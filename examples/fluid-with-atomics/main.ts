// example: fluid-with-atomics
// Moves water down and sideways through one atomic grid under a pointer brush.
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
  atomicFlow_ENTRY,
  atomicFlow_LAYOUT0,
  atomicFlow_WGSL,
  atomicFluidRender_FRAGMENT_ENTRY,
  atomicFluidRender_LAYOUT0,
  atomicFluidRender_TARGET_FORMAT,
  atomicFluidRender_VERTEX_ENTRY,
  atomicFluidRender_VERTEX_LAYOUT0,
  atomicFluidRender_WGSL,
} from "./main.typegpu";

const GRID_SIZE: u32 = 64;
const CELL_COUNT: u32 = GRID_SIZE * GRID_SIZE;
const MAX_WATER: u32 = 1024;
const WALL_LEVEL: u32 = 2147483648;
const GRAVITY_STEP: u32 = 96;
const SIDE_STEP: u32 = 24;
const BRUSH_WATER: u32 = 192;
const BRUSH_RADIUS: f32 = 2.4;
const BRUSH_ERASE: u32 = 0;
const BRUSH_WATER_MODE: u32 = 1;
const BRUSH_WALL: u32 = 2;

@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

@CStruct
class WaterCell {
  level: AtomicU32;

  constructor(level: AtomicU32) {
    this.level = level;
  }
}

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

@CStruct
class Varyings {
  position: Vec4f;
  uv: Vec2f;

  constructor(position: Vec4f, uv: Vec2f) {
    this.position = position;
    this.uv = uv;
  }
}

class AtomicFluidLayout {
  cells!: MutStorage<WaterCell>;
  brush!: Uniform<BrushParams>;
}

class AtomicFluidRenderLayout {
  cells!: MutStorage<WaterCell>;
}

function minU32(left: u32, right: u32): u32 {
  return left < right ? left : right;
}

// Each invocation owns one source cell. Gravity runs first, and one transfer toward the lower
// side neighbor follows it. TypeGPU runs a separate brush pass over two buffers, and this port
// folds the brush in and updates one atomic buffer in place.
// Gravity runs first, and one transfer toward the lower side neighbor follows it.
function atomicFlowKernel(res: AtomicFluidLayout, ctx: ComputeInvocation): void {
  const x: u32 = ctx.globalId.x;
  const y: u32 = ctx.globalId.y;
  const index: u32 = y * GRID_SIZE + x;
  const brush: BrushParams = res.brush.get();
  if (brush.active !== 0) {
    const point = new Vec2f(x as f32, y as f32);
    if (sdLine(point, brush.previous, brush.current) <= BRUSH_RADIUS) {
      if (brush.mode === BRUSH_WALL) {
        res.cells[index].level.store(WALL_LEVEL);
        return;
      }
      if (brush.mode === BRUSH_ERASE) {
        res.cells[index].level.store(0);
        return;
      }
      const brushLevel: u32 = res.cells[index].level.load();
      if (brushLevel >= WALL_LEVEL) {
        res.cells[index].level.store(BRUSH_WATER);
      } else if (brushLevel < MAX_WATER) {
        res.cells[index].level.add(minU32(BRUSH_WATER, MAX_WATER - brushLevel));
      }
    }
  }

  let level: u32 = res.cells[index].level.load();
  if (level === 0 || level >= WALL_LEVEL) return;

  if (y > 0) {
    const belowIndex: u32 = index - GRID_SIZE;
    const belowLevel: u32 = res.cells[belowIndex].level.load();
    if (belowLevel < MAX_WATER) {
      const gravity: u32 = minU32(
        level,
        minU32(MAX_WATER - belowLevel, GRAVITY_STEP),
      );
      if (gravity > 0) {
        res.cells[index].level.sub(gravity);
        res.cells[belowIndex].level.add(gravity);
        level -= gravity;
      }
    }
  }
  if (level === 0) return;

  let targetIndex: u32 = index;
  let targetLevel: u32 = level;
  if (x > 0) {
    const leftIndex: u32 = index - 1;
    const leftLevel: u32 = res.cells[leftIndex].level.load();
    if (leftLevel < targetLevel) {
      targetIndex = leftIndex;
      targetLevel = leftLevel;
    }
  }
  if (x + 1 < GRID_SIZE) {
    const rightIndex: u32 = index + 1;
    const rightLevel: u32 = res.cells[rightIndex].level.load();
    if (rightLevel < targetLevel) {
      targetIndex = rightIndex;
      targetLevel = rightLevel;
    }
  }
  if (targetIndex !== index && targetLevel < WALL_LEVEL && level > targetLevel + 1) {
    const sideways: u32 = minU32((level - targetLevel) / 4, SIDE_STEP);
    if (sideways > 0) {
      res.cells[index].level.sub(sideways);
      res.cells[targetIndex].level.add(sideways);
    }
  }
}

// TypeGPU draws one full-screen triangle and picks its three corners from the vertex index.
// This port draws a four-corner strip from a typed vertex buffer.
// This port stores the four corners in a typed vertex buffer.
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

export const atomicFlow: ComputePipelineSpec = computePipeline<AtomicFluidLayout>(
  atomicFlowKernel,
  { name: "atomicFlow", workgroupSize: [8, 8, 1] },
);

export const atomicFluidRender: RenderPipelineSpec = renderPipelineL<
  AtomicFluidRenderLayout,
  Vertex,
  Varyings
