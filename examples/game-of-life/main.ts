// example: game-of-life
// Advances a texture-backed Conway grid and accepts pointer drawing and keyboard clearing.
// This port keeps the naive strategy at a fixed 128-square grid. It drops the workgroup-tiled
// strategy, the bit-packed strategy, the size selector, the zoom view, and the pause controls.
// One glider replaces the upstream random seed, and a fixed square brush replaces the
// upstream brush radius, brush modes, and stroke line. The cell texture is `r32float`,
// not `r32uint`, and the neighbor lookup wraps into a torus. The view selector, the
// timestep, the steps-per-frame, and the Step controls do not port.
// Ported from TypeGPU's game-of-life example (https://github.com/software-mansion/TypeGPU).

import {
  ComputeInvocation,
  ComputePipeline,
  ComputePipelineSpec,
  FragmentInvocation,
  R32float,
  ReadStorageTexture2d,
  ReadWriteStorageTexture2d,
  RenderPipeline,
  RenderPipelineSpec,
  StorageTexture2d,
  Uniform,
  VertexInvocation,
  bufferResource,
  computePipeline,
  createBindGroupHost,
  createComputePipelineHost,
  createRenderPipelineHost,
  renderPipelineL,
  textureResource,
  writeTexturePixels,
} from "./typegpu";
import {
  Vec2f,
  Vec2i,
  Vec4f,
} from "./typegpu-types";
import {
  GPUBindGroup,
  GPUBuffer,
  GPUBufferUsage,
  GPUHostOwnedDevice,
  GPUTexture,
  GPUTextureUsage,
  GPUTextureView,
  hostOwnedGPUDevice,
} from "./webgpu";
import {
  EditParams_SIZE,
  Vertex_STRIDE,
  lifeEdit_ENTRY,
  lifeEdit_LAYOUT0,
  lifeEdit_WGSL,
  lifeRender_FRAGMENT_ENTRY,
  lifeRender_LAYOUT0,
  lifeRender_TARGET_FORMAT,
  lifeRender_VERTEX_ENTRY,
  lifeRender_VERTEX_LAYOUT0,
  lifeRender_WGSL,
  lifeStep_ENTRY,
  lifeStep_LAYOUT0,
  lifeStep_WGSL,
} from "./main.typegpu";

// The grid is fixed at 128 cells per axis, and 128 divides by the workgroup size of 8.
// BRUSH_RADIUS is in cells, so the brush covers a 5 by 5 square.
const GRID_SIZE: u32 = 128;
const BRUSH_RADIUS: f32 = 2.5;
const EDIT_NONE: u32 = 0;
const EDIT_CLEAR: u32 = 1;
const EDIT_DRAW: u32 = 2;

// The vertex record. The generator emits Vertex_STRIDE from this class, so the host code
// never counts bytes.
@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// The edit uniform. `point` is in cell coordinates, not pixels, and `mode` carries one of
// the three EDIT_ constants.
@CStruct
class EditParams {
  point: Vec2f;
  mode: u32;

  constructor(point: Vec2f, mode: u32) {
    this.point = point;
    this.mode = mode;
  }
}

// The inter-stage record. The field named `position` becomes the clip-space builtin, and
// `uv` becomes location 0.
@CStruct
class Varyings {
  position: Vec4f;
  uv: Vec2f;

  constructor(position: Vec4f, uv: Vec2f) {
    this.position = position;
    this.uv = uv;
  }
}

// The step bind group. `ReadStorageTexture2d` is read-only and `StorageTexture2d` is
// write-only, so the types state that one dispatch never writes the texture it reads.
class LifeStepLayout {
  generation: ReadStorageTexture2d<R32float>;
  next: StorageTexture2d<R32float>;

  constructor(generation: ReadStorageTexture2d<R32float>, next: StorageTexture2d<R32float>) {
    this.generation = generation;
    this.next = next;
  }
}

// The edit bind group. `ReadWriteStorageTexture2d` reads and writes one texture, because the
// brush changes cells in place.
class LifeEditLayout {
  generation: ReadWriteStorageTexture2d<R32float>;
  edit: Uniform<EditParams>;

  constructor(generation: ReadWriteStorageTexture2d<R32float>, edit: Uniform<EditParams>) {
    this.generation = generation;
    this.edit = edit;
  }
}

// The render bind group. The fragment loads a texel directly, so the pass needs no sampler.
class LifeRenderLayout {
  generation: ReadStorageTexture2d<R32float>;

  constructor(generation: ReadStorageTexture2d<R32float>) {
    this.generation = generation;
  }
}

