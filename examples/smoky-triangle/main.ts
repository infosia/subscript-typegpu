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

@CStruct
class Varyings {
  position: Vec4f;
  uv: Vec2f;

  constructor(position: Vec4f, uv: Vec2f) {
    this.position = position;
    this.uv = uv;
  }
}

class SmokeLayout {
  frame!: Uniform<FrameData>;
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

function smokeVertex(res: SmokeLayout, value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    value.uv,
  );
}

function smokeFragment(
  res: SmokeLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const frame: FrameData = res.frame.get();
  const field: f32 = smokeField(input.uv, frame.motion.x) * frame.motion.y;
  let smoke: f32 = 0.5 + field;
  if (smoke < 0.0) smoke = 0.0;
  if (smoke > 1.0) smoke = 1.0;
  const low = new Vec3f(0.08, 0.025, 0.12);
  const high = new Vec3f(0.95, 0.42, 0.2);
  const color: Vec3f = low.add(high.sub(low).scale(smoke));
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

export const smoke: RenderPipelineSpec = renderPipelineL<SmokeLayout, Vertex, Varyings>(
  smokeVertex,
  smokeFragment,
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
  if (format !== smoke_TARGET_FORMAT) {
    print(`FAIL format expected=${smoke_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const vertices = hostDevice.createBuffer({
    label: "smoky-triangle-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  const frameBuffer = hostDevice.createBuffer({
    label: "smoky-triangle-frame",
    size: FrameData_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>([
    new Vertex(new Vec2f(-0.78, -0.68), new Vec2f(0.0, 0.0)),
    new Vertex(new Vec2f(0.78, -0.68), new Vec2f(1.0, 0.0)),
    new Vertex(new Vec2f(0.0, 0.78), new Vec2f(0.5, 1.0)),
  ]));
  queue.writeBuffer(frameBuffer, 0, Context.bytesOf<FrameData>(new FrameData(0.0, 0.82)));
  hostDevice.pushErrorScope("validation");
  const pipeline = createRenderPipelineHost(
    hostDevice,
    smoke_WGSL,
    smoke_VERTEX_ENTRY,
    smoke_FRAGMENT_ENTRY,
    [smoke_LAYOUT0],
    [smoke_VERTEX_LAYOUT0],
    smoke,
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
    smoke_LAYOUT0,
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
    Context.bytesOf<FrameData>(new FrameData((frameCount as f32) / 60.0, 0.82)),
  );
  const target = new GPUTextureView(view);
  using encoder = device.createCommandEncoderDefault();
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.018, g: 0.008, b: 0.035, a: 1.0 },
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
