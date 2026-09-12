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

// One corner of the full-surface triangle in normalized device coordinates.
// TypeGPU takes the same triangle from a shared vertex helper.
@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// hue is an angle in radians. alpha is the gamut clip strength. pointer is the probe
// position in the 0 to 1 surface range. TypeGPU drives all three from HTML controls.
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
// binding numbers, and the generator emits the WGSL declaration and oklab_LAYOUT0.
class OklabLayout {
  uniforms: Uniform<OklabUniforms>;

  constructor(uniforms: Uniform<OklabUniforms>) {
    this.uniforms = uniforms;
  }
}

// A small alpha keeps the adaptive clip near the original lightness. TypeGPU offers the
// same number as a slider.
const GAMUT_CLIP_ALPHA: f32 = 0.05;

// The vertex stage widens the corner to clip space and maps it from the -1 to 1 range
// into the 0 to 1 uv range. It reads no binding.
function oklabVertex(res: OklabLayout, value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

// Per pixel the fragment stage builds one Oklab color. uv.y becomes the lightness and
// uv.x becomes the chroma along the hue direction. 0.3 and 1.2 frame the visible slice.
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
  // The hue angle gives the direction of the a and b axes. Lightness rises with y, and
  // negative x carries the opposite hue.
  const lab = new Vec3f(
    position.y,
    hueVector.cos().x * position.x,
    hueVector.sin().x * position.x,
  );
  // The unclipped conversion tells whether the color leaves the sRGB cube. The clipped
  // conversion below gives the color the surface really shows.
  const linear: Vec3f = oklabToLinearRgb(lab);
  const outOfGamut: boolean = linear.x < 0.0 || linear.x > 1.0
    || linear.y < 0.0 || linear.y > 1.0
    || linear.z < 0.0 || linear.z > 1.0;
  let color: Vec3f = linearToSrgb(
    oklabToLinearRgb(oklabGamutClipAdaptiveL05(lab, uniforms.alpha)),
  );
  // Out-of-gamut regions keep the clipped color and the checker dims every second cell.
  // TypeGPU picks this pattern through a slot at run time. The port compiles one pattern,
  // because the generator emits the WGSL before the program runs.
  if (outOfGamut) {
    const cell = input.uv.scale(24.0).floor();
    const checker: f32 = (((cell.x as i32) + (cell.y as i32)) & 1) === 0 ? 0.0 : 1.0;
    color = color.scale(0.1 + 0.9 * checker);
  }
  // A thin bright ring marks the pointer probe. TypeGPU moves an HTML element over the
  // canvas instead, which a windowed script cannot reach.
  const probeDistance: f32 = input.uv.distance(uniforms.pointer);
  if (probeDistance > 0.018 && probeDistance < 0.026) {
    color = color.mix(new Vec3f(1.0, 1.0, 1.0), 0.8);
  }
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

// The declaration ties the layout class, the two kernels, and the target format together.
// The generator reads it at compile time and emits oklab_WGSL into ./main.typegpu.
// TypeGPU rebuilds its pipeline whenever a control picks another clip or pattern.
export const oklab: RenderPipelineSpec = renderPipelineL<OklabLayout, Vertex, Varyings>(
  oklabVertex,
  oklabFragment,
  { format: "bgra8unorm" },
);

// The window host calls init, frame, and shutdown as separate entry points, so the
// handles live in module state. The script owns each one and frees it in shutdown.
let activeDevice: GPUHostOwnedDevice | null = null;
let activePipeline: RenderPipeline | null = null;
let activeVertices: GPUBuffer | null = null;
let activeUniforms: GPUBuffer | null = null;
let activeGroup: GPUBindGroup | null = null;
let hue: f32 = 0.0;

// The host calls init once, before the first frame, with the device it owns.
export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  // The host picks the surface format. The generator baked one format into the pipeline,
  // so a mismatch stops the example here instead of at pipeline creation.
  if (format !== oklab_TARGET_FORMAT) {
    print(`FAIL format expected=${oklab_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The host owns the device and the queue. This wrapper borrows both and never destroys them.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // The vertex buffer holds three vertices. Vertex_STRIDE is the generated byte stride,
  // so the size never repeats a number counted by hand.
  const vertices = hostDevice.createBuffer({
    label: "oklab-fullscreen",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // The uniform buffer holds one OklabUniforms. OklabUniforms_SIZE comes from the layout
  // generator, so the host size and the WGSL struct size always agree.
  const uniforms = hostDevice.createBuffer({
    label: "oklab-uniforms",
    size: OklabUniforms_SIZE as u64,
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
  // The uniform needs a valid value before the first draw. The pointer starts far outside
  // the 0 to 1 range, so the probe ring stays hidden until the pointer enters the window.
  queue.writeBuffer(
    uniforms,
    0,
    Context.bytesOf<OklabUniforms>(new OklabUniforms(
      hue,
      GAMUT_CLIP_ALPHA,
      new Vec2f(-10.0, -10.0),
    )),
  );
  // Pipeline creation raises no exception. The error scope turns a bad shader or a bad
  // layout into a value the code below tests.
  hostDevice.pushErrorScope("validation");
  // The generated WGSL, the entry names, and the layout constants build the pipeline.
  const pipeline = createRenderPipelineHost(
    hostDevice,
    oklab_WGSL,
    oklab_VERTEX_ENTRY,
    oklab_FRAGMENT_ENTRY,
    [oklab_LAYOUT0],
    [oklab_VERTEX_LAYOUT0],
    oklab,
  );
  // A null check replaces the exception a browser port throws. The failure path frees the
  // three handles this function already created.
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    pipeline.dispose();
    uniforms.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  // The layout comes back from the pipeline, so the shader and the bind group cannot
  // disagree about group 0.
  using bindLayout = pipeline.bindGroupLayout(0);
  // The resource list follows the field order of OklabLayout. One uniform buffer fills binding 0.
  const group = createBindGroupHost(
    hostDevice,
    bindLayout,
    oklab_LAYOUT0,
    [bufferResource(uniforms)],
  );
  // The handles reach module state only after every step succeeds, so frame never sees a
  // half-built set.
  activeDevice = hostDevice;
  activePipeline = pipeline;
  activeVertices = vertices;
  activeUniforms = uniforms;
  activeGroup = group;
}

// The host calls frame once per presented image. The key and the pointer replace the hue
// slider and the HTML probe of the upstream example.
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
  const uniforms = activeUniforms;
  const group = activeGroup;
  if (device === null) return;
  if (pipeline === null) return;
  if (vertices === null) return;
  if (uniforms === null) return;
  if (group === null) return;
  // The host stores the last key press as a Unicode scalar and clears the slot after this
  // call. 49 and 50 are the codes for 1 and 2, and each press turns the slice by 0.1 radians.
  if (key === 49) hue -= 0.1;
  if (key === 50) hue += 0.1;
  // The host reports -1, -1 until the pointer first enters the window. The division turns
  // surface pixels into the 0 to 1 range, and the flip matches the uv origin.
  let pointer = new Vec2f(-10.0, -10.0);
  if (pointerX >= 0.0 && pointerY >= 0.0) {
    pointer = new Vec2f(pointerX / (width as f32), 1.0 - pointerY / (height as f32));
  }
  using queue = device.queue();
  // The uniform update enters the queue before the submit below. The queue keeps that
  // order, so the draw reads this frame's hue and pointer.
  queue.writeBuffer(
    uniforms,
    0,
    Context.bytesOf<OklabUniforms>(new OklabUniforms(hue, GAMUT_CLIP_ALPHA, pointer)),
  );
  // The host passes the current surface view as a raw handle. The wrapper does not own it,
  // and the host presents the surface after frame returns.
  const target = new GPUTextureView(view);
  using encoder = device.createCommandEncoderDefault();
  // The attachment clears to a near-black gray. The triangle covers every pixel, so the
  // clear color only shows when the draw fails.
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.015, g: 0.02, b: 0.03, a: 1.0 },
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
  if (activeUniforms !== null) activeUniforms.dispose();
  if (activeVertices !== null) activeVertices.dispose();
  if (activePipeline !== null) activePipeline.dispose();
  activeUniforms = null;
  activeVertices = null;
  activePipeline = null;
  activeGroup = null;
  activeDevice = null;
}