>(atomicFluidVertex, atomicFluidFragment, {
  format: "bgra8unorm",
  topology: "triangle-strip",
});

let activeDevice: GPUHostOwnedDevice | null = null;
let activeFlow: ComputePipeline | null = null;
let activeRender: RenderPipeline | null = null;
let activeFlowGroup: GPUBindGroup | null = null;
let activeRenderGroup: GPUBindGroup | null = null;
let activeVertices: GPUBuffer | null = null;
let activeCells: GPUBuffer | null = null;
let activeBrush: GPUBuffer | null = null;
let brushMode: u32 = BRUSH_WATER_MODE;
let previousPointerX: f32 = -1.0;
let previousPointerY: f32 = -1.0;

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  if (format !== atomicFluidRender_TARGET_FORMAT) {
    print(`FAIL format expected=${atomicFluidRender_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const vertexValues: FixedArray<Vertex, 4> = [
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(1.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 1.0)),
    new Vertex(new Vec2f(1.0, 1.0)),
  ];
  const vertices = hostDevice.createBuffer({
    label: "atomic-fluid-vertices",
    size: (Vertex_STRIDE * 4) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  const cells = hostDevice.createBuffer({
    label: "atomic-fluid-cells",
    size: (WaterCell_STRIDE * CELL_COUNT) as u64,
    usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
  });
  const brush = hostDevice.createBuffer({
    label: "atomic-fluid-brush",
    size: BrushParams_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });

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
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 4>>(vertexValues));
  queue.writeBuffer(cells, 0, initialCells);
  const idlePoint = new Vec2f(0.0, 0.0);
  queue.writeBuffer(
    brush,
    0,
    Context.bytesOf<BrushParams>(
      new BrushParams(idlePoint, new Vec2f(0.001, 0.0), brushMode, 0),
    ),
  );

  hostDevice.pushErrorScope("validation");
  const flowPipeline = createComputePipelineHost(
    hostDevice,
    atomicFlow_WGSL,
    atomicFlow_ENTRY,
    [atomicFlow_LAYOUT0],
    [8, 8, 1],
  );
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
    flowPipeline.dispose();
    brush.dispose();
    cells.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }

  using flowLayout = flowPipeline.bindGroupLayout(0);
  using renderLayout = renderPipeline.bindGroupLayout(0);
  const flowGroup = createBindGroupHost(
    hostDevice,
    flowLayout,
    atomicFlow_LAYOUT0,
    [bufferResource(cells), bufferResource(brush)],
  );
  const renderGroup = createBindGroupHost(
    hostDevice,
    renderLayout,
    atomicFluidRender_LAYOUT0,
    [bufferResource(cells)],
  );
  activeDevice = hostDevice;
  activeFlow = flowPipeline;
  activeRender = renderPipeline;
  activeFlowGroup = flowGroup;
  activeRenderGroup = renderGroup;
  activeVertices = vertices;
  activeCells = cells;
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
  const device = activeDevice;
  const flowPipeline = activeFlow;
  const renderPipeline = activeRender;
  const flowGroup = activeFlowGroup;
  const renderGroup = activeRenderGroup;
  const vertices = activeVertices;
  const brush = activeBrush;
  if (device === null) return;
  if (flowPipeline === null) return;
  if (renderPipeline === null) return;
  if (flowGroup === null) return;
  if (renderGroup === null) return;
  if (vertices === null) return;
  if (brush === null) return;

  if (key === 49) brushMode = BRUSH_WATER_MODE;
  if (key === 50) brushMode = BRUSH_WALL;
  if (key === 48) brushMode = BRUSH_ERASE;

  let currentX: f32 = 0.0;
  let currentY: f32 = 0.0;
  let previousX: f32 = 0.0;
  let previousY: f32 = 0.0;
  let brushActive: u32 = 0;
  if (pointerX >= 0.0 && pointerY >= 0.0) {
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
    if (previousX === currentX && previousY === currentY) {
      previousX = currentX > 0.001 ? currentX - 0.001 : currentX + 0.001;
    }
    if ((buttons & 1) !== 0) brushActive = 1;
    previousPointerX = currentX;
    previousPointerY = currentY;
  }

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
  using encoder = device.createCommandEncoderDefault();
  flowPipeline.dispatch(encoder, [flowGroup], GRID_SIZE / 8, GRID_SIZE / 8, 1);
  const target = new GPUTextureView(view);
  using renderPass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.01, g: 0.015, b: 0.03, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  renderPass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  renderPass.setScissorRect(0, 0, width, height);
  renderPipeline.bind(renderPass, [renderGroup], [vertices]);
  renderPass.draw(4);
  renderPass.end();
  using command = encoder.finishDefault();
  queue.submit([command]);
}

export function shutdown(): void {
  if (activeRenderGroup !== null) activeRenderGroup.dispose();
  if (activeFlowGroup !== null) activeFlowGroup.dispose();
  if (activeBrush !== null) activeBrush.dispose();
  if (activeCells !== null) activeCells.dispose();
  if (activeVertices !== null) activeVertices.dispose();
  if (activeRender !== null) activeRender.dispose();
  if (activeFlow !== null) activeFlow.dispose();
  activeRenderGroup = null;
  activeFlowGroup = null;
  activeBrush = null;
  activeCells = null;
  activeVertices = null;
  activeRender = null;
  activeFlow = null;
  activeDevice = null;
}