// One invocation counts the eight neighbors and writes the next state of one cell.
// TypeGPU counts an out-of-range neighbor as dead. This port wraps the grid into a torus.
function lifeStepKernel(res: LifeStepLayout, ctx: ComputeInvocation): void {
  const x: i32 = ctx.globalId.x as i32;
  const y: i32 = ctx.globalId.y as i32;
  const limit: i32 = GRID_SIZE as i32;
  const left: i32 = x > 0 ? x - 1 : limit - 1;
  const right: i32 = x + 1 < limit ? x + 1 : 0;
  const down: i32 = y > 0 ? y - 1 : limit - 1;
  const up: i32 = y + 1 < limit ? y + 1 : 0;
  const neighbors: f32 =
    res.generation.load(new Vec2i(left, down)).x
    + res.generation.load(new Vec2i(x, down)).x
    + res.generation.load(new Vec2i(right, down)).x
    + res.generation.load(new Vec2i(left, y)).x
    + res.generation.load(new Vec2i(right, y)).x
    + res.generation.load(new Vec2i(left, up)).x
    + res.generation.load(new Vec2i(x, up)).x
    + res.generation.load(new Vec2i(right, up)).x;
  // A cell is one r32float texel, so the test compares against 0.5. Upstream packs the same
  // state as r32uint.
  const alive: boolean = res.generation.load(new Vec2i(x, y)).x > 0.5;
  let next: f32 = 0.0;
  // The neighbor count is a float sum, so each rule compares against a range. Three neighbors
  // create a cell, and two keep a live one.
  if (neighbors > 2.5 && neighbors < 3.5) {
    next = 1.0;
  } else if (alive && neighbors > 1.5 && neighbors < 2.5) {
    next = 1.0;
  }
  res.next.store(new Vec2i(x, y), new Vec4f(next, 0.0, 0.0, 1.0));
}

// The edit pass runs over the whole grid and applies the current pointer or key action.
// TypeGPU draws a capsule between two pointer samples with a radius and a mode control.
// This port draws a fixed square around one pointer sample and only sets cells alive.
function lifeEditKernel(res: LifeEditLayout, ctx: ComputeInvocation): void {
  const params: EditParams = res.edit.$;
  const cell = new Vec2i(ctx.globalId.x as i32, ctx.globalId.y as i32);
  if (params.mode === EDIT_CLEAR) {
    res.generation.store(cell, new Vec4f(0.0, 0.0, 0.0, 1.0));
    return;
  }
  let dx: f32 = (ctx.globalId.x as f32) - params.point.x;
  let dy: f32 = (ctx.globalId.y as f32) - params.point.y;
  if (dx < 0.0) dx = -dx;
  if (dy < 0.0) dy = -dy;
  if (dx <= BRUSH_RADIUS && dy <= BRUSH_RADIUS) {
    res.generation.store(cell, new Vec4f(1.0, 0.0, 0.0, 1.0));
  }
}

// The vertex stage passes the corner through and derives the uv. Four corners and the strip
// topology cover the surface with two triangles.
function lifeVertex(
  res: LifeRenderLayout,
  value: Vertex,
  ctx: VertexInvocation,
): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

// The uv maps to a cell index. A uv of exactly 1.0 lands one cell past the last row, so the
// clamp pulls it back.
function lifeFragment(
  res: LifeRenderLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  let x: u32 = (input.uv.x * (GRID_SIZE as f32)) as u32;
  let y: u32 = (input.uv.y * (GRID_SIZE as f32)) as u32;
  if (x >= GRID_SIZE) x = GRID_SIZE - 1;
  if (y >= GRID_SIZE) y = GRID_SIZE - 1;
  const alive: f32 = res.generation.load(new Vec2i(x as i32, y as i32)).x;
  return new Vec4f(
    0.015 + alive * 0.88,
    0.02 + alive * 0.76,
    0.035 + alive * 0.32,
    1.0,
  );
}

// The workgroup is 8 by 8, so one workgroup covers 64 cells. 128 divides by 8, and no
// invocation falls outside the grid.
export const lifeStep: ComputePipelineSpec = computePipeline<LifeStepLayout>(
  lifeStepKernel,
  { name: "lifeStep", workgroupSize: [8, 8, 1] },
);

// The edit kernel covers the whole grid with the same workgroup size, so one dispatch count
// serves both compute pipelines.
export const lifeEdit: ComputePipelineSpec = computePipeline<LifeEditLayout>(
  lifeEditKernel,
  { name: "lifeEdit", workgroupSize: [8, 8, 1] },
);

