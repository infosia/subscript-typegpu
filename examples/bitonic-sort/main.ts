// example: bitonic-sort
// Shows an ascending u32 bitonic sort converging across a 64-by-64 grayscale grid.
// This port commits the upstream comparator to ascending order, maps upstream reshuffle
// and sort buttons to keys 1 and 2, and submits one step per frame instead of one submit.
// Ported from TypeGPU's bitonic-sort example (https://github.com/software-mansion/TypeGPU).

import {
  ComputePipeline,
  ComputePipelineSpec,
  FragmentInvocation,
  RenderPipeline,
  RenderPipelineSpec,
  Storage,
  VertexInvocation,
  bufferResource,
  computePipeline,
  createBindGroupHost,
  createComputePipelineHost,
  createRenderPipelineHost,
  renderPipelineL,
} from "./typegpu";
import {
  RandomF32,
  randF32,
  randSeed,
} from "./typegpu-noise";
import {
  BitonicSortPass,
  BitonicSortResources,
  bitonicSortPass,
  bitonicSortPassCount,
  bitonicSortStep,
} from "./typegpu-sort";
import {
  Vec2f,
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
  BitonicSortPass_SIZE,
  FullscreenVertex_STRIDE,
  bitonicDisplay_FRAGMENT_ENTRY,
  bitonicDisplay_LAYOUT0,
  bitonicDisplay_TARGET_FORMAT,
  bitonicDisplay_VERTEX_ENTRY,
  bitonicDisplay_VERTEX_LAYOUT0,
  bitonicDisplay_WGSL,
  bitonicStepPipeline_ENTRY,
  bitonicStepPipeline_LAYOUT0,
  bitonicStepPipeline_WGSL,
} from "./main.typegpu";

// 4096 values fill the 64 by 64 grid the display draws. The sort helper needs a power of
// two. One invocation owns one comparator pair, so half the values gives the thread count.
const VALUE_COUNT: u32 = 4096;
const COMPARATOR_COUNT: u32 = VALUE_COUNT / 2;

// One corner of the full-surface triangle in normalized device coordinates.
// TypeGPU takes the same triangle from a shared vertex helper.
@CStruct
class FullscreenVertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// The vertex stage returns this record and the fragment stage receives it.
// The field named position becomes the clip position, and uv becomes location 0.
@CStruct
class DisplayVaryings {
  position: Vec4f;
  uv: Vec2f;

  constructor(position: Vec4f, uv: Vec2f) {
    this.position = position;
    this.uv = uv;
  }
}

// The display binds the same buffer as read-only storage. The sort kernel binds it as
// mutable storage through its own layout, so the two passes never share a bind group.
class BitonicDisplayResources {
  values!: Storage<u32>;
}

