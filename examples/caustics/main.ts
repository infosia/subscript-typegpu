// example: caustics
// Layers animated three-dimensional noise into bright caustic bands over a dark floor.
// TypeGPU exposes one tile density slider. This port drops the tile pattern, the
// perspective skew, the god rays, the fog, the second caustic layer, and the Perlin blend
// between the layers, and commits the scale and the colors.
// Ported from TypeGPU's caustics example (https://github.com/software-mansion/TypeGPU).

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
  caustic_FRAGMENT_ENTRY,
  caustic_LAYOUT0,
  caustic_TARGET_FORMAT,
  caustic_VERTEX_ENTRY,
  caustic_VERTEX_LAYOUT0,
  caustic_WGSL,
} from "./main.typegpu";

// One triangle corner in normalized device coordinates. TypeGPU reads the same three
// corners from a constant array. This port sends them through a vertex buffer.
@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// The only uniform. time is in seconds and aspect is the window width over its height.
// TypeGPU keeps one uniform per value and adds a tile density slider this port drops.
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
// binding numbers, and the generator emits the WGSL declaration and caustic_LAYOUT0.
class CausticLayout {
  frame: Uniform<FrameData>;

  constructor(frame: Uniform<FrameData>) {
    this.frame = frame;
  }
}

// Four octaves of absolute noise build a ridged field. One minus that field, cubed,
// leaves narrow bright bands. TypeGPU distorts the sample coordinates first instead.
function causticIntensity(point: Vec2f, time: f32): f32 {
  let frequency: f32 = 1.6;
  let amplitude: f32 = 0.58;
  let total: f32 = 0.0;
  for (let octave: u32 = 0; octave < 4; octave += 1) {
    const samplePoint = new Vec3f(
      point.x * frequency,
      point.y * frequency,
      time * (0.24 + (octave as f32) * 0.035),
    );
    let noise: f32 = perlin3d(samplePoint);
    if (noise < 0.0) noise = -noise;
    total += noise * amplitude;
    frequency *= 2.05;
    amplitude *= 0.52;
  }
  const ridge: f32 = 1.0 - total;
  const sharpened: f32 = ridge * ridge * ridge;
  if (sharpened < 0.0) return 0.0;
  if (sharpened > 1.0) return 1.0;
  return sharpened;
}

// The vertex stage widens the corner to clip space and maps it from the -1 to 1 range
// into the 0 to 1 uv range. It reads no binding.
function causticVertex(
  res: CausticLayout,
  value: Vertex,
  ctx: VertexInvocation,
): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

// Per pixel the fragment stage rebuilds a centered point. It scales x by the aspect ratio,
// so the bands stay round when the window is not square. The glow adds to a constant
// dark floor color, which nothing ever removes.
function causticFragment(
  res: CausticLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const frame: FrameData = res.frame.$;
  const point = new Vec2f(
    (input.uv.x * 2.0 - 1.0) * frame.aspect,
    input.uv.y * 2.0 - 1.0,
  );
  const light: f32 = causticIntensity(point, frame.time);
  const floor = new Vec3f(0.015, 0.07, 0.095);
  const glow = new Vec3f(0.18, 0.92, 0.96).scale(light);
  const color: Vec3f = floor.add(glow);
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

// The declaration ties the layout class, the two kernels, and the target format together.
// The generator reads it at compile time and emits caustic_WGSL into ./main.typegpu.
// TypeGPU builds the equivalent shader at run time from the same functions.
export const caustic: RenderPipelineSpec = renderPipelineL<CausticLayout, Vertex, Varyings>(
  causticVertex,
  causticFragment,
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
  if (format !== caustic_TARGET_FORMAT) {
    print(`FAIL format expected=${caustic_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The host owns the device and the queue. This wrapper borrows both and never destroys them.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // The vertex buffer holds three vertices. Vertex_STRIDE is the generated byte stride,
  // so the size never repeats a number counted by hand.
  const vertices = hostDevice.createBuffer({
    label: "caustics-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // The uniform buffer holds one FrameData. FrameData_SIZE comes from the layout generator,
  // so the host size and the WGSL struct size always agree.
  const frameBuffer = hostDevice.createBuffer({
    label: "caustics-frame",
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
    caustic_WGSL,
    caustic_VERTEX_ENTRY,
    caustic_FRAGMENT_ENTRY,
    [caustic_LAYOUT0],
    [caustic_VERTEX_LAYOUT0],
    caustic,
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
  // The resource list follows the field order of CausticLayout. One uniform buffer fills binding 0.
  const group = createBindGroupHost(
    hostDevice,
    bindLayout,
    caustic_LAYOUT0,
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
// because the port fixes the tile density the upstream slider controls.
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
  // The attachment clears to a deep teal water color. loadOp clear drops the previous image,
  // so a resize leaves no stale pixel.
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.01, g: 0.035, b: 0.05, a: 1.0 },
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
