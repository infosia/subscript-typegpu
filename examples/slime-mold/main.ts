// example: slime-mold
// Moves trail-sensing agents and diffuses their deposits through a swapped texture pair.
// TypeGPU exposes move speed, sensor angle, sensor distance, turn speed, and evaporation
// rate as sliders. This port fixes step 1, sensor angle 0.5, sensor distance 5, turn 0.32,
// and decay 0.96.
// TypeGPU runs 200000 agents over a canvas-sized texture pair, and one deposit adds 1.0 to
// a cell. This port fixes 4096 agents, a 256-square trail, and deposit 0.2.
// TypeGPU seeds agents in a disc and points them toward its center. This port derives
// full-grid positions and full-circle headings from a Wang-seeded xorshift32 PRNG.
// The same PRNG adds centered heading jitter in [-0.15, 0.15) radians each frame.
// It also selects a turn when both sides win or all samples tie at zero. At the border, this
// port wraps the trail instead of the TypeGPU clamp, reflection, and jitter.
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
  RandomF32,
  randF32,
  randSeed,
} from "./typegpu-noise";
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

// Distances count trail cells and angles count radians. `dispatch` takes workgroup counts, so
// 256 divides by the diffuse workgroup of 8 and 4096 divides by the move workgroup of 64.
const TRAIL_SIZE: u32 = 256;
const AGENT_COUNT: u32 = 4096;
const SENSOR_DISTANCE: f32 = 5.0;
const SENSOR_ANGLE: f32 = 0.5;
const TURN_SPEED: f32 = 0.32;
const STEP_SIZE: f32 = 1.0;
const DEPOSIT_AMOUNT: f32 = 0.2;
const TRAIL_DECAY: f32 = 0.96;
const TAU: f32 = 6.2831855;

// One corner of the render strip. The generator derives `Vertex_STRIDE` from this class.
@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// One agent. The position counts trail cells, the heading counts radians, and `randomState`
// carries the PRNG state forward. TypeGPU stores position and angle and reseeds from the index.
@CStruct
class Agent {
  position: Vec2f;
  heading: f32;
  randomState: u32;

