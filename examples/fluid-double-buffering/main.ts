// example: fluid-double-buffering
// Moves a density field along a per-cell velocity through three grid passes around an
// obstacle that follows the pointer or the A and D keys. Each cell picks the cheapest open
// neighbor as its velocity and sends density there, as upstream does. The source tracks the
// obstacle, where upstream holds it at a fixed position, and the wall slider becomes a fixed
// value. The grid is 256 by 256, as upstream.
// Ported from TypeGPU's fluid-double-buffering example (https://github.com/software-mansion/TypeGPU).

import {
  ComputeInvocation,
  ComputePipeline,
  ComputePipelineSpec,
  FragmentInvocation,
  MutStorage,
  RenderPipelineSpec,
  RenderPipeline,
  Storage,
  Uniform,
  VertexInvocation,
  computePipeline,
  createComputePipelineHost,
  createRenderPipelineHost,
  renderPipelineL,
} from "./typegpu";
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
  FluidCell_STRIDE,
  FluidParams_SIZE,
  Vertex_STRIDE,
  evaporate_ENTRY,
  evaporate_LAYOUT0,
  evaporate_WGSL,
  flow_ENTRY,
  flow_LAYOUT0,
  flow_WGSL,
  fluidRender_FRAGMENT_ENTRY,
  fluidRender_LAYOUT0,
  fluidRender_TARGET_FORMAT,
  fluidRender_VERTEX_ENTRY,
  fluidRender_VERTEX_LAYOUT0,
  fluidRender_WGSL,
  obstacle_ENTRY,
  obstacle_LAYOUT0,
  obstacle_WGSL,
} from "./main.typegpu";

// The grid is 256 by 256 cells, so one compute dispatch covers it with 32 by 32 workgroups.
// TypeGPU runs a 256-square grid inside an array sized for 1024 squared cells.
const GRID_SIZE: u32 = 256;
const CELL_COUNT: u32 = GRID_SIZE * GRID_SIZE;

// One corner of the render strip. The generator derives `Vertex_STRIDE` from this class, and
// the vertex buffer layout follows the field order.
@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// One grid cell. The velocity counts cells per step and the density is a unitless amount near
// 0 to 1. TypeGPU packs the same three values into one `vec4f` and leaves the fourth free.
@CStruct
class FluidCell {
  velocity: Vec2f;
  density: f32;

  constructor(velocity: Vec2f, density: f32) {
    this.velocity = velocity;
    this.density = density;
  }
}

// The obstacle center on the normalized X axis, where -1 is the left edge and 1 the right edge.
// TypeGPU carries four integer obstacle boxes and moves them from slider callbacks.
@CStruct
class FluidParams {
  obstacleX: f32;

