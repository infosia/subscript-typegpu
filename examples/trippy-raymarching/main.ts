// example: trippy-raymarching
// Raymarches a twisted, infinitely repeated sphere lattice inside a tunnel.
// This port commits the upstream slider defaults and drops the time terms of the wave,
// the sphere radius, and the light direction. It uses its own palette coefficients and
// reads the pointer in surface coordinates without the vertical flip.
// Ported from TypeGPU's trippy-raymarching example (https://github.com/software-mansion/TypeGPU).

import {
  FragmentInvocation,
  RenderPipeline,
  RenderPipelineSpec,
  Uniform,
  VertexInvocation,
  bufferResource,
  createBindGroupHost,
  createRenderPipelineHost,
  renderPipelineL,
} from "./typegpu";
import {
  Vec2f,
  Vec3f,
  Vec4f,
} from "./typegpu-types";
import {
  GPUBindGroup,
  GPUBuffer,
  GPUBufferUsage,
  GPUHostOwnedDevice,
  GPUTextureView,
  hostOwnedGPUDevice,
} from "./webgpu";
import {
  FrameData_SIZE,
  Vertex_STRIDE,
  trippy_FRAGMENT_ENTRY,
  trippy_LAYOUT0,
  trippy_TARGET_FORMAT,
  trippy_VERTEX_ENTRY,
  trippy_VERTEX_LAYOUT0,
  trippy_WGSL,
} from "./main.typegpu";

// The vertex record. The generator emits Vertex_STRIDE from this class, so the host code
// never counts bytes.
@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// The per-frame uniform. `time` carries seconds derived from the frame count, and `pointer`
// carries the surface position in [-1, 1] per axis.
@CStruct
class FrameData {
  time: f32;
  aspect: f32;
  pointer: Vec2f;

  constructor(time: f32, aspect: f32, pointer: Vec2f) {
    this.time = time;
    this.aspect = aspect;
    this.pointer = pointer;
  }
}

// The inter-stage record. The field named `position` becomes the clip-space builtin, and
// `uv` becomes location 0.
@CStruct
class Varyings {
  position: Vec4f;
  uv: Vec2f;

  constructor(position: Vec4f, uv: Vec2f) {
    this.position = position;
    this.uv = uv;
  }
}

// The bind group layout. TypeGPU splits time, aspect, mouse, and the slider parameters into
// four uniforms. One record replaces them here, because the parameters are constants.
class TrippyLayout {
  frame!: Uniform<FrameData>;
}

// These constants hold the upstream slider defaults. MAX_STEPS bounds the march per pixel,
// and MAX_DISTANCE ends a ray that escapes down the tunnel.
const SPHERE_SPACING: f32 = 4.0;
const MAX_DISTANCE: f32 = 60.0;
const MAX_STEPS: u32 = 96;
const SPHERE_RADIUS: f32 = 0.5;
const WAVE_FREQUENCY_X: f32 = 2.0;
const WAVE_FREQUENCY_Y: f32 = 2.0;
const WAVE_AMPLITUDE: f32 = 0.2;
const TWIST_FACTOR: f32 = 0.01;
const TUNNEL_RADIUS: f32 = 0.697;

// The cosine palette maps one scalar to a color. The 6.28318 factor turns the argument into
// a full turn.
function cosinePalette(a: Vec3f, b: Vec3f, c: Vec3f, d: Vec3f, value: f32): Vec3f {
  return a.add(b.mul(c.scale(value).add(d).scale(6.28318).cos()));
}

