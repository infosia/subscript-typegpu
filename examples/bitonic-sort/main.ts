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

const VALUE_COUNT: u32 = 4096;
const COMPARATOR_COUNT: u32 = VALUE_COUNT / 2;

@CStruct
class FullscreenVertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

@CStruct
class DisplayVaryings {
  position: Vec4f;
  uv: Vec2f;

  constructor(position: Vec4f, uv: Vec2f) {
    this.position = position;
    this.uv = uv;
  }
}

class BitonicDisplayResources {
  values!: Storage<u32>;
}

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

// Each pixel maps to one of the 4096 values. Brightness is the value itself.
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

export const bitonicStepPipeline: ComputePipelineSpec = computePipeline<
  BitonicSortResources
>(bitonicSortStep, {
  name: "bitonicStepPipeline",
  workgroupSize: [256, 1, 1],
});

export const bitonicDisplay: RenderPipelineSpec = renderPipelineL<
  BitonicDisplayResources,
  FullscreenVertex,
  DisplayVaryings
>(displayVertex, displayFragment, { format: "bgra8unorm" });

let activeDevice: GPUHostOwnedDevice | null = null;
let activeComputePipeline: ComputePipeline | null = null;
let activeRenderPipeline: RenderPipeline | null = null;
let activeComputeGroup: GPUBindGroup | null = null;
let activeRenderGroup: GPUBindGroup | null = null;
let activeValues: GPUBuffer | null = null;
let activePass: GPUBuffer | null = null;
let activeVertices: GPUBuffer | null = null;
let randomState: u32 = randSeed(20260831);
let sortActive: boolean = false;
let sortPassIndex: u32 = 0;
let sortPassCount: u32 = 0;

function appendBitonicBytes(target: u8[], source: u8[]): void {
  let index: i32 = 0;
  while (index < source.length) {
    target.push(source[index]);
    index += 1;
  }
}

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

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  if (format !== bitonicDisplay_TARGET_FORMAT) {
    print(`FAIL format expected=${bitonicDisplay_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const values = hostDevice.createBuffer({
    label: "bitonic-sort-values",
    size: (VALUE_COUNT as u64) * 4,
    usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
  });
  const pass = hostDevice.createBuffer({
    label: "bitonic-sort-pass",
    size: BitonicSortPass_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  const vertices = hostDevice.createBuffer({
    label: "bitonic-sort-fullscreen",
    size: (FullscreenVertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  using queue = hostDevice.queue();
  queue.writeBuffer(values, 0, shuffledValueBytes());
  queue.writeBuffer(
    pass,
    0,
    Context.bytesOf<BitonicSortPass>(new BitonicSortPass(2, 0)),
  );
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<FullscreenVertex, 3>>([
    new FullscreenVertex(new Vec2f(-1.0, -1.0)),
    new FullscreenVertex(new Vec2f(3.0, -1.0)),
    new FullscreenVertex(new Vec2f(-1.0, 3.0)),
  ]));

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
  // Key 1 reshuffles. Key 2 restarts the sort from pass zero.
  if (key === 49) {
    queue.writeBuffer(values, 0, shuffledValueBytes());
    sortActive = false;
    sortPassIndex = 0;
  } else if (key === 50) {
    sortActive = true;
    sortPassIndex = 0;
  }
  using encoder = device.createCommandEncoderDefault();
  // One comparator pass per frame keeps the convergence visible.
  if (sortActive) {
    const currentPass: BitonicSortPass = bitonicSortPass(VALUE_COUNT, sortPassIndex);
    queue.writeBuffer(passBuffer, 0, Context.bytesOf<BitonicSortPass>(currentPass));
    computePipeline.dispatchThreads(encoder, [computeGroup], COMPARATOR_COUNT, 1, 1);
    sortPassIndex += 1;
    if (sortPassIndex === sortPassCount) sortActive = false;
  }

  const target = new GPUTextureView(view);
  using renderPass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.02, g: 0.025, b: 0.035, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  renderPass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  renderPass.setScissorRect(0, 0, width, height);
  renderPipeline.bind(renderPass, [renderGroup], [vertices]);
  renderPass.draw(3);
  renderPass.end();
  using command = encoder.finishDefault();
  queue.submit([command]);
}

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
