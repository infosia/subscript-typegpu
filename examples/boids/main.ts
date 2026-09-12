// example: boids
// Updates a three-rule flock in double-buffered storage and draws velocity-oriented instances.
// This port uses one fixed rule preset with 96 boids and drops the upstream preset and color
// controls. A deterministic grid seed replaces the random start, so every run is identical.
// Ported from TypeGPU's boids example (https://github.com/software-mansion/TypeGPU).

import {
  ComputeInvocation,
  ComputePipeline,
  ComputePipelineSpec,
  FragmentInvocation,
  MutStorage,
  RenderPipelineSpec,
  RenderPipeline,
  Storage,
  VertexInvocation,
  bufferResource,
  computePipeline,
  createBindGroupHost,
  createComputePipelineHost,
  createRenderPipelineHost,
  renderPipelineInstanced,
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
  Boid_STRIDE,
  Vertex_STRIDE,
  boidRender_FRAGMENT_ENTRY,
  boidRender_TARGET_FORMAT,
  boidRender_VERTEX_ENTRY,
  boidRender_VERTEX_LAYOUT0,
  boidRender_VERTEX_LAYOUT1,
  boidRender_WGSL,
  boidUpdate_ENTRY,
  boidUpdate_LAYOUT0,
  boidUpdate_WGSL,
} from "./main.typegpu";

// One committed preset replaces the upstream preset buttons. Each invocation scans every
// other boid, so the cost grows with the square of BOID_COUNT, and upstream flies 1000.
const BOID_COUNT: u32 = 96;
const PERCEPTION_SQUARED: f32 = 0.10;
const COHESION_WEIGHT: f32 = 0.0018;
const ALIGNMENT_WEIGHT: f32 = 0.028;
const SEPARATION_WEIGHT: f32 = 0.0014;
const MAX_SPEED: f32 = 0.018;

// The vertex record. The generator emits Vertex_STRIDE from this class, so the host code
// never counts bytes.
@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// The instance record. One buffer serves the kernel as storage and the draw as an instance
// vertex buffer. No copy moves the state between the two passes.
@CStruct
class Boid {
  position: Vec2f;
  velocity: Vec2f;

  constructor(position: Vec2f, velocity: Vec2f) {
    this.position = position;
    this.velocity = velocity;
  }
}

// The inter-stage record. The field named `position` becomes the clip-space builtin, and
// `color` becomes location 0.
@CStruct
class Varyings {
  position: Vec4f;
  color: Vec4f;

  constructor(position: Vec4f, color: Vec4f) {
    this.position = position;
    this.color = color;
  }
}

// The compute bind group. `Storage` is read-only and `MutStorage` is read-write, so the
// types state that one dispatch never writes the buffer it reads.
class BoidLayout {
  previous: Storage<Boid>;
  next: MutStorage<Boid>;

  constructor(previous: Storage<Boid>, next: MutStorage<Boid>) {
    this.previous = previous;
    this.next = next;
  }
}