// The topology is a triangle strip, so four vertices make two triangles. TypeGPU draws its
// display pass from a fullscreen triangle instead.
export const lifeRender: RenderPipelineSpec = renderPipelineL<
  LifeRenderLayout,
  Vertex,
  Varyings
>(lifeVertex, lifeFragment, {
  format: "bgra8unorm",
  topology: "triangle-strip",
});

// One record holds every handle that outlives `init`. TypeGPU keeps the same handles in a
// closure, and `frame` here needs one null check instead of fourteen.
class LifeState {
  device: GPUHostOwnedDevice;
  step: ComputePipeline;
  edit: ComputePipeline;
  render: RenderPipeline;
  stepAB: GPUBindGroup;
  stepBA: GPUBindGroup;
  editA: GPUBindGroup;
  editB: GPUBindGroup;
  renderA: GPUBindGroup;
  renderB: GPUBindGroup;
  vertices: GPUBuffer;
  editParams: GPUBuffer;
  generationA: GPUTexture;
  generationB: GPUTexture;
  viewA: GPUTextureView;
  viewB: GPUTextureView;

  constructor(
    device: GPUHostOwnedDevice,
    step: ComputePipeline,
    edit: ComputePipeline,
    render: RenderPipeline,
    stepAB: GPUBindGroup,
    stepBA: GPUBindGroup,
    editA: GPUBindGroup,
    editB: GPUBindGroup,
    renderA: GPUBindGroup,
    renderB: GPUBindGroup,
    vertices: GPUBuffer,
    editParams: GPUBuffer,
    generationA: GPUTexture,
    generationB: GPUTexture,
    viewA: GPUTextureView,
    viewB: GPUTextureView,
  ) {
    this.device = device;
    this.step = step;
    this.edit = edit;
    this.render = render;
    this.stepAB = stepAB;
    this.stepBA = stepBA;
    this.editA = editA;
    this.editB = editB;
    this.renderA = renderA;
    this.renderB = renderB;
    this.vertices = vertices;
    this.editParams = editParams;
    this.generationA = generationA;
    this.generationB = generationB;
    this.viewA = viewA;
    this.viewB = viewB;
  }
}

let activeState: LifeState | null = null;
let frameCount: u32 = 0;

// TypeGPU seeds the grid at random. This port writes one glider, so a reader sees the
// same motion on every run.
function gliderSeed(): Vec4f[] {
  const pixels: Vec4f[] = [];
  const center: u32 = GRID_SIZE / 2;
  let index: u32 = 0;
  while (index < GRID_SIZE * GRID_SIZE) {
    const x: u32 = index % GRID_SIZE;
    const y: u32 = index / GRID_SIZE;
    const alive: boolean =
      (x === center && y === center)
      || (x === center + 1 && y === center + 1)
      || (x + 1 === center && y === center + 2)
      || (x === center && y === center + 2)
      || (x === center + 1 && y === center + 2);
    pixels.push(new Vec4f(alive ? 1.0 : 0.0, 0.0, 0.0, 1.0));
    index += 1;
  }
  return pixels;
}

// A storage texture starts undefined, so the second generation takes an explicit zero fill.
function emptyGeneration(): Vec4f[] {
  const pixels: Vec4f[] = [];
  let index: u32 = 0;
  while (index < GRID_SIZE * GRID_SIZE) {
    pixels.push(new Vec4f(0.0, 0.0, 0.0, 1.0));
    index += 1;
  }
  return pixels;
}

