// example: slime-mold
// Moves trail-sensing agents and diffuses their deposits through a swapped texture pair.
// This port fixes 4096 agents, a 128-square trail, sensor distance 5, sensor angle 0.5,
// turn 0.32, step 1, deposit 0.2, and decay 0.985 in place of the upstream sliders.
// Ported from TypeGPU's slime-mold example (https://github.com/software-mansion/TypeGPU).

import {
  ComputeInvocation,
  ComputePipeline,
  ComputePipelineSpec,
  FragmentInvocation,
  MutStorage,
  R32float,
  ReadStorageTexture2d,
  ReadWriteStorageTexture2d,
  RenderPipeline,
  RenderPipelineSpec,
  StorageTexture2d,
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
  Agent_STRIDE,
  Vertex_STRIDE,
  slimeDiffuse_ENTRY,
  slimeDiffuse_LAYOUT0,
  slimeDiffuse_WGSL,
  slimeMove_ENTRY,
  slimeMove_LAYOUT0,
  slimeMove_WGSL,
  slimeRender_FRAGMENT_ENTRY,
  slimeRender_LAYOUT0,
  slimeRender_TARGET_FORMAT,
  slimeRender_VERTEX_ENTRY,
  slimeRender_VERTEX_LAYOUT0,
  slimeRender_WGSL,
} from "./main.typegpu";

const TRAIL_SIZE: u32 = 128;
const AGENT_COUNT: u32 = 4096;
const SENSOR_DISTANCE: f32 = 5.0;
const SENSOR_ANGLE: f32 = 0.5;
const TURN_SPEED: f32 = 0.32;
const STEP_SIZE: f32 = 1.0;
const DEPOSIT_AMOUNT: f32 = 0.2;
const TRAIL_DECAY: f32 = 0.985;
const TAU: f32 = 6.2831855;

@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

@CStruct
class Agent {
  position: Vec2f;
  heading: f32;