  constructor(obstacleX: f32) {
    this.obstacleX = obstacleX;
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

// The compute layout. All three passes share it, so one class names the source cells, the target
// cells, and the obstacle uniform. TypeGPU binds the same two roles through slots per pipeline.
class FluidLayout {
  source: Storage<FluidCell>;
  target: MutStorage<FluidCell>;
  params: Uniform<FluidParams>;

  constructor(
    source: Storage<FluidCell>,
    target: MutStorage<FluidCell>,
    params: Uniform<FluidParams>,
  ) {
    this.source = source;
    this.target = target;
    this.params = params;
  }
}

// The render pass reads one grid, so its layout carries a single read-only binding. Two bind
// groups on this layout then select cells A or cells B without a second pipeline.
class FluidRenderLayout {
  cells: Storage<FluidCell>;

  constructor(cells: Storage<FluidCell>) {
    this.cells = cells;
  }
}

function isValidFlowOut(x: i32, y: i32, params: FluidParams): boolean {
  const radius: f32 = ((GRID_SIZE - 1) as f32) * 0.5;
  let horizontal: f32 = (x as f32) / radius - 1.0 - params.obstacleX;
  if (horizontal < 0.0) horizontal = -horizontal;
  let vertical: f32 = (y as f32) / radius - 1.0;
  if (vertical < 0.0) vertical = -vertical;
  return x >= 0 && y >= 0 && x < (GRID_SIZE as i32) && y < (GRID_SIZE as i32)
    && !(horizontal < 0.12 && vertical < 0.35);
}

function computeVelocity(
  x: i32,
  y: i32,
  params: FluidParams,
  density: f32,
  up: f32,
  down: f32,
  right: f32,
  left: f32,
): Vec2f {
  let best: f32 = density;
  let velocity: Vec2f = new Vec2f(0.0, 0.0);
  if (isValidFlowOut(x, y + 1, params) && up + 0.5 < best) {
    best = up + 0.5;
    velocity = new Vec2f(0.0, 1.0);
  }
  if (isValidFlowOut(x, y - 1, params) && down - 0.5 < best) {
    best = down - 0.5;
    velocity = new Vec2f(0.0, -1.0);
  }
  if (isValidFlowOut(x + 1, y, params) && right < best) {
    best = right;
    velocity = new Vec2f(1.0, 0.0);
  }
  if (isValidFlowOut(x - 1, y, params) && left < best) {
    velocity = new Vec2f(-1.0, 0.0);
  }
  return velocity;
}

function flowFromCell(
  myX: i32,
  myY: i32,
  x: i32,
  y: i32,
  source: FluidCell,
  destinationDensity: f32,
): f32 {
  let amount: f32 = 0.0;
  if (source.velocity.length() >= 0.5) {
    amount = 0.3 + (source.density - destinationDensity) * 0.1;
    if (amount < 0.01) amount = 0.01;
    if (amount > source.density) amount = source.density;
  }
  let contribution: f32 = 0.0;
  if (myX === x && myY === y) {
    contribution = source.density - amount;
  } else if (myX === x + (source.velocity.x as i32) && myY === y + (source.velocity.y as i32)) {
    contribution += amount;
  }
  return contribution;
}

// TypeGPU advances the grid with one compute pass per step. This port splits the step into
// three passes that alternate source and target. The obstacle pass writes the render state.
function flowKernel(res: FluidLayout, ctx: ComputeInvocation): void {
  const x: u32 = ctx.globalId.x;
  const y: u32 = ctx.globalId.y;
  const index: u32 = y * GRID_SIZE + x;
  // A neighbor outside the grid falls back to the cell itself for the velocity choice. The
  // transport loop below skips a source outside the grid, so a border cell exchanges density
  // with its valid neighbors only.
  const leftIndex: u32 = x > 0 ? index - 1 : index;
  const rightIndex: u32 = x + 1 < GRID_SIZE ? index + 1 : index;
  const downIndex: u32 = y > 0 ? index - GRID_SIZE : index;
  const upIndex: u32 = y + 1 < GRID_SIZE ? index + GRID_SIZE : index;
  const cell: FluidCell = res.source[index];
  const left: FluidCell = res.source[leftIndex];
  const right: FluidCell = res.source[rightIndex];
  const down: FluidCell = res.source[downIndex];
  const up: FluidCell = res.source[upIndex];
  const params: FluidParams = res.params.$;
  // The velocity is the unit step toward the open neighbor with the least cost, where cost is
  // the neighbor's density plus 0.5 per row upward. The cell then keeps what it does not send
  // and receives the out-flow of each neighbor whose velocity points at it.
  cell.velocity = computeVelocity(x as i32, y as i32, params, cell.density,
    up.density, down.density, right.density, left.density);
  cell.density = 0.0;
  for (let neighbor: i32 = 0; neighbor < 5; neighbor += 1) {
    let sourceX: i32 = x as i32;
    let sourceY: i32 = y as i32;
    if (neighbor === 1) sourceY += 1;
    if (neighbor === 2) sourceY -= 1;
    if (neighbor === 3) sourceX += 1;
    if (neighbor === 4) sourceX -= 1;
    if (sourceX >= 0 && sourceY >= 0 && sourceX < (GRID_SIZE as i32) && sourceY < (GRID_SIZE as i32)) {
      const source: FluidCell = res.source[(sourceY as u32) * GRID_SIZE + (sourceX as u32)];
      const destinationX: i32 = sourceX + (source.velocity.x as i32);
      const destinationY: i32 = sourceY + (source.velocity.y as i32);
      let destinationDensity: f32 = 0.0;
      if (destinationX >= 0 && destinationY >= 0 && destinationX < (GRID_SIZE as i32) && destinationY < (GRID_SIZE as i32)) {
        destinationDensity = res.source[(destinationY as u32) * GRID_SIZE + (destinationX as u32)].density;
      }
      cell.density += flowFromCell(x as i32, y as i32, sourceX, sourceY, source, destinationDensity);
    }
  }
  res.target[index] = cell;
}

// The second pass multiplies every density by 0.992. The slow loss keeps the grid clear of
// saturation under a constant source. TypeGPU has no evaporation pass.
function evaporateKernel(res: FluidLayout, ctx: ComputeInvocation): void {
  const index: u32 = ctx.globalId.y * GRID_SIZE + ctx.globalId.x;
  const cell: FluidCell = res.source[index];
  cell.density *= 0.992;
  res.target[index] = cell;
}

// The third pass applies the two boundary rules. The obstacle box holds its cells at zero.
// The top three rows near the same X hold density at 1.0 with a downward velocity, so the
// source follows the obstacle and the fluid falls onto it.
// TypeGPU moves four obstacle boxes in a separate pass and holds its source at one fixed point.
function obstacleKernel(res: FluidLayout, ctx: ComputeInvocation): void {
  const x: u32 = ctx.globalId.x;
  const y: u32 = ctx.globalId.y;
  const index: u32 = y * GRID_SIZE + x;
  const cell: FluidCell = res.source[index];
  const params: FluidParams = res.params.$;
  // Cell coordinates become the -1 to 1 space the uniform uses, so one value fits any grid size.
  const gridRadius: f32 = ((GRID_SIZE - 1) as f32) * 0.5;
  const normalizedX: f32 = (x as f32) / gridRadius - 1.0;
  const normalizedY: f32 = (y as f32) / gridRadius - 1.0;
  let distanceX: f32 = normalizedX - params.obstacleX;
  if (distanceX < 0.0) distanceX = -distanceX;
  let distanceY: f32 = normalizedY;
  if (distanceY < 0.0) distanceY = -distanceY;
  if (distanceX < 0.12 && distanceY < 0.35) {
    cell.velocity = new Vec2f(0.0, 0.0);
    cell.density = 0.0;
  } else if (y >= GRID_SIZE - 3 && distanceX < 0.18) {
    cell.velocity = new Vec2f(0.0, -1.0);
    cell.density = 1.0;
  }
  res.target[index] = cell;
}

// TypeGPU selects the four full-screen strip corners from the vertex index.
// This port stores the same corners in a typed vertex buffer.
function fluidVertex(
  res: FluidRenderLayout,
  value: Vertex,
  ctx: VertexInvocation,
): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

// The fragment maps its uv to the nearest cell and turns the density into a blue ramp.
// TypeGPU steps through four color thresholds and paints the obstacle cells apart.
function fluidFragment(
  res: FluidRenderLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  let x: u32 = (input.uv.x * (GRID_SIZE as f32)) as u32;
  let y: u32 = (input.uv.y * (GRID_SIZE as f32)) as u32;
  if (x >= GRID_SIZE) x = GRID_SIZE - 1;
  if (y >= GRID_SIZE) y = GRID_SIZE - 1;
  const cell: FluidCell = res.cells[y * GRID_SIZE + x];
  return new Vec4f(
    0.03 + cell.density * 0.12,
    0.05 + cell.density * 0.52,
    0.09 + cell.density * 0.86,
    1.0,
  );
}

// The four declarations name the kernel, the layout type, and the workgroup size. The generator
// reads them ahead of the run and emits the WGSL, the entry names, and the layout facts.
// TypeGPU builds the same WGSL at run time from the kernel function.
export const flow: ComputePipelineSpec = computePipeline<FluidLayout>(flowKernel, {
  name: "flow",
  workgroupSize: [8, 8, 1],
});

export const evaporate: ComputePipelineSpec = computePipeline<FluidLayout>(evaporateKernel, {
  name: "evaporate",
  workgroupSize: [8, 8, 1],
});

export const obstacle: ComputePipelineSpec = computePipeline<FluidLayout>(obstacleKernel, {
  name: "obstacle",
  workgroupSize: [8, 8, 1],
});

// The render declaration fixes the color target format and the strip topology. `init` compares
// the same format constant against the surface format the host reports.
export const fluidRender: RenderPipelineSpec = renderPipelineL<
  FluidRenderLayout,
  Vertex,
  Varyings
>(fluidVertex, fluidFragment, {
  format: "bgra8unorm",
  topology: "triangle-strip",
});

// The host calls `init`, `frame`, and `shutdown` as separate entries, so every handle lives here.
// This example owns each handle from creation until `shutdown` releases it. TypeGPU frees the
// same resources through garbage collection and `root.destroy()`.
let activeDevice: GPUHostOwnedDevice | null = null;
let activeFlow: ComputePipeline | null = null;
let activeEvaporate: ComputePipeline | null = null;
let activeObstacle: ComputePipeline | null = null;
let activeRender: RenderPipeline | null = null;
let activeFlowAB: GPUBindGroup | null = null;
let activeFlowBA: GPUBindGroup | null = null;
let activeEvaporateAB: GPUBindGroup | null = null;
let activeEvaporateBA: GPUBindGroup | null = null;
let activeObstacleAB: GPUBindGroup | null = null;
let activeObstacleBA: GPUBindGroup | null = null;
let activeRenderA: GPUBindGroup | null = null;
let activeRenderB: GPUBindGroup | null = null;
let activeVertices: GPUBuffer | null = null;
let activeCellsA: GPUBuffer | null = null;
let activeCellsB: GPUBuffer | null = null;
let activeParams: GPUBuffer | null = null;
let obstacleX: f32 = 0.0;
let frameCount: u32 = 0;

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  // The pipeline declares its target format literally. A surface with another format ends the
  // example before any draw.
  if (format !== fluidRender_TARGET_FORMAT) {
    print(`FAIL format expected=${fluidRender_TARGET_FORMAT} actual=${format}`);
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
    label: "fluid-vertices",
    size: (Vertex_STRIDE * 4) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // The two cell grids alternate between source and target. Both carry the same size, so a bind
  // group can pair them in either direction.
  const cellsA = hostDevice.createBuffer({
    label: "fluid-cells-a",
    size: (FluidCell_STRIDE * CELL_COUNT) as u64,
    usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
  });
  const cellsB = hostDevice.createBuffer({
    label: "fluid-cells-b",
    size: (FluidCell_STRIDE * CELL_COUNT) as u64,
    usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
  });
  // The obstacle uniform. One frame writes it once, and all three passes read the same value.
  const params = hostDevice.createBuffer({
    label: "fluid-params",
    size: FluidParams_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  // Queue writes land before the commands that a later submit carries, so the initial state
  // reaches the buffers before the first dispatch.
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 4>>(vertexValues));
  queue.writeBuffer(params, 0, Context.bytesOf<FluidParams>(new FluidParams(0.0)));
  // The initial state fills a dense block near the bottom center. `Context.bytesOf` returns the
  // bytes of one cell in the generated layout, and the loop appends cell after cell.
  const initialCells: u8[] = [];
  for (let index: u32 = 0; index < CELL_COUNT; index += 1) {
    const x: u32 = index % GRID_SIZE;
    const y: u32 = index / GRID_SIZE;
    let density: f32 = 0.0;
    if (x > GRID_SIZE * 3 / 8 && x < GRID_SIZE * 5 / 8 && y < GRID_SIZE / 4) density = 0.8;
    const cell = new FluidCell(new Vec2f(0.0, 0.0), density);
    const bytes: u8[] = Context.bytesOf<FluidCell>(cell);
    for (let byteIndex: i32 = 0; byteIndex < bytes.length; byteIndex += 1) {
      initialCells.push(bytes[byteIndex]);
    }
  }
  queue.writeBuffer(cellsA, 0, initialCells);

  // The error scope catches a pipeline creation failure. TypeGPU rejects a promise. Here the pop
  // returns a value, so this example releases what it created and prints the first error line.
  hostDevice.pushErrorScope("validation");
  // Each compute pipeline takes the generated WGSL text, the entry name, the layout facts, and
  // the workgroup size. That size must equal the size in the declaration above.
  const flowPipeline = createComputePipelineHost(
    hostDevice,
    flow_WGSL,
    flow_ENTRY,
    [flow_LAYOUT0],
    [8, 8, 1],
  );
  const evaporatePipeline = createComputePipelineHost(
    hostDevice,
    evaporate_WGSL,
    evaporate_ENTRY,
    [evaporate_LAYOUT0],
    [8, 8, 1],
  );
  const obstaclePipeline = createComputePipelineHost(
    hostDevice,
    obstacle_WGSL,
    obstacle_ENTRY,
    [obstacle_LAYOUT0],
    [8, 8, 1],
  );
  // The render pipeline also takes the vertex buffer layout and the declaration, which carries
  // the target format and the topology.
  const renderPipeline = createRenderPipelineHost(
    hostDevice,
    fluidRender_WGSL,
    fluidRender_VERTEX_ENTRY,
    fluidRender_FRAGMENT_ENTRY,
    [fluidRender_LAYOUT0],
    [fluidRender_VERTEX_LAYOUT0],
    fluidRender,
  );
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    renderPipeline.dispose();
    obstaclePipeline.dispose();
    evaporatePipeline.dispose();
    flowPipeline.dispose();
    params.dispose();
    cellsB.dispose();
    cellsA.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  // The pipeline reports the layout WebGPU built for group 0. Each call returns a new handle, and
  // the bind groups below need it only at creation.
  using flowBindLayout = flowPipeline.bindGroupLayout(0);
  using evaporateBindLayout = evaporatePipeline.bindGroupLayout(0);
  using obstacleBindLayout = obstaclePipeline.bindGroupLayout(0);
  using renderBindLayout = renderPipeline.bindGroupLayout(0);
  // Six compute bind groups cover both directions of each pass, and two render groups select the
  // grid the frame displays. The binding numbers come from the generated layout facts.
  // TypeGPU swaps the same roles with two pipeline sets built from slots.
  const flowAB = hostDevice.createBindGroup({
    layout: flowBindLayout,
    entries: [
      { binding: flow_LAYOUT0.entries[0].binding, buffer: cellsA, size: cellsA.size },
      { binding: flow_LAYOUT0.entries[1].binding, buffer: cellsB, size: cellsB.size },
      { binding: flow_LAYOUT0.entries[2].binding, buffer: params, size: FluidParams_SIZE as u64 },
    ],
  });
  const flowBA = hostDevice.createBindGroup({
    layout: flowBindLayout,
    entries: [
      { binding: flow_LAYOUT0.entries[0].binding, buffer: cellsB, size: cellsB.size },
      { binding: flow_LAYOUT0.entries[1].binding, buffer: cellsA, size: cellsA.size },
      { binding: flow_LAYOUT0.entries[2].binding, buffer: params, size: FluidParams_SIZE as u64 },
    ],
  });
  const evaporateAB = hostDevice.createBindGroup({
    layout: evaporateBindLayout,
    entries: [
      { binding: evaporate_LAYOUT0.entries[0].binding, buffer: cellsA, size: cellsA.size },
      { binding: evaporate_LAYOUT0.entries[1].binding, buffer: cellsB, size: cellsB.size },
      { binding: evaporate_LAYOUT0.entries[2].binding, buffer: params, size: FluidParams_SIZE as u64 },
    ],
  });
  const evaporateBA = hostDevice.createBindGroup({
    layout: evaporateBindLayout,
    entries: [
      { binding: evaporate_LAYOUT0.entries[0].binding, buffer: cellsB, size: cellsB.size },
      { binding: evaporate_LAYOUT0.entries[1].binding, buffer: cellsA, size: cellsA.size },
      { binding: evaporate_LAYOUT0.entries[2].binding, buffer: params, size: FluidParams_SIZE as u64 },
    ],
  });
  const obstacleAB = hostDevice.createBindGroup({
    layout: obstacleBindLayout,
    entries: [
      { binding: obstacle_LAYOUT0.entries[0].binding, buffer: cellsA, size: cellsA.size },
      { binding: obstacle_LAYOUT0.entries[1].binding, buffer: cellsB, size: cellsB.size },
      { binding: obstacle_LAYOUT0.entries[2].binding, buffer: params, size: FluidParams_SIZE as u64 },
    ],
  });
  const obstacleBA = hostDevice.createBindGroup({
    layout: obstacleBindLayout,
    entries: [
      { binding: obstacle_LAYOUT0.entries[0].binding, buffer: cellsB, size: cellsB.size },
      { binding: obstacle_LAYOUT0.entries[1].binding, buffer: cellsA, size: cellsA.size },
      { binding: obstacle_LAYOUT0.entries[2].binding, buffer: params, size: FluidParams_SIZE as u64 },
    ],
  });
  const renderA = hostDevice.createBindGroup({
    layout: renderBindLayout,
    entries: [{
      binding: fluidRender_LAYOUT0.entries[0].binding,
      buffer: cellsA,
      size: cellsA.size,
    }],
  });
  const renderB = hostDevice.createBindGroup({
    layout: renderBindLayout,
    entries: [{
      binding: fluidRender_LAYOUT0.entries[0].binding,
      buffer: cellsB,
      size: cellsB.size,
    }],
  });
  // The handles reach module state only after every creation succeeds, so a failed init leaves
  // no partial state behind.
  activeDevice = hostDevice;
  activeFlow = flowPipeline;
  activeEvaporate = evaporatePipeline;
  activeObstacle = obstaclePipeline;
  activeRender = renderPipeline;
  activeFlowAB = flowAB;
  activeFlowBA = flowBA;
  activeEvaporateAB = evaporateAB;
  activeEvaporateBA = evaporateBA;
  activeObstacleAB = obstacleAB;
  activeObstacleBA = obstacleBA;
  activeRenderA = renderA;
  activeRenderB = renderB;
  activeVertices = vertices;
  activeCellsA = cellsA;
  activeCellsB = cellsB;
  activeParams = params;
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
  const evaporatePipeline = activeEvaporate;
  const obstaclePipeline = activeObstacle;
  const renderPipeline = activeRender;
  const flowAB = activeFlowAB;
  const flowBA = activeFlowBA;
  const evaporateAB = activeEvaporateAB;
  const evaporateBA = activeEvaporateBA;
  const obstacleAB = activeObstacleAB;
  const obstacleBA = activeObstacleBA;
  const renderA = activeRenderA;
  const renderB = activeRenderB;
  const vertices = activeVertices;
  const params = activeParams;
  if (device === null) return;
  if (flowPipeline === null) return;
  if (evaporatePipeline === null) return;
  if (obstaclePipeline === null) return;
  if (renderPipeline === null) return;
  if (flowAB === null) return;
  if (flowBA === null) return;
  if (evaporateAB === null) return;
  if (evaporateBA === null) return;
  if (obstacleAB === null) return;
  if (obstacleBA === null) return;
  if (renderA === null) return;
  if (renderB === null) return;
  if (vertices === null) return;
  if (params === null) return;
  // Key 65 and key 97 are `A`, and key 68 and key 100 are `D`, in upper and lower case.
  // The pointer maps its surface X through a grid cell into the same -1 to 1 obstacle space.
  if (key === 65 || key === 97) obstacleX -= 0.08;
  if (key === 68 || key === 100) obstacleX += 0.08;
  if ((buttons & 1) !== 0 && pointerX >= 0.0) {
    const gridRadius: f32 = ((GRID_SIZE - 1) as f32) * 0.5;
    const pointerCell: f32 =
      (pointerX / (width as f32)) * ((GRID_SIZE - 1) as f32);
    obstacleX = pointerCell / gridRadius - 1.0;
  }
  // The limit keeps the whole obstacle box inside the grid.
  if (obstacleX < -0.75) obstacleX = -0.75;
  if (obstacleX > 0.75) obstacleX = 0.75;
  frameCount += 1;
  // The uniform write reaches the device before the submit below, so all three passes read the
  // obstacle position of this frame.
  using queue = device.queue();
  queue.writeBuffer(
    params,
    0,
    Context.bytesOf<FluidParams>(new FluidParams(obstacleX)),
  );
  const writesToB: boolean = frameCount % 2 === 1;
  // Each pass reads what the pass before it wrote, so flow, evaporate, and obstacle chain
  // inside one frame. The last pass leaves the result in the grid the display group reads.
  // The next frame reverses every role.
  const flowGroup: GPUBindGroup = writesToB ? flowAB : flowBA;
  const evaporateGroup: GPUBindGroup = writesToB ? evaporateBA : evaporateAB;
  const obstacleGroup: GPUBindGroup = writesToB ? obstacleAB : obstacleBA;
  const displayGroup: GPUBindGroup = writesToB ? renderB : renderA;
  // Each dispatch records its own compute pass, and the passes run in record order.
  // The counts are workgroups, so 256 cells over a workgroup of 8 need 32 groups per axis.
  using encoder = device.createCommandEncoderDefault();
  flowPipeline.dispatch(encoder, [flowGroup], GRID_SIZE / 8, GRID_SIZE / 8, 1);
  evaporatePipeline.dispatch(encoder, [evaporateGroup], GRID_SIZE / 8, GRID_SIZE / 8, 1);
  obstaclePipeline.dispatch(encoder, [obstacleGroup], GRID_SIZE / 8, GRID_SIZE / 8, 1);
  // The host owns the presented view, so this example wraps it and releases nothing.
  const target = new GPUTextureView(view);
  using renderPass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.0, g: 0.0, b: 0.02, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  // The surface size changes when the window resizes, so both rectangles follow the frame size.
  renderPass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  renderPass.setScissorRect(0, 0, width, height);
  // Four vertices draw the strip, and the display group selects the grid the last pass wrote.
  renderPipeline.bind(renderPass, [displayGroup], [vertices]);
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
  if (activeRenderB !== null) activeRenderB.dispose();
  if (activeRenderA !== null) activeRenderA.dispose();
  if (activeObstacleBA !== null) activeObstacleBA.dispose();
  if (activeObstacleAB !== null) activeObstacleAB.dispose();
  if (activeEvaporateBA !== null) activeEvaporateBA.dispose();
  if (activeEvaporateAB !== null) activeEvaporateAB.dispose();
  if (activeFlowBA !== null) activeFlowBA.dispose();
  if (activeFlowAB !== null) activeFlowAB.dispose();
  if (activeParams !== null) activeParams.dispose();
  if (activeCellsB !== null) activeCellsB.dispose();
  if (activeCellsA !== null) activeCellsA.dispose();
  if (activeVertices !== null) activeVertices.dispose();
  if (activeRender !== null) activeRender.dispose();
  if (activeObstacle !== null) activeObstacle.dispose();
  if (activeEvaporate !== null) activeEvaporate.dispose();
  if (activeFlow !== null) activeFlow.dispose();
  // The cleared state leaves no released handle reachable from module scope.
  activeRenderB = null;
  activeRenderA = null;
  activeObstacleBA = null;
  activeObstacleAB = null;
  activeEvaporateBA = null;
  activeEvaporateAB = null;
  activeFlowBA = null;
  activeFlowAB = null;
  activeParams = null;
  activeCellsB = null;
  activeCellsA = null;
  activeVertices = null;
  activeRender = null;
  activeObstacle = null;
  activeEvaporate = null;
  activeFlow = null;
  activeDevice = null;
}