// Twist about z, repeat on the sphere spacing, and clip to the tunnel bore.
function trippyDistance(point: Vec3f, time: f32, pointer: Vec2f): f32 {
  const angle: f32 = point.z * TWIST_FACTOR + time * 0.2;
  const phases = new Vec2f(angle, angle);
  const cosine: f32 = phases.cos().x;
  const sine: f32 = phases.sin().x;
  const twisted = new Vec3f(
    point.x * cosine - point.y * sine,
    point.x * sine + point.y * cosine,
    point.z,
  );
  // The pointer scrolls the repeated lattice sideways. The 7/3 factor matches the world scale
  // that upstream applies to its mouse uniform.
  const scroll = pointer.scale(7.0 / 3.0);
  const repeated = new Vec3f(
    twisted.x / SPHERE_SPACING + scroll.x,
    twisted.y / SPHERE_SPACING + scroll.y,
    twisted.z / SPHERE_SPACING,
  ).fract().scale(SPHERE_SPACING).sub(new Vec3f(
    SPHERE_SPACING * 0.5,
    SPHERE_SPACING * 0.5,
    SPHERE_SPACING * 0.5,
  ));
  const waveX: f32 = new Vec2f(
    twisted.x * WAVE_FREQUENCY_X,
    twisted.x * WAVE_FREQUENCY_X,
  ).sin().x;
  const waveY: f32 = new Vec2f(
    twisted.y * WAVE_FREQUENCY_Y,
    twisted.y * WAVE_FREQUENCY_Y,
  ).cos().x;
  const sphereDistance: f32 = repeated.length() - SPHERE_RADIUS
    + WAVE_AMPLITUDE * waveX * waveY;
  // The tunnel is an inverted cylinder. The larger of the two distances keeps the spheres
  // inside the bore.
  const tunnelDistance: f32 = TUNNEL_RADIUS - twisted.xy.length();
  if (sphereDistance > tunnelDistance) return sphereDistance;
  return tunnelDistance;
}

// The normal is the gradient of the distance field. Three forward differences cost three
// extra field evaluations per hit pixel.
function trippyNormal(point: Vec3f, time: f32, pointer: Vec2f): Vec3f {
  const epsilon: f32 = 0.001;
  const center: f32 = trippyDistance(point, time, pointer);
  return new Vec3f(
    trippyDistance(point.add(new Vec3f(epsilon, 0.0, 0.0)), time, pointer) - center,
    trippyDistance(point.add(new Vec3f(0.0, epsilon, 0.0)), time, pointer) - center,
    trippyDistance(point.add(new Vec3f(0.0, 0.0, epsilon)), time, pointer) - center,
  ).normalize();
}

