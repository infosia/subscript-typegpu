// example: jump-flood-voronoi
// Floods random color seeds across a grid with one visible jump-flood step per frame.
// The canvas-sized textures commit to a 512-square grid.
// The example starts idle with the seed pattern. Key 1 replaces the Random Seeds button and
// stops. Key 2 replaces Run Algorithm, where upstream runs the flood on load.
// Seed density commits to threshold 0.999, step delay to one frame, and range to 100%.
// Ported from TypeGPU's jump-flood-voronoi example (https://github.com/software-mansion/TypeGPU).

import {
  ComputeInvocation,
  ComputePipeline,
  ComputePipelineSpec,
  FragmentInvocation,
  ReadStorageTexture2dArray,
  RenderPipeline,
  RenderPipelineSpec,
  Rgba16float,
  Sampler,
  Texture2d,
  Uniform,
  VertexInvocation,
  WriteStorageTexture2dArray,
  bufferResource,
  computePipeline,
  createBindGroupHost,
  createComputePipelineHost,
  createRenderPipelineHost,
  renderPipelineL,
  samplerResource,
  textureResource,
} from "./typegpu";
import {
  Vec2f,
  Vec2i,
  Vec3f,
  Vec4f,
} from "./typegpu-types";
import {
  RandomF32,
  randF32,
  randSeed,
} from "./typegpu-noise";
import {
  GPUBindGroup,
  GPUBuffer,
  GPUBufferUsage,
  GPUHostOwnedDevice,
  GPUSampler,
  GPUSamplerDescriptor,
  GPUTexture,
  GPUTextureUsage,
  GPUTextureView,
  hostOwnedGPUDevice,
} from "./webgpu";
import {
  SeedParams_SIZE,
  StepParams_SIZE,
  Vertex_STRIDE,
  jumpFloodStep_ENTRY,
  jumpFloodStep_LAYOUT0,
  jumpFloodStep_WGSL,
  seedVoronoi_ENTRY,
  seedVoronoi_LAYOUT0,
  seedVoronoi_WGSL,
  voronoiRender_FRAGMENT_ENTRY,
  voronoiRender_LAYOUT0,
  voronoiRender_TARGET_FORMAT,
  voronoiRender_VERTEX_ENTRY,
  voronoiRender_VERTEX_LAYOUT0,
  voronoiRender_WGSL,
} from "./main.typegpu";

// The grid commits to 512 squared cells, and the first jump covers half of it. Every later step
// halves the offset, so nine steps carry a seed to any cell.
const GRID_SIZE: u32 = 512;
const LAYER_COUNT: u32 = 2;
const SEED_THRESHOLD: f32 = 0.999;
const START_OFFSET: u32 = 256;

// One corner of the oversized triangle that covers the surface. TypeGPU picks the same three
// corners from the vertex index and binds no vertex buffer.
@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// The reseed counter. A new value changes the seed of every cell, so one key press draws a new
// pattern. TypeGPU reseeds from the clock instead.
@CStruct
class SeedParams {
  reseed: u32;

  constructor(reseed: u32) {
    this.reseed = reseed;
  }
}

// The jump distance of the current step, in cells. The frame writes it before each dispatch.
@CStruct
class StepParams {
  offset: i32;

  constructor(offset: i32) {
    this.offset = offset;
  }
}

// The vertex output. `position` is clip space and `uv` runs 0 to 1 across the surface.
@CStruct
class Varyings {
  position: Vec4f;
  uv: Vec2f;

  constructor(position: Vec4f, uv: Vec2f) {
    this.position = position;
    this.uv = uv;
  }
}

// The seed layout writes both layers of one texture. Layer 0 carries the color, and layer 1
// carries the seed coordinate as a 0 to 1 pair.
class SeedLayout {
  target: WriteStorageTexture2dArray<Rgba16float>;
  params: Uniform<SeedParams>;

  constructor(target: WriteStorageTexture2dArray<Rgba16float>, params: Uniform<SeedParams>) {
    this.target = target;
    this.params = params;
  }
}

// The step layout reads one texture and writes the other, so no invocation reads a cell that
// another invocation already changed.
class StepLayout {
  source: ReadStorageTexture2dArray<Rgba16float>;
  target: WriteStorageTexture2dArray<Rgba16float>;
  params: Uniform<StepParams>;

