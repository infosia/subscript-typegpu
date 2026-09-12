// example: confetti
// Advances colored particles in compute and draws them as instanced cards.
// This port starts from a fixed layout of 64 particles and drops the upstream randomize button.
// Constant gravity, a per-index spin, and edge wrap replace the upstream seeded sine motion.
// Ported from TypeGPU's confetti example (https://github.com/software-mansion/TypeGPU).

import {
  ComputeInvocation,
  ComputePipeline,
  ComputePipelineSpec,
  FragmentInvocation,
  MutStorage,
  RenderPipelineSpec,
  RenderPipeline,
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
  Particle_STRIDE,
  Vertex_STRIDE,
  confettiRender_FRAGMENT_ENTRY,
  confettiRender_TARGET_FORMAT,
  confettiRender_VERTEX_ENTRY,
  confettiRender_VERTEX_LAYOUT0,
  confettiRender_VERTEX_LAYOUT1,
  confettiRender_WGSL,
  confettiUpdate_ENTRY,
  confettiUpdate_LAYOUT0,
  confettiUpdate_WGSL,
} from "./main.typegpu";

// 64 particles fill one workgroup of the update kernel exactly.
// TypeGPU runs 200 particles and randomizes them from a button.
const PARTICLE_COUNT: u32 = 64;

// One corner of the card, in normalized device coordinates around the particle center.
// The four corners form a triangle strip, and every instance reuses the same four.
@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// One instance record. One buffer holds them all. The update kernel binds it as mutable
// storage, and the draw reads it as the instance stream, so no copy moves between passes.
@CStruct
class Particle {
  position: Vec2f;
  velocity: Vec2f;
  color: Vec4f;
  angle: f32;

  constructor(position: Vec2f, velocity: Vec2f, color: Vec4f, angle: f32) {
    this.position = position;
    this.velocity = velocity;
    this.color = color;
    this.angle = angle;
  }
}

// The vertex stage returns this record and the fragment stage receives it.
// The field named position becomes the clip position, and color becomes location 0.
@CStruct
class Varyings {
  position: Vec4f;
  color: Vec4f;

  constructor(position: Vec4f, color: Vec4f) {
    this.position = position;
    this.color = color;
  }
}

// A bind group layout is a class here, not a runtime object. Only the update kernel binds
// the particles. The draw reaches the same buffer through a vertex slot instead.
class ParticleLayout {
  particles: MutStorage<Particle>;

  constructor(particles: MutStorage<Particle>) {
    this.particles = particles;
  }
}

// One storage buffer becomes the instance stream after this pass completes.
// TypeGPU adds a seeded sine offset to the position and never wraps.
// This port applies constant gravity, a spin, and a wrap to the top edge.
function updateParticles(res: ParticleLayout, ctx: ComputeInvocation): void {
  const index: u32 = ctx.globalId.x;
  const particle: Particle = res.particles[index];
  particle.velocity.y -= 0.00032;
  particle.position.x += particle.velocity.x;
  particle.position.y += particle.velocity.y;
  particle.angle += 0.025 + (index as f32) * 0.0003;
  if (particle.position.y < -1.12) {
    particle.position.y = 1.12;
    particle.position.x += 0.17;
  }
  if (particle.position.x > 1.12) {
    particle.position.x = -1.12;
  }
  // particle is a copy of the storage element, so this line stores the changes back.
  res.particles[index] = particle;
}

// TypeGPU rotates the card by a fixed per-particle angle and corrects for the canvas aspect ratio.
// This port animates the angle in compute and applies no aspect correction.
function confettiVertex(
  value: Vertex,
  particle: Particle,
  ctx: VertexInvocation,
): Varyings {
  // One vector carries the angle twice, so `cos()` and `sin()` reach the WGSL builtins.
  // A scalar `Math.cos` needs an `f64` cast, and K12 rejects that cast inside a kernel.
  const angles = new Vec2f(particle.angle, particle.angle);
  const cosine: f32 = angles.cos().x;
  const sine: f32 = angles.sin().x;
  const rotated = new Vec2f(
    value.position.x * cosine - value.position.y * sine,
    value.position.x * sine + value.position.y * cosine,
  );
  return new Varyings(
    new Vec4f(
      particle.position.x + rotated.x,
      particle.position.y + rotated.y,
      0.0,
      1.0,
    ),
    particle.color,
  );
}