  constructor(position: Vec2f, heading: f32) {
    this.position = position;
    this.heading = heading;
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

class SlimeMoveLayout {
  agents!: MutStorage<Agent>;
  sense!: ReadStorageTexture2d<R32float>;
  trail!: ReadWriteStorageTexture2d<R32float>;
}

class SlimeDiffuseLayout {
  source!: ReadStorageTexture2d<R32float>;
  target!: StorageTexture2d<R32float>;
}

class SlimeRenderLayout {
  trail!: ReadStorageTexture2d<R32float>;
}

function wrapTrail(value: f32): f32 {
  let wrapped: f32 = value;
  if (wrapped < 0.0) wrapped += TRAIL_SIZE as f32;
  if (wrapped >= (TRAIL_SIZE as f32)) wrapped -= TRAIL_SIZE as f32;
  return wrapped;
}

function moveAgents(res: SlimeMoveLayout, ctx: ComputeInvocation): void {
  const index: u32 = ctx.globalId.x;
  const agent: Agent = res.agents.get(index);
  const forwardAngle = new Vec2f(agent.heading, agent.heading);
  const forwardX: f32 = wrapTrail(
    agent.position.x + forwardAngle.cos().x * SENSOR_DISTANCE,
  );
  const forwardY: f32 = wrapTrail(
    agent.position.y + forwardAngle.sin().x * SENSOR_DISTANCE,
  );
  const leftAngle: f32 = agent.heading + SENSOR_ANGLE;
  const leftDirection = new Vec2f(leftAngle, leftAngle);
  const leftX: f32 = wrapTrail(
    agent.position.x + leftDirection.cos().x * SENSOR_DISTANCE,
  );
  const leftY: f32 = wrapTrail(
    agent.position.y + leftDirection.sin().x * SENSOR_DISTANCE,
  );
  const rightAngle: f32 = agent.heading - SENSOR_ANGLE;
  const rightDirection = new Vec2f(rightAngle, rightAngle);
  const rightX: f32 = wrapTrail(
    agent.position.x + rightDirection.cos().x * SENSOR_DISTANCE,
  );
  const rightY: f32 = wrapTrail(
    agent.position.y + rightDirection.sin().x * SENSOR_DISTANCE,
  );
  const forward: f32 = res.sense.load(
    new Vec2i(forwardX as i32, forwardY as i32),
  ).x;
  const left: f32 = res.sense.load(new Vec2i(leftX as i32, leftY as i32)).x;
  const right: f32 = res.sense.load(new Vec2i(rightX as i32, rightY as i32)).x;
  if (left > forward && left > right) {
    agent.heading += TURN_SPEED;
  } else if (right > forward && right > left) {
    agent.heading -= TURN_SPEED;
  }
  const stepDirection = new Vec2f(agent.heading, agent.heading);
  agent.position.x = wrapTrail(
    agent.position.x + stepDirection.cos().x * STEP_SIZE,
  );
  agent.position.y = wrapTrail(
    agent.position.y + stepDirection.sin().x * STEP_SIZE,
  );
  res.agents.set(index, agent);
  const cell = new Vec2i(agent.position.x as i32, agent.position.y as i32);
  const previous: f32 = res.trail.load(cell).x;
  res.trail.store(cell, new Vec4f(previous + DEPOSIT_AMOUNT, 0.0, 0.0, 1.0));
}

function diffuseTrail(res: SlimeDiffuseLayout, ctx: ComputeInvocation): void {
  const x: i32 = ctx.globalId.x as i32;
  const y: i32 = ctx.globalId.y as i32;
  const limit: i32 = TRAIL_SIZE as i32;
  const left: i32 = x > 0 ? x - 1 : limit - 1;
  const right: i32 = x + 1 < limit ? x + 1 : 0;
  const down: i32 = y > 0 ? y - 1 : limit - 1;
  const up: i32 = y + 1 < limit ? y + 1 : 0;
  const center: f32 = res.source.load(new Vec2i(x, y)).x;
  const neighbors: f32 = (
    res.source.load(new Vec2i(left, y)).x
    + res.source.load(new Vec2i(right, y)).x
    + res.source.load(new Vec2i(x, down)).x
    + res.source.load(new Vec2i(x, up)).x
  ) * 0.15;
  const value: f32 = (center * 0.4 + neighbors) * TRAIL_DECAY;
  res.target.store(new Vec2i(x, y), new Vec4f(value, 0.0, 0.0, 1.0));
}

function slimeVertex(
  res: SlimeRenderLayout,
  value: Vertex,
  ctx: VertexInvocation,
): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

function slimeFragment(
  res: SlimeRenderLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  let x: u32 = (input.uv.x * (TRAIL_SIZE as f32)) as u32;
  let y: u32 = (input.uv.y * (TRAIL_SIZE as f32)) as u32;
  if (x >= TRAIL_SIZE) x = TRAIL_SIZE - 1;
  if (y >= TRAIL_SIZE) y = TRAIL_SIZE - 1;
  let amount: f32 = res.trail.load(new Vec2i(x as i32, y as i32)).x;
  if (amount > 1.0) amount = 1.0;
  return new Vec4f(
    0.008 + amount * 0.18,
    0.012 + amount * 0.82,
    0.018 + amount * 0.46,
    1.0,
  );
}

export const slimeMove: ComputePipelineSpec = computePipeline<SlimeMoveLayout>(
  moveAgents,
  { name: "slimeMove", workgroupSize: [64, 1, 1] },
);

export const slimeDiffuse: ComputePipelineSpec = computePipeline<SlimeDiffuseLayout>(
  diffuseTrail,
  { name: "slimeDiffuse", workgroupSize: [8, 8, 1] },
);

export const slimeRender: RenderPipelineSpec = renderPipelineL<
  SlimeRenderLayout,
  Vertex,
  Varyings
>(slimeVertex, slimeFragment, {
  format: "bgra8unorm",
  topology: "triangle-strip",
});

let activeDevice: GPUHostOwnedDevice | null = null;
let activeMove: ComputePipeline | null = null;
let activeDiffuse: ComputePipeline | null = null;
let activeRender: RenderPipeline | null = null;
let activeMoveA: GPUBindGroup | null = null;
let activeMoveB: GPUBindGroup | null = null;
let activeDiffuseAB: GPUBindGroup | null = null;
let activeDiffuseBA: GPUBindGroup | null = null;
let activeRenderA: GPUBindGroup | null = null;
let activeRenderB: GPUBindGroup | null = null;
let activeVertices: GPUBuffer | null = null;
let activeAgents: GPUBuffer | null = null;
let activeTrailA: GPUTexture | null = null;
let activeTrailB: GPUTexture | null = null;
let activeViewA: GPUTextureView | null = null;
let activeViewB: GPUTextureView | null = null;
let frameCount: u32 = 0;

function zeroTrail(): Vec4f[] {
  const pixels: Vec4f[] = [];
  let index: u32 = 0;
  while (index < TRAIL_SIZE * TRAIL_SIZE) {
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
  if (format !== slimeRender_TARGET_FORMAT) {
    print(`FAIL format expected=${slimeRender_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const vertices = hostDevice.createBuffer({
    label: "slime-vertices",
    size: (Vertex_STRIDE * 4) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  const agents = hostDevice.createBuffer({
    label: "slime-agents",
    size: (Agent_STRIDE * AGENT_COUNT) as u64,
    usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
  });
  const textureUsage: u64 = GPUTextureUsage.STORAGE_BINDING + GPUTextureUsage.COPY_DST;
  const trailA = hostDevice.createTexture({
    label: "slime-trail-a",
    size: { width: TRAIL_SIZE, height: TRAIL_SIZE },
    format: "r32float",
    usage: textureUsage,
  });
  const trailB = hostDevice.createTexture({
    label: "slime-trail-b",
    size: { width: TRAIL_SIZE, height: TRAIL_SIZE },
    format: "r32float",
    usage: textureUsage,
  });
  const viewA = trailA.createView();
  const viewB = trailB.createView();
  const vertexValues: FixedArray<Vertex, 4> = [
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(1.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 1.0)),
    new Vertex(new Vec2f(1.0, 1.0)),
  ];
  const agentBytes: u8[] = [];
  let agentIndex: u32 = 0;
  while (agentIndex < AGENT_COUNT) {
    const heading: f32 = (agentIndex as f32) * TAU / (AGENT_COUNT as f32);
    const radius: f32 = 10.0 + ((agentIndex % 29) as f32) * 0.12;
    const position = new Vec2f(
      (TRAIL_SIZE as f32) * 0.5 + (Math.cos(heading as f64) as f32) * radius,
      (TRAIL_SIZE as f32) * 0.5 + (Math.sin(heading as f64) as f32) * radius,
    );
    const bytes: u8[] = Context.bytesOf<Agent>(new Agent(position, heading + 1.5707963));
    let byteIndex: i32 = 0;
    while (byteIndex < bytes.length) {
      agentBytes.push(bytes[byteIndex]);
      byteIndex += 1;
    }
    agentIndex += 1;
  }
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 4>>(vertexValues));
  queue.writeBuffer(agents, 0, agentBytes);
  const empty: Vec4f[] = zeroTrail();
  writeTexturePixels(queue, trailA, empty, TRAIL_SIZE, TRAIL_SIZE);
  writeTexturePixels(queue, trailB, empty, TRAIL_SIZE, TRAIL_SIZE);

  hostDevice.pushErrorScope("validation");
  const movePipeline = createComputePipelineHost(
    hostDevice,
    slimeMove_WGSL,
    slimeMove_ENTRY,
    [slimeMove_LAYOUT0],
    [64, 1, 1],
  );
  const diffusePipeline = createComputePipelineHost(
    hostDevice,
    slimeDiffuse_WGSL,
    slimeDiffuse_ENTRY,
    [slimeDiffuse_LAYOUT0],
    [8, 8, 1],
  );
  const renderPipeline = createRenderPipelineHost(
    hostDevice,
    slimeRender_WGSL,
    slimeRender_VERTEX_ENTRY,
    slimeRender_FRAGMENT_ENTRY,
    [slimeRender_LAYOUT0],
    [slimeRender_VERTEX_LAYOUT0],
    slimeRender,
  );
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    renderPipeline.dispose();
    diffusePipeline.dispose();
    movePipeline.dispose();
    viewB.dispose();
    viewA.dispose();
    trailB.dispose();
    trailA.dispose();
    agents.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }

  using moveLayout = movePipeline.bindGroupLayout(0);
  using diffuseLayout = diffusePipeline.bindGroupLayout(0);
  using renderLayout = renderPipeline.bindGroupLayout(0);
  const moveA = createBindGroupHost(hostDevice, moveLayout, slimeMove_LAYOUT0, [
    bufferResource(agents),
    textureResource(viewA),
    textureResource(viewA),
  ]);
  const moveB = createBindGroupHost(hostDevice, moveLayout, slimeMove_LAYOUT0, [
    bufferResource(agents),
    textureResource(viewB),
    textureResource(viewB),
  ]);
  const diffuseAB = createBindGroupHost(hostDevice, diffuseLayout, slimeDiffuse_LAYOUT0, [
    textureResource(viewA),
    textureResource(viewB),
  ]);
  const diffuseBA = createBindGroupHost(hostDevice, diffuseLayout, slimeDiffuse_LAYOUT0, [
    textureResource(viewB),
    textureResource(viewA),
  ]);
  const renderA = createBindGroupHost(hostDevice, renderLayout, slimeRender_LAYOUT0, [
    textureResource(viewA),
  ]);
  const renderB = createBindGroupHost(hostDevice, renderLayout, slimeRender_LAYOUT0, [
    textureResource(viewB),
  ]);
  activeDevice = hostDevice;
  activeMove = movePipeline;
  activeDiffuse = diffusePipeline;
  activeRender = renderPipeline;
  activeMoveA = moveA;
  activeMoveB = moveB;
  activeDiffuseAB = diffuseAB;
  activeDiffuseBA = diffuseBA;
  activeRenderA = renderA;
  activeRenderB = renderB;
  activeVertices = vertices;
  activeAgents = agents;
  activeTrailA = trailA;
  activeTrailB = trailB;
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
  const movePipeline = activeMove;
  const diffusePipeline = activeDiffuse;
  const renderPipeline = activeRender;
  const moveA = activeMoveA;
  const moveB = activeMoveB;
  const diffuseAB = activeDiffuseAB;
  const diffuseBA = activeDiffuseBA;
  const renderA = activeRenderA;
  const renderB = activeRenderB;
  const vertices = activeVertices;
  if (device === null) return;
  if (movePipeline === null) return;
  if (diffusePipeline === null) return;
  if (renderPipeline === null) return;
  if (moveA === null) return;
  if (moveB === null) return;
  if (diffuseAB === null) return;
  if (diffuseBA === null) return;
  if (renderA === null) return;
  if (renderB === null) return;
  if (vertices === null) return;
  const readsA: boolean = frameCount % 2 === 0;
  const moveGroup: GPUBindGroup = readsA ? moveA : moveB;
  const diffuseGroup: GPUBindGroup = readsA ? diffuseAB : diffuseBA;
  const displayGroup: GPUBindGroup = readsA ? renderB : renderA;
  using encoder = device.createCommandEncoderDefault();
  movePipeline.dispatch(encoder, [moveGroup], AGENT_COUNT / 64, 1, 1);
  diffusePipeline.dispatch(
    encoder,
    [diffuseGroup],
    TRAIL_SIZE / 8,
    TRAIL_SIZE / 8,
    1,
  );
  const target = new GPUTextureView(view);
  using renderPass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.008, g: 0.012, b: 0.018, a: 1.0 },
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
  using queue = device.queue();
  queue.submit([command]);
  frameCount += 1;
}

export function shutdown(): void {
  if (activeRenderB !== null) activeRenderB.dispose();
  if (activeRenderA !== null) activeRenderA.dispose();
  if (activeDiffuseBA !== null) activeDiffuseBA.dispose();
  if (activeDiffuseAB !== null) activeDiffuseAB.dispose();
  if (activeMoveB !== null) activeMoveB.dispose();
  if (activeMoveA !== null) activeMoveA.dispose();
  if (activeViewB !== null) activeViewB.dispose();
  if (activeViewA !== null) activeViewA.dispose();
  if (activeTrailB !== null) activeTrailB.dispose();
  if (activeTrailA !== null) activeTrailA.dispose();
  if (activeAgents !== null) activeAgents.dispose();
  if (activeVertices !== null) activeVertices.dispose();
  if (activeRender !== null) activeRender.dispose();
  if (activeDiffuse !== null) activeDiffuse.dispose();
  if (activeMove !== null) activeMove.dispose();
  activeRenderB = null;
  activeRenderA = null;
  activeDiffuseBA = null;
  activeDiffuseAB = null;
  activeMoveB = null;
  activeMoveA = null;
  activeViewB = null;
  activeViewA = null;
  activeTrailB = null;
  activeTrailA = null;
  activeAgents = null;
  activeVertices = null;
  activeRender = null;
  activeDiffuse = null;
  activeMove = null;
  activeDevice = null;
  frameCount = 0;
}