  constructor(position: Vec2f, heading: f32, randomState: u32) {
    this.position = position;
    this.heading = heading;
    this.randomState = randomState;
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

// The move layout binds the agents, the trail they sense, and the trail they mark. A deposit
// reads and then writes the same cell, so the target binding is read-write.
// The two textures swap roles every frame, so one layout serves both directions.
class SlimeMoveLayout {
  agents!: MutStorage<Agent>;
  sense!: ReadStorageTexture2d<R32float>;
  trail!: ReadWriteStorageTexture2d<R32float>;
}

// The diffuse layout reads one trail and writes the other, so no invocation reads a cell that
// another invocation already changed.
class SlimeDiffuseLayout {
  source!: ReadStorageTexture2d<R32float>;
  target!: StorageTexture2d<R32float>;
}

// The render layout reads one trail. Two bind groups on it select the texture the frame shows.
class SlimeRenderLayout {
  trail!: ReadStorageTexture2d<R32float>;
}

// The trail is a torus. A coordinate one step outside the grid returns on the opposite edge.
// TypeGPU clamps at the border, reflects the heading, and adds jitter instead.
function wrapTrail(value: f32): f32 {
  let wrapped: f32 = value;
  if (wrapped < 0.0) wrapped += TRAIL_SIZE as f32;
  if (wrapped >= (TRAIL_SIZE as f32)) wrapped -= TRAIL_SIZE as f32;
  return wrapped;
}

// The helper returns the trail cell one sensor step along the angle, wrapped into the grid.
// A texture wrapper cannot be a helper parameter under K5, so the caller loads the cell.
// TypeGPU's sense helper reads the texture itself and returns the summed sample.
function senseTrailCell(position: Vec2f, angle: f32): Vec2i {
  // One vector carries the angle twice, so `cos()` and `sin()` reach the WGSL builtins.
  // A scalar `Math.sin` needs an `f64` cast, and K12 rejects that cast.
  const angles = new Vec2f(angle, angle);
  const x: f32 = wrapTrail(
    position.x + angles.cos().x * SENSOR_DISTANCE,
  );
  const y: f32 = wrapTrail(
    position.y + angles.sin().x * SENSOR_DISTANCE,
  );
  return new Vec2i(x as i32, y as i32);
}

// One invocation senses three cells, turns the agent, moves it, and deposits a trail mark.
function moveAgents(res: SlimeMoveLayout, ctx: ComputeInvocation): void {
  const index: u32 = ctx.globalId.x;
  const agent: Agent = res.agents[index];
  // The stored state advances once per frame, so each agent walks its own fixed sequence.
  const jitter: RandomF32 = randF32(agent.randomState);
  agent.randomState = jitter.state;
  agent.heading += (jitter.value - 0.5) * 0.3;
  const forward: f32 = res.sense.load(
    senseTrailCell(agent.position, agent.heading),
  ).x;
  const left: f32 = res.sense.load(
    senseTrailCell(agent.position, agent.heading + SENSOR_ANGLE),
  ).x;
  const right: f32 = res.sense.load(
    senseTrailCell(agent.position, agent.heading - SENSOR_ANGLE),
  ).x;
  // Two better sides trap the agent, and three zero samples give it nothing to follow. Both
  // cases pick a side at random. Otherwise the agent turns toward the stronger sensor.
  if (
    (left > forward && right > forward)
    || (forward === 0.0 && left === 0.0 && right === 0.0)
  ) {
    const random: RandomF32 = randF32(agent.randomState);
    agent.randomState = random.state;
    if ((agent.randomState & 1) === 0) {
      agent.heading += TURN_SPEED;
    } else {
      agent.heading -= TURN_SPEED;
    }
  } else if (right > left && right >= forward) {
    agent.heading -= TURN_SPEED;
  } else if (left > right && left >= forward) {
    agent.heading += TURN_SPEED;
  }
  // The agent moves one cell along its heading, and the wrap keeps it on the torus.
  const stepAngles = new Vec2f(agent.heading, agent.heading);
  agent.position.x = wrapTrail(
    agent.position.x + stepAngles.cos().x * STEP_SIZE,
  );
  agent.position.y = wrapTrail(
    agent.position.y + stepAngles.sin().x * STEP_SIZE,
  );
  res.agents[index] = agent;
  // The deposit adds to the cell the agent now occupies. The read and the write on one binding
  // need the read-write access the layout declares.
  const cell = new Vec2i(agent.position.x as i32, agent.position.y as i32);
  const previous: f32 = res.trail.load(cell).x;
  res.trail.store(cell, new Vec4f(previous + DEPOSIT_AMOUNT, 0.0, 0.0, 1.0));
}

// Each cell blends itself with its four wrapped neighbors. The five weights sum to one,
// so a flat trail keeps its value. TypeGPU averages a 3x3 window and subtracts the
// evaporation rate, and this port multiplies by a decay factor instead.
function diffuseTrail(res: SlimeDiffuseLayout, ctx: ComputeInvocation): void {
  const x: i32 = ctx.globalId.x as i32;
  const y: i32 = ctx.globalId.y as i32;
  const limit: i32 = TRAIL_SIZE as i32;
  // The neighbor indices wrap, so the diffusion crosses the border the same way the agents do.
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

// The full-screen strip scales the complete trail across the current window viewport.
// TypeGPU emits one oversized triangle from the vertex index and binds no vertex buffer.
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

// The fragment loads one trail cell and maps the amount through a fixed color ramp.
// TypeGPU samples a filtered rgba8unorm texture, so its trail is smooth between cells.
// A storage texture takes no sampler, so this port reads the nearest cell.
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

// The three declarations name the kernel, the layout type, and the workgroup size. The generator
// reads them ahead of the run and emits the WGSL, the entry names, and the layout facts.
// TypeGPU builds the same WGSL at run time from the kernel function.
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

// One holder for everything `init` creates. The host calls `init`, `frame`, and `shutdown` as
// separate entries, so the handles outlive each call and `shutdown` owns their release.
// TypeGPU frees the same resources through garbage collection and `root.destroy()`.
class SlimeState {
  device: GPUHostOwnedDevice;
  move: ComputePipeline;
  diffuse: ComputePipeline;
  render: RenderPipeline;
  moveAB: GPUBindGroup;
  moveBA: GPUBindGroup;
  diffuseAB: GPUBindGroup;
  diffuseBA: GPUBindGroup;
  renderA: GPUBindGroup;
  renderB: GPUBindGroup;
  vertices: GPUBuffer;
  agents: GPUBuffer;
  trailA: GPUTexture;
  trailB: GPUTexture;
  viewA: GPUTextureView;
  viewB: GPUTextureView;

  constructor(
    device: GPUHostOwnedDevice,
    move: ComputePipeline,
    diffuse: ComputePipeline,
    render: RenderPipeline,
    moveAB: GPUBindGroup,
    moveBA: GPUBindGroup,
    diffuseAB: GPUBindGroup,
    diffuseBA: GPUBindGroup,
    renderA: GPUBindGroup,
    renderB: GPUBindGroup,
    vertices: GPUBuffer,
    agents: GPUBuffer,
    trailA: GPUTexture,
    trailB: GPUTexture,
    viewA: GPUTextureView,
    viewB: GPUTextureView,
  ) {
    this.device = device;
    this.move = move;
    this.diffuse = diffuse;
    this.render = render;
    this.moveAB = moveAB;
    this.moveBA = moveBA;
    this.diffuseAB = diffuseAB;
    this.diffuseBA = diffuseBA;
    this.renderA = renderA;
    this.renderB = renderB;
    this.vertices = vertices;
    this.agents = agents;
    this.trailA = trailA;
    this.trailB = trailB;
    this.viewA = viewA;
    this.viewB = viewB;
  }
}

let activeState: SlimeState | null = null;
let frameCount: u32 = 0;

// The initial trail is empty. The upload takes one `Vec4f` per cell, and only the first
// component reaches an `r32float` texture.
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
  // The pipeline declares its target format literally. A surface with another format ends the
  // example before any draw.
  if (format !== slimeRender_TARGET_FORMAT) {
    print(`FAIL format expected=${slimeRender_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The host owns the device and the instance. The wrapper carries no `dispose`, so this example
  // never releases what it did not create.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // Buffer sizes come from the generated stride constants, never from a hand count.
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
  // The trail pair carries one `f32` per cell. `STORAGE_BINDING` lets a kernel load and store
  // the texture, and `COPY_DST` lets the upload below clear it.
  // TypeGPU sizes its pair from the canvas and keeps four channels per cell.
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
  // Every binding takes a view, not the texture. Both views live until `shutdown` releases them.
  const viewA = trailA.createView();
  const viewB = trailB.createView();
  // The four strip corners in clip space, in the order the triangle-strip topology needs.
  const vertexValues: FixedArray<Vertex, 4> = [
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(1.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 1.0)),
    new Vertex(new Vec2f(1.0, 1.0)),
  ];
  // Each agent index seeds the PRNG. Three samples set the full-grid position and
  // full-circle heading. The stored state continues the deterministic sequence.
  const agentBytes: u8[] = [];
  let agentIndex: u32 = 0;
  while (agentIndex < AGENT_COUNT) {
    let randomState: u32 = randSeed(agentIndex);
    let sample: RandomF32 = randF32(randomState);
    randomState = sample.state;
    const x: f32 = sample.value * (TRAIL_SIZE as f32);
    sample = randF32(randomState);
    randomState = sample.state;
    const y: f32 = sample.value * (TRAIL_SIZE as f32);
    sample = randF32(randomState);
    randomState = sample.state;
    const heading: f32 = sample.value * TAU;
    const position = new Vec2f(
      x,
      y,
    );
    const bytes: u8[] = Context.bytesOf<Agent>(
      new Agent(position, heading, randomState),
    );
    let byteIndex: i32 = 0;
    while (byteIndex < bytes.length) {
      agentBytes.push(bytes[byteIndex]);
      byteIndex += 1;
    }
    agentIndex += 1;
  }
  // Queue writes land before the commands that a later submit carries, so the agents and the
  // empty trails reach the device before the first dispatch.
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 4>>(vertexValues));
  queue.writeBuffer(agents, 0, agentBytes);
  const empty: Vec4f[] = zeroTrail();
  writeTexturePixels(queue, trailA, empty, TRAIL_SIZE, TRAIL_SIZE);
  writeTexturePixels(queue, trailB, empty, TRAIL_SIZE, TRAIL_SIZE);

  // The error scope catches a pipeline creation failure. TypeGPU rejects a promise. Here the pop
  // returns a value, so this example releases what it created and prints the first error line.
  hostDevice.pushErrorScope("validation");
  // Each compute pipeline takes the generated WGSL text, the entry name, the layout facts, and
  // the workgroup size. That size must equal the size in the declaration above.
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
  // The render pipeline also takes the vertex buffer layout and the declaration, which carries
  // the target format and the topology.
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

  // The pipeline reports the layout WebGPU built for group 0. Each call returns a new handle, and
  // the bind groups below need it only at creation.
  using moveLayout = movePipeline.bindGroupLayout(0);
  using diffuseLayout = diffusePipeline.bindGroupLayout(0);
  using renderLayout = renderPipeline.bindGroupLayout(0);
  // Six bind groups cover both trail directions for each pass and both display choices.
  // The resource order follows the field order of the layout class.
  const moveAB = createBindGroupHost(hostDevice, moveLayout, slimeMove_LAYOUT0, [
    bufferResource(agents),
    textureResource(viewA),
    textureResource(viewB),
  ]);
  const moveBA = createBindGroupHost(hostDevice, moveLayout, slimeMove_LAYOUT0, [
    bufferResource(agents),
    textureResource(viewB),
    textureResource(viewA),
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
  // The state reaches module scope only after every creation succeeds.
  activeState = new SlimeState(
    hostDevice,
    movePipeline,
    diffusePipeline,
    renderPipeline,
    moveAB,
    moveBA,
    diffuseAB,
    diffuseBA,
    renderA,
    renderB,
    vertices,
    agents,
    trailA,
    trailB,
    viewA,
    viewB,
  );
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
  // Frame parity diffuses A into B, then agents sense A and deposit into B.
  // The render pass displays B before the pair swaps on the next frame.
  const readsA: boolean = frameCount % 2 === 0;
  const moveGroup: GPUBindGroup = readsA ? active.moveAB : active.moveBA;
  const diffuseGroup: GPUBindGroup = readsA ? active.diffuseAB : active.diffuseBA;
  const displayGroup: GPUBindGroup = readsA ? active.renderB : active.renderA;
  // Each dispatch records its own compute pass, and the passes run in record order. The diffuse
  // pass fills the target first, so it never overwrites the deposits the move pass adds.
  using encoder = active.device.createCommandEncoderDefault();
  // The counts are workgroups, not threads. The trail needs 32 groups per axis, and the agents
  // need 64 groups on one axis.
  active.diffuse.dispatch(
    encoder,
    [diffuseGroup],
    TRAIL_SIZE / 8,
    TRAIL_SIZE / 8,
    1,
  );
  active.move.dispatch(encoder, [moveGroup], AGENT_COUNT / 64, 1, 1);
  // The host owns the presented view, so this example wraps it and releases nothing.
  const target = new GPUTextureView(view);
  using renderPass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.008, g: 0.012, b: 0.018, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  // The surface size changes when the window resizes, so both rectangles follow the frame size.
  renderPass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  renderPass.setScissorRect(0, 0, width, height);
  // Four vertices draw the strip, and the display group selects the trail the deposits reached.
  active.render.bind(renderPass, [displayGroup], [active.vertices]);
  renderPass.draw(4);
  renderPass.end();
  // The command buffer reaches the device queue after the encoder finishes. The frame count then
  // reverses the two texture roles for the next frame.
  using command = encoder.finishDefault();
  using queue = active.device.queue();
  queue.submit([command]);
  frameCount += 1;
}

// The host calls this once before it releases the device. This example disposes in reverse
// creation order, so a bind group never outlives the textures and the buffers it names.
// TypeGPU releases the same resources with one `root.destroy()` call.
export function shutdown(): void {
  if (activeState === null) return;
  const active = activeState;
  active.renderB.dispose();
  active.renderA.dispose();
  active.diffuseBA.dispose();
  active.diffuseAB.dispose();
  active.moveBA.dispose();
  active.moveAB.dispose();
  active.viewB.dispose();
  active.viewA.dispose();
  active.trailB.dispose();
  active.trailA.dispose();
  active.agents.dispose();
  active.vertices.dispose();
  active.render.dispose();
  active.diffuse.dispose();
  active.move.dispose();
  // The cleared state leaves no released handle reachable from module scope.
  activeState = null;
  frameCount = 0;
}