  constructor(
    source: ReadStorageTexture2dArray<Rgba16float>,
    target: WriteStorageTexture2dArray<Rgba16float>,
    params: Uniform<StepParams>,
  ) {
    this.source = source;
    this.target = target;
    this.params = params;
  }
}

// The render layout takes a sampled view of layer 0 and a linear sampler. A storage binding
// takes no sampler, so the display path needs a second view of the same texture.
class VoronoiRenderLayout {
  colors: Texture2d<f32>;
  linear: Sampler;

  constructor(colors: Texture2d<f32>, linear: Sampler) {
    this.colors = colors;
    this.linear = linear;
  }
}

// These are the four committed upstream palette literals. The random variation below
// changes each channel by at most 0.075.
function paletteColor(index: u32): Vec3f {
  if (index === 0) return new Vec3f(0.9215686, 0.8117647, 1.0);
  if (index === 1) return new Vec3f(0.7176471, 0.5450980, 0.9803922);
  if (index === 2) return new Vec3f(0.5450980, 0.3607843, 0.9647059);
  return new Vec3f(0.4274510, 0.2666667, 0.9490196);
}

// One invocation seeds one cell. Empty coordinates use the sentinel consumed by the
// flood pass, while their color remains transparent black.
function seedKernel(res: SeedLayout, ctx: ComputeInvocation): void {
  const x: u32 = ctx.globalId.x;
  const y: u32 = ctx.globalId.y;
  const coords = new Vec2i(x as i32, y as i32);
  const cellIndex: u32 = y * GRID_SIZE + x;
  // The cell index and the scaled reseed counter give each cell an independent stream, so the
  // result never depends on the dispatch order. The odd multiplier separates two counters.
  let random: RandomF32 = randF32(randSeed(cellIndex + res.params.$.reseed * 747796405));
  if (random.value < SEED_THRESHOLD) {
    res.target.store(coords, 0, new Vec4f(0.0, 0.0, 0.0, 0.0));
    res.target.store(coords, 1, new Vec4f(-1.0, -1.0, 0.0, 0.0));
    return;
  }

  // A seed cell takes one of the four palette colors and stores its own normalized coordinate.
  // Every later step compares against that coordinate.
  random = randF32(random.state);
  const base: Vec3f = paletteColor((random.value * 4.0) as u32);
  random = randF32(random.state);
  const variationX: f32 = (random.value - 0.5) * 0.15;
  random = randF32(random.state);
  const variationY: f32 = (random.value - 0.5) * 0.15;
  random = randF32(random.state);
  const variationZ: f32 = (random.value - 0.5) * 0.15;
  const color: Vec3f = base.add(new Vec3f(
    variationX,
    variationY,
    variationZ,
  )).clamp(new Vec3f(0.0, 0.0, 0.0), new Vec3f(1.0, 1.0, 1.0));
  res.target.store(coords, 0, new Vec4f(color.x, color.y, color.z, 1.0));
  res.target.store(coords, 1, new Vec4f(
    (x as f32) / (GRID_SIZE as f32),
    (y as f32) / (GRID_SIZE as f32),
    0.0,
    0.0,
  ));
}

// The comparison uses the squared distance, so no square root runs per candidate. A negative x
// marks an empty cell, and the large return keeps that candidate last.
// TypeGPU compares the true distance and skips an empty sample.
function seedDistance(x: f32, y: f32, seed: Vec4f): f32 {
  if (seed.x < 0.0) return 100000000000000000000.0;
  const dx: f32 = x - seed.x * (GRID_SIZE as f32);
  const dy: f32 = y - seed.y * (GRID_SIZE as f32);
  return dx * dx + dy * dy;
}