// The vertex stage widens the corner to clip space and maps it from the -1 to 1 range
// into the 0 to 1 uv range. It reads no binding.
function displayVertex(
  resources: BitonicDisplayResources,
  value: FullscreenVertex,
  invocation: VertexInvocation,
): DisplayVaryings {
  return new DisplayVaryings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

// Each pixel maps to one of the 4096 values through a 64 by 64 grid. Brightness is the
// value divided by the largest u32, so a sorted array reads as a smooth ramp.
// TypeGPU derives the grid side from the array length and stores values 0 through 255.
function displayFragment(
  resources: BitonicDisplayResources,
  input: DisplayVaryings,
  invocation: FragmentInvocation,
): Vec4f {
  let column: u32 = (input.uv.x * 64.0) as u32;
  let row: u32 = (input.uv.y * 64.0) as u32;
  if (column > 63) column = 63;
  if (row > 63) row = 63;
  const value: f32 = (resources.values[row * 64 + column] as f32) / 4294967295.0;
  return new Vec4f(value, value, value, 1.0);
}

// The kernel and its layout come from the sort library, so this example declares only the
// pipeline. 2048 comparators divide by 256 exactly, so the dispatch needs no guard.
export const bitonicStepPipeline: ComputePipelineSpec = computePipeline<
  BitonicSortResources
>(bitonicSortStep, {
  name: "bitonicStepPipeline",
  workgroupSize: [256, 1, 1],
});

// The declaration ties the layout class, the two kernels, and the target format together.
// The generator reads it at compile time and emits bitonicDisplay_WGSL into ./main.typegpu.
export const bitonicDisplay: RenderPipelineSpec = renderPipelineL<
  BitonicDisplayResources,
  FullscreenVertex,
  DisplayVaryings
>(displayVertex, displayFragment, { format: "bgra8unorm" });

// The window host calls init, frame, and shutdown as separate entry points, so the handles
// and the sort progress live in module state. The script owns each handle.
let activeDevice: GPUHostOwnedDevice | null = null;
let activeComputePipeline: ComputePipeline | null = null;
let activeRenderPipeline: RenderPipeline | null = null;
let activeComputeGroup: GPUBindGroup | null = null;
let activeRenderGroup: GPUBindGroup | null = null;
let activeValues: GPUBuffer | null = null;
let activePass: GPUBuffer | null = null;
let activeVertices: GPUBuffer | null = null;
let randomState: u32 = randSeed(7);
let sortActive: boolean = false;
let sortPassIndex: u32 = 0;
let sortPassCount: u32 = 0;

// Context.bytesOf returns the bytes of one value, so the caller concatenates 4096 of them
// into a single upload.
function appendBitonicBytes(target: u8[], source: u8[]): void {
  let index: i32 = 0;
  while (index < source.length) {
    target.push(source[index]);
    index += 1;
  }
}

// The host runs the random chain and takes each new state as the next value.
// TypeGPU fills the same buffer with a compute kernel and a seed uniform.
function shuffledValueBytes(): u8[] {
  const bytes: u8[] = [];
  let index: u32 = 0;
  while (index < VALUE_COUNT) {
    const sample: RandomF32 = randF32(randomState);
    randomState = sample.state;
    appendBitonicBytes(bytes, Context.bytesOf<FixedArray<u32, 1>>([sample.state]));
    index += 1;
  }
  return bytes;
}

// The host calls init once, before the first frame, with the device it owns.
export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  // The host picks the surface format. The generator baked one format into the pipeline,
  // so a mismatch stops the example here instead of at pipeline creation.
  if (format !== bitonicDisplay_TARGET_FORMAT) {
    print(`FAIL format expected=${bitonicDisplay_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The host owns the device and the queue. This wrapper borrows both and never destroys them.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // One storage buffer of 4096 u32 keys. Both passes bind it, and no readback exists,
  // so the buffer carries no COPY_SRC.
  const values = hostDevice.createBuffer({
    label: "bitonic-sort-values",
    size: (VALUE_COUNT as u64) * 4,
    usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
  });
  // The uniform carries the two parameters of the current comparator pass.
  // A frame that runs a pass rewrites it, because every pass uses a different stride.
  const pass = hostDevice.createBuffer({
    label: "bitonic-sort-pass",
    size: BitonicSortPass_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  // The vertex buffer holds three vertices. FullscreenVertex_STRIDE is the generated byte
  // stride, so the size never repeats a number counted by hand.
  const vertices = hostDevice.createBuffer({
    label: "bitonic-sort-fullscreen",
    size: (FullscreenVertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // A host-owned device returns a new queue wrapper from each call, so this block frees it.
  using queue = hostDevice.queue();
  // One upload seeds all 4096 values. The shuffle runs on the host, so the first image shows
  // noise before any dispatch.
  queue.writeBuffer(values, 0, shuffledValueBytes());
  // The first pass parameters land before any dispatch, so a draw before the first sort
  // still reads a valid uniform.
  queue.writeBuffer(
    pass,
    0,
    Context.bytesOf<BitonicSortPass>(new BitonicSortPass(2, 0)),
  );
  // One oversized triangle covers the whole surface. The parts outside the viewport clip
  // away, so this example needs no quad and no index buffer.
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<FullscreenVertex, 3>>([
    new FullscreenVertex(new Vec2f(-1.0, -1.0)),
    new FullscreenVertex(new Vec2f(3.0, -1.0)),
    new FullscreenVertex(new Vec2f(-1.0, 3.0)),
  ]));

  // Pipeline creation raises no exception. One error scope covers both pipelines and turns
  // a bad shader or a bad layout into a value the code below tests.
  hostDevice.pushErrorScope("validation");
  const computePipeline = createComputePipelineHost(
    hostDevice,
    bitonicStepPipeline_WGSL,
    bitonicStepPipeline_ENTRY,
    [bitonicStepPipeline_LAYOUT0],
    [256, 1, 1],
  );
  const renderPipeline = createRenderPipelineHost(
    hostDevice,
    bitonicDisplay_WGSL,
    bitonicDisplay_VERTEX_ENTRY,
    bitonicDisplay_FRAGMENT_ENTRY,
    [bitonicDisplay_LAYOUT0],
    [bitonicDisplay_VERTEX_LAYOUT0],
    bitonicDisplay,
  );
  // A null check replaces the exception a browser port throws. The failure path frees the
  // five handles this function already created.
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    renderPipeline.dispose();
    computePipeline.dispose();
    vertices.dispose();
    pass.dispose();
    values.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  // Two bind groups over one values buffer: the sort group adds the pass uniform, and the
  // display group binds the values alone. Each layout comes back from its own pipeline.
  using computeLayout = computePipeline.bindGroupLayout(0);
  const computeGroup = createBindGroupHost(
    hostDevice,
    computeLayout,
    bitonicStepPipeline_LAYOUT0,
    [bufferResource(values), bufferResource(pass)],
  );
  using renderLayout = renderPipeline.bindGroupLayout(0);
  const renderGroup = createBindGroupHost(
    hostDevice,
    renderLayout,
    bitonicDisplay_LAYOUT0,
    [bufferResource(values)],
  );
  // The handles reach module state only after every step succeeds, so frame never sees a
  // half-built set. 4096 values need 78 comparator passes.
  activeDevice = hostDevice;
  activeComputePipeline = computePipeline;
  activeRenderPipeline = renderPipeline;
  activeComputeGroup = computeGroup;
  activeRenderGroup = renderGroup;
  activeValues = values;
  activePass = pass;
  activeVertices = vertices;
  sortPassCount = bitonicSortPassCount(VALUE_COUNT);
}

// The host calls frame once per presented image. It draws the values every frame and
// advances the sort by one pass while a sort runs.
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
  const computePipeline = activeComputePipeline;
  const renderPipeline = activeRenderPipeline;
  const computeGroup = activeComputeGroup;
  const renderGroup = activeRenderGroup;
  const values = activeValues;
  const passBuffer = activePass;
  const vertices = activeVertices;
  if (device === null) return;
  if (computePipeline === null) return;
  if (renderPipeline === null) return;
  if (computeGroup === null) return;
  if (renderGroup === null) return;
  if (values === null) return;
  if (passBuffer === null) return;
  if (vertices === null) return;

  using queue = device.queue();
  // The host stores the last key press as a Unicode scalar and clears the slot after this
  // call. 49 and 50 are the codes for 1 and 2, which replace the upstream buttons.
  if (key === 49) {
    queue.writeBuffer(values, 0, shuffledValueBytes());
    sortActive = false;
    sortPassIndex = 0;
  } else if (key === 50) {
    sortActive = true;
    sortPassIndex = 0;
  }
  using encoder = device.createCommandEncoderDefault();
  // One comparator pass per frame keeps the convergence visible. The upstream example
  // submits every pass at once and shows only the sorted result.
  // The uniform write reaches the queue before the submit, so the dispatch reads this pass.
  if (sortActive) {
    const currentPass: BitonicSortPass = bitonicSortPass(VALUE_COUNT, sortPassIndex);
    queue.writeBuffer(passBuffer, 0, Context.bytesOf<BitonicSortPass>(currentPass));
    computePipeline.dispatchThreads(encoder, [computeGroup], COMPARATOR_COUNT, 1, 1);
    sortPassIndex += 1;
    if (sortPassIndex === sortPassCount) sortActive = false;
  }

  // The host passes the current surface view as a raw handle. The wrapper does not own it,
  // and the host presents the surface after frame returns.
  const target = new GPUTextureView(view);
  // The attachment clears to a near-black gray. The triangle covers every pixel, so the
  // clear color only shows when the draw fails.
  using renderPass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.02, g: 0.025, b: 0.035, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  // The window resizes between frames. The viewport and the scissor follow the size the host
  // reports, so the grid keeps the whole surface.
  renderPass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  renderPass.setScissorRect(0, 0, width, height);
  // One call sets the pipeline, the bind groups by group index, and the vertex buffers by slot.
  renderPipeline.bind(renderPass, [renderGroup], [vertices]);
  renderPass.draw(3);
  renderPass.end();
  // The pass must end before the encoder finishes. The compute pass sits earlier in the same
  // command buffer, so the draw reads the values this frame produced.
  using command = encoder.finishDefault();
  queue.submit([command]);
}

// The host calls shutdown once. The script frees every GPU handle by hand, because this
// library keeps no finalizer and no reference count for scripts.
export function shutdown(): void {
  if (activeRenderGroup !== null) activeRenderGroup.dispose();
  if (activeComputeGroup !== null) activeComputeGroup.dispose();
  if (activeVertices !== null) activeVertices.dispose();
  if (activePass !== null) activePass.dispose();
  if (activeValues !== null) activeValues.dispose();
  if (activeRenderPipeline !== null) activeRenderPipeline.dispose();
  if (activeComputePipeline !== null) activeComputePipeline.dispose();
  activeRenderGroup = null;
  activeComputeGroup = null;
  activeVertices = null;
  activePass = null;
  activeValues = null;
  activeRenderPipeline = null;
  activeComputePipeline = null;
  activeDevice = null;
  sortActive = false;
  sortPassIndex = 0;
  sortPassCount = 0;
}
