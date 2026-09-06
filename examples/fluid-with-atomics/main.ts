// example: fluid-with-atomics
// Moves water down and sideways through two storage grids that swap roles under a pointer brush.
// TypeGPU picks the brush from a select and erases with a right click. This port maps keys 1, 2,
// and 0 to water, wall, and erase, and drops the source, drain, pressure, and upward flow rules.
// Ported from TypeGPU's fluid-with-atomics example (https://github.com/software-mansion/TypeGPU).

import {
  ComputeInvocation,
  ComputePipeline,
  ComputePipelineSpec,
  FragmentInvocation,
  MutStorage,
  RenderPipeline,
  RenderPipelineSpec,
  Storage,
  Uniform,
  VertexInvocation,
  bufferResource,
  computePipeline,
  createBindGroupHost,
  createComputePipelineHost,
  createRenderPipelineHost,
  renderPipelineL,
} from "./typegpu";
import {
  AtomicI32,
  AtomicU32,
  Vec2f,
  Vec4f,
} from "./typegpu-types";
import {
  sdLine,
} from "./typegpu-sdf";
import {
  GPUBindGroup,
  GPUBuffer,
  GPUBufferUsage,
  GPUHostOwnedDevice,
  GPUTextureView,
  hostOwnedGPUDevice,
} from "./webgpu";
import {
  BrushParams_SIZE,
  Vertex_STRIDE,
  WaterCell_STRIDE,
  atomicBrush_ENTRY,
  atomicBrush_LAYOUT0,
  atomicBrush_WGSL,
  atomicFlow_ENTRY,
  atomicFlow_LAYOUT0,
  atomicFlow_WGSL,
  atomicFinalize_ENTRY,
  atomicFinalize_LAYOUT0,
  atomicFinalize_WGSL,
  atomicFluidRender_FRAGMENT_ENTRY,
  atomicFluidRender_LAYOUT0,
  atomicFluidRender_TARGET_FORMAT,
  atomicFluidRender_VERTEX_ENTRY,
  atomicFluidRender_VERTEX_LAYOUT0,
  atomicFluidRender_WGSL,
} from "./main.typegpu";

// The grid commits to 64 squared cells, and a full cell holds 1024 water. `WALL_LEVEL` is 2 to
// the 31st power, far above any water level, so one `u32` carries the level and the wall mark.
// TypeGPU keeps a separate flags buffer with four cell kinds.
const GRID_SIZE: u32 = 64;
const CELL_COUNT: u32 = GRID_SIZE * GRID_SIZE;
const MAX_WATER: u32 = 1024;
const WALL_LEVEL: u32 = 2147483648;
// The caps bound the water one cell moves in one frame, so a tall column drains over many
// frames. TypeGPU derives the same bound from a viscosity slider.
const GRAVITY_STEP: u32 = 96;
const SIDE_STEP: u32 = 24;
const BRUSH_WATER: u32 = 192;
const BRUSH_RADIUS: f32 = 2.4;
const BRUSH_ERASE: u32 = 0;
const BRUSH_WATER_MODE: u32 = 1;
const BRUSH_WALL: u32 = 2;

// One corner of the render strip. The generator derives `Vertex_STRIDE` from this class.
@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// The same cell buffer carries three declarations. `WaterCell` is the atomic `u32` the brush
// edits, `WaterLevel` is the plain read the flow pass takes, and `WaterDelta` is the change.
// Each layout picks the declaration its pass needs, and the bytes never change shape.
@CStruct
class WaterCell {
  level: AtomicU32;

  constructor(level: AtomicU32) {
    this.level = level;
  }
}

@CStruct
class WaterLevel {
  level: u32;

  constructor(level: u32) {
    this.level = level;
  }
}

@CStruct
class WaterDelta {
  level: AtomicI32;

  constructor(level: AtomicI32) {
    this.level = level;
  }
}

// The brush segment in cell coordinates, with the mode and an active flag. One frame writes it
// once. TypeGPU passes a radius and a cell kind and erases from the right mouse button.
@CStruct
class BrushParams {
  previous: Vec2f;
  current: Vec2f;
  mode: u32;
  active: u32;