// One invocation keeps the nearest seed among the cell itself and its eight neighbors at the
// current offset. The nine candidates are written out to make the upstream unroll explicit.
// Each in-bounds candidate carries its color and seed-coordinate layers together.
function stepKernel(res: StepLayout, ctx: ComputeInvocation): void {
  const x: i32 = ctx.globalId.x as i32;
  const y: i32 = ctx.globalId.y as i32;
  const offset: i32 = res.params.$.offset;
  const coords = new Vec2i(x, y);
  let bestColor: Vec4f = res.source.load(coords, 0);
  let bestSeed: Vec4f = res.source.load(coords, 1);
  let bestDistance: f32 = seedDistance(x as f32, y as f32, bestSeed);

  const nw = new Vec2i(x - offset, y - offset);
  if (nw.x >= 0 && nw.y >= 0) {
    const seed: Vec4f = res.source.load(nw, 1);
    const color: Vec4f = res.source.load(nw, 0);
    const distance: f32 = seedDistance(x as f32, y as f32, seed);
    if (distance < bestDistance) {
      bestDistance = distance;
      bestSeed = seed;
      bestColor = color;
    }
  }
  const north = new Vec2i(x, y - offset);
  if (north.y >= 0) {
    const seed: Vec4f = res.source.load(north, 1);
    const color: Vec4f = res.source.load(north, 0);
    const distance: f32 = seedDistance(x as f32, y as f32, seed);
    if (distance < bestDistance) {
      bestDistance = distance;
      bestSeed = seed;
      bestColor = color;
    }
  }
  const ne = new Vec2i(x + offset, y - offset);
  if (ne.x < (GRID_SIZE as i32) && ne.y >= 0) {
    const seed: Vec4f = res.source.load(ne, 1);
    const color: Vec4f = res.source.load(ne, 0);
    const distance: f32 = seedDistance(x as f32, y as f32, seed);
    if (distance < bestDistance) {
      bestDistance = distance;
      bestSeed = seed;
      bestColor = color;
    }
  }
  const west = new Vec2i(x - offset, y);
  if (west.x >= 0) {
    const seed: Vec4f = res.source.load(west, 1);
    const color: Vec4f = res.source.load(west, 0);
    const distance: f32 = seedDistance(x as f32, y as f32, seed);
    if (distance < bestDistance) {
      bestDistance = distance;
      bestSeed = seed;
      bestColor = color;
    }
  }
  const east = new Vec2i(x + offset, y);
  if (east.x < (GRID_SIZE as i32)) {
    const seed: Vec4f = res.source.load(east, 1);
    const color: Vec4f = res.source.load(east, 0);
    const distance: f32 = seedDistance(x as f32, y as f32, seed);
    if (distance < bestDistance) {
      bestDistance = distance;
      bestSeed = seed;
      bestColor = color;
    }
  }
  const sw = new Vec2i(x - offset, y + offset);
  if (sw.x >= 0 && sw.y < (GRID_SIZE as i32)) {
    const seed: Vec4f = res.source.load(sw, 1);
    const color: Vec4f = res.source.load(sw, 0);
    const distance: f32 = seedDistance(x as f32, y as f32, seed);
    if (distance < bestDistance) {
      bestDistance = distance;
      bestSeed = seed;
      bestColor = color;
    }
  }
  const south = new Vec2i(x, y + offset);
  if (south.y < (GRID_SIZE as i32)) {
    const seed: Vec4f = res.source.load(south, 1);
    const color: Vec4f = res.source.load(south, 0);
    const distance: f32 = seedDistance(x as f32, y as f32, seed);
    if (distance < bestDistance) {
      bestDistance = distance;
      bestSeed = seed;
      bestColor = color;
    }
  }
  const se = new Vec2i(x + offset, y + offset);
  if (se.x < (GRID_SIZE as i32) && se.y < (GRID_SIZE as i32)) {
    const seed: Vec4f = res.source.load(se, 1);
    const color: Vec4f = res.source.load(se, 0);
    const distance: f32 = seedDistance(x as f32, y as f32, seed);
    if (distance < bestDistance) {
      bestDistance = distance;
      bestSeed = seed;
      bestColor = color;
    }
  }

  res.target.store(coords, 0, bestColor);
  res.target.store(coords, 1, bestSeed);
}

// Three vertices cover the surface, and the uv follows the clip position. TypeGPU emits the
// same oversized triangle from a shared helper.
function voronoiVertex(
  res: VoronoiRenderLayout,
  value: Vertex,
  ctx: VertexInvocation,
): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

// The fragment samples the color layer through the linear filter, so the flood result blends
// between cells. TypeGPU samples the same layer with `textureSample`.
function voronoiFragment(
  res: VoronoiRenderLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  return res.colors.sampleLevel(res.linear, input.uv, 0.0);
}

// The three declarations name the kernel, the layout type, and the workgroup size. The generator
// reads them ahead of the run and emits the WGSL, the entry names, and the layout facts.
// TypeGPU builds the same WGSL at run time from the kernel function.
export const seedVoronoi: ComputePipelineSpec = computePipeline<SeedLayout>(seedKernel, {
  name: "seedVoronoi",
  workgroupSize: [8, 8, 1],
});

