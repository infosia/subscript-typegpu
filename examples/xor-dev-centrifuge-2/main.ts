// example: xor-dev-centrifuge-2
// Animates a polar tunnel from layered trigonometric bands and the host frame count.
// The six upstream sliders become fixed constants inside the shader body.
// Ported from TypeGPU's xor-dev-centrifuge-2 example (https://github.com/software-mansion/TypeGPU).

import {
  FragmentInvocation,
  RenderPipeline,
  RenderPipelineSpec,
  Uniform,
  VertexInvocation,
  WgslShellSpec,
  createRenderPipelineHost,
  renderPipelineL,
  wgslShell,
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
  tunnel_FRAGMENT_ENTRY,
  tunnel_LAYOUT0,
  tunnel_TARGET_FORMAT,
  tunnel_VERTEX_ENTRY,
  tunnel_VERTEX_LAYOUT0,
  tunnel_WGSL,
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
// The other six values of the upstream struct are constants in the shell body below.
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

// TypeGPU declares an eight-field `Params` struct and creates the uniform at run time.
// This port sends time and the aspect ratio only. A bind group layout is a class here,
// and field order fixes the binding numbers.
class TunnelLayout {
  frame!: Uniform<FrameData>;
}

// A shell function has two bodies. This subscript body is the host implementation,
// and the CPU lane runs it.
function tunnelBands(point: Vec2f, time: f32): Vec3f {
  const radius: f32 = point.length();
  // The host body uses Math.atan2 because the typed kernel method table has no scalar mapping.
  const angle: f32 = Math.atan2(point.y as f64, point.x as f64) as f32;
  let color = new Vec3f(0.0, 0.0, 0.0);
  let layer: u32 = 0;
  while (layer < 12) {
    const depth: f32 = (layer as f32) * 0.21 + time * 0.35;
    const wave: f32 = Math.sin((radius * 11.0 - depth * 3.0) as f64) as f32;
    const ring: f32 = 0.045 / ((Math.abs(wave as f64) as f32) + 0.07);
    const spoke: f32 = 0.35
      + 0.65 * (Math.cos((angle * 7.0 + depth * 2.0) as f64) as f32);
    const visibleSpoke: f32 = Math.max(spoke as f64, 0.0) as f32;
    const fade: f32 = 1.0 / (1.0 + (layer as f32) * 0.32);
    color = color.add(new Vec3f(0.22, 0.05, 0.48).scale(ring * visibleSpoke * fade));
    layer += 1;
  }
  return new Vec3f(
    Math.tanh((color.x * 2.0) as f64) as f32,
    Math.tanh((color.y * 2.0) as f64) as f32,
    Math.tanh((color.z * 2.0) as f64) as f32,
  );
}

// K11 maps no `atan2`, so the ring angle needs a WGSL shell. TypeGPU marches a 3D
// ray instead. This shell accumulates twelve polar rings and tone-maps with `tanh`.
const tunnelBandsGpu: WgslShellSpec = wgslShell<(point: Vec2f, time: f32) => Vec3f>(
  tunnelBands,
  {
    body: "let radius = length(point); let angle = atan2(point.y, point.x); var color = vec3f(0.0); for (var layer = 0u; layer < 12u; layer = layer + 1u) { let depth = f32(layer) * 0.21 + time * 0.35; let ring = 0.045 / (abs(sin(radius * 11.0 - depth * 3.0)) + 0.07); let spoke = 0.35 + 0.65 * cos(angle * 7.0 + depth * 2.0); let visibleSpoke = max(spoke, 0.0); let fade = 1.0 / (1.0 + f32(layer) * 0.32); color = color + vec3f(0.22, 0.05, 0.48) * ring * visibleSpoke * fade; } return tanh(color * 2.0);",
  },
);

// The vertex stage widens the corner to clip space and maps it from the -1 to 1 range
// into the 0 to 1 uv range. It reads no binding.
function tunnelVertex(
  res: TunnelLayout,
  value: Vertex,
  ctx: VertexInvocation,
): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

// `res.frame.$` is a real accessor method, not the run-time proxy read TypeGPU builds.
// The port re-centers uv here, because the vertex stage wrote it into the 0 to 1 range.
function tunnelFragment(
  res: TunnelLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const frame: FrameData = res.frame.$;
  const centered = new Vec2f(
    (input.uv.x * 2.0 - 1.0) * frame.aspect,
    input.uv.y * 2.0 - 1.0,
  );
  // The call names the host function. The generator puts the shell body in its place, so
  // the emitted WGSL never contains the subscript body.
  const color: Vec3f = tunnelBands(centered, frame.time);
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

// The declaration ties the layout class, the two kernels, and the target format together.
// The generator reads it at compile time and emits tunnel_WGSL into ./main.typegpu.
export const tunnel: RenderPipelineSpec = renderPipelineL<TunnelLayout, Vertex, Varyings>(
  tunnelVertex,
  tunnelFragment,
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
  if (format !== tunnel_TARGET_FORMAT) {
    print(`FAIL format expected=${tunnel_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The host owns the device and the queue. This wrapper borrows both and never destroys them.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // One oversized triangle covers the whole surface. The parts outside the viewport clip
  // away, so this example needs no quad and no index buffer.
  const values: FixedArray<Vertex, 3> = [
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(3.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 3.0)),
  ];
  // The vertex buffer holds three vertices. Vertex_STRIDE is the generated byte stride,
  // so the size never repeats a number counted by hand.
  const vertices = hostDevice.createBuffer({
    label: "centrifuge-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // The uniform buffer holds one FrameData. FrameData_SIZE comes from the layout generator,
  // so the host size and the WGSL struct size always agree.
  const frameBuffer = hostDevice.createBuffer({
    label: "centrifuge-frame",
    size: FrameData_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  // A host-owned device returns a new queue wrapper from each call, so this block frees it.
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>(values));
  // The uniform needs a valid value before the first draw. The aspect ratio starts at 1.0,
  // and the first frame replaces it with the real window ratio.
  queue.writeBuffer(frameBuffer, 0, Context.bytesOf<FrameData>(new FrameData(0.0, 1.0)));
  // Pipeline creation raises no exception. The error scope turns a bad shader or a bad
  // layout into a value the code below tests.
  hostDevice.pushErrorScope("validation");
  // The generated WGSL, the entry names, and the layout constants build the pipeline.
  const createdPipeline = createRenderPipelineHost(
    hostDevice,
    tunnel_WGSL,
    tunnel_VERTEX_ENTRY,
    tunnel_FRAGMENT_ENTRY,
    [tunnel_LAYOUT0],
    [tunnel_VERTEX_LAYOUT0],
    tunnel,
  );
  // A null check replaces the exception a browser port throws. The failure path frees the
  // three handles this function already created.
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    createdPipeline.dispose();
    frameBuffer.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  // The layout comes back from the pipeline, so the shader and the bind group cannot
  // disagree about group 0.
  using bindLayout = createdPipeline.bindGroupLayout(0);
  // One binding needs no helper, so this call builds the entry directly. The binding number
  // still comes from the generated layout, never from a number written here.
  const group = hostDevice.createBindGroup({
    layout: bindLayout,
    entries: [{
      binding: tunnel_LAYOUT0.entries[0].binding,
      buffer: frameBuffer,
      size: FrameData_SIZE as u64,
    }],
  });
  // The handles reach module state only after every step succeeds, so frame never sees a
  // half-built set.
  activeDevice = hostDevice;
  activeVertices = vertices;
  activeFrameBuffer = frameBuffer;
  activeGroup = group;
  activePipeline = createdPipeline;
}

// The host calls frame once per presented image. This example reads no key and no pointer,
// because the port fixes every value the upstream sliders control.
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
  const device: GPUHostOwnedDevice | null = activeDevice;
  const pipeline: RenderPipeline | null = activePipeline;
  const vertices: GPUBuffer | null = activeVertices;
  const frameBuffer: GPUBuffer | null = activeFrameBuffer;
  const group: GPUBindGroup | null = activeGroup;
  if (device === null) {
    return;
  }
  if (pipeline === null) {
    return;
  }
  if (vertices === null) {
    return;
  }
  if (frameBuffer === null) {
    return;
  }
  if (group === null) {
    return;
  }
  // Time comes from the frame count divided by sixty. TypeGPU reads the
  // `requestAnimationFrame` timestamp instead.
  frameCount += 1;
  const aspect: f32 = (width as f32) / (height as f32);
  using queue = device.queue();
  // The uniform write enters the queue before the submit below. The queue keeps that order,
  // so the draw reads this frame's time.
  queue.writeBuffer(
    frameBuffer,
    0,
    Context.bytesOf<FrameData>(new FrameData(frameCount as f32 / 60.0, aspect)),
  );
  // The host passes the current surface view as a raw handle. The wrapper does not own it,
  // and the host presents the surface after frame returns.
  const target = new GPUTextureView(view);
  using encoder = device.createCommandEncoderDefault();
  // The attachment clears to black. The triangle covers every pixel, so the clear color only
  // shows when the draw fails.
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
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
  if (activeGroup !== null) {
    activeGroup.dispose();
    activeGroup = null;
  }
  if (activeFrameBuffer !== null) {
    activeFrameBuffer.dispose();
    activeFrameBuffer = null;
  }
  if (activeVertices !== null) {
    activeVertices.dispose();
    activeVertices = null;
  }
  if (activePipeline !== null) {
    activePipeline.dispose();
    activePipeline = null;
  }
  activeDevice = null;
}
