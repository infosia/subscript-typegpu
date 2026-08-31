// example: oklab
// Renders one hue slice of the Oklab color solid with adaptive gamut clipping.
// The upstream fwidth projection lines reduce to an out-of-gamut checker because the
// kernel subset has no derivatives. Keys 1 and 2 step hue for each key press.
// The upstream CSS probe dot reduces to a bright fragment-shader ring.
// Ported from TypeGPU's oklab example (https://github.com/software-mansion/TypeGPU).

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
  linearToSrgb,
  oklabGamutClipAdaptiveL05,
  oklabToLinearRgb,
} from "./typegpu-color";
import {
  GPUBindGroup,
  GPUBuffer,
  GPUBufferUsage,
  GPUHostOwnedDevice,
  GPUTextureView,
  hostOwnedGPUDevice,
} from "./webgpu";
import {
  OklabUniforms_SIZE,
  Vertex_STRIDE,
  oklab_FRAGMENT_ENTRY,
  oklab_LAYOUT0,
  oklab_TARGET_FORMAT,
  oklab_VERTEX_ENTRY,
  oklab_VERTEX_LAYOUT0,
  oklab_WGSL,
} from "./main.typegpu";

@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

@CStruct
class OklabUniforms {
  hue: f32;
  alpha: f32;
  pointer: Vec2f;

  constructor(hue: f32, alpha: f32, pointer: Vec2f) {
    this.hue = hue;
    this.alpha = alpha;
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

class OklabLayout {
  uniforms!: Uniform<OklabUniforms>;
}

const GAMUT_CLIP_ALPHA: f32 = 0.05;

function oklabVertex(res: OklabLayout, value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

function oklabFragment(
  res: OklabLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const uniforms: OklabUniforms = res.uniforms.$;
  const x: f32 = input.uv.x * 2.0 - 1.0;
  const y: f32 = input.uv.y * 2.0 - 1.0;
  const position = new Vec2f(0.3 * x, (y * 1.2 + 1.0) * 0.5);
  const hueVector = new Vec2f(uniforms.hue, uniforms.hue);
  // Lightness rises with y; negative x shows the opposite hue.
  const lab = new Vec3f(
    position.y,
    hueVector.cos().x * position.x,
    hueVector.sin().x * position.x,
  );
  const linear: Vec3f = oklabToLinearRgb(lab);
  const outOfGamut: boolean = linear.x < 0.0 || linear.x > 1.0
    || linear.y < 0.0 || linear.y > 1.0
    || linear.z < 0.0 || linear.z > 1.0;
  let color: Vec3f = linearToSrgb(
    oklabToLinearRgb(oklabGamutClipAdaptiveL05(lab, uniforms.alpha)),
  );
  // Out-of-gamut regions keep the clipped color, dimmed by the checker.
  if (outOfGamut) {
    const cell = input.uv.scale(24.0).floor();
    const checker: f32 = (((cell.x as i32) + (cell.y as i32)) & 1) === 0 ? 0.0 : 1.0;
    color = color.scale(0.1 + 0.9 * checker);
  }
  // A thin bright ring marks the pointer probe.
  const probeDistance: f32 = input.uv.distance(uniforms.pointer);
  if (probeDistance > 0.018 && probeDistance < 0.026) {
    color = color.mix(new Vec3f(1.0, 1.0, 1.0), 0.8);
  }
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

export const oklab: RenderPipelineSpec = renderPipelineL<OklabLayout, Vertex, Varyings>(
  oklabVertex,
  oklabFragment,
  { format: "bgra8unorm" },
);

let activeDevice: GPUHostOwnedDevice | null = null;
let activePipeline: RenderPipeline | null = null;
let activeVertices: GPUBuffer | null = null;
let activeUniforms: GPUBuffer | null = null;
let activeGroup: GPUBindGroup | null = null;
let hue: f32 = 0.0;

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  if (format !== oklab_TARGET_FORMAT) {
    print(`FAIL format expected=${oklab_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const vertices = hostDevice.createBuffer({
    label: "oklab-fullscreen",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  const uniforms = hostDevice.createBuffer({
    label: "oklab-uniforms",
    size: OklabUniforms_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>([
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(3.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 3.0)),
  ]));
  queue.writeBuffer(
    uniforms,
    0,
    Context.bytesOf<OklabUniforms>(new OklabUniforms(
      hue,
      GAMUT_CLIP_ALPHA,
      new Vec2f(-10.0, -10.0),
    )),
  );
  hostDevice.pushErrorScope("validation");
  const pipeline = createRenderPipelineHost(
    hostDevice,
    oklab_WGSL,
    oklab_VERTEX_ENTRY,
    oklab_FRAGMENT_ENTRY,
    [oklab_LAYOUT0],
    [oklab_VERTEX_LAYOUT0],
    oklab,
  );
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    pipeline.dispose();
    uniforms.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  using bindLayout = pipeline.bindGroupLayout(0);
  const group = createBindGroupHost(
    hostDevice,
    bindLayout,
    oklab_LAYOUT0,
    [bufferResource(uniforms)],
  );
  activeDevice = hostDevice;
  activePipeline = pipeline;
  activeVertices = vertices;
  activeUniforms = uniforms;
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
  const uniforms = activeUniforms;
  const group = activeGroup;
  if (device === null) return;
  if (pipeline === null) return;
  if (vertices === null) return;
  if (uniforms === null) return;
  if (group === null) return;
  if (key === 49) hue -= 0.1;
  if (key === 50) hue += 0.1;
  let pointer = new Vec2f(-10.0, -10.0);
  if (pointerX >= 0.0 && pointerY >= 0.0) {
    pointer = new Vec2f(pointerX / (width as f32), 1.0 - pointerY / (height as f32));
  }
  using queue = device.queue();
  queue.writeBuffer(
    uniforms,
    0,
    Context.bytesOf<OklabUniforms>(new OklabUniforms(hue, GAMUT_CLIP_ALPHA, pointer)),
  );
  const target = new GPUTextureView(view);
  using encoder = device.createCommandEncoderDefault();
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.015, g: 0.02, b: 0.03, a: 1.0 },
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
  if (activeUniforms !== null) activeUniforms.dispose();
  if (activeVertices !== null) activeVertices.dispose();
  if (activePipeline !== null) activePipeline.dispose();
  activeUniforms = null;
  activeVertices = null;
  activePipeline = null;
  activeGroup = null;
  activeDevice = null;
}
