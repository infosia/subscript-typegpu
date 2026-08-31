// example: trippy-raymarching
// Raymarches a twisted, infinitely repeated sphere lattice inside a tunnel.
// This port commits the upstream slider defaults and reduces pointer input to the
// window host's surface coordinates.
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
  pointer: Vec2f;

  constructor(time: f32, aspect: f32, pointer: Vec2f) {
    this.time = time;
    this.aspect = aspect;
    this.pointer = pointer;
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

class TrippyLayout {
  frame!: Uniform<FrameData>;
}

const SPHERE_SPACING: f32 = 4.0;
const MAX_DISTANCE: f32 = 60.0;
const MAX_STEPS: u32 = 96;
const SPHERE_RADIUS: f32 = 0.5;
const WAVE_FREQUENCY_X: f32 = 2.0;
const WAVE_FREQUENCY_Y: f32 = 2.0;
const WAVE_AMPLITUDE: f32 = 0.2;
const TWIST_FACTOR: f32 = 0.01;
const TUNNEL_RADIUS: f32 = 0.697;

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
  const tunnelDistance: f32 = TUNNEL_RADIUS - twisted.xy.length();
  if (sphereDistance > tunnelDistance) return sphereDistance;
  return tunnelDistance;
}

function trippyNormal(point: Vec3f, time: f32, pointer: Vec2f): Vec3f {
  const epsilon: f32 = 0.001;
  const center: f32 = trippyDistance(point, time, pointer);
  return new Vec3f(
    trippyDistance(point.add(new Vec3f(epsilon, 0.0, 0.0)), time, pointer) - center,
    trippyDistance(point.add(new Vec3f(0.0, epsilon, 0.0)), time, pointer) - center,
    trippyDistance(point.add(new Vec3f(0.0, 0.0, epsilon)), time, pointer) - center,
  ).normalize();
}

function trippyVertex(res: TrippyLayout, value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

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
  if (!hit) {
    const background = new Vec3f(0.008, 0.003, 0.025).scale(glow);
    return new Vec4f(background.x, background.y, background.z, 1.0);
  }
  const point: Vec3f = origin.add(direction.scale(travel));
  const normal: Vec3f = trippyNormal(point, frame.time, frame.pointer);
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

export const trippy: RenderPipelineSpec = renderPipelineL<TrippyLayout, Vertex, Varyings>(
  trippyVertex,
  trippyFragment,
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
  if (format !== trippy_TARGET_FORMAT) {
    print(`FAIL format expected=${trippy_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const vertices = hostDevice.createBuffer({
    label: "trippy-raymarching-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  const frameBuffer = hostDevice.createBuffer({
    label: "trippy-raymarching-frame",
    size: FrameData_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>([
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(3.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 3.0)),
  ]));
  queue.writeBuffer(
    frameBuffer,
    0,
    Context.bytesOf<FrameData>(new FrameData(0.0, 1.0, new Vec2f(0.0, 0.0))),
  );
  hostDevice.pushErrorScope("validation");
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
    trippy_LAYOUT0,
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
  // Map the pointer to [-1, 1] per axis. (0, 0) stands for a pointer outside.
  let pointer = new Vec2f(0.0, 0.0);
  if (pointerX >= 0.0 && pointerY >= 0.0) {
    pointer = new Vec2f(
      pointerX / (width as f32) * 2.0 - 1.0,
      pointerY / (height as f32) * 2.0 - 1.0,
    );
  }
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
  const target = new GPUTextureView(view);
  using encoder = device.createCommandEncoderDefault();
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
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
  if (activeGroup !== null) activeGroup.dispose();
  if (activeFrameBuffer !== null) activeFrameBuffer.dispose();
  if (activeVertices !== null) activeVertices.dispose();
  if (activePipeline !== null) activePipeline.dispose();
  activeFrameBuffer = null;
  activeVertices = null;
  activePipeline = null;
  activeGroup = null;
  activeDevice = null;
  frameCount = 0;
}
