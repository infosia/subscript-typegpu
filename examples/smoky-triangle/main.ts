// example: smoky-triangle
// Fills a single triangle with layered animated smoke instead of a flat color.
// TypeGPU exposes distortion, sharpness, two gradient colors, and two mode toggles.
// This port commits one gradient and one density, and drops the grain and the polar mode.
// Ported from TypeGPU's smoky-triangle example (https://github.com/software-mansion/TypeGPU).

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
  perlin3d,
} from "./typegpu-noise";
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
  smoke_FRAGMENT_ENTRY,
  smoke_LAYOUT0,
  smoke_TARGET_FORMAT,
  smoke_VERTEX_ENTRY,
  smoke_VERTEX_LAYOUT0,
  smoke_WGSL,
} from "./main.typegpu";

// One triangle corner in normalized device coordinates plus its texture coordinate.
// TypeGPU reads the same values from two constant arrays by vertex index.
// This port sends them through a vertex buffer, so the generator fixes the stride.
@CStruct
class Vertex {
  position: Vec2f;
  uv: Vec2f;

  constructor(position: Vec2f, uv: Vec2f) {
    this.position = position;
    this.uv = uv;
  }
}

@CStruct
class FrameData {
  // One vec2f carries time and density. TypeGPU holds a full parameter struct that
  // its sliders patch on every frame.
  motion: Vec2f;

  constructor(time: f32, density: f32) {
    this.motion = new Vec2f(time, density);
  }
}

// The vertex stage returns this record and the fragment stage receives it.
// The field named position becomes the clip position, and uv becomes location 0.
@CStruct
class Varyings {
  position: Vec4f;
  uv: Vec2f;

  constructor(position: Vec4f, uv: Vec2f) {
    this.position = position;
    this.uv = uv;
  }
}

// A bind group layout is a class here, not a runtime object. Field order fixes the
// binding numbers, and the generator emits the WGSL declaration and smoke_LAYOUT0.
class SmokeLayout {
  frame: Uniform<FrameData>;

  constructor(frame: Uniform<FrameData>) {
    this.frame = frame;
  }
}

// Four octaves of noise drift across the surface. The third noise axis carries the
// time and one offset per octave, so no two octaves repeat each other.
function smokeField(uv: Vec2f, time: f32): f32 {
  let frequency: f32 = 2.2;
  let amplitude: f32 = 0.58;
  let value: f32 = 0.0;
  for (let octave: u32 = 0; octave < 4; octave += 1) {
    value += perlin3d(new Vec3f(
      uv.x * frequency + time * 0.16,
      uv.y * frequency - time * 0.11,
      time * 0.23 + (octave as f32) * 5.0,
    )) * amplitude;
    frequency *= 2.0;
    amplitude *= 0.5;
  }
  return value;
}

// The vertex stage passes the vertex buffer values through. It widens the position to
// four components with w at 1.0, so the triangle needs no projection matrix.
function smokeVertex(res: SmokeLayout, value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    value.uv,
  );
}