// Each invocation reads the complete previous flock and writes one next-state record.
function updateBoids(res: BoidLayout, ctx: ComputeInvocation): void {
  const index: u32 = ctx.globalId.x;
  const boid: Boid = res.previous[index];
  let positionX: f32 = 0.0;
  let positionY: f32 = 0.0;
  let velocityX: f32 = 0.0;
  let velocityY: f32 = 0.0;
  let separationX: f32 = 0.0;
  let separationY: f32 = 0.0;
  let neighbors: u32 = 0;
  // TypeGPU gives each rule its own radius and sums raw offsets for separation.
  // This port uses one perception radius and weights separation by inverse square distance.
  for (let otherIndex: u32 = 0; otherIndex < BOID_COUNT; otherIndex += 1) {
    if (otherIndex !== index) {
      const other: Boid = res.previous[otherIndex];
      const deltaX: f32 = other.position.x - boid.position.x;
      const deltaY: f32 = other.position.y - boid.position.y;
      const distanceSquared: f32 = deltaX * deltaX + deltaY * deltaY;
      if (distanceSquared < PERCEPTION_SQUARED && distanceSquared > 0.00001) {
        positionX += other.position.x;
        positionY += other.position.y;
        velocityX += other.velocity.x;
        velocityY += other.velocity.y;
        separationX -= deltaX / distanceSquared;
        separationY -= deltaY / distanceSquared;
        neighbors += 1;
      }
    }
  }
  // Cohesion steers to the mean neighbor position, alignment to the mean neighbor velocity,
  // and separation away from each neighbor.
  if (neighbors > 0) {
    const inverseCount: f32 = 1.0 / (neighbors as f32);
    boid.velocity.x += (positionX * inverseCount - boid.position.x) * COHESION_WEIGHT;
    boid.velocity.y += (positionY * inverseCount - boid.position.y) * COHESION_WEIGHT;
    boid.velocity.x += (velocityX * inverseCount - boid.velocity.x) * ALIGNMENT_WEIGHT;
    boid.velocity.y += (velocityY * inverseCount - boid.velocity.y) * ALIGNMENT_WEIGHT;
    boid.velocity.x += separationX * SEPARATION_WEIGHT;
    boid.velocity.y += separationY * SEPARATION_WEIGHT;
  }
  // The speed clamp holds the flock together. The three rules add velocity without a bound of
  // their own.
  const speed: f32 = boid.velocity.length();
  if (speed > MAX_SPEED) {
    boid.velocity.x = boid.velocity.x * MAX_SPEED / speed;
    boid.velocity.y = boid.velocity.y * MAX_SPEED / speed;
  }
  boid.position.x += boid.velocity.x;
  boid.position.y += boid.velocity.y;
  // The domain wraps just outside clip space, so a boid leaves the surface before it reappears
  // on the far side.
  if (boid.position.x < -1.05) boid.position.x = 1.05;
  if (boid.position.x > 1.05) boid.position.x = -1.05;
  if (boid.position.y < -1.05) boid.position.y = 1.05;
  if (boid.position.y > 1.05) boid.position.y = -1.05;
  res.next[index] = boid;
}

// The rotation matches upstream. Upstream colors each triangle from a palette uniform
// and the heading angle, and this port derives the color from speed.
function boidVertex(value: Vertex, boid: Boid, ctx: VertexInvocation): Varyings {
  const speed: f32 = boid.velocity.length();
  let directionX: f32 = 0.0;
  let directionY: f32 = 1.0;
  if (speed > 0.00001) {
    directionX = boid.velocity.x / speed;
    directionY = boid.velocity.y / speed;
  }
  const worldX: f32 = value.position.x * directionY + value.position.y * directionX;
  const worldY: f32 = value.position.x * -directionX + value.position.y * directionY;
  const tone: f32 = 0.35 + speed * 24.0;
  return new Varyings(
    new Vec4f(boid.position.x + worldX, boid.position.y + worldY, 0.0, 1.0),
    new Vec4f(0.2 + tone * 0.3, 0.45 + tone * 0.35, 0.95, 1.0),
  );
}

function boidFragment(input: Varyings, ctx: FragmentInvocation): Vec4f {
  return input.color;
}

// `guarded: true` makes the kernel skip an invocation past the end of the flock. A workgroup
// of 64 over 96 boids starts 128 invocations, and 32 of them do no work.
export const boidUpdate: ComputePipelineSpec = computePipeline<BoidLayout>(updateBoids, {
  name: "boidUpdate",
  workgroupSize: [64, 1, 1],
  guarded: true,
});

// The instanced form takes a vertex schema and an instance schema. It needs no bind group,
// because the boid state arrives through vertex buffer slot 1.
export const boidRender: RenderPipelineSpec = renderPipelineInstanced<
  Vertex,
  Boid,
  Varyings
>(boidVertex, boidFragment, { format: "bgra8unorm" });

// The host calls `init`, `frame`, and `shutdown` separately, so every handle that outlives
// `init` lives in module state. A `using` declaration disposes a handle too early here.
let activeDevice: GPUHostOwnedDevice | null = null;
let activeCompute: ComputePipeline | null = null;
let activeRender: RenderPipeline | null = null;
let activeGroupAB: GPUBindGroup | null = null;
let activeGroupBA: GPUBindGroup | null = null;
let activeVertices: GPUBuffer | null = null;
let activeBoidsA: GPUBuffer | null = null;
let activeBoidsB: GPUBuffer | null = null;
let frameCount: u32 = 0;