  constructor(previous: Vec2f, current: Vec2f, mode: u32, active: u32) {
    this.previous = previous;
    this.current = current;
    this.mode = mode;
    this.active = active;
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

// The flow layout reads the current levels and accumulates signed changes into the next buffer.
// No pass writes the source, so a plain read never sees a partial update.
class AtomicFluidLayout {
  current!: Storage<WaterLevel>;
  next!: MutStorage<WaterDelta>;
}

// The finalize layout binds the same pair. Its pass turns the accumulated changes into levels.
class AtomicFinalizeLayout {
  current!: Storage<WaterLevel>;
  next!: MutStorage<WaterDelta>;
}

// The brush layout writes the new state directly, so a stroke lands on the buffer the frame
// displays.
class AtomicBrushLayout {
  cells!: MutStorage<WaterCell>;
  brush!: Uniform<BrushParams>;
}

// The render layout binds the cells as mutable storage, because an atomic load needs that
// access.
class AtomicFluidRenderLayout {
  cells!: MutStorage<WaterCell>;
}

function minU32(left: u32, right: u32): u32 {
  return left < right ? left : right;
}

// Each invocation reads current and atomically accumulates signed changes in the cleared next buffer.
// A finalize pass clamps next, the brush follows, and the frame swaps the buffer roles.
// Four neighbors can change one cell in the same dispatch, so every change needs an atomic.
function atomicFlowKernel(res: AtomicFluidLayout, ctx: ComputeInvocation): void {
  const x: u32 = ctx.globalId.x;
  const y: u32 = ctx.globalId.y;
  const index: u32 = y * GRID_SIZE + x;
  let level: u32 = res.current[index].level;
  if (level === 0 || level >= WALL_LEVEL) return;

  // Gravity first. The cell gives the space below what it can take, up to the per-frame cap.
  if (y > 0) {
    const belowIndex: u32 = index - GRID_SIZE;
    const belowLevel: u32 = res.current[belowIndex].level;
    if (belowLevel < MAX_WATER) {
      const gravity: u32 = minU32(
        level,
        minU32(MAX_WATER - belowLevel, GRAVITY_STEP),
      );
      if (gravity > 0) {
        res.next[index].level.sub(gravity as i32);
        res.next[belowIndex].level.add(gravity as i32);
        level -= gravity;
      }
    }
  }
  if (level === 0) return;

  // Then one sideways move toward the lower of the two horizontal neighbors.
  let targetIndex: u32 = index;
  let targetLevel: u32 = level;
  if (x > 0) {
    const leftIndex: u32 = index - 1;
    const leftLevel: u32 = res.current[leftIndex].level;
    if (leftLevel < targetLevel) {
      targetIndex = leftIndex;
      targetLevel = leftLevel;
    }
  }
  if (x + 1 < GRID_SIZE) {
    const rightIndex: u32 = index + 1;
    const rightLevel: u32 = res.current[rightIndex].level;
    if (rightLevel < targetLevel) {
      targetIndex = rightIndex;
      targetLevel = rightLevel;
    }
  }
  // A difference of one or less stops the move, so a flat surface never swaps water forever.
  // The quarter share levels the pair over several frames and never overshoots.
  if (targetIndex !== index && level > targetLevel + 1) {
    const sideways: u32 = minU32((level - targetLevel) / 4, SIDE_STEP);
    if (sideways > 0) {
      res.next[index].level.sub(sideways as i32);
      res.next[targetIndex].level.add(sideways as i32);
    }
  }
}

// The second pass turns changes into levels. A wall keeps its marker, and every other cell
// clamps the sum of its old level and its change into the 0 to `MAX_WATER` range.
// The signed store of the lowest `i32` reads back as `WALL_LEVEL` through the unsigned view.
function atomicFinalizeKernel(res: AtomicFinalizeLayout, ctx: ComputeInvocation): void {
  const index: u32 = ctx.globalId.y * GRID_SIZE + ctx.globalId.x;
  const current: u32 = res.current[index].level;
  if (current >= WALL_LEVEL) {
    res.next[index].level.store(-2147483648);
    return;
  }
  const changed: i32 = (current as i32) + res.next[index].level.load();
  const positive: u32 = changed > 0 ? changed as u32 : 0;
  res.next[index].level.store(minU32(positive, MAX_WATER) as i32);
}

// The third pass paints one stroke. `sdLine` gives the distance from the cell to the segment
// between the last two pointer positions, so a fast drag leaves no gap.
// TypeGPU uses the same helper and takes a point distance when the segment has no length.
function atomicBrushKernel(res: AtomicBrushLayout, ctx: ComputeInvocation): void {
  const x: u32 = ctx.globalId.x;
  const y: u32 = ctx.globalId.y;
  const index: u32 = y * GRID_SIZE + x;
  const brush: BrushParams = res.brush.$;
  if (brush.active === 0) return;
  const point = new Vec2f(x as f32, y as f32);
  if (sdLine(point, brush.previous, brush.current) > BRUSH_RADIUS) return;
  if (brush.mode === BRUSH_WALL) {
    res.cells[index].level.store(WALL_LEVEL);
    return;
  }
  if (brush.mode === BRUSH_ERASE) {
    res.cells[index].level.store(0);
    return;
  }
  const brushLevel: u32 = res.cells[index].level.load();
  if (brushLevel < WALL_LEVEL && brushLevel < MAX_WATER) {
    res.cells[index].level.add(minU32(BRUSH_WATER, MAX_WATER - brushLevel));
  }
}

// TypeGPU draws one full-screen triangle and picks its three corners from the vertex index.
// This port draws a four-corner strip from a typed vertex buffer.
function atomicFluidVertex(
  res: AtomicFluidRenderLayout,
  value: Vertex,
  ctx: VertexInvocation,
): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

// The fragment maps its uv to the nearest cell. A wall paints stone gray, and the water level
// drives a blue ramp. TypeGPU maps its level through a logistic curve and paints four kinds.
function atomicFluidFragment(
  res: AtomicFluidRenderLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  let x: u32 = (input.uv.x * (GRID_SIZE as f32)) as u32;
  let y: u32 = (input.uv.y * (GRID_SIZE as f32)) as u32;
  if (x >= GRID_SIZE) x = GRID_SIZE - 1;
  if (y >= GRID_SIZE) y = GRID_SIZE - 1;
  const level: u32 = res.cells[y * GRID_SIZE + x].level.load();
  if (level >= WALL_LEVEL) {
    return new Vec4f(0.24, 0.22, 0.20, 1.0);
  }
  let amount: f32 = (level as f32) / (MAX_WATER as f32);
  if (amount > 1.0) amount = 1.0;
  return new Vec4f(
    0.015 + amount * 0.05,
    0.025 + amount * 0.34,
    0.055 + amount * 0.80,
    1.0,
  );
}

// The four declarations name the kernel, the layout type, and the workgroup size. The generator
// reads them ahead of the run and emits the WGSL, the entry names, and the layout facts.
// TypeGPU builds the same WGSL at run time from the kernel function.
export const atomicFlow: ComputePipelineSpec = computePipeline<AtomicFluidLayout>(
  atomicFlowKernel,
  { name: "atomicFlow", workgroupSize: [8, 8, 1] },
);

export const atomicFinalize: ComputePipelineSpec = computePipeline<AtomicFinalizeLayout>(
  atomicFinalizeKernel,
  { name: "atomicFinalize", workgroupSize: [8, 8, 1] },
);

export const atomicBrush: ComputePipelineSpec = computePipeline<AtomicBrushLayout>(
  atomicBrushKernel,
  { name: "atomicBrush", workgroupSize: [8, 8, 1] },
);

export const atomicFluidRender: RenderPipelineSpec = renderPipelineL<
  AtomicFluidRenderLayout,
  Vertex,
  Varyings
>(atomicFluidVertex, atomicFluidFragment, {
  format: "bgra8unorm",
  topology: "triangle-strip",
});

// The host calls `init`, `frame`, and `shutdown` as separate entries, so every handle lives here.
// This example owns each handle from creation until `shutdown` releases it. TypeGPU frees the
// same resources through garbage collection and `root.destroy()`.
let activeDevice: GPUHostOwnedDevice | null = null;
let activeFlow: ComputePipeline | null = null;
let activeFinalize: ComputePipeline | null = null;
let activeBrushPipeline: ComputePipeline | null = null;
let activeRender: RenderPipeline | null = null;
let activeFlowGroupAB: GPUBindGroup | null = null;
let activeFlowGroupBA: GPUBindGroup | null = null;
let activeFinalizeGroupAB: GPUBindGroup | null = null;
let activeFinalizeGroupBA: GPUBindGroup | null = null;
let activeBrushGroupA: GPUBindGroup | null = null;
let activeBrushGroupB: GPUBindGroup | null = null;
let activeRenderGroupA: GPUBindGroup | null = null;
let activeRenderGroupB: GPUBindGroup | null = null;
let activeVertices: GPUBuffer | null = null;
let activeCellsA: GPUBuffer | null = null;
let activeCellsB: GPUBuffer | null = null;
let activeBrush: GPUBuffer | null = null;
let brushMode: u32 = BRUSH_WATER_MODE;
let previousPointerX: f32 = -1.0;
let previousPointerY: f32 = -1.0;
let frameCount: u32 = 0;

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  // The pipeline declares its target format literally. A surface with another format ends the
  // example before any draw.
  if (format !== atomicFluidRender_TARGET_FORMAT) {
    print(`FAIL format expected=${atomicFluidRender_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The host owns the device and the instance. The wrapper carries no `dispose`, so this example
  // never releases what it did not create.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // The four strip corners in clip space, in the order the triangle-strip topology needs.
  const vertexValues: FixedArray<Vertex, 4> = [
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(1.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 1.0)),
    new Vertex(new Vec2f(1.0, 1.0)),
  ];
  // Buffer sizes come from the generated stride and size constants, never from a hand count.
  const vertices = hostDevice.createBuffer({
    label: "atomic-fluid-vertices",
    size: (Vertex_STRIDE * 4) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // The two cell grids alternate between source and target. Both carry the same size, so a bind
  // group can pair them in either direction.
  const cellsA = hostDevice.createBuffer({
    label: "atomic-fluid-cells-a",
    size: (WaterCell_STRIDE * CELL_COUNT) as u64,
    usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
  });
  const cellsB = hostDevice.createBuffer({
    label: "atomic-fluid-cells-b",
    size: (WaterCell_STRIDE * CELL_COUNT) as u64,
    usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
  });
  // The brush uniform. One frame writes it once, and the brush pass reads it in every cell.
  const brush = hostDevice.createBuffer({
    label: "atomic-fluid-brush",
    size: BrushParams_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });

  // The initial scene walls the floor and both sides, adds a shelf across the middle, and fills
  // the rows below 14 with water. `Context.bytesInto` writes one cell in place at its offset.
  const initialCells: u8[] = [];
  let initialByte: u32 = 0;
  while (initialByte < WaterCell_STRIDE * CELL_COUNT) {
    initialCells.push(0);
    initialByte += 1;
  }
  for (let index: u32 = 0; index < CELL_COUNT; index += 1) {
    const x: u32 = index % GRID_SIZE;
    const y: u32 = index / GRID_SIZE;
    const border: boolean = x === 0 || x + 1 === GRID_SIZE || y === 0;
    const shelf: boolean = y === 18 && x > 10 && x < 27;
    let level: u32 = 0;
    if (border || shelf) {
      level = WALL_LEVEL;
    } else if (y < 14) {
      level = 640;
    }
    Context.bytesInto<WaterCell>(
      new WaterCell(new AtomicU32(level)),
      initialCells,
      index * WaterCell_STRIDE,
    );
  }
  // Both grids start from the same bytes, so either one can act as the source on the first
  // frame. Queue writes land before the commands that a later submit carries.
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 4>>(vertexValues));
  queue.writeBuffer(cellsA, 0, initialCells);
  queue.writeBuffer(cellsB, 0, initialCells);
  // The idle brush stays inactive. The two points differ, so `sdLine` never takes a zero-length
  // segment.
  const idlePoint = new Vec2f(0.0, 0.0);
  queue.writeBuffer(
    brush,
    0,
    Context.bytesOf<BrushParams>(
      new BrushParams(idlePoint, new Vec2f(0.001, 0.0), brushMode, 0),
    ),
  );

  // The error scope catches a pipeline creation failure. TypeGPU rejects a promise. Here the pop
  // returns a value, so this example releases what it created and prints the first error line.
  hostDevice.pushErrorScope("validation");
  // Each compute pipeline takes the generated WGSL text, the entry name, the layout facts, and
  // the workgroup size. That size must equal the size in the declaration above.
  const flowPipeline = createComputePipelineHost(
    hostDevice,
    atomicFlow_WGSL,
    atomicFlow_ENTRY,
    [atomicFlow_LAYOUT0],
    [8, 8, 1],
  );
  const finalizePipeline = createComputePipelineHost(
    hostDevice,
    atomicFinalize_WGSL,
    atomicFinalize_ENTRY,
    [atomicFinalize_LAYOUT0],
    [8, 8, 1],
  );
  const brushPipeline = createComputePipelineHost(
    hostDevice,
    atomicBrush_WGSL,
    atomicBrush_ENTRY,
    [atomicBrush_LAYOUT0],
    [8, 8, 1],
  );
  // The render pipeline also takes the vertex buffer layout and the declaration, which carries
  // the target format and the topology.
  const renderPipeline = createRenderPipelineHost(
    hostDevice,
    atomicFluidRender_WGSL,
    atomicFluidRender_VERTEX_ENTRY,
    atomicFluidRender_FRAGMENT_ENTRY,
    [atomicFluidRender_LAYOUT0],
    [atomicFluidRender_VERTEX_LAYOUT0],
    atomicFluidRender,
  );
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    renderPipeline.dispose();
    brushPipeline.dispose();
    finalizePipeline.dispose();
    flowPipeline.dispose();
    brush.dispose();
    cellsB.dispose();
    cellsA.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }

  // The pipeline reports the layout WebGPU built for group 0. Each call returns a new handle, and
  // the bind groups below need it only at creation.
  using flowLayout = flowPipeline.bindGroupLayout(0);
  using finalizeLayout = finalizePipeline.bindGroupLayout(0);
  using brushLayout = brushPipeline.bindGroupLayout(0);
  using renderLayout = renderPipeline.bindGroupLayout(0);
  // Eight bind groups cover both buffer directions for the flow and the finalize pass, and both
  // choices for the brush and the render pass. The resource order follows the layout fields.
  const flowGroupAB = createBindGroupHost(
    hostDevice,
    flowLayout,
    atomicFlow_LAYOUT0,
    [bufferResource(cellsA), bufferResource(cellsB)],
  );
  const flowGroupBA = createBindGroupHost(
    hostDevice,
    flowLayout,
    atomicFlow_LAYOUT0,
    [bufferResource(cellsB), bufferResource(cellsA)],
  );
  const finalizeGroupAB = createBindGroupHost(
    hostDevice,
    finalizeLayout,
    atomicFinalize_LAYOUT0,
    [bufferResource(cellsA), bufferResource(cellsB)],
  );
  const finalizeGroupBA = createBindGroupHost(
    hostDevice,
    finalizeLayout,
    atomicFinalize_LAYOUT0,
    [bufferResource(cellsB), bufferResource(cellsA)],
  );
  const brushGroupA = createBindGroupHost(
    hostDevice,
    brushLayout,
    atomicBrush_LAYOUT0,
    [bufferResource(cellsA), bufferResource(brush)],
  );
  const brushGroupB = createBindGroupHost(
    hostDevice,
    brushLayout,
    atomicBrush_LAYOUT0,
    [bufferResource(cellsB), bufferResource(brush)],
  );
  const renderGroupA = createBindGroupHost(
    hostDevice,
    renderLayout,
    atomicFluidRender_LAYOUT0,
    [bufferResource(cellsA)],
  );
  const renderGroupB = createBindGroupHost(
    hostDevice,
    renderLayout,
    atomicFluidRender_LAYOUT0,
    [bufferResource(cellsB)],
  );
  // The handles reach module state only after every creation succeeds, so a failed init leaves
  // no partial state behind.
  activeDevice = hostDevice;
  activeFlow = flowPipeline;
  activeFinalize = finalizePipeline;
  activeBrushPipeline = brushPipeline;
  activeRender = renderPipeline;
  activeFlowGroupAB = flowGroupAB;
  activeFlowGroupBA = flowGroupBA;
  activeFinalizeGroupAB = finalizeGroupAB;
  activeFinalizeGroupBA = finalizeGroupBA;
  activeBrushGroupA = brushGroupA;
  activeBrushGroupB = brushGroupB;
  activeRenderGroupA = renderGroupA;
  activeRenderGroupB = renderGroupB;
  activeVertices = vertices;
  activeCellsA = cellsA;
  activeCellsB = cellsB;
  activeBrush = brush;
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
  // The frame copies module state into locals and checks each handle. An empty handle means init
  // failed, and the frame ends without a draw. TypeGPU reports the same failure as an exception.
  const device = activeDevice;
  const flowPipeline = activeFlow;
  const finalizePipeline = activeFinalize;
  const brushPipeline = activeBrushPipeline;
  const renderPipeline = activeRender;
  const flowGroupAB = activeFlowGroupAB;
  const flowGroupBA = activeFlowGroupBA;
  const finalizeGroupAB = activeFinalizeGroupAB;
  const finalizeGroupBA = activeFinalizeGroupBA;
  const brushGroupA = activeBrushGroupA;
  const brushGroupB = activeBrushGroupB;
  const renderGroupA = activeRenderGroupA;
  const renderGroupB = activeRenderGroupB;
  const vertices = activeVertices;
  const cellsA = activeCellsA;
  const cellsB = activeCellsB;
  const brush = activeBrush;
  if (device === null) return;
  if (flowPipeline === null) return;
  if (finalizePipeline === null) return;
  if (brushPipeline === null) return;
  if (renderPipeline === null) return;
  if (flowGroupAB === null) return;
  if (flowGroupBA === null) return;
  if (finalizeGroupAB === null) return;
  if (finalizeGroupBA === null) return;
  if (brushGroupA === null) return;
  if (brushGroupB === null) return;
  if (renderGroupA === null) return;
  if (renderGroupB === null) return;
  if (vertices === null) return;
  if (cellsA === null) return;
  if (cellsB === null) return;
  if (brush === null) return;

  // Key 49, key 50, and key 48 are `1`, `2`, and `0`. TypeGPU picks the same brush from a select
  // and erases while the right button is down.
  if (key === 49) brushMode = BRUSH_WATER_MODE;
  if (key === 50) brushMode = BRUSH_WALL;
  if (key === 48) brushMode = BRUSH_ERASE;

  // The brush needs a segment. A release forgets the previous point, so the first sample after a
  // press starts and ends the segment on the same cell.
  let currentX: f32 = 0.0;
  let currentY: f32 = 0.0;
  let previousX: f32 = 0.0;
  let previousY: f32 = 0.0;
  let brushActive: u32 = 0;
  const primaryDown: boolean = (buttons & 1) !== 0;
  if (!primaryDown) {
    previousPointerX = -1.0;
    previousPointerY = -1.0;
  }
  if (primaryDown && pointerX >= 0.0 && pointerY >= 0.0) {
    const gridMax: f32 = (GRID_SIZE - 1) as f32;
    currentX = (pointerX / (width as f32)) * gridMax;
    // Surface y grows down, while the rendered grid y grows up.
    currentY = (1.0 - pointerY / (height as f32)) * gridMax;
    if (currentX < 0.0) currentX = 0.0;
    if (currentX > gridMax) currentX = gridMax;
    if (currentY < 0.0) currentY = 0.0;
    if (currentY > gridMax) currentY = gridMax;
    previousX = previousPointerX >= 0.0 ? previousPointerX : currentX;
    previousY = previousPointerY >= 0.0 ? previousPointerY : currentY;
    // A zero-length segment has no direction, so a still pointer moves the start of the segment
    // by one thousandth of a cell.
    if (previousX === currentX && previousY === currentY) {
      previousX = currentX > 0.001 ? currentX - 0.001 : currentX + 0.001;
    }
    brushActive = 1;
    previousPointerX = currentX;
    previousPointerY = currentY;
  }

  // Frame parity names the source and the target. The brush and the render group both take the
  // target, so a stroke lands on the state the frame displays.
  const useAB: boolean = frameCount % 2 === 0;
  const nextCells: GPUBuffer = useAB ? cellsB : cellsA;
  const flowGroup: GPUBindGroup = useAB ? flowGroupAB : flowGroupBA;
  const finalizeGroup: GPUBindGroup = useAB ? finalizeGroupAB : finalizeGroupBA;
  const brushGroup: GPUBindGroup = useAB ? brushGroupB : brushGroupA;
  const renderGroup: GPUBindGroup = useAB ? renderGroupB : renderGroupA;
  frameCount += 1;

  // The brush write reaches the device before the submit below, so the stroke of this frame uses
  // the pointer of this frame.
  using queue = device.queue();
  queue.writeBuffer(
    brush,
    0,
    Context.bytesOf<BrushParams>(new BrushParams(
      new Vec2f(previousX, previousY),
      new Vec2f(currentX, currentY),
      brushMode,
      brushActive,
    )),
  );
  // The clear starts the target at zero, because the flow pass accumulates changes into it.
  // Each dispatch records its own compute pass, and the passes run in record order.
  // The counts are workgroups, so 64 cells over a workgroup of 8 need eight groups per axis.
  using encoder = device.createCommandEncoderDefault();
  encoder.clearBuffer(nextCells, 0, (WaterCell_STRIDE * CELL_COUNT) as u64);
  flowPipeline.dispatch(encoder, [flowGroup], GRID_SIZE / 8, GRID_SIZE / 8, 1);
  finalizePipeline.dispatch(encoder, [finalizeGroup], GRID_SIZE / 8, GRID_SIZE / 8, 1);
  brushPipeline.dispatch(encoder, [brushGroup], GRID_SIZE / 8, GRID_SIZE / 8, 1);
  // The host owns the presented view, so this example wraps it and releases nothing.
  const target = new GPUTextureView(view);
  using renderPass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.01, g: 0.015, b: 0.03, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  // The surface size changes when the window resizes, so both rectangles follow the frame size.
  renderPass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  renderPass.setScissorRect(0, 0, width, height);
  // Four vertices draw the strip, and the render group selects the grid the brush pass wrote.
  renderPipeline.bind(renderPass, [renderGroup], [vertices]);
  renderPass.draw(4);
  renderPass.end();
  // The command buffer reaches the device queue after the encoder finishes. The host presents
  // the surface after this call returns.
  using command = encoder.finishDefault();
  queue.submit([command]);
}

// The host calls this once before it releases the device. This example disposes in reverse
// creation order, so a bind group never outlives the buffers it names.
// TypeGPU releases the same resources with one `root.destroy()` call.
export function shutdown(): void {
  if (activeRenderGroupB !== null) activeRenderGroupB.dispose();
  if (activeRenderGroupA !== null) activeRenderGroupA.dispose();
  if (activeBrushGroupB !== null) activeBrushGroupB.dispose();
  if (activeBrushGroupA !== null) activeBrushGroupA.dispose();
  if (activeFinalizeGroupBA !== null) activeFinalizeGroupBA.dispose();
  if (activeFinalizeGroupAB !== null) activeFinalizeGroupAB.dispose();
  if (activeFlowGroupBA !== null) activeFlowGroupBA.dispose();
  if (activeFlowGroupAB !== null) activeFlowGroupAB.dispose();
  if (activeBrush !== null) activeBrush.dispose();
  if (activeCellsB !== null) activeCellsB.dispose();
  if (activeCellsA !== null) activeCellsA.dispose();
  if (activeVertices !== null) activeVertices.dispose();
  if (activeRender !== null) activeRender.dispose();
  if (activeBrushPipeline !== null) activeBrushPipeline.dispose();
  if (activeFinalize !== null) activeFinalize.dispose();
  if (activeFlow !== null) activeFlow.dispose();
  // The cleared state leaves no released handle reachable from module scope.
  activeRenderGroupB = null;
  activeRenderGroupA = null;
  activeBrushGroupB = null;
  activeBrushGroupA = null;
  activeFinalizeGroupBA = null;
  activeFinalizeGroupAB = null;
  activeFlowGroupBA = null;
  activeFlowGroupAB = null;
  activeBrush = null;
  activeCellsB = null;
  activeCellsA = null;
  activeVertices = null;
  activeRender = null;
  activeBrushPipeline = null;
  activeFinalize = null;
  activeFlow = null;
  activeDevice = null;
}
