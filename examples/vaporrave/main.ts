// example: vaporrave
// Raymarches a noise-warped sphere above a luminous retro grid floor.
// TypeGPU exposes glow intensity, a floor speed, a sphere speed, a sphere color, and
// a floor pattern. This port commits one grid floor and one palette, and drops the
// glow accumulation and the sky fog.
// Ported from TypeGPU's vaporrave example (https://github.com/software-mansion/TypeGPU).

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
  opUnion,
  sdPlane,
  sdSphere,
} from "./typegpu-sdf";
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
  vapor_FRAGMENT_ENTRY,
  vapor_LAYOUT0,
  vapor_TARGET_FORMAT,
  vapor_VERTEX_ENTRY,
  vapor_VERTEX_LAYOUT0,
  vapor_WGSL,
} from "./main.typegpu";

// One corner of the full-surface triangle in normalized device coordinates.
// TypeGPU reads the same three corners from a constant array inside its vertex stage.
@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// The only uniform. time is in seconds and aspect is the window width over its height.
// TypeGPU keeps five uniforms, because its controls change the scene while it runs.
@CStruct
class FrameData {
  time: f32;
  aspect: f32;

  constructor(time: f32, aspect: f32) {
    this.time = time;
    this.aspect = aspect;
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
// binding numbers, and the generator emits the WGSL declaration and vapor_LAYOUT0.
class VaporLayout {
  frame!: Uniform<FrameData>;
}

// The whole scene is one signed distance field. The march below needs the distance to the
// nearest surface, so the sphere and the floor combine through a union.
function vaporDistance(point: Vec3f, time: f32): f32 {
  const spherePoint = point.sub(new Vec3f(0.0, 1.05, 0.0));
  // Noise added to the sphere distance warps the surface. The amplitude stays small,
  // so the marched field stays close to a true distance.
  const noise: f32 = perlin3d(spherePoint.scale(2.4).add(new Vec3f(0.0, time * 0.35, 0.0)));
  const sphere: f32 = sdSphere(spherePoint, 0.82) + noise * 0.07;
  const floor: f32 = sdPlane(point, new Vec3f(0.0, 1.0, 0.0), 0.0);
  return opUnion(sphere, floor);
}

// The surface normal is the gradient of the distance field. Three offset evaluations give
// a forward difference, so the scene needs no analytic derivative.
function vaporNormal(point: Vec3f, time: f32): Vec3f {
  const epsilon: f32 = 0.002;
  const center: f32 = vaporDistance(point, time);
  return new Vec3f(
    vaporDistance(point.add(new Vec3f(epsilon, 0.0, 0.0)), time) - center,
    vaporDistance(point.add(new Vec3f(0.0, epsilon, 0.0)), time) - center,
    vaporDistance(point.add(new Vec3f(0.0, 0.0, epsilon)), time) - center,
  ).normalize();
}

// Points near the floor plane take a grid line color from the fractional cell
// distance. Every other point takes the sphere palette. TypeGPU selects the floor
// pattern through a slot and rebuilds the pipeline.
function retroSurface(point: Vec3f, normal: Vec3f, time: f32): Vec3f {
  if (point.y < 0.025) {
    const cell: Vec2f = point.xz.fract();
    let xEdge: f32 = cell.x;
    if (1.0 - cell.x < xEdge) xEdge = 1.0 - cell.x;
    let yEdge: f32 = cell.y;
    if (1.0 - cell.y < yEdge) yEdge = 1.0 - cell.y;
    let edge: f32 = xEdge;
    if (yEdge < edge) edge = yEdge;
    if (edge < 0.035) return new Vec3f(0.06, 0.88, 1.0);
    return new Vec3f(0.035, 0.01, 0.09);
  }
  let facing: f32 = normal.dot(new Vec3f(-0.35, 0.75, 0.4).normalize());
  if (facing < 0.0) facing = 0.0;
  const pulse: f32 = perlin3d(point.scale(3.0).add(new Vec3f(time * 0.2, 0.0, 0.0))) * 0.5 + 0.5;
  return new Vec3f(
    0.55 + pulse * 0.42,
    0.08 + facing * 0.18,
    0.62 + facing * 0.34,
  );
}

// The vertex stage widens the corner to clip space and maps it from the -1 to 1 range
// into the 0 to 1 uv range. It reads no binding.
function vaporVertex(res: VaporLayout, value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

// Per pixel the fragment stage marches one ray from a fixed camera.
// TypeGPU marches the same scene and adds a glow accumulator and a fog blend.
function vaporFragment(
  res: VaporLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const frame: FrameData = res.frame.$;
  const screen = new Vec2f(
    (input.uv.x * 2.0 - 1.0) * frame.aspect,
    input.uv.y * 2.0 - 1.0,
  );
  // The camera stands above the floor and looks toward negative z. The -0.18 offset tilts
  // the view down, so the horizon stays above the middle of the surface.
  const origin = new Vec3f(0.0, 1.45, 4.4);
  const direction = new Vec3f(screen.x, screen.y - 0.18, -1.75).normalize();
  let travel: f32 = 0.0;
  let hit: boolean = false;
  // Sphere tracing: each step advances by the field value, which never passes a surface.
  // The 80 steps and the 14.0 limit bound the work per pixel.
  for (let stepIndex: u32 = 0; stepIndex < 80; stepIndex += 1) {
    const field: f32 = vaporDistance(origin.add(direction.scale(travel)), frame.time);
    if (field < 0.0015) {
      hit = true;
      break;
    }
    travel += field;
    if (travel > 14.0) break;
  }
  // A ray that leaves the scene paints a vertical sky gradient.
  // TypeGPU mixes the same sky into every pixel through a fog factor.
  if (!hit) {
    const sky: f32 = input.uv.y;
    return new Vec4f(0.03 + sky * 0.12, 0.005, 0.09 + sky * 0.2, 1.0);
  }
  const point: Vec3f = origin.add(direction.scale(travel));
  const normal: Vec3f = vaporNormal(point, frame.time);
  const color: Vec3f = retroSurface(point, normal, frame.time);
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

// The declaration ties the layout class, the two kernels, and the target format together.
// The generator reads it at compile time and emits vapor_WGSL into ./main.typegpu.
// TypeGPU rebuilds its pipeline whenever the floor pattern control changes.
export const vapor: RenderPipelineSpec = renderPipelineL<VaporLayout, Vertex, Varyings>(
  vaporVertex,
  vaporFragment,
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
  if (format !== vapor_TARGET_FORMAT) {
    print(`FAIL format expected=${vapor_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The host owns the device and the queue. This wrapper borrows both and never destroys them.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // The vertex buffer holds three vertices. Vertex_STRIDE is the generated byte stride,
  // so the size never repeats a number counted by hand.
  const vertices = hostDevice.createBuffer({
    label: "vaporrave-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // The uniform buffer holds one FrameData. FrameData_SIZE comes from the layout generator,
  // so the host size and the WGSL struct size always agree.
  const frameBuffer = hostDevice.createBuffer({
    label: "vaporrave-frame",
    size: FrameData_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  // A host-owned device returns a new queue wrapper from each call, so this block frees it.
  using queue = hostDevice.queue();
  // One oversized triangle covers the whole surface. The parts outside the viewport clip
  // away, so this example needs no quad and no index buffer.
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>([
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(3.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 3.0)),
  ]));
  // The uniform needs a valid value before the first draw. The aspect ratio starts at 1.0,
  // and the first frame replaces it with the real window ratio.
  queue.writeBuffer(frameBuffer, 0, Context.bytesOf<FrameData>(new FrameData(0.0, 1.0)));
  // Pipeline creation raises no exception. The error scope turns a bad shader or a bad
  // layout into a value the code below tests.
  hostDevice.pushErrorScope("validation");
  // The generated WGSL, the entry names, and the layout constants build the pipeline.
  const pipeline = createRenderPipelineHost(
    hostDevice,
    vapor_WGSL,
    vapor_VERTEX_ENTRY,
    vapor_FRAGMENT_ENTRY,
    [vapor_LAYOUT0],
    [vapor_VERTEX_LAYOUT0],
    vapor,
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
  // The resource list follows the field order of VaporLayout. One uniform buffer fills binding 0.
  const group = createBindGroupHost(
    hostDevice,
    bindLayout,
    vapor_LAYOUT0,
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
// because the port fixes every value the upstream controls change.
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
  // The window host carries no clock, so the frame count is the only time source.
  frameCount += 1;
  using queue = device.queue();
  // The frame count divided by 60.0 gives seconds at the nominal present rate. The aspect
  // ratio follows the reported size, so a resize needs no other work.
  queue.writeBuffer(
    frameBuffer,
    0,
    Context.bytesOf<FrameData>(new FrameData(
      (frameCount as f32) / 60.0,
      (width as f32) / (height as f32),
    )),
  );
  // The host passes the current surface view as a raw handle. The wrapper does not own it,
  // and the host presents the surface after frame returns.
  const target = new GPUTextureView(view);
  using encoder = device.createCommandEncoderDefault();
  // The attachment clears to a dark violet. The triangle covers every pixel, so the clear
  // color only shows when the draw fails.
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.02, g: 0.0, b: 0.07, a: 1.0 },
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
