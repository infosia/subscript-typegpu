// example: disco
// Cycles three animated fullscreen patterns from one shared palette.
// Keys 1, 2, and 3 select rings, swirl, and kaleidoscope.
// TypeGPU ships seven selectable patterns. This port commits three.
// Ported from TypeGPU's disco example (https://github.com/software-mansion/TypeGPU).

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
  smoothstep,
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
  kaleidoscope_FRAGMENT_ENTRY,
  kaleidoscope_LAYOUT0,
  kaleidoscope_TARGET_FORMAT,
  kaleidoscope_VERTEX_ENTRY,
  kaleidoscope_VERTEX_LAYOUT0,
  kaleidoscope_WGSL,
  rings_FRAGMENT_ENTRY,
  rings_LAYOUT0,
  rings_TARGET_FORMAT,
  rings_VERTEX_ENTRY,
  rings_VERTEX_LAYOUT0,
  rings_WGSL,
  swirl_FRAGMENT_ENTRY,
  swirl_LAYOUT0,
  swirl_TARGET_FORMAT,
  swirl_VERTEX_ENTRY,
  swirl_VERTEX_LAYOUT0,
  swirl_WGSL,
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
  resolution: Vec2f;

  constructor(time: f32, resolution: Vec2f) {
    this.time = time;
    this.resolution = resolution;
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

class DiscoLayout {
  frame!: Uniform<FrameData>;
}

function cosinePalette(a: Vec3f, b: Vec3f, c: Vec3f, d: Vec3f, value: f32): Vec3f {
  return a.add(b.mul(c.scale(value).add(d).scale(6.28318).cos()));
}

function rotate(point: Vec2f, angle: f32): Vec2f {
  const phase = new Vec2f(angle, angle);
  const cosine: f32 = phase.cos().x;
  const sine: f32 = phase.sin().x;
  return new Vec2f(
    point.x * cosine - point.y * sine,
    point.x * sine + point.y * cosine,
  );
}

function absolute(value: f32): f32 {
  if (value < 0.0) return -value;
  return value;
}

function discoUv(input: Varyings, resolution: Vec2f): Vec2f {
  return new Vec2f(
    (input.uv.x * 2.0 - 1.0) * resolution.x / resolution.y,
    input.uv.y * 2.0 - 1.0,
  );
}

function discoVertex(res: DiscoLayout, value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

// Rings: rotated, scaled cells with a pulsed ring radius per iteration.
function ringsFragment(
  res: DiscoLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const frame: FrameData = res.frame.$;
  let point: Vec2f = discoUv(input, frame.resolution);
  let glow: f32 = 0.0;
  for (let iteration: u32 = 0; iteration < 4; iteration += 1) {
    point = rotate(point, frame.time * 0.08 + (iteration as f32) * 0.38);
    const cell: Vec2f = point.scale(1.45).fract().sub(new Vec2f(0.5, 0.5));
    const pulse: f32 = new Vec2f(
      frame.time * 1.4 + (iteration as f32),
      frame.time * 1.4 + (iteration as f32),
    ).sin().x;
    const edge: f32 = absolute(cell.length() - (0.24 + pulse * 0.035));
    glow += smoothstep(0.075, 0.005, edge);
    point = point.scale(1.18).add(new Vec2f(0.13, -0.09));
  }
  const color = cosinePalette(
    new Vec3f(0.48, 0.48, 0.48),
    new Vec3f(0.52, 0.52, 0.52),
    new Vec3f(1.0, 1.0, 1.0),
    new Vec3f(0.0, 0.22, 0.48),
    glow * 0.16 + frame.time * 0.04,
  ).scale(0.12 + glow * 0.46);
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

// Swirl: a radius-driven rotation feeds a horizontal wave per iteration.
function swirlFragment(
  res: DiscoLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const frame: FrameData = res.frame.$;
  let point: Vec2f = discoUv(input, frame.resolution);
  let glow: f32 = 0.0;
  for (let iteration: u32 = 0; iteration < 5; iteration += 1) {
    const radius: f32 = point.length();
    point = rotate(
      point,
      radius * 1.3 - frame.time * 0.18 + (iteration as f32) * 0.46,
    ).scale(1.3).fract().sub(new Vec2f(0.5, 0.5));
    const wave: f32 = new Vec2f(
      point.x * 7.0 + frame.time * 1.7,
      point.x * 7.0 + frame.time * 1.7,
    ).sin().x;
    glow += smoothstep(0.095, 0.01, absolute(point.y + wave * 0.14));
  }
  const color = cosinePalette(
    new Vec3f(0.5, 0.45, 0.5),
    new Vec3f(0.5, 0.48, 0.5),
    new Vec3f(1.0, 0.8, 1.2),
    new Vec3f(0.62, 0.08, 0.3),
    glow * 0.12 - frame.time * 0.05,
  ).scale(0.1 + glow * 0.34);
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

// Kaleidoscope: mirrored folds cut the plane into diagonal cells.
function kaleidoscopeFragment(
  res: DiscoLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const frame: FrameData = res.frame.$;
  let point: Vec2f = discoUv(input, frame.resolution);
  let glow: f32 = 0.0;
  for (let iteration: u32 = 0; iteration < 3; iteration += 1) {
    point = rotate(
      point.abs(),
      0.74 + frame.time * 0.12 + (iteration as f32) * 0.63,
    ).scale(1.72).fract().sub(new Vec2f(0.5, 0.5));
    const diagonal: f32 = absolute(absolute(point.x) - absolute(point.y));
    glow += smoothstep(0.12, 0.008, diagonal);
  }
  const color = cosinePalette(
    new Vec3f(0.46, 0.5, 0.52),
    new Vec3f(0.54, 0.48, 0.46),
    new Vec3f(1.0, 1.3, 0.8),
    new Vec3f(0.12, 0.44, 0.72),
    glow * 0.18 + frame.time * 0.035,
  ).scale(0.1 + glow * 0.42);
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

export const rings: RenderPipelineSpec = renderPipelineL<DiscoLayout, Vertex, Varyings>(
  discoVertex,
  ringsFragment,
  { format: "bgra8unorm" },
);

export const swirl: RenderPipelineSpec = renderPipelineL<DiscoLayout, Vertex, Varyings>(
  discoVertex,
  swirlFragment,
  { format: "bgra8unorm" },
);

export const kaleidoscope: RenderPipelineSpec = renderPipelineL<DiscoLayout, Vertex, Varyings>(
  discoVertex,
  kaleidoscopeFragment,
  { format: "bgra8unorm" },
);

let activeDevice: GPUHostOwnedDevice | null = null;
let activeRings: RenderPipeline | null = null;
let activeSwirl: RenderPipeline | null = null;
let activeKaleidoscope: RenderPipeline | null = null;
let activeVertices: GPUBuffer | null = null;
let activeFrameBuffer: GPUBuffer | null = null;
let activeGroup: GPUBindGroup | null = null;
let frameCount: u32 = 0;
let patternIndex: u32 = 0;

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  if (
    format !== rings_TARGET_FORMAT
    || format !== swirl_TARGET_FORMAT
    || format !== kaleidoscope_TARGET_FORMAT
  ) {
    print(`FAIL format expected=${rings_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const vertices = hostDevice.createBuffer({
    label: "disco-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  const frameBuffer = hostDevice.createBuffer({
    label: "disco-frame",
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
    Context.bytesOf<FrameData>(new FrameData(0.0, new Vec2f(1.0, 1.0))),
  );
  hostDevice.pushErrorScope("validation");
  const ringsPipeline = createRenderPipelineHost(
    hostDevice,
    rings_WGSL,
    rings_VERTEX_ENTRY,
    rings_FRAGMENT_ENTRY,
    [rings_LAYOUT0],
    [rings_VERTEX_LAYOUT0],
    rings,
  );
  const swirlPipeline = createRenderPipelineHost(
    hostDevice,
    swirl_WGSL,
    swirl_VERTEX_ENTRY,
    swirl_FRAGMENT_ENTRY,
    [swirl_LAYOUT0],
    [swirl_VERTEX_LAYOUT0],
    swirl,
  );
  const kaleidoscopePipeline = createRenderPipelineHost(
    hostDevice,
    kaleidoscope_WGSL,
    kaleidoscope_VERTEX_ENTRY,
    kaleidoscope_FRAGMENT_ENTRY,
    [kaleidoscope_LAYOUT0],
    [kaleidoscope_VERTEX_LAYOUT0],
    kaleidoscope,
  );
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    kaleidoscopePipeline.dispose();
    swirlPipeline.dispose();
    ringsPipeline.dispose();
    frameBuffer.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  using bindLayout = ringsPipeline.bindGroupLayout(0);
  const group = createBindGroupHost(
    hostDevice,
    bindLayout,
    rings_LAYOUT0,
    [bufferResource(frameBuffer)],
  );
  activeDevice = hostDevice;
  activeRings = ringsPipeline;
  activeSwirl = swirlPipeline;
  activeKaleidoscope = kaleidoscopePipeline;
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
  const vertices = activeVertices;
  const frameBuffer = activeFrameBuffer;
  const group = activeGroup;
  if (key === 49) patternIndex = 0;
  if (key === 50) patternIndex = 1;
  if (key === 51) patternIndex = 2;
  let pipeline = activeRings;
  if (patternIndex === 1) pipeline = activeSwirl;
  if (patternIndex === 2) pipeline = activeKaleidoscope;
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
      new Vec2f(width as f32, height as f32),
    )),
  );
  const target = new GPUTextureView(view);
  using encoder = device.createCommandEncoderDefault();
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.005, g: 0.005, b: 0.012, a: 1.0 },
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
  if (activeKaleidoscope !== null) activeKaleidoscope.dispose();
  if (activeSwirl !== null) activeSwirl.dispose();
  if (activeRings !== null) activeRings.dispose();
  activeFrameBuffer = null;
  activeVertices = null;
  activeKaleidoscope = null;
  activeSwirl = null;
  activeRings = null;
  activeGroup = null;
  activeDevice = null;
  frameCount = 0;
  patternIndex = 0;
}