export const jumpFloodStep: ComputePipelineSpec = computePipeline<StepLayout>(stepKernel, {
  name: "jumpFloodStep",
  workgroupSize: [8, 8, 1],
});

export const voronoiRender: RenderPipelineSpec = renderPipelineL<
  VoronoiRenderLayout,
  Vertex,
  Varyings
>(voronoiVertex, voronoiFragment, { format: "bgra8unorm" });

// One holder for everything `init` creates. `compute` holds the seed pipeline and then the step
// pipeline. `computeGroups` holds seed A, seed B, step A to B, and step B to A.
// TypeGPU frees the same resources through garbage collection and `root.destroy()`.
class VoronoiState {
  device: GPUHostOwnedDevice;
  compute: ComputePipeline[];
  render: RenderPipeline;
  computeGroups: GPUBindGroup[];
  renderGroups: GPUBindGroup[];
  vertices: GPUBuffer;
  seedParams: GPUBuffer;
  stepParams: GPUBuffer;
  textures: GPUTexture[];
  views: GPUTextureView[];
  linearSampler: GPUSampler;

  constructor(
    device: GPUHostOwnedDevice,
    compute: ComputePipeline[],
    render: RenderPipeline,
    computeGroups: GPUBindGroup[],
    renderGroups: GPUBindGroup[],
    vertices: GPUBuffer,
    seedParams: GPUBuffer,
    stepParams: GPUBuffer,
    textures: GPUTexture[],
    views: GPUTextureView[],
    linearSampler: GPUSampler,
  ) {
    this.device = device;
    this.compute = compute;
    this.render = render;
    this.computeGroups = computeGroups;
    this.renderGroups = renderGroups;
    this.vertices = vertices;
    this.seedParams = seedParams;
    this.stepParams = stepParams;
    this.textures = textures;
    this.views = views;
    this.linearSampler = linearSampler;
  }
}

