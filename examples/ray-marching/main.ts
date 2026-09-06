// example: ray-marching
// Raymarches a sphere, a framed box, and a floor with animated soft shadows.
// TypeGPU exposes no controls here. This port drops the rotating two-sphere blend,
// the per-shape colors, the checkered floor, the orbiting light, and the distance fog.
// Ported from TypeGPU's ray-marching example (https://github.com/software-mansion/TypeGPU).

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
  opSmoothUnion,
  opUnion,
  sdBoxFrame,
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
  scene_FRAGMENT_ENTRY,
  scene_LAYOUT0,
  scene_TARGET_FORMAT,
  scene_VERTEX_ENTRY,
  scene_VERTEX_LAYOUT0,
  scene_WGSL,
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
// TypeGPU keeps time and the canvas resolution in two separate uniforms.
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
// binding numbers, and the generator emits the WGSL declaration and scene_LAYOUT0.
class SceneLayout {
  frame!: Uniform<FrameData>;
}

// The whole scene is one signed distance field. The smooth union melts the sphere into
// the box frame, and TypeGPU carries a color beside each distance to blend the shapes.
function sceneDistance(point: Vec3f, time: f32): f32 {
  // One vector carries the time twice, so `sin()` reaches the WGSL builtin. A scalar
  // `Math.sin` needs an `f64` cast, and K12 rejects that cast inside a kernel.
  const phase: f32 = new Vec2f(time * 0.7, time * 0.7).sin().x;
  const sphere: f32 = sdSphere(point.sub(new Vec3f(-0.8, 0.75 + phase * 0.18, 0.0)), 0.72);
  const frame: f32 = sdBoxFrame(
    point.sub(new Vec3f(0.85, 0.72, 0.15)),
    new Vec3f(0.58, 0.58, 0.58),
    0.09,
  );
  const joined: f32 = opSmoothUnion(sphere, frame, 0.22);
  return opUnion(joined, sdPlane(point, new Vec3f(0.0, 1.0, 0.0), 0.0));
}

// The surface normal is the gradient of the distance field. Three offset evaluations give
// a forward difference, so the scene needs no analytic derivative.
function sceneNormal(point: Vec3f, time: f32): Vec3f {
  const epsilon: f32 = 0.002;
  const center: f32 = sceneDistance(point, time);
  return new Vec3f(
    sceneDistance(point.add(new Vec3f(epsilon, 0.0, 0.0)), time) - center,
    sceneDistance(point.add(new Vec3f(0.0, epsilon, 0.0)), time) - center,
    sceneDistance(point.add(new Vec3f(0.0, 0.0, epsilon)), time) - center,
  ).normalize();
}

// The shadow ray keeps the smallest ratio of field distance to travel distance.
// TypeGPU marches toward an orbiting light position. This port uses one fixed
// direction and one floor value.
function softShadow(origin: Vec3f, direction: Vec3f, time: f32): f32 {
  let visibility: f32 = 1.0;
  let distance: f32 = 0.03;
  for (let stepIndex: u32 = 0; stepIndex < 24; stepIndex += 1) {
    const field: f32 = sceneDistance(origin.add(direction.scale(distance)), time);
    // 10.0 sets the penumbra hardness. A larger number narrows the soft edge of the shadow.
    const estimate: f32 = 10.0 * field / distance;
    if (estimate < visibility) visibility = estimate;
    if (field < 0.001 || distance > 7.0) break;
    distance += field;
  }
  if (visibility < 0.12) return 0.12;
  if (visibility > 1.0) return 1.0;
  return visibility;
}

// The vertex stage widens the corner to clip space and maps it from the -1 to 1 range
// into the 0 to 1 uv range. It reads no binding.
function sceneVertex(res: SceneLayout, value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

// Per pixel the fragment stage marches one ray, shades the hit, and traces one shadow ray.
// TypeGPU shades with per-shape colors and blends distance fog over the result.
function sceneFragment(
  res: SceneLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const frame: FrameData = res.frame.$;
  const screen = new Vec2f(
    (input.uv.x * 2.0 - 1.0) * frame.aspect,
    input.uv.y * 2.0 - 1.0,
  );
  // The camera stands above the floor and looks toward negative z. The -0.12 offset tilts
  // the view down, so the horizon stays above the middle of the surface.
  const origin = new Vec3f(0.0, 1.15, 4.2);
  const direction = new Vec3f(screen.x, screen.y - 0.12, -1.8).normalize();
  let travel: f32 = 0.0;
  let hit: boolean = false;
  // Sphere tracing: each step advances by the field value, which never passes a surface.
  // The 72 steps and the 12.0 limit bound the work per pixel.
  for (let stepIndex: u32 = 0; stepIndex < 72; stepIndex += 1) {
    const point: Vec3f = origin.add(direction.scale(travel));
    const field: f32 = sceneDistance(point, frame.time);
    if (field < 0.0015) {
      hit = true;
      break;
    }
    travel += field;
    if (travel > 12.0) break;
  }
  // A ray that leaves the scene paints a vertical sky gradient.
  if (!hit) {
    const horizon: f32 = input.uv.y * 0.2;
    return new Vec4f(0.025 + horizon, 0.035 + horizon, 0.08 + horizon, 1.0);
  }
  const point: Vec3f = origin.add(direction.scale(travel));
  const normal: Vec3f = sceneNormal(point, frame.time);
  const lightDirection = new Vec3f(-0.45, 0.8, 0.35).normalize();
  let diffuse: f32 = normal.dot(lightDirection);
  if (diffuse < 0.0) diffuse = 0.0;
  // The shadow ray starts a small step along the normal. Without that offset the ray hits
  // the surface it starts on and every lit pixel turns black.
  const shadow: f32 = softShadow(point.add(normal.scale(0.006)), lightDirection, frame.time);
  const light: f32 = 0.16 + diffuse * shadow * 0.84;
  const base = new Vec3f(0.24, 0.58, 0.78);
  return new Vec4f(base.x * light, base.y * light, base.z * light, 1.0);
}

// The declaration ties the layout class, the two kernels, and the target format together.
// The generator reads it at compile time and emits scene_WGSL into ./main.typegpu.
// TypeGPU builds the equivalent shader at run time from the same functions.
export const scene: RenderPipelineSpec = renderPipelineL<SceneLayout, Vertex, Varyings>(
  sceneVertex,
  sceneFragment,
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
  if (format !== scene_TARGET_FORMAT) {
    print(`FAIL format expected=${scene_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The host owns the device and the queue. This wrapper borrows both and never destroys them.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // The vertex buffer holds three vertices. Vertex_STRIDE is the generated byte stride,
  // so the size never repeats a number counted by hand.
  const vertices = hostDevice.createBuffer({
    label: "ray-marching-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // The uniform buffer holds one FrameData. FrameData_SIZE comes from the layout generator,
  // so the host size and the WGSL struct size always agree.
  const frameBuffer = hostDevice.createBuffer({
    label: "ray-marching-frame",
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
    scene_WGSL,
    scene_VERTEX_ENTRY,
    scene_FRAGMENT_ENTRY,
    [scene_LAYOUT0],
    [scene_VERTEX_LAYOUT0],
    scene,
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
  // The resource list follows the field order of SceneLayout. One uniform buffer fills binding 0.
  const group = createBindGroupHost(
    hostDevice,
    bindLayout,
    scene_LAYOUT0,
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
// because the upstream example carries no controls either.
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
  // The attachment clears to a near-black blue. The triangle covers every pixel, so the
  // clear color only shows when the draw fails.
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.01, g: 0.015, b: 0.04, a: 1.0 },
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
