// example: perlin-noise
// Animates a two-dimensional slice through the noise module's perlin3d field.
// This port commits grid, depth, and sharpness and omits the upstream gradient cache,
// because perlin3d reads committed tables directly.
// Ported from TypeGPU's perlin-noise example (https://github.com/software-mansion/TypeGPU).

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
  sign,
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
  noise_FRAGMENT_ENTRY,
  noise_LAYOUT0,
  noise_TARGET_FORMAT,
  noise_VERTEX_ENTRY,
  noise_VERTEX_LAYOUT0,
  noise_WGSL,
} from "./main.typegpu";

// The vertex schema. The generator lays it out for the vertex buffer and emits
// `Vertex_STRIDE`, the byte distance between two vertices.
@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// The uniform record. `time` counts seconds, and the generator emits `FrameData_SIZE`
// for the buffer.
@CStruct
class FrameData {
  time: f32;

  constructor(time: f32) {
    this.time = time;
  }
}

// The record that travels from the vertex stage to the fragment stage. The `position`
// field carries the clip-space position builtin, and `uv` takes location 0.
@CStruct
class Varyings {
  position: Vec4f;
  uv: Vec2f;

  constructor(position: Vec4f, uv: Vec2f) {
    this.position = position;
    this.uv = uv;
  }
}

// The bind group of the pipeline: one uniform buffer. TypeGPU declares the same binding
// with `root.createUniform`, and the generator emits `noise_LAYOUT0` from this class.
class NoiseLayout {
  frame!: Uniform<FrameData>;
}

// GRID is the noise cells across the surface, DEPTH is the loop period in seconds, and
// SHARPNESS is the contrast exponent. TypeGPU drives all three from page controls.
const GRID: f32 = 4.0;
const DEPTH: f32 = 10.0;
const SHARPNESS: f32 = 0.5;

