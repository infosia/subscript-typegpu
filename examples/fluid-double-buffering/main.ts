// example: fluid-double-buffering
// Simulates a three-pass grid fluid whose obstacle moves horizontally from the A and D key scalar.
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

const GRID_SIZE: u32 = 32;
const CELL_COUNT: u32 = 1024;

@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

@CStruct
class FluidCell {
  velocity: Vec2f;
  density: f32;

  constructor(velocity: Vec2f, density: f32) {
    this.velocity = velocity;
    this.density = density;
  }
}

@CStruct
class FluidParams {
  values: Vec4f;

  constructor(obstacleX: f32, time: f32) {
    this.values = new Vec4f(obstacleX, time, GRID_SIZE as f32, 0.0);
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

class FluidLayout {
  source!: Storage<FluidCell>;
  target!: MutStorage<FluidCell>;
  params!: Uniform<FluidParams>;
}

class FluidRenderLayout {
  cells!: Storage<FluidCell>;
}

// Three passes alternate source and target. The obstacle pass writes the render state.
function flowKernel(res: FluidLayout, ctx: ComputeInvocation): void {
  const x: u32 = ctx.globalId.x;
  const y: u32 = ctx.globalId.y;
  const index: u32 = y * GRID_SIZE + x;
  const leftIndex: u32 = x > 0 ? index - 1 : index;
  const rightIndex: u32 = x + 1 < GRID_SIZE ? index + 1 : index;
  const downIndex: u32 = y > 0 ? index - GRID_SIZE : index;
  const upIndex: u32 = y + 1 < GRID_SIZE ? index + GRID_SIZE : index;
  const cell: FluidCell = res.source.get(index);
  const left: FluidCell = res.source.get(leftIndex);
  const right: FluidCell = res.source.get(rightIndex);
  const down: FluidCell = res.source.get(downIndex);
  const up: FluidCell = res.source.get(upIndex);
  const params: FluidParams = res.params.get();
  const neighborDensity: f32 = (left.density + right.density + down.density + up.density) * 0.25;
  cell.density = cell.density * 0.91 + neighborDensity * 0.09;
  cell.velocity.x = cell.velocity.x * 0.97 + (left.density - right.density) * 0.001;
  cell.velocity.y = cell.velocity.y * 0.97 + (down.density - up.density) * 0.001;
  cell.density += params.values.w;
  res.target.set(index, cell);
}

function evaporateKernel(res: FluidLayout, ctx: ComputeInvocation): void {
  const index: u32 = ctx.globalId.y * GRID_SIZE + ctx.globalId.x;
  const cell: FluidCell = res.source.get(index);
  const params: FluidParams = res.params.get();
  cell.density *= 0.992 + params.values.w;
  res.target.set(index, cell);
}

function obstacleKernel(res: FluidLayout, ctx: ComputeInvocation): void {
  const x: u32 = ctx.globalId.x;
  const y: u32 = ctx.globalId.y;
  const index: u32 = y * GRID_SIZE + x;
  const cell: FluidCell = res.source.get(index);
  const params: FluidParams = res.params.get();
  const normalizedX: f32 = (x as f32) / 15.5 - 1.0;
  const normalizedY: f32 = (y as f32) / 15.5 - 1.0;
  let distanceX: f32 = normalizedX - params.values.x;
  if (distanceX < 0.0) distanceX = -distanceX;
  let distanceY: f32 = normalizedY;
  if (distanceY < 0.0) distanceY = -distanceY;
  if (distanceX < 0.12 && distanceY < 0.35) {
    cell.velocity = new Vec2f(0.0, 0.0);
    cell.density = 0.0;
  } else if (y < 3 && distanceX < 0.18) {
    cell.velocity.y += 0.012;
    cell.density = 1.0;
  }
  res.target.set(index, cell);
}

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

function fluidFragment(
  res: FluidRenderLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  let x: u32 = (input.uv.x * (GRID_SIZE as f32)) as u32;
  let y: u32 = (input.uv.y * (GRID_SIZE as f32)) as u32;
  if (x >= GRID_SIZE) x = GRID_SIZE - 1;
  if (y >= GRID_SIZE) y = GRID_SIZE - 1;
  const cell: FluidCell = res.cells.get(y * GRID_SIZE + x);
  return new Vec4f(
    0.03 + cell.density * 0.12,
    0.05 + cell.density * 0.52,
    0.09 + cell.density * 0.86,
    1.0,
  );
}

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

export const fluidRender: RenderPipelineSpec = renderPipelineL<
  FluidRenderLayout,
  Vertex,
  Varyings
>(fluidVertex, fluidFragment, { format: "bgra8unorm" });

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
  if (format !== fluidRender_TARGET_FORMAT) {
    print(`FAIL format expected=${fluidRender_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const vertexValues: FixedArray<Vertex, 3> = [
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(3.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 3.0)),
  ];
  const vertices = hostDevice.createBuffer({
    label: "fluid-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
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
  const params = hostDevice.createBuffer({
    label: "fluid-params",
    size: FluidParams_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>(vertexValues));
  queue.writeBuffer(params, 0, Context.bytesOf<FluidParams>(new FluidParams(0.0, 0.0)));
  for (let index: u32 = 0; index < CELL_COUNT; index += 1) {
    const x: u32 = index % GRID_SIZE;
    const y: u32 = index / GRID_SIZE;
    let density: f32 = 0.0;
    if (x > 12 && x < 20 && y < 8) density = 0.8;
    const cell = new FluidCell(new Vec2f(0.0, 0.0), density);
    const offset: u64 = (index as u64) * (FluidCell_STRIDE as u64);
    const bytes: u8[] = Context.bytesOf<FluidCell>(cell);
    queue.writeBuffer(cellsA, offset, bytes);
    queue.writeBuffer(cellsB, offset, bytes);
  }

  hostDevice.pushErrorScope("validation");
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
  using flowBindLayout = flowPipeline.bindGroupLayout(0);
  using evaporateBindLayout = evaporatePipeline.bindGroupLayout(0);
  using obstacleBindLayout = obstaclePipeline.bindGroupLayout(0);
  using renderBindLayout = renderPipeline.bindGroupLayout(0);
  const flowAB = hostDevice.createBindGroup({
    layout: flowBindLayout,
    entries: [
      { binding: flow_LAYOUT0.entries[0].binding, buffer: cellsA, size: cellsA.size() },
      { binding: flow_LAYOUT0.entries[1].binding, buffer: cellsB, size: cellsB.size() },
      { binding: flow_LAYOUT0.entries[2].binding, buffer: params, size: FluidParams_SIZE as u64 },
    ],
  });
  const flowBA = hostDevice.createBindGroup({
    layout: flowBindLayout,
    entries: [
      { binding: flow_LAYOUT0.entries[0].binding, buffer: cellsB, size: cellsB.size() },
      { binding: flow_LAYOUT0.entries[1].binding, buffer: cellsA, size: cellsA.size() },
      { binding: flow_LAYOUT0.entries[2].binding, buffer: params, size: FluidParams_SIZE as u64 },
    ],
  });
  const evaporateAB = hostDevice.createBindGroup({
    layout: evaporateBindLayout,
    entries: [
      { binding: evaporate_LAYOUT0.entries[0].binding, buffer: cellsA, size: cellsA.size() },
      { binding: evaporate_LAYOUT0.entries[1].binding, buffer: cellsB, size: cellsB.size() },
      { binding: evaporate_LAYOUT0.entries[2].binding, buffer: params, size: FluidParams_SIZE as u64 },
    ],
  });
  const evaporateBA = hostDevice.createBindGroup({
    layout: evaporateBindLayout,
    entries: [
      { binding: evaporate_LAYOUT0.entries[0].binding, buffer: cellsB, size: cellsB.size() },
      { binding: evaporate_LAYOUT0.entries[1].binding, buffer: cellsA, size: cellsA.size() },
      { binding: evaporate_LAYOUT0.entries[2].binding, buffer: params, size: FluidParams_SIZE as u64 },
    ],
  });
  const obstacleAB = hostDevice.createBindGroup({
    layout: obstacleBindLayout,
    entries: [
      { binding: obstacle_LAYOUT0.entries[0].binding, buffer: cellsA, size: cellsA.size() },
      { binding: obstacle_LAYOUT0.entries[1].binding, buffer: cellsB, size: cellsB.size() },
      { binding: obstacle_LAYOUT0.entries[2].binding, buffer: params, size: FluidParams_SIZE as u64 },
    ],
  });
  const obstacleBA = hostDevice.createBindGroup({
    layout: obstacleBindLayout,
    entries: [
      { binding: obstacle_LAYOUT0.entries[0].binding, buffer: cellsB, size: cellsB.size() },
      { binding: obstacle_LAYOUT0.entries[1].binding, buffer: cellsA, size: cellsA.size() },
      { binding: obstacle_LAYOUT0.entries[2].binding, buffer: params, size: FluidParams_SIZE as u64 },
    ],
  });
  const renderA = hostDevice.createBindGroup({
    layout: renderBindLayout,
    entries: [{
      binding: fluidRender_LAYOUT0.entries[0].binding,
      buffer: cellsA,
      size: cellsA.size(),
    }],
  });
  const renderB = hostDevice.createBindGroup({
    layout: renderBindLayout,
    entries: [{
      binding: fluidRender_LAYOUT0.entries[0].binding,
      buffer: cellsB,
      size: cellsB.size(),
    }],
  });
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
): void {
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
  // The A and D key scalar replaces the upstream obstacle sliders.
  if (key === 65 || key === 97) obstacleX -= 0.08;
  if (key === 68 || key === 100) obstacleX += 0.08;
  if (obstacleX < -0.75) obstacleX = -0.75;
  if (obstacleX > 0.75) obstacleX = 0.75;
  frameCount += 1;
  using queue = device.queue();
  queue.writeBuffer(
    params,
    0,
    Context.bytesOf<FluidParams>(new FluidParams(obstacleX, frameCount as f32 / 60.0)),
  );
  const even: boolean = frameCount % 2 === 1;
  // The next frame reverses all buffer roles and preserves the three-pass sequence.
  const flowGroup: GPUBindGroup = even ? flowAB : flowBA;
  const evaporateGroup: GPUBindGroup = even ? evaporateBA : evaporateAB;
  const obstacleGroup: GPUBindGroup = even ? obstacleAB : obstacleBA;
  const displayGroup: GPUBindGroup = even ? renderB : renderA;
  using encoder = device.createCommandEncoderDefault();
  flowPipeline.dispatch(encoder, [flowGroup], GRID_SIZE / 8, GRID_SIZE / 8, 1);
  evaporatePipeline.dispatch(encoder, [evaporateGroup], GRID_SIZE / 8, GRID_SIZE / 8, 1);
  obstaclePipeline.dispatch(encoder, [obstacleGroup], GRID_SIZE / 8, GRID_SIZE / 8, 1);
  const target = new GPUTextureView(view);
  using renderPass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.0, g: 0.0, b: 0.02, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  renderPass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  renderPass.setScissorRect(0, 0, width, height);
  renderPipeline.bind(renderPass, [displayGroup], [vertices]);
  renderPass.draw(3);
  renderPass.end();
  using command = encoder.finishDefault();
  queue.submit([command]);
}

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
