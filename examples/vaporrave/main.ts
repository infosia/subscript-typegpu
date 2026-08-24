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

class VaporLayout {
  frame!: Uniform<FrameData>;
}

function vaporDistance(point: Vec3f, time: f32): f32 {
  const spherePoint = point.sub(new Vec3f(0.0, 1.05, 0.0));
  // Noise added to the sphere distance warps the surface. The amplitude stays small,
  // so the marched field stays close to a true distance.
  const noise: f32 = perlin3d(spherePoint.scale(2.4).add(new Vec3f(0.0, time * 0.35, 0.0)));
  const sphere: f32 = sdSphere(spherePoint, 0.82) + noise * 0.07;
  const floor: f32 = sdPlane(point, new Vec3f(0.0, 1.0, 0.0), 0.0);
  return opUnion(sphere, floor);
}

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
    const cell: Vec2f = point.xz().fract();
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

function vaporVertex(res: VaporLayout, value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

function vaporFragment(
  res: VaporLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const frame: FrameData = res.frame.get();
  const screen = new Vec2f(
    (input.uv.x * 2.0 - 1.0) * frame.aspect,
    input.uv.y * 2.0 - 1.0,
  );
  const origin = new Vec3f(0.0, 1.45, 4.4);
  const direction = new Vec3f(screen.x, screen.y - 0.18, -1.75).normalize();
  let travel: f32 = 0.0;
  let hit: boolean = false;
  for (let stepIndex: u32 = 0; stepIndex < 80; stepIndex += 1) {
    const field: f32 = vaporDistance(origin.add(direction.scale(travel)), frame.time);
    if (field < 0.0015) {
      hit = true;
      break;
    }
    travel += field;
    if (travel > 14.0) break;
  }
  if (!hit) {
    const sky: f32 = input.uv.y;
    return new Vec4f(0.03 + sky * 0.12, 0.005, 0.09 + sky * 0.2, 1.0);
  }
  const point: Vec3f = origin.add(direction.scale(travel));
  const normal: Vec3f = vaporNormal(point, frame.time);
  const color: Vec3f = retroSurface(point, normal, frame.time);
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

export const vapor: RenderPipelineSpec = renderPipelineL<VaporLayout, Vertex, Varyings>(
  vaporVertex,
  vaporFragment,
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
  if (format !== vapor_TARGET_FORMAT) {
    print(`FAIL format expected=${vapor_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const vertices = hostDevice.createBuffer({
    label: "vaporrave-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  const frameBuffer = hostDevice.createBuffer({
    label: "vaporrave-frame",
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
    vapor_WGSL,
    vapor_VERTEX_ENTRY,
    vapor_FRAGMENT_ENTRY,
    [vapor_LAYOUT0],
    [vapor_VERTEX_LAYOUT0],
    vapor,
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
    vapor_LAYOUT0,
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
      clearValue: { r: 0.02, g: 0.0, b: 0.07, a: 1.0 },
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