// The vertex kernel. It raises the position into clip space and maps the clip range to the
// 0-to-1 uv range that the fragment kernel samples.
function noiseVertex(res: NoiseLayout, value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

// The fragment kernel. It samples one z slice of the three-dimensional field per pixel, so
// the still image animates as time advances the slice.
function noiseFragment(
  res: NoiseLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const time: f32 = res.frame.$.time;
  // Bound time to DEPTH so the sampled z coordinate does not grow without limit.
  const depthCycle: f32 = new Vec2f(time / DEPTH, time / DEPTH).floor().x;
  const tz: f32 = time - DEPTH * depthCycle;
  const n: f32 = perlin3d(new Vec3f(input.uv.x * GRID, input.uv.y * GRID, tz));
  // `abs` and `pow` exist on vectors only, so the scalar travels through a two-component
  // vector and comes back as its x component.
  const magnitude: f32 = new Vec2f(n, n).abs().pow(new Vec2f(
    1.0 - SHARPNESS,
    1.0 - SHARPNESS,
  )).x;
  // Signed-power sharpening, the upstream exponential curve.
  const n2: f32 = sign(n) * magnitude;
  // Perlin output spans -1 to 1, so the shift maps it to the 0-to-1 range the gradient takes.
  const n01: f32 = n2 * 0.5 + 0.5;
  const color = new Vec3f(0.0, 0.2, 1.0).mix(new Vec3f(1.0, 0.3, 0.5), n01);
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

// The declaration pairs the two kernels with the layout class, so both stages reach the
// uniform. The generator reads it and emits the WGSL, the layout, and the vertex layout.
export const noise: RenderPipelineSpec = renderPipelineL<NoiseLayout, Vertex, Varyings>(
  noiseVertex,
  noiseFragment,
  { format: "bgra8unorm" },
);

// The host calls `init`, `frame`, and `shutdown` at different times, so the handles live
// at module scope. The script owns the pipeline, both buffers, and the bind group.
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
  // The declaration fixes the target format literally. A surface with another format ends
  // the example before it creates any resource.
  if (format !== noise_TARGET_FORMAT) {
    print(`FAIL format expected=${noise_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The host owns the device and the instance. The wrapper adds the API-layer surface and
  // disposes neither.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // `Vertex_STRIDE` is a generated constant, so the size needs no hand arithmetic. VERTEX
  // admits the vertex buffer slot, and COPY_DST admits the queue write.
  const vertices = hostDevice.createBuffer({
    label: "perlin-noise-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // `FrameData_SIZE` is a generated constant, so the uniform size follows the schema layout.
  // UNIFORM binds the buffer to the fragment stage, and COPY_DST admits the per-frame write.
  const frameBuffer = hostDevice.createBuffer({
    label: "perlin-noise-frame",
    size: FrameData_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  // Two queue writes seed both buffers. The three vertices form one oversized triangle whose
  // 3.0 coordinates fall outside the clip volume, so it covers the whole surface.
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>([
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(3.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 3.0)),
  ]));
  // The uniform starts at time zero, so the first frame reads a defined value.
  queue.writeBuffer(frameBuffer, 0, Context.bytesOf<FrameData>(new FrameData(0.0)));
  // The generated entry names, the generated layouts, and the declaration build the pipeline.
  // The scope catches a validation error from that call.
  hostDevice.pushErrorScope("validation");
  const pipeline = createRenderPipelineHost(
    hostDevice,
    noise_WGSL,
    noise_VERTEX_ENTRY,
    noise_FRAGMENT_ENTRY,
    [noise_LAYOUT0],
    [noise_VERTEX_LAYOUT0],
    noise,
  );
  // The host-owned device pumps the event loop itself, so the scope result arrives without
  // an await.
  const validationError = hostDevice.popErrorScope();
  // All three handles belong to the script here, so it releases them before it reports the
  // failure and leaves the module state empty.
  if (validationError !== null) {
    pipeline.dispose();
    frameBuffer.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  // The bind group joins the uniform buffer to the generated layout. It survives every frame,
  // because the buffer handle never changes.
  using bindLayout = pipeline.bindGroupLayout(0);
  const group = createBindGroupHost(
    hostDevice,
    bindLayout,
    noise_LAYOUT0,
    [bufferResource(frameBuffer)],
  );
  // The module state takes the handles only after validation passes, so `frame` never sees
  // a half-built pipeline.
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
  // A failed `init` leaves the module state empty. The layers carry no exceptions, so a null
  // check ends the frame where TypeGPU throws.
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
  // The host reports no clock, so the frame count divided by 60 stands for seconds on a
  // 60 Hz surface. TypeGPU reads the animation timestamp instead.
  frameCount += 1;
  // The queue keeps its work in the order it receives it, so this write lands before the
  // submitted pass reads the uniform.
  using queue = device.queue();
  queue.writeBuffer(
    frameBuffer,
    0,
    Context.bytesOf<FrameData>(new FrameData((frameCount as f32) / 60.0)),
  );
  // The host acquires and presents the surface texture. The wrapper borrows the view for one
  // frame, and the script disposes neither.
  const target = new GPUTextureView(view);
  // One encoder per frame records the render pass. The triangle covers every pixel, so the
  // clear color never stays on the surface.
  using encoder = device.createCommandEncoderDefault();
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  // The window size changes between frames, so the pass takes the viewport and the scissor
  // from the size the host reports.
  pass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  pass.setScissorRect(0, 0, width, height);
  // `bind` sets the pipeline, the bind groups, and each vertex buffer at its full size. Group
  // 0 is the uniform the fragment kernel reads.
  pipeline.bind(pass, [group], [vertices]);
  pass.draw(3);
  pass.end();
  // The GPU runs nothing until the queue receives the command buffer. The host presents the
  // surface after this call returns.
  using command = encoder.finishDefault();
  queue.submit([command]);
}

// The host calls this one time before it releases the device. TypeGPU frees the same kind of
// resource through `root.destroy`, and this layer disposes each handle by name.
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