// `init` runs once, after the host configures the surface. It creates every long-lived
// resource. The instance and the device stay with the host.
export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  // The generator pins the target format into the pipeline. A mismatch with the host surface
  // fails here, not inside pipeline creation.
  if (format !== lifeRender_TARGET_FORMAT) {
    print(`FAIL format expected=${lifeRender_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The wrapper adapts the host handles to the API layer. It carries no `dispose`, because
  // the host owns the device.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // The vertex buffer holds the four corners of the strip. Vertex_STRIDE keeps the size right
  // when the schema changes.
  const vertices = hostDevice.createBuffer({
    label: "life-vertices",
    size: (Vertex_STRIDE * 4) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // One uniform buffer serves both edit bind groups. COPY_DST admits the write that a frame
  // with input makes.
  const editParams = hostDevice.createBuffer({
    label: "life-edit",
    size: EditParams_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  // STORAGE_BINDING covers every load and store in the three shaders, and COPY_DST covers the
  // seed write below.
  const textureUsage: u64 = GPUTextureUsage.STORAGE_BINDING + GPUTextureUsage.COPY_DST;
  // Two textures hold the generations and swap roles each frame. The r32float format keeps one
  // cell per texel.
  const generationA = hostDevice.createTexture({
    label: "life-generation-a",
    size: { width: GRID_SIZE, height: GRID_SIZE },
    format: "r32float",
    usage: textureUsage,
  });
  const generationB = hostDevice.createTexture({
    label: "life-generation-b",
    size: { width: GRID_SIZE, height: GRID_SIZE },
    format: "r32float",
    usage: textureUsage,
  });
  // A bind group holds a view, not a texture, so each generation needs a view handle of its own.
  const viewA = generationA.createView();
  const viewB = generationB.createView();
  // The four corners run in strip order: lower left, lower right, upper left, upper right.
  const vertexValues: FixedArray<Vertex, 4> = [
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(1.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 1.0)),
    new Vertex(new Vec2f(1.0, 1.0)),
  ];
  // The queue wrapper is a handle. `using` disposes it at the end of `init`, and `frame`
  // takes a fresh one.
  using queue = hostDevice.queue();
  // `Context.bytesOf` lays out the values with the generated C layout, so the bytes match the
  // WGSL that the generator emits.
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 4>>(vertexValues));
  // The helper pads each row to the 256-byte row alignment that a texture write needs.
  writeTexturePixels(queue, generationA, gliderSeed(), GRID_SIZE, GRID_SIZE);
  writeTexturePixels(queue, generationB, emptyGeneration(), GRID_SIZE, GRID_SIZE);

  // One error scope covers all three pipeline creations. The layers return the failure as a
  // value, so a `null` check replaces an exception.
  hostDevice.pushErrorScope("validation");
  // Each compute call takes the generated WGSL, the entry name, the bind group layout, and the
  // workgroup size that the declaration above names.
  const stepPipeline = createComputePipelineHost(
    hostDevice,
    lifeStep_WGSL,
    lifeStep_ENTRY,
    [lifeStep_LAYOUT0],
    [8, 8, 1],
  );
  const editPipeline = createComputePipelineHost(
    hostDevice,
    lifeEdit_WGSL,
    lifeEdit_ENTRY,
    [lifeEdit_LAYOUT0],
    [8, 8, 1],
  );
  // The render call adds the two entry names and the vertex layout. No shader text is built here.
  const renderPipeline = createRenderPipelineHost(
    hostDevice,
    lifeRender_WGSL,
    lifeRender_VERTEX_ENTRY,
    lifeRender_FRAGMENT_ENTRY,
    [lifeRender_LAYOUT0],
    [lifeRender_VERTEX_LAYOUT0],
    lifeRender,
  );
  const validationError = hostDevice.popErrorScope();
  // The error path disposes every handle that the failed run already created, because no
  // finalizer runs later.
  if (validationError !== null) {
    renderPipeline.dispose();
    editPipeline.dispose();
    stepPipeline.dispose();
    viewB.dispose();
    viewA.dispose();
    generationB.dispose();
    generationA.dispose();
    editParams.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }

  // Three native layouts come from the three pipelines. Every bind group reads its layout at
  // creation, so `using` releases all three at the end of `init`.
  using stepLayout = stepPipeline.bindGroupLayout(0);
  using editLayout = editPipeline.bindGroupLayout(0);
  using renderLayout = renderPipeline.bindGroupLayout(0);
  // Six bind groups cover both directions of the swap for each pass. The frame then picks
  // three of them and builds none.
  const stepAB = createBindGroupHost(hostDevice, stepLayout, lifeStep_LAYOUT0, [
    textureResource(viewA),
    textureResource(viewB),
  ]);
  const stepBA = createBindGroupHost(hostDevice, stepLayout, lifeStep_LAYOUT0, [
    textureResource(viewB),
    textureResource(viewA),
  ]);
  const editA = createBindGroupHost(hostDevice, editLayout, lifeEdit_LAYOUT0, [
    textureResource(viewA),
    bufferResource(editParams),
  ]);
  const editB = createBindGroupHost(hostDevice, editLayout, lifeEdit_LAYOUT0, [
    textureResource(viewB),
    bufferResource(editParams),
  ]);
  const renderA = createBindGroupHost(hostDevice, renderLayout, lifeRender_LAYOUT0, [
    textureResource(viewA),
  ]);
  const renderB = createBindGroupHost(hostDevice, renderLayout, lifeRender_LAYOUT0, [
    textureResource(viewB),
  ]);
  // The state moves to module scope only after every step passes. A failed `init` leaves
  // `activeState` null, and `frame` returns at once.
  activeState = new LifeState(
    hostDevice,
    stepPipeline,
    editPipeline,
    renderPipeline,
    stepAB,
    stepBA,
    editA,
    editB,
    renderA,
    renderB,
    vertices,
    editParams,
    generationA,
    generationB,
    viewA,
    viewB,
  );
}

// The host calls `frame` once per presented frame. `view` is the swapchain view the host
// owns, and `width` and `height` are surface pixels.
export function frame(
  view: SubscriptTypegpuTextureView,
  width: u32,
  height: u32,
  key: u32,
  pointerX: f32,
  pointerY: f32,
  buttons: u32,
): void {
  // A null state means `init` failed or never ran. The frame returns, because the layers
  // report failure as a value.
  if (activeState === null) return;
  const active = activeState;
  // The host passes the key as a Unicode scalar, and 48 is the `0` key. Bit 0 of `buttons`
  // is the left button, and the host reports -1, -1 until the pointer first enters the window.
  // Grid row 0 sits at the bottom of the surface, so the pointer Y is flipped.
  let editMode: u32 = EDIT_NONE;
  let editPoint = new Vec2f(0.0, 0.0);
  if (key === 48) {
    editMode = EDIT_CLEAR;
  } else if ((buttons & 1) !== 0 && pointerX >= 0.0 && pointerY >= 0.0) {
    editMode = EDIT_DRAW;
    editPoint = new Vec2f(
      (pointerX / (width as f32)) * ((GRID_SIZE - 1) as f32),
      (1.0 - pointerY / (height as f32)) * ((GRID_SIZE - 1) as f32),
    );
  }
  using queue = active.device.queue();
  // The frame parity picks the step source and target. The edit pass and the render pass
  // both use the step target, so an edit lands on the grid the frame displays.
  const readsA: boolean = frameCount % 2 === 0;
  const stepGroup: GPUBindGroup = readsA ? active.stepAB : active.stepBA;
  const editGroup: GPUBindGroup = readsA ? active.editB : active.editA;
  const displayGroup: GPUBindGroup = readsA ? active.renderB : active.renderA;
  // One encoder records the step pass, the edit pass, and the render pass. The device runs
  // them in the recorded order.
  using encoder = active.device.createCommandEncoderDefault();
  // `dispatch` takes workgroup counts, not thread counts. 128 cells over a workgroup of 8 give
  // 16 workgroups per axis, with no partial workgroup.
  active.step.dispatch(encoder, [stepGroup], GRID_SIZE / 8, GRID_SIZE / 8, 1);
  // The edit pass runs only on a frame with input. It runs after the step pass, so the brush
  // survives into the next generation.
  if (editMode !== EDIT_NONE) {
    queue.writeBuffer(
      active.editParams,
      0,
      Context.bytesOf<EditParams>(new EditParams(editPoint, editMode)),
    );
    active.edit.dispatch(encoder, [editGroup], GRID_SIZE / 8, GRID_SIZE / 8, 1);
  }
  // The host owns the swapchain view. The wrapper adds no ownership, and `shutdown` never
  // disposes it.
  const target = new GPUTextureView(view);
  // The pass clears to the dead-cell color, so no pixel of an earlier frame survives a resize.
  using renderPass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.015, g: 0.02, b: 0.035, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  // The viewport and the scissor follow the current surface size, because the host resizes the
  // swapchain without a new pipeline.
  renderPass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  renderPass.setScissorRect(0, 0, width, height);
  // `bind` sets the pipeline, the bind groups, and the vertex buffers in one call. TypeGPU
  // spells the same step as `.with(bindGroup)`.
  active.render.bind(renderPass, [displayGroup], [active.vertices]);
  // Four vertices and the strip topology give the two triangles that cover the surface.
  renderPass.draw(4);
  renderPass.end();
  // `finishDefault` closes the encoder, and `submit` hands the command buffer to the queue.
  // The host presents the surface after `frame` returns.
  using command = encoder.finishDefault();
  queue.submit([command]);
  // The counter advances after the submit, so the next frame reads the texture this frame wrote.
  frameCount += 1;
}

// The host calls `shutdown` once, before it releases the device. The bind groups go first,
// because they name the views and the buffers.
export function shutdown(): void {
  if (activeState === null) return;
  const active = activeState;
  active.renderB.dispose();
  active.renderA.dispose();
  active.editB.dispose();
  active.editA.dispose();
  active.stepBA.dispose();
  active.stepAB.dispose();
  active.viewB.dispose();
  active.viewA.dispose();
  active.generationB.dispose();
  active.generationA.dispose();
  active.editParams.dispose();
  active.vertices.dispose();
  active.render.dispose();
  active.edit.dispose();
  active.step.dispose();
  // The null assignment makes a second `shutdown` call safe.
  activeState = null;
  frameCount = 0;
}