// Per pixel the fragment stage reads the noise field, scales it by the density, and
// clamps the result to 0.0 through 1.0. The two colors are the ends of the gradient:
// dark violet for thin smoke and orange for dense smoke.
function smokeFragment(
  res: SmokeLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const frame: FrameData = res.frame.$;
  const field: f32 = smokeField(input.uv, frame.motion.x) * frame.motion.y;
  let smoke: f32 = 0.5 + field;
  if (smoke < 0.0) smoke = 0.0;
  if (smoke > 1.0) smoke = 1.0;
  const low = new Vec3f(0.08, 0.025, 0.12);
  const high = new Vec3f(0.95, 0.42, 0.2);
  const color: Vec3f = low.add(high.sub(low).scale(smoke));
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

// The declaration ties the layout class, the two kernels, and the target format together.
// The generator reads it at compile time and emits smoke_WGSL into ./main.typegpu.
// TypeGPU builds the equivalent shader at run time from the same functions.
export const smoke: RenderPipelineSpec = renderPipelineL<SmokeLayout, Vertex, Varyings>(
  smokeVertex,
  smokeFragment,
  { format: "bgra8unorm" },
);

// The window host calls init, frame, and shutdown as separate entry points, so the
// handles live in module state. The script owns each one and frees it in shutdown.
let activeDevice: GPUHostOwnedDevice | null = null;
let activePipeline: RenderPipeline | null = null;
let activeVertices: GPUBuffer | null = null;
let activeFrameBuffer: GPUBuffer | null = null;
let activeGroup: GPUBindGroup | null = null;
let frameCount: u32 = 0;

// The host calls init once, before the first frame, with the device it owns.
export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  // The host picks the surface format. The generator baked one format into the pipeline,
  // so a mismatch stops the example here instead of at pipeline creation.
  if (format !== smoke_TARGET_FORMAT) {
    print(`FAIL format expected=${smoke_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The host owns the device and the queue. This wrapper borrows both and never destroys them.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // The vertex buffer holds three vertices. Vertex_STRIDE is the generated byte stride,
  // so the size never repeats a number counted by hand.
  const vertices = hostDevice.createBuffer({
    label: "smoky-triangle-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // The uniform buffer holds one FrameData. FrameData_SIZE comes from the layout generator,
  // so the host size and the WGSL struct size always agree.
  const frameBuffer = hostDevice.createBuffer({
    label: "smoky-triangle-frame",
    size: FrameData_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  // A host-owned device returns a new queue wrapper from each call, so this block frees it.
  using queue = hostDevice.queue();
  // The triangle covers most of the surface. Context.bytesOf packs the array with the layout
  // rules the shader uses, so the host never counts offsets.
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>([
    new Vertex(new Vec2f(-0.78, -0.68), new Vec2f(0.0, 0.0)),
    new Vertex(new Vec2f(0.78, -0.68), new Vec2f(1.0, 0.0)),
    new Vertex(new Vec2f(0.0, 0.78), new Vec2f(0.5, 1.0)),
  ]));
  // The uniform needs a valid value before the first draw. 0.82 is the fixed density that
  // replaces the density slider of the upstream example.
  queue.writeBuffer(frameBuffer, 0, Context.bytesOf<FrameData>(new FrameData(0.0, 0.82)));
  // Pipeline creation raises no exception. The error scope turns a bad shader or a bad
  // layout into a value the code below tests.
  hostDevice.pushErrorScope("validation");
  // The generated WGSL, the entry names, and the layout constants build the pipeline.
  const pipeline = createRenderPipelineHost(
    hostDevice,
    smoke_WGSL,
    smoke_VERTEX_ENTRY,
    smoke_FRAGMENT_ENTRY,
    [smoke_LAYOUT0],
    [smoke_VERTEX_LAYOUT0],
    smoke,
  );
  // A null check replaces the exception a browser port throws. The failure path frees the
  // three handles this function already created.
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    pipeline.dispose();
    frameBuffer.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  // The layout comes back from the pipeline, so the shader and the bind group cannot
  // disagree about group 0.
  using bindLayout = pipeline.bindGroupLayout(0);
  // The resource list follows the field order of SmokeLayout. One uniform buffer fills binding 0.
  const group = createBindGroupHost(
    hostDevice,
    bindLayout,
    smoke_LAYOUT0,
    [bufferResource(frameBuffer)],
  );
  // The handles reach module state only after every step succeeds, so frame never sees a
  // half-built set.
  activeDevice = hostDevice;
  activePipeline = pipeline;
  activeVertices = vertices;
  activeFrameBuffer = frameBuffer;
  activeGroup = group;
}

// The host calls frame once per presented image. This example reads no key and no pointer,
// because the port fixes every value the upstream sliders control.
export function frame(
  view: SubscriptTypegpuTextureView,
  width: u32,
  height: u32,
  key: u32,
  pointerX: f32,
  pointerY: f32,
  buttons: u32,
): void {
  // init can fail, so frame proves that every handle exists before it records commands.
  // These null checks replace the exception a browser port throws.
  const device = activeDevice;
  const pipeline = activePipeline;
  const vertices = activeVertices;
  const frameBuffer = activeFrameBuffer;
  const group = activeGroup;
  if (device === null) return;
  if (pipeline === null) return;
  if (vertices === null) return;
  if (frameBuffer === null) return;
  if (group === null) return;
  // The window host carries no clock. The frame count divided by 60.0 gives seconds at the
  // nominal present rate, which drives the noise drift.
  frameCount += 1;
  using queue = device.queue();
  // The uniform write enters the queue before the submit below. The queue keeps that order,
  // so the draw reads this frame's time.
  queue.writeBuffer(
    frameBuffer,
    0,
    Context.bytesOf<FrameData>(new FrameData((frameCount as f32) / 60.0, 0.82)),
  );
  // The host passes the current surface view as a raw handle. The wrapper does not own it,
  // and the host presents the surface after frame returns.
  const target = new GPUTextureView(view);
  using encoder = device.createCommandEncoderDefault();
  // The attachment clears to a near-black violet. loadOp clear drops the previous image, so
  // no stale pixel stays outside the triangle.
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.018, g: 0.008, b: 0.035, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  // The window resizes between frames. The viewport and the scissor follow the size the host
  // reports, so the triangle keeps the whole surface.
  pass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  pass.setScissorRect(0, 0, width, height);
  // One call sets the pipeline, the bind groups by group index, and the vertex buffers by slot.
  pipeline.bind(pass, [group], [vertices]);
  pass.draw(3);
  pass.end();
  // The pass must end before the encoder finishes. submit hands the command buffer to the
  // queue, which runs it before the host presents the surface.
  using command = encoder.finishDefault();
  queue.submit([command]);
}

// The host calls shutdown once. The script frees every GPU handle by hand, because this
// library keeps no finalizer and no reference count for scripts.
export function shutdown(): void {
  if (activeGroup !== null) activeGroup.dispose();
  if (activeFrameBuffer !== null) activeFrameBuffer.dispose();
  if (activeVertices !== null) activeVertices.dispose();
  if (activePipeline !== null) activePipeline.dispose();
  activeFrameBuffer = null;
  activeVertices = null;
  activePipeline = null;
  activeGroup = null;
  activeDevice = null;
}