// Every corner of one card carries the same color, so the interpolated value stays flat.
function confettiFragment(input: Varyings, ctx: FragmentInvocation): Vec4f {
  return input.color;
}

// TypeGPU uses a guarded pipeline. This port keeps the guard and dispatches the exact
// particle thread count through `dispatchThreads`. The generator wraps the body in a
// bounds test against a hidden uniform, which `dispatchThreads` writes before the pass.
export const confettiUpdate: ComputePipelineSpec = computePipeline<ParticleLayout>(
  updateParticles,
  { name: "confettiUpdate", workgroupSize: [64, 1, 1], guarded: true },
);

// The instanced declaration takes the vertex schema first and the instance schema second.
// Slot 0 steps per vertex and slot 1 steps per instance. The strip draws one card in four
// vertices, so the draw below needs no index buffer.
export const confettiRender: RenderPipelineSpec = renderPipelineInstanced<
  Vertex,
  Particle,
  Varyings
>(confettiVertex, confettiFragment, {
  format: "bgra8unorm",
  topology: "triangle-strip",
});

// The window host calls init, frame, and shutdown as separate entry points, so the
// handles live in module state. The script owns each one and frees it in shutdown.
let activeDevice: GPUHostOwnedDevice | null = null;
let activeCompute: ComputePipeline | null = null;
let activeRender: RenderPipeline | null = null;
let activeGroup: GPUBindGroup | null = null;
let activeVertices: GPUBuffer | null = null;
let activeParticles: GPUBuffer | null = null;