// `currentIsA` names the texture that holds the newest result. `jumpOffset` counts down to
// zero, and a zero offset stops the flood until a key restarts it.
let activeState: VoronoiState | null = null;
let currentIsA: boolean = true;
let jumpOffset: u32 = 0;
let reseedCounter: u32 = 1;

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  // The pipeline declares its target format literally. A surface with another format ends the
  // example before any draw.
  if (format !== voronoiRender_TARGET_FORMAT) {
    print(`FAIL format expected=${voronoiRender_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The host owns the device and the instance. The wrapper carries no `dispose`, so this example
  // never releases what it did not create.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // Buffer sizes come from the generated stride and size constants, never from a hand count.
  const vertices = hostDevice.createBuffer({
    label: "voronoi-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // Two uniforms. The seed pass reads the counter, and the step pass reads the current offset.
  const seedParams = hostDevice.createBuffer({
    label: "voronoi-seed-params",
    size: SeedParams_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  const stepParams = hostDevice.createBuffer({
    label: "voronoi-step-params",
    size: StepParams_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  // Each texture carries two array layers. `STORAGE_BINDING` lets a kernel store into a layer,
  // and `TEXTURE_BINDING` lets the render pass sample layer 0.
  const textureUsage: u64 = GPUTextureUsage.STORAGE_BINDING + GPUTextureUsage.TEXTURE_BINDING;
  const textureA = hostDevice.createTexture({
    label: "voronoi-a",
    size: { width: GRID_SIZE, height: GRID_SIZE, depthOrArrayLayers: LAYER_COUNT },
    format: "rgba16float",
    usage: textureUsage,
  });
  const textureB = hostDevice.createTexture({
    label: "voronoi-b",
    size: { width: GRID_SIZE, height: GRID_SIZE, depthOrArrayLayers: LAYER_COUNT },
    format: "rgba16float",
    usage: textureUsage,
  });
  // A compute binding takes the two-layer array view, and the render binding takes a 2D view of
  // layer 0. One texture therefore needs two views, and all four live until `shutdown`.
  const arrayViewA = textureA.createView({
    dimension: "2d-array",
    mipLevelCount: 1,
    arrayLayerCount: LAYER_COUNT,
  });
  const arrayViewB = textureB.createView({
    dimension: "2d-array",
    mipLevelCount: 1,
    arrayLayerCount: LAYER_COUNT,
  });
  const colorViewA = textureA.createView({
    dimension: "2d",
    mipLevelCount: 1,
    baseArrayLayer: 0,
    arrayLayerCount: 1,
  });
  const colorViewB = textureB.createView({
    dimension: "2d",
    mipLevelCount: 1,
    baseArrayLayer: 0,
    arrayLayerCount: 1,
  });
  // The linear filter blends the color layer across cells, so the display hides the grid step.
  const samplerDescriptor: GPUSamplerDescriptor = {
    minFilter: "linear",
    magFilter: "linear",
  };
  const linearSampler = hostDevice.createSampler(samplerDescriptor);
  // Three vertices in clip space cover the surface. Queue writes land before the commands that a
  // later submit carries, so both uniforms hold a value before the first dispatch.
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>([
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(3.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 3.0)),
  ]));
  queue.writeBuffer(seedParams, 0, Context.bytesOf<SeedParams>(new SeedParams(reseedCounter)));
  queue.writeBuffer(stepParams, 0, Context.bytesOf<StepParams>(new StepParams(0)));

  // The error scope catches a pipeline creation failure. TypeGPU rejects a promise. Here the pop
  // returns a value, so this example releases what it created and prints the first error line.
  hostDevice.pushErrorScope("validation");
  // Each compute pipeline takes the generated WGSL text, the entry name, the layout facts, and
  // the workgroup size. That size must equal the size in the declaration above.
  const seedPipeline = createComputePipelineHost(
    hostDevice,
    seedVoronoi_WGSL,
    seedVoronoi_ENTRY,
    [seedVoronoi_LAYOUT0],
    [8, 8, 1],
  );
  const stepPipeline = createComputePipelineHost(
    hostDevice,
    jumpFloodStep_WGSL,
    jumpFloodStep_ENTRY,
    [jumpFloodStep_LAYOUT0],
    [8, 8, 1],
  );
  // The render pipeline also takes the vertex buffer layout and the declaration, which carries
  // the target format.
  const renderPipeline = createRenderPipelineHost(
    hostDevice,
    voronoiRender_WGSL,
    voronoiRender_VERTEX_ENTRY,
    voronoiRender_FRAGMENT_ENTRY,
    [voronoiRender_LAYOUT0],
    [voronoiRender_VERTEX_LAYOUT0],
    voronoiRender,
  );
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    renderPipeline.dispose();
    stepPipeline.dispose();
    seedPipeline.dispose();
    linearSampler.dispose();
    colorViewB.dispose();
    colorViewA.dispose();
    arrayViewB.dispose();
    arrayViewA.dispose();
    textureB.dispose();
    textureA.dispose();
    stepParams.dispose();
    seedParams.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }

  // The pipeline reports the layout WebGPU built for group 0. Each call returns a new handle, and
  // the bind groups below need it only at creation.
  using seedLayout = seedPipeline.bindGroupLayout(0);
  using stepLayout = stepPipeline.bindGroupLayout(0);
  using renderLayout = renderPipeline.bindGroupLayout(0);
  // Two seed groups pick the texture the seed pass fills. Two step groups cover both flood
  // directions, and two render groups pick the texture the frame displays.
  const seedA = createBindGroupHost(hostDevice, seedLayout, seedVoronoi_LAYOUT0, [
    textureResource(arrayViewA),
    bufferResource(seedParams),
  ]);
  const seedB = createBindGroupHost(hostDevice, seedLayout, seedVoronoi_LAYOUT0, [
    textureResource(arrayViewB),
    bufferResource(seedParams),
  ]);
  const stepAB = createBindGroupHost(hostDevice, stepLayout, jumpFloodStep_LAYOUT0, [
    textureResource(arrayViewA),
    textureResource(arrayViewB),
    bufferResource(stepParams),
  ]);
  const stepBA = createBindGroupHost(hostDevice, stepLayout, jumpFloodStep_LAYOUT0, [
    textureResource(arrayViewB),
    textureResource(arrayViewA),
    bufferResource(stepParams),
  ]);
  const renderA = createBindGroupHost(hostDevice, renderLayout, voronoiRender_LAYOUT0, [
    textureResource(colorViewA),
    samplerResource(linearSampler),
  ]);
  const renderB = createBindGroupHost(hostDevice, renderLayout, voronoiRender_LAYOUT0, [
    textureResource(colorViewB),
    samplerResource(linearSampler),
  ]);

  // The state reaches module scope only after every creation succeeds.
  activeState = new VoronoiState(
    hostDevice,
    [seedPipeline, stepPipeline],
    renderPipeline,
    [seedA, seedB, stepAB, stepBA],
    [renderA, renderB],
    vertices,
    seedParams,
    stepParams,
    [textureA, textureB],
    [arrayViewA, arrayViewB, colorViewA, colorViewB],
    linearSampler,
  );
  // The first seed pass runs here, so the first frame displays a seeded grid. It fills texture A,
  // which `currentIsA` names at start.
  using encoder = hostDevice.createCommandEncoderDefault();
  seedPipeline.dispatch(encoder, [seedA], GRID_SIZE / 8, GRID_SIZE / 8, 1);
  using command = encoder.finishDefault();
  queue.submit([command]);
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
  // A failed `init` leaves the state empty, and the frame ends without a draw. TypeGPU reports
  // the same failure as an exception.
  if (activeState === null) return;
  const active = activeState;
  using queue = active.device.queue();
  // One encoder carries the whole frame. The key action, the flood step, and the render pass all
  // record into it, and one submit sends them in order.
  using encoder = active.device.createCommandEncoderDefault();

  // Key 49 is `1` and key 50 is `2`. `1` reseeds the current texture and stops the flood.
  // `2` restarts the flood at the largest offset. TypeGPU carries both actions as buttons.
  if (key === 49) {
    reseedCounter += 1;
    jumpOffset = 0;
    queue.writeBuffer(
      active.seedParams,
      0,
      Context.bytesOf<SeedParams>(new SeedParams(reseedCounter)),
    );
    const seedGroup: GPUBindGroup = currentIsA
      ? active.computeGroups[0]
      : active.computeGroups[1];
    active.compute[0].dispatch(encoder, [seedGroup], GRID_SIZE / 8, GRID_SIZE / 8, 1);
  } else if (key === 50) {
    jumpOffset = START_OFFSET;
  }

  // One step per frame makes the flood visible. The step reads the current texture and writes
  // the other, so the flip names the new result. The halved offset ends the flood at nine steps.
  if (jumpOffset >= 1) {
    queue.writeBuffer(
      active.stepParams,
      0,
      Context.bytesOf<StepParams>(new StepParams(jumpOffset as i32)),
    );
    const stepGroup: GPUBindGroup = currentIsA
      ? active.computeGroups[2]
      : active.computeGroups[3];
    active.compute[1].dispatch(encoder, [stepGroup], GRID_SIZE / 8, GRID_SIZE / 8, 1);
    currentIsA = !currentIsA;
    jumpOffset /= 2;
  }

  // The host owns the presented view, so this example wraps it and releases nothing.
  const target = new GPUTextureView(view);
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.008, g: 0.006, b: 0.015, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  // The surface size changes when the window resizes, so both rectangles follow the frame size.
  pass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  pass.setScissorRect(0, 0, width, height);
  // Three vertices draw the triangle, and the render group selects the texture the last step
  // filled.
  const renderGroup: GPUBindGroup = currentIsA
    ? active.renderGroups[0]
    : active.renderGroups[1];
  active.render.bind(pass, [renderGroup], [active.vertices]);
  pass.draw(3);
  pass.end();
  // The command buffer reaches the device queue after the encoder finishes. The host presents
  // the surface after this call returns.
  using command = encoder.finishDefault();
  queue.submit([command]);
}

// The host calls this once before it releases the device. This example disposes in reverse
// creation order, so a bind group never outlives the views and the buffers it names.
// TypeGPU releases the same resources with one `root.destroy()` call.
export function shutdown(): void {
  if (activeState === null) return;
  const active = activeState;
  let index: i32 = 0;
  while (index < active.renderGroups.length) {
    active.renderGroups[index].dispose();
    index += 1;
  }
  index = 0;
  while (index < active.computeGroups.length) {
    active.computeGroups[index].dispose();
    index += 1;
  }
  active.linearSampler.dispose();
  index = 0;
  while (index < active.views.length) {
    active.views[index].dispose();
    index += 1;
  }
  index = 0;
  while (index < active.textures.length) {
    active.textures[index].dispose();
    index += 1;
  }
  active.stepParams.dispose();
  active.seedParams.dispose();
  active.vertices.dispose();
  active.render.dispose();
  index = 0;
  while (index < active.compute.length) {
    active.compute[index].dispose();
    index += 1;
  }
  // The cleared state leaves no released handle reachable from module scope.
  activeState = null;
  currentIsA = true;
  jumpOffset = 0;
  reseedCounter = 1;
}