// `init` runs once, after the host configures the surface. It creates every long-lived
// resource. The instance and the device stay with the host.
export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  // The generator pins the target format into the pipeline. A mismatch with the host surface
  // fails here, not inside pipeline creation.
  if (format !== boidRender_TARGET_FORMAT) {
    print(`FAIL format expected=${boidRender_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The wrapper adapts the host handles to the API layer. It carries no `dispose`, because
  // the host owns the device.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // One narrow triangle points along +y in local space. The vertex stage turns it onto the
  // velocity of each boid.
  const vertexValues: FixedArray<Vertex, 3> = [
    new Vertex(new Vec2f(0.0, 0.035)),
    new Vertex(new Vec2f(-0.014, -0.022)),
    new Vertex(new Vec2f(0.014, -0.022)),
  ];
  // The vertex buffer holds the three corners once. Every instance reads the same three.
  const vertices = hostDevice.createBuffer({
    label: "boids-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // Two state buffers alternate roles each frame. Each one carries STORAGE for the kernel and
  // VERTEX for the instanced draw.
  const boidsA = hostDevice.createBuffer({
    label: "boids-state-a",
    size: (Boid_STRIDE * BOID_COUNT) as u64,
    usage: GPUBufferUsage.STORAGE + GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  const boidsB = hostDevice.createBuffer({
    label: "boids-state-b",
    size: (Boid_STRIDE * BOID_COUNT) as u64,
    usage: GPUBufferUsage.STORAGE + GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>(vertexValues));
  // A fixed grid seed replaces the upstream random start, so every run shows the same flock.
  // Both buffers take the same bytes, so neither one starts with undefined content.
  for (let index: u32 = 0; index < BOID_COUNT; index += 1) {
    const column: f32 = (index % 12) as f32;
    const row: f32 = (index / 12) as f32;
    const direction: f32 = (index % 7) as f32 - 3.0;
    const value = new Boid(
      new Vec2f(-0.88 + column * 0.16, -0.7 + row * 0.2),
      new Vec2f(direction * 0.002, 0.006 + ((index % 5) as f32) * 0.0006),
    );
    const offset: u64 = (index as u64) * (Boid_STRIDE as u64);
    const bytes: u8[] = Context.bytesOf<Boid>(value);
    queue.writeBuffer(boidsA, offset, bytes);
    queue.writeBuffer(boidsB, offset, bytes);
  }

  // One error scope covers both pipeline creations. The layers return the failure as a value,
  // so a `null` check replaces an exception.
  hostDevice.pushErrorScope("validation");
  // The call takes the generated WGSL, the entry name, the bind group layout, and the
  // workgroup size that the declaration above names.
  const computePipeline = createComputePipelineHost(
    hostDevice,
    boidUpdate_WGSL,
    boidUpdate_ENTRY,
    [boidUpdate_LAYOUT0],
    [64, 1, 1],
  );
  // The empty layout list matches a pipeline with no bind group. The two vertex layouts cover
  // slot 0 per vertex and slot 1 per instance.
  const renderPipeline = createRenderPipelineHost(
    hostDevice,
    boidRender_WGSL,
    boidRender_VERTEX_ENTRY,
    boidRender_FRAGMENT_ENTRY,
    [],
    [boidRender_VERTEX_LAYOUT0, boidRender_VERTEX_LAYOUT1],
    boidRender,
  );
  const validationError = hostDevice.popErrorScope();
  // The error path disposes every handle that the failed run already created, because no
  // finalizer runs later.
  if (validationError !== null) {
    renderPipeline.dispose();
    computePipeline.dispose();
    boidsB.dispose();
    boidsA.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  // The native layout comes from the pipeline. Both bind groups read it at creation, so `using`
  // releases the handle right after.
  using computeBindLayout = computePipeline.bindGroupLayout(0);
  // The guard buffer carries the dispatched thread count. It gives the kernel the bound
  // that TypeGPU's createGuardedComputePipeline applies for the caller.
  const groupAB = createBindGroupHost(
    hostDevice,
    computeBindLayout,
    boidUpdate_LAYOUT0,
    [bufferResource(boidsA), bufferResource(boidsB)],
    computePipeline.guardBuffer(0),
  );
  // The second group swaps the two buffers. Both groups exist once, so no frame builds a bind
  // group.
  const groupBA = createBindGroupHost(
    hostDevice,
    computeBindLayout,
    boidUpdate_LAYOUT0,
    [bufferResource(boidsB), bufferResource(boidsA)],
    computePipeline.guardBuffer(0),
  );
  // The state moves to module scope only after every step passes. A failed `init` leaves the
  // fields null, and `frame` returns at once.
  activeDevice = hostDevice;
  activeCompute = computePipeline;
  activeRender = renderPipeline;
  activeGroupAB = groupAB;
  activeGroupBA = groupBA;
  activeVertices = vertices;
  activeBoidsA = boidsA;
  activeBoidsB = boidsB;
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
  const device = activeDevice;
  const computePipeline = activeCompute;
  const renderPipeline = activeRender;
  const groupAB = activeGroupAB;
  const groupBA = activeGroupBA;
  const vertices = activeVertices;
  const boidsA = activeBoidsA;
  const boidsB = activeBoidsB;
  // A null field means `init` failed or never ran. The frame returns, because the layers
  // report failure as a value.
  if (device === null) return;
  if (computePipeline === null) return;
  if (renderPipeline === null) return;
  if (groupAB === null) return;
  if (groupBA === null) return;
  if (vertices === null) return;
  if (boidsA === null) return;
  if (boidsB === null) return;
  const useAB: boolean = frameCount % 2 === 0;
  // The frame swaps the storage roles and renders the buffer that compute writes.
  const computeGroup: GPUBindGroup = useAB ? groupAB : groupBA;
  const instanceBuffer: GPUBuffer = useAB ? boidsB : boidsA;
  frameCount += 1;
  // One encoder records the compute pass and the render pass. The device runs them in the
  // recorded order, so the draw reads the state that the kernel wrote.
  using encoder = device.createCommandEncoderDefault();
  // `dispatchThreads` takes a thread count and divides it by the workgroup size. TypeGPU
  // spells the same call on a guarded pipeline.
  computePipeline.dispatchThreads(encoder, [computeGroup], BOID_COUNT, 1, 1);
  // The host owns the swapchain view. The wrapper adds no ownership, and `shutdown` never
  // disposes it.
  const target = new GPUTextureView(view);
  // The clear paints the background, because the triangles cover a small part of the surface.
  using renderPass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.025, g: 0.035, b: 0.065, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  // The viewport and the scissor follow the current surface size, because the host resizes the
  // swapchain without a new pipeline.
  renderPass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  renderPass.setScissorRect(0, 0, width, height);
  // The empty group list matches the pipeline with no bind group. The two buffers fill vertex
  // slot 0 and instance slot 1.
  renderPipeline.bind(renderPass, [], [vertices, instanceBuffer]);
  // Three vertices per instance, one instance per boid. The instance index selects the record
  // in slot 1.
  renderPass.draw(3, BOID_COUNT);
  renderPass.end();
  // `finishDefault` closes the encoder, and `submit` hands the command buffer to the queue.
  // The host presents the surface after `frame` returns.
  using command = encoder.finishDefault();
  using queue = device.queue();
  queue.submit([command]);
}

// The host calls `shutdown` once, before it releases the device. The bind groups go first,
// because they name the other handles.
export function shutdown(): void {
  if (activeGroupBA !== null) activeGroupBA.dispose();
  if (activeGroupAB !== null) activeGroupAB.dispose();
  if (activeBoidsB !== null) activeBoidsB.dispose();
  if (activeBoidsA !== null) activeBoidsA.dispose();
  if (activeVertices !== null) activeVertices.dispose();
  if (activeRender !== null) activeRender.dispose();
  if (activeCompute !== null) activeCompute.dispose();
  // The null assignments make a second `shutdown` call safe.
  activeGroupBA = null;
  activeGroupAB = null;
  activeBoidsB = null;
  activeBoidsA = null;
  activeVertices = null;
  activeRender = null;
  activeCompute = null;
  activeDevice = null;
}
