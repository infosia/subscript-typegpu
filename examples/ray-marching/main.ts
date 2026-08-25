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

@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

@CStruct
class FrameData {
  time: f32;
  aspect: f32;

  constructor(time: f32, aspect: f32) {
    this.time = time;
    this.aspect = aspect;
  }
}

@CStruct
class Varyings {
  position: Vec4f;
  uv: Vec2f;

  constructor(position: Vec4f, uv: Vec2f) {
    this.position = position;
    this.uv = uv;
  }
}

class SceneLayout {
  frame!: Uniform<FrameData>;
}

function sceneDistance(point: Vec3f, time: f32): f32 {
  // The type library exposes sine on vectors only. One lane of a two-component sine
  // gives the scalar phase.
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
    const estimate: f32 = 10.0 * field / distance;
    if (estimate < visibility) visibility = estimate;
    if (field < 0.001 || distance > 7.0) break;
    distance += field;
  }
  if (visibility < 0.12) return 0.12;
  if (visibility > 1.0) return 1.0;
  return visibility;
}

function sceneVertex(res: SceneLayout, value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

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
  const origin = new Vec3f(0.0, 1.15, 4.2);
  const direction = new Vec3f(screen.x, screen.y - 0.12, -1.8).normalize();
  let travel: f32 = 0.0;
  let hit: boolean = false;
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
  if (!hit) {
    const horizon: f32 = input.uv.y * 0.2;
    return new Vec4f(0.025 + horizon, 0.035 + horizon, 0.08 + horizon, 1.0);
  }
  const point: Vec3f = origin.add(direction.scale(travel));
  const normal: Vec3f = sceneNormal(point, frame.time);
  const lightDirection = new Vec3f(-0.45, 0.8, 0.35).normalize();
  let diffuse: f32 = normal.dot(lightDirection);
  if (diffuse < 0.0) diffuse = 0.0;
  const shadow: f32 = softShadow(point.add(normal.scale(0.006)), lightDirection, frame.time);
  const light: f32 = 0.16 + diffuse * shadow * 0.84;
  const base = new Vec3f(0.24, 0.58, 0.78);
  return new Vec4f(base.x * light, base.y * light, base.z * light, 1.0);
}

export const scene: RenderPipelineSpec = renderPipelineL<SceneLayout, Vertex, Varyings>(
  sceneVertex,
  sceneFragment,
  { format: "bgra8unorm" },
);

let activeDevice: GPUHostOwnedDevice | null = null;
let activePipeline: RenderPipeline | null = null;
let activeVertices: GPUBuffer | null = null;
let activeFrameBuffer: GPUBuffer | null = null;
let activeGroup: GPUBindGroup | null = null;
let frameCount: u32 = 0;

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  if (format !== scene_TARGET_FORMAT) {
    print(`FAIL format expected=${scene_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const vertices = hostDevice.createBuffer({
    label: "ray-marching-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  const frameBuffer = hostDevice.createBuffer({
    label: "ray-marching-frame",
    size: FrameData_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>([
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(3.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 3.0)),
  ]));
  queue.writeBuffer(frameBuffer, 0, Context.bytesOf<FrameData>(new FrameData(0.0, 1.0)));
  hostDevice.pushErrorScope("validation");
  const pipeline = createRenderPipelineHost(
    hostDevice,
    scene_WGSL,
    scene_VERTEX_ENTRY,
    scene_FRAGMENT_ENTRY,
    [scene_LAYOUT0],
    [scene_VERTEX_LAYOUT0],
    scene,
  );
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    pipeline.dispose();
    frameBuffer.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  using bindLayout = pipeline.bindGroupLayout(0);
  const group = createBindGroupHost(
    hostDevice,
    bindLayout,
    scene_LAYOUT0,
    [bufferResource(frameBuffer)],
  );
  activeDevice = hostDevice;
  activePipeline = pipeline;
  activeVertices = vertices;
  activeFrameBuffer = frameBuffer;
  activeGroup = group;
}

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
  if (device === null) return;
  if (pipeline === null) return;
  if (vertices === null) return;
  if (frameBuffer === null) return;
  if (group === null) return;
  frameCount += 1;
  using queue = device.queue();
  queue.writeBuffer(
    frameBuffer,
    0,
    Context.bytesOf<FrameData>(new FrameData(
      (frameCount as f32) / 60.0,
      (width as f32) / (height as f32),
    )),
  );
  const target = new GPUTextureView(view);
  using encoder = device.createCommandEncoderDefault();
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.01, g: 0.015, b: 0.04, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  pass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  pass.setScissorRect(0, 0, width, height);
  pipeline.bind(pass, [group], [vertices]);
  pass.draw(3);
  pass.end();
  using command = encoder.finishDefault();
  queue.submit([command]);
}

export function shutdown(): void {
  if (activeFrameBuffer !== null) activeFrameBuffer.dispose();
  if (activeVertices !== null) activeVertices.dispose();
  if (activePipeline !== null) activePipeline.dispose();
  activeFrameBuffer = null;
  activeVertices = null;
  activePipeline = null;
  activeGroup = null;
  activeDevice = null;
}
