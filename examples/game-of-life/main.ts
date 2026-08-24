// example: game-of-life
// Advances a texture-backed Conway grid and accepts pointer drawing and keyboard clearing.
// This port keeps the naive strategy at a fixed 128-square grid. It drops the workgroup-tiled
// strategy, the bit-packed strategy, the size selector, the zoom view, and the pause controls.
// One glider replaces the upstream random seed, and a fixed square brush replaces the
// upstream brush radius, brush modes, and stroke line.
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

const GRID_SIZE: u32 = 128;
const BRUSH_RADIUS: f32 = 2.5;
const EDIT_NONE: u32 = 0;
const EDIT_CLEAR: u32 = 1;
const EDIT_DRAW: u32 = 2;

@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

@CStruct
class EditParams {
  point: Vec2f;
  mode: u32;

  constructor(point: Vec2f, mode: u32) {
    this.point = point;
    this.mode = mode;
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

class LifeStepLayout {
  generation!: ReadStorageTexture2d<R32float>;
  next!: StorageTexture2d<R32float>;
}

class LifeEditLayout {
  generation!: ReadWriteStorageTexture2d<R32float>;
  edit!: Uniform<EditParams>;
}

class LifeRenderLayout {
  generation!: ReadStorageTexture2d<R32float>;
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
  const alive: boolean = res.generation.load(new Vec2i(x, y)).x > 0.5;
  let next: f32 = 0.0;
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
  const params: EditParams = res.edit.get();
  const cell = new Vec2i(ctx.globalId.x as i32, ctx.globalId.y as i32);
  if (params.mode === EDIT_CLEAR) {
    res.generation.store(cell, new Vec4f(0.0, 0.0, 0.0, 1.0));
    return;
  }
  if (params.mode !== EDIT_DRAW) return;
  let dx: f32 = (ctx.globalId.x as f32) - params.point.x;
  let dy: f32 = (ctx.globalId.y as f32) - params.point.y;
  if (dx < 0.0) dx = -dx;
  if (dy < 0.0) dy = -dy;
  if (dx <= BRUSH_RADIUS && dy <= BRUSH_RADIUS) {
    res.generation.store(cell, new Vec4f(1.0, 0.0, 0.0, 1.0));
  }
}

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

export const lifeStep: ComputePipelineSpec = computePipeline<LifeStepLayout>(
  lifeStepKernel,
  { name: "lifeStep", workgroupSize: [8, 8, 1] },
);

export const lifeEdit: ComputePipelineSpec = computePipeline<LifeEditLayout>(
  lifeEditKernel,
  { name: "lifeEdit", workgroupSize: [8, 8, 1] },
);

export const lifeRender: RenderPipelineSpec = renderPipelineL<
  LifeRenderLayout,
  Vertex,
  Varyings
>(lifeVertex, lifeFragment, {
  format: "bgra8unorm",
  topology: "triangle-strip",
});

let activeDevice: GPUHostOwnedDevice | null = null;
let activeStep: ComputePipeline | null = null;
let activeEdit: ComputePipeline | null = null;
let activeRender: RenderPipeline | null = null;
let activeStepAB: GPUBindGroup | null = null;
let activeStepBA: GPUBindGroup | null = null;
let activeEditA: GPUBindGroup | null = null;
let activeEditB: GPUBindGroup | null = null;
let activeRenderA: GPUBindGroup | null = null;
let activeRenderB: GPUBindGroup | null = null;
let activeVertices: GPUBuffer | null = null;
let activeEditParams: GPUBuffer | null = null;
let activeGenerationA: GPUTexture | null = null;
let activeGenerationB: GPUTexture | null = null;
let activeViewA: GPUTextureView | null = null;
let activeViewB: GPUTextureView | null = null;
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

function emptyGeneration(): Vec4f[] {
  const pixels: Vec4f[] = [];
  let index: u32 = 0;
  while (index < GRID_SIZE * GRID_SIZE) {
    pixels.push(new Vec4f(0.0, 0.0, 0.0, 1.0));
    index += 1;
  }
  return pixels;
}

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  if (format !== lifeRender_TARGET_FORMAT) {
    print(`FAIL format expected=${lifeRender_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const vertices = hostDevice.createBuffer({
    label: "life-vertices",
    size: (Vertex_STRIDE * 4) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  const editParams = hostDevice.createBuffer({
    label: "life-edit",
    size: EditParams_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  const textureUsage: u64 = GPUTextureUsage.STORAGE_BINDING + GPUTextureUsage.COPY_DST;
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
  const viewA = generationA.createView();
  const viewB = generationB.createView();
  const vertexValues: FixedArray<Vertex, 4> = [
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(1.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 1.0)),
    new Vertex(new Vec2f(1.0, 1.0)),
  ];
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 4>>(vertexValues));
  queue.writeBuffer(
    editParams,
    0,
    Context.bytesOf<EditParams>(new EditParams(new Vec2f(0.0, 0.0), EDIT_NONE)),
  );
  writeTexturePixels(queue, generationA, gliderSeed(), GRID_SIZE, GRID_SIZE);
  writeTexturePixels(queue, generationB, emptyGeneration(), GRID_SIZE, GRID_SIZE);

  hostDevice.pushErrorScope("validation");
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

  using stepLayout = stepPipeline.bindGroupLayout(0);
  using editLayout = editPipeline.bindGroupLayout(0);
  using renderLayout = renderPipeline.bindGroupLayout(0);
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
  activeDevice = hostDevice;
  activeStep = stepPipeline;
  activeEdit = editPipeline;
  activeRender = renderPipeline;
  activeStepAB = stepAB;
  activeStepBA = stepBA;
  activeEditA = editA;
  activeEditB = editB;
  activeRenderA = renderA;
  activeRenderB = renderB;
  activeVertices = vertices;
  activeEditParams = editParams;
  activeGenerationA = generationA;
  activeGenerationB = generationB;
  activeViewA = viewA;
  activeViewB = viewB;
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
  const stepPipeline = activeStep;
  const editPipeline = activeEdit;
  const renderPipeline = activeRender;
  const stepAB = activeStepAB;
  const stepBA = activeStepBA;
  const editA = activeEditA;
  const editB = activeEditB;
  const renderA = activeRenderA;
  const renderB = activeRenderB;
  const vertices = activeVertices;
  const editParams = activeEditParams;
  if (device === null) return;
  if (stepPipeline === null) return;
  if (editPipeline === null) return;
  if (renderPipeline === null) return;
  if (stepAB === null) return;
  if (stepBA === null) return;
  if (editA === null) return;
  if (editB === null) return;
  if (renderA === null) return;
  if (renderB === null) return;
  if (vertices === null) return;
  if (editParams === null) return;
  // The host passes the key as a Unicode scalar, and 48 is the `0` key.
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
  using queue = device.queue();
  queue.writeBuffer(
    editParams,
    0,
    Context.bytesOf<EditParams>(new EditParams(editPoint, editMode)),
  );
  // The frame parity picks the step source and target. The edit pass and the render pass
  // both use the step target, so an edit lands on the grid the frame displays.
  const readsA: boolean = frameCount % 2 === 0;
  const stepGroup: GPUBindGroup = readsA ? stepAB : stepBA;
  const editGroup: GPUBindGroup = readsA ? editB : editA;
  const displayGroup: GPUBindGroup = readsA ? renderB : renderA;
  using encoder = device.createCommandEncoderDefault();
  stepPipeline.dispatch(encoder, [stepGroup], GRID_SIZE / 8, GRID_SIZE / 8, 1);
  if (editMode !== EDIT_NONE) {
    editPipeline.dispatch(encoder, [editGroup], GRID_SIZE / 8, GRID_SIZE / 8, 1);
  }
  const target = new GPUTextureView(view);
  using renderPass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.015, g: 0.02, b: 0.035, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  renderPass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  renderPass.setScissorRect(0, 0, width, height);
  renderPipeline.bind(renderPass, [displayGroup], [vertices]);
  renderPass.draw(4);
  renderPass.end();
  using command = encoder.finishDefault();
  queue.submit([command]);
  frameCount += 1;
}

export function shutdown(): void {
  if (activeRenderB !== null) activeRenderB.dispose();
  if (activeRenderA !== null) activeRenderA.dispose();
  if (activeEditB !== null) activeEditB.dispose();
  if (activeEditA !== null) activeEditA.dispose();
  if (activeStepBA !== null) activeStepBA.dispose();
  if (activeStepAB !== null) activeStepAB.dispose();
  if (activeViewB !== null) activeViewB.dispose();
  if (activeViewA !== null) activeViewA.dispose();
  if (activeGenerationB !== null) activeGenerationB.dispose();
  if (activeGenerationA !== null) activeGenerationA.dispose();
  if (activeEditParams !== null) activeEditParams.dispose();
  if (activeVertices !== null) activeVertices.dispose();
  if (activeRender !== null) activeRender.dispose();
  if (activeEdit !== null) activeEdit.dispose();
  if (activeStep !== null) activeStep.dispose();
  activeRenderB = null;
  activeRenderA = null;
  activeEditB = null;
  activeEditA = null;
  activeStepBA = null;
  activeStepAB = null;
  activeViewB = null;
  activeViewA = null;
  activeGenerationB = null;
  activeGenerationA = null;
  activeEditParams = null;
  activeVertices = null;
  activeRender = null;
  activeEdit = null;
  activeStep = null;
  activeDevice = null;
  frameCount = 0;
}