// The host calls init once, before the first frame, with the device it owns.
export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  // The host picks the surface format. The generator baked one format into the pipeline,
  // so a mismatch stops the example here instead of at pipeline creation.
  if (format !== confettiRender_TARGET_FORMAT) {
    print(`FAIL format expected=${confettiRender_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The host owns the device and the queue. This wrapper borrows both and never destroys them.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // Four strip vertices form one card, and each Particle supplies one instance record.
  // This port retains a typed vertex buffer for the upstream strip corners.
  const vertices: FixedArray<Vertex, 4> = [
    new Vertex(new Vec2f(-0.012, -0.022)),
    new Vertex(new Vec2f(0.012, -0.022)),
    new Vertex(new Vec2f(-0.012, 0.022)),
    new Vertex(new Vec2f(0.012, 0.022)),
  ];
  const vertexBuffer = hostDevice.createBuffer({
    label: "confetti-vertices",
    size: (Vertex_STRIDE * 4) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // One buffer carries both usages: STORAGE for the update kernel and VERTEX for the draw.
  // Particle_STRIDE is the generated element size, so the two views agree on the layout.
  const particleBuffer = hostDevice.createBuffer({
    label: "confetti-particles",
    size: (Particle_STRIDE * PARTICLE_COUNT) as u64,
    usage: GPUBufferUsage.STORAGE + GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // A host-owned device returns a new queue wrapper from each call, so this block frees it.
  using queue = hostDevice.queue();
  queue.writeBuffer(vertexBuffer, 0, Context.bytesOf<FixedArray<Vertex, 4>>(vertices));
  // The particles start on an 8 by 8 grid with a color that follows the cell.
  // A fixed start replaces the random start of the upstream example, so every run looks alike.
  for (let index: u32 = 0; index < PARTICLE_COUNT; index += 1) {
    const column: f32 = (index % 8) as f32;
    const row: f32 = (index / 8) as f32;
    const color = new Vec4f(
      0.25 + column * 0.085,
      0.85 - row * 0.075,
      0.35 + (((index * 3) % 7) as f32) * 0.08,
      1.0,
    );
    const particle = new Particle(
      new Vec2f(-0.92 + column * 0.26, -0.82 + row * 0.25),
      new Vec2f(((index % 5) as f32 - 2.0) * 0.0007, 0.010 + row * 0.0004),
      color,
      index as f32 * 0.37,
    );
    queue.writeBuffer(
      particleBuffer,
      (index as u64) * (Particle_STRIDE as u64),
      Context.bytesOf<Particle>(particle),
    );
  }

  // Pipeline creation raises no exception. One error scope covers both pipelines and turns
  // a bad shader or a bad layout into a value the code below tests.
  hostDevice.pushErrorScope("validation");
  const computePipeline = createComputePipelineHost(
    hostDevice,
    confettiUpdate_WGSL,
    confettiUpdate_ENTRY,
    [confettiUpdate_LAYOUT0],
    [64, 1, 1],
  );
  // The render pipeline declares no bind group layout. Every value it draws arrives through
  // the two vertex buffer slots.
  const renderPipeline = createRenderPipelineHost(
    hostDevice,
    confettiRender_WGSL,
    confettiRender_VERTEX_ENTRY,
    confettiRender_FRAGMENT_ENTRY,
    [],
    [confettiRender_VERTEX_LAYOUT0, confettiRender_VERTEX_LAYOUT1],
    confettiRender,
  );
  // A null check replaces the exception a browser port throws. The failure path frees the
  // four handles this function already created.
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    renderPipeline.dispose();
    computePipeline.dispose();
    particleBuffer.dispose();
    vertexBuffer.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  // The guard buffer belongs to the pipeline, and the helper appends it after the author's
  // resources. The bind group entry order stays the field order of ParticleLayout.
  using computeBindLayout = computePipeline.bindGroupLayout(0);
  const group = createBindGroupHost(
    hostDevice,
    computeBindLayout,
    confettiUpdate_LAYOUT0,
    [bufferResource(particleBuffer)],
    computePipeline.guardBuffer(0),
  );
  // The handles reach module state only after every step succeeds, so frame never sees a
  // half-built set.
  activeDevice = hostDevice;
  activeCompute = computePipeline;
  activeRender = renderPipeline;
  activeGroup = group;
  activeVertices = vertexBuffer;
  activeParticles = particleBuffer;
}

// The host calls frame once per presented image. The motion follows the particle state
// alone, so this example needs no clock and no input.
export function frame(
  view: SubscriptTypegpuTextureView,
  width: u32,
  height: u32,
  key: u32,
  pointerX: f32,
  pointerY: f32,
  buttons: u32,
): void {
  // init can fail, so frame proves that every handle exists before it records commands.
  // These null checks replace the exception a browser port throws.
  const device = activeDevice;
  const computePipeline = activeCompute;
  const renderPipeline = activeRender;
  const group = activeGroup;
  const vertices = activeVertices;
  const particles = activeParticles;
  if (device === null) return;
  if (computePipeline === null) return;
  if (renderPipeline === null) return;
  if (group === null) return;
  if (vertices === null) return;
  if (particles === null) return;
  // One encoder records the update and the draw. The compute pass comes first, so the draw
  // reads the positions this frame produced.
  using encoder = device.createCommandEncoderDefault();
  computePipeline.dispatchThreads(encoder, [group], PARTICLE_COUNT, 1, 1);
  // The host passes the current surface view as a raw handle. The wrapper does not own it,
  // and the host presents the surface after frame returns.
  const target = new GPUTextureView(view);
  // The attachment clears to a near-black blue. The cards cover a small part of the surface,
  // so loadOp clear removes the previous frame.
  using renderPass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.015, g: 0.02, b: 0.04, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  // The window resizes between frames. The viewport and the scissor follow the size the host
  // reports. The cards keep their normalized size, so a wide window stretches them.
  renderPass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  renderPass.setScissorRect(0, 0, width, height);
  // No bind group, and two vertex slots: the card corners first, the particles second.
  // The draw takes four strip vertices and one instance per particle.
  renderPipeline.bind(renderPass, [], [vertices, particles]);
  renderPass.draw(4, PARTICLE_COUNT);
  renderPass.end();
  // The pass must end before the encoder finishes. submit hands the command buffer to the
  // queue, which runs it before the host presents the surface.
  using command = encoder.finishDefault();
  using queue = device.queue();
  queue.submit([command]);
}

// The host calls shutdown once. The script frees every GPU handle by hand, because this
// library keeps no finalizer and no reference count for scripts.
export function shutdown(): void {
  if (activeGroup !== null) activeGroup.dispose();
  if (activeParticles !== null) activeParticles.dispose();
  if (activeVertices !== null) activeVertices.dispose();
  if (activeRender !== null) activeRender.dispose();
  if (activeCompute !== null) activeCompute.dispose();
  activeGroup = null;
  activeParticles = null;
  activeVertices = null;
  activeRender = null;
  activeCompute = null;
  activeDevice = null;
}