// The vertex stage passes the corner through and derives the uv. TypeGPU builds the same
// triangle from the vertex index, and this port reads it from a vertex buffer.
function trippyVertex(res: TrippyLayout, value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

// One invocation marches one ray. The march stops at a hit, at MAX_DISTANCE, or after
// MAX_STEPS, so a pixel costs a bounded amount of work.
function trippyFragment(
  res: TrippyLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const frame: FrameData = res.frame.$;
  const uv = new Vec2f(
    (input.uv.x * 2.0 - 1.0) * frame.aspect,
    input.uv.y * 2.0 - 1.0,
  );
  // The camera travels 3 units per second along z. The direction takes z as 1, which fixes the
  // field of view.
  const origin = new Vec3f(0.0, 0.0, frame.time * 3.0);
  const direction = new Vec3f(uv.x, uv.y, 1.0).normalize();
  let travel: f32 = 0.0;
  let glow: f32 = 0.0;
  let hit: boolean = false;
  for (let stepIndex: u32 = 0; stepIndex < MAX_STEPS; stepIndex += 1) {
    const distance: f32 = trippyDistance(
      origin.add(direction.scale(travel)),
      frame.time,
      frame.pointer,
    );
    // Every step feeds the glow, so near misses light the fog.
    glow += 0.015 / (0.01 + distance * distance);
    if (distance < 0.001) {
      hit = true;
      break;
    }
    travel += distance;
    if (travel > MAX_DISTANCE) break;
  }
  // A miss returns only the fog that the glow accumulated. Upstream tints the same miss with
  // the palette.
  if (!hit) {
    const background = new Vec3f(0.008, 0.003, 0.025).scale(glow);
    return new Vec4f(background.x, background.y, background.z, 1.0);
  }
  const point: Vec3f = origin.add(direction.scale(travel));
  const normal: Vec3f = trippyNormal(point, frame.time, frame.pointer);
  // The light direction is fixed. Upstream swings it with time, and this port drops that
  // motion with the sliders.
  const lightDirection = new Vec3f(-0.45, 0.8, -0.35).normalize();
  const halfLambert: f32 = normal.dot(lightDirection) * 0.5 + 0.5;
  const color = cosinePalette(
    new Vec3f(0.5, 0.5, 0.5),
    new Vec3f(0.5, 0.5, 0.5),
    new Vec3f(1.0, 1.0, 1.0),
    new Vec3f(0.0, 0.18, 0.42),
    travel * 0.035 + frame.time * 0.08,
  ).scale(0.18 + halfLambert * 0.82);
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

// The pipeline declaration joins the layout class, the vertex schema, and the varyings.
// `subscript-typegpu-gen` emits the WGSL for both stages before the run.
export const trippy: RenderPipelineSpec = renderPipelineL<TrippyLayout, Vertex, Varyings>(
  trippyVertex,
  trippyFragment,
  { format: "bgra8unorm" },
);

// The host calls `init`, `frame`, and `shutdown` separately, so every handle that outlives
// `init` lives in module state. A `using` declaration disposes a handle too early here.
let activeDevice: GPUHostOwnedDevice | null = null;
let activePipeline: RenderPipeline | null = null;
let activeVertices: GPUBuffer | null = null;
let activeFrameBuffer: GPUBuffer | null = null;
let activeGroup: GPUBindGroup | null = null;
let frameCount: u32 = 0;

// `init` runs once, after the host configures the surface. It creates every long-lived
// resource. The instance and the device stay with the host.
export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  // The generator pins the target format into the pipeline. A mismatch with the host surface
  // fails here, not inside pipeline creation.
  if (format !== trippy_TARGET_FORMAT) {
    print(`FAIL format expected=${trippy_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The wrapper adapts the host handles to the API layer. It carries no `dispose`, because
  // the host owns the device.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // The vertex buffer holds the three corners of the fullscreen triangle. Vertex_STRIDE keeps
  // the size right when the schema changes.
  const vertices = hostDevice.createBuffer({
    label: "trippy-raymarching-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // One uniform buffer holds FrameData. COPY_DST admits the queue write that each frame makes.
  const frameBuffer = hostDevice.createBuffer({
    label: "trippy-raymarching-frame",
    size: FrameData_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  // The queue wrapper is a handle. `using` disposes it at the end of `init`, and `frame`
  // takes a fresh one.
  using queue = hostDevice.queue();
  // `Context.bytesOf` lays out the values with the generated C layout, so the bytes match the
  // WGSL that the generator emits.
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>([
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(3.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 3.0)),
  ]));
  // The first uniform write gives the buffer a defined value before the first frame runs.
  queue.writeBuffer(
    frameBuffer,
    0,
    Context.bytesOf<FrameData>(new FrameData(0.0, 1.0, new Vec2f(0.0, 0.0))),
  );
  // The error scope catches a validation failure from pipeline creation. The layers return the
  // failure as a value, so a `null` check replaces an exception.
  hostDevice.pushErrorScope("validation");
  // The call takes the generated WGSL text, the two entry names, the bind group layout, and
  // the vertex layout. No shader text is built here.
  const pipeline = createRenderPipelineHost(
    hostDevice,
    trippy_WGSL,
    trippy_VERTEX_ENTRY,
    trippy_FRAGMENT_ENTRY,
    [trippy_LAYOUT0],
    [trippy_VERTEX_LAYOUT0],
    trippy,
  );
  const validationError = hostDevice.popErrorScope();
  // The error path disposes every handle that the failed run already created, because no
  // finalizer runs later.
  if (validationError !== null) {
    pipeline.dispose();
    frameBuffer.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  // The native layout comes from the pipeline. The bind group reads it at creation, so `using`
  // releases the handle right after.
  using bindLayout = pipeline.bindGroupLayout(0);
  // One resource fills the group, in the field order of TrippyLayout.
  const group = createBindGroupHost(
    hostDevice,
    bindLayout,
    trippy_LAYOUT0,
    [bufferResource(frameBuffer)],
  );
  // The state moves to module scope only after every step passes. A failed `init` leaves the
  // fields null, and `frame` returns at once.
  activeDevice = hostDevice;
  activePipeline = pipeline;
  activeVertices = vertices;
  activeFrameBuffer = frameBuffer;
  activeGroup = group;
}

// The host calls `frame` once per presented frame. `view` is the swapchain view the host
// owns, and `width` and `height` are surface pixels.
export function frame(
  view: SubscriptTypegpuTextureView,
  width: u32,
  height: u32,
  key: u32,
  pointerX: f32,
  pointerY: f32,
  buttons: u32,
): void {
  const device = activeDevice;
  const pipeline = activePipeline;
  const vertices = activeVertices;
  const frameBuffer = activeFrameBuffer;
  const group = activeGroup;
  // A null field means `init` failed or never ran. The frame returns, because the layers
  // report failure as a value.
  if (device === null) return;
  if (pipeline === null) return;
  if (vertices === null) return;
  if (frameBuffer === null) return;
  if (group === null) return;
  // The frame count is the only clock the window host offers. 60 frames stand for one second
  // of shader time.
  frameCount += 1;
  // Map the pointer to [-1, 1] per axis. (0, 0) stands for a pointer outside.
  let pointer = new Vec2f(0.0, 0.0);
  if (pointerX >= 0.0 && pointerY >= 0.0) {
    pointer = new Vec2f(
      pointerX / (width as f32) * 2.0 - 1.0,
      pointerY / (height as f32) * 2.0 - 1.0,
    );
  }
  // The queue applies the write below before it runs the command buffer that the submit sends,
  // so the fragment reads the values of this frame.
  using queue = device.queue();
  queue.writeBuffer(
    frameBuffer,
    0,
    Context.bytesOf<FrameData>(new FrameData(
      (frameCount as f32) / 60.0,
      (width as f32) / (height as f32),
      pointer,
    )),
  );
  // The host owns the swapchain view. The wrapper adds no ownership, and `shutdown` never
  // disposes it.
  const target = new GPUTextureView(view);
  // One encoder records the whole frame. `using` disposes it after the submit.
  using encoder = device.createCommandEncoderDefault();
  // The pass clears to black first, so no pixel of an earlier frame survives a resize.
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  // The viewport and the scissor follow the current surface size, because the host resizes the
  // swapchain without a new pipeline.
  pass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  pass.setScissorRect(0, 0, width, height);
  // `bind` sets the pipeline, the bind groups, and the vertex buffers in one call. TypeGPU
  // spells the same step as `.with(bindGroup)`.
  pipeline.bind(pass, [group], [vertices]);
  // The three vertices reach past the surface, and the rasterizer clips the excess.
  pass.draw(3);
  pass.end();
  // `finishDefault` closes the encoder, and `submit` hands the command buffer to the queue.
  // The host presents the surface after `frame` returns.
  using command = encoder.finishDefault();
  queue.submit([command]);
}

// The host calls `shutdown` once, before it releases the device. The bind group goes first,
// because it names the other handles.
export function shutdown(): void {
  if (activeGroup !== null) activeGroup.dispose();
  if (activeFrameBuffer !== null) activeFrameBuffer.dispose();
  if (activeVertices !== null) activeVertices.dispose();
  if (activePipeline !== null) activePipeline.dispose();
  // The null assignments make a second `shutdown` call safe.
  activeFrameBuffer = null;
  activeVertices = null;
  activePipeline = null;
  activeGroup = null;
  activeDevice = null;
  frameCount = 0;
}
