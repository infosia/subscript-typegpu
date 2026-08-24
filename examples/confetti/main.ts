// example: confetti
// Advances colored particles in compute and draws them as instanced cards.
// This port starts from a fixed layout of 64 particles and drops the upstream randomize button.
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
  computePipeline,
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

const PARTICLE_COUNT: u32 = 64;

@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

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

@CStruct
class Varyings {
  position: Vec4f;
  color: Vec4f;

  constructor(position: Vec4f, color: Vec4f) {
    this.position = position;
    this.color = color;
  }
}

class ParticleLayout {
  particles!: MutStorage<Particle>;
}

// One storage buffer becomes the instance stream after this pass completes.
// Upstream adds a seeded sine offset to the position and never wraps.
// This port applies constant gravity, a spin, and a wrap to the top edge.
function updateParticles(res: ParticleLayout, ctx: ComputeInvocation): void {
  const index: u32 = ctx.globalId.x;
  const particle: Particle = res.particles.get(index);
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
  res.particles.set(index, particle);
}

// Upstream rotates the card by a fixed per-particle angle and corrects for the canvas aspect ratio.
// This port animates the angle in compute and applies no aspect correction.
function confettiVertex(
  value: Vertex,
  particle: Particle,
  ctx: VertexInvocation,
): Varyings {
  const cosine: f32 = new Vec2f(particle.angle, particle.angle).cos().x;
  const sine: f32 = new Vec2f(particle.angle, particle.angle).sin().x;
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

function confettiFragment(input: Varyings, ctx: FragmentInvocation): Vec4f {
  return input.color;
}

// Upstream uses a guarded pipeline. The particle count here is a whole number of workgroups,
// so this pipeline dispatches workgroups directly and needs no guard.
export const confettiUpdate: ComputePipelineSpec = computePipeline<ParticleLayout>(
  updateParticles,
  { name: "confettiUpdate", workgroupSize: [64, 1, 1] },
);

export const confettiRender: RenderPipelineSpec = renderPipelineInstanced<
  Vertex,
  Particle,
  Varyings
>(confettiVertex, confettiFragment, { format: "bgra8unorm" });

let activeDevice: GPUHostOwnedDevice | null = null;
let activeCompute: ComputePipeline | null = null;
let activeRender: RenderPipeline | null = null;
let activeGroup: GPUBindGroup | null = null;
let activeVertices: GPUBuffer | null = null;
let activeParticles: GPUBuffer | null = null;

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  if (format !== confettiRender_TARGET_FORMAT) {
    print(`FAIL format expected=${confettiRender_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // Two triangles form one card, and each Particle supplies one instance record.
  // Upstream draws a four-vertex triangle strip with no vertex buffer.
  // This port draws six triangle-list vertices from one vertex buffer.
  const vertices: FixedArray<Vertex, 6> = [
    new Vertex(new Vec2f(-0.012, -0.022)),
    new Vertex(new Vec2f(0.012, -0.022)),
    new Vertex(new Vec2f(-0.012, 0.022)),
    new Vertex(new Vec2f(-0.012, 0.022)),
    new Vertex(new Vec2f(0.012, -0.022)),
    new Vertex(new Vec2f(0.012, 0.022)),
  ];
  const vertexBuffer = hostDevice.createBuffer({
    label: "confetti-vertices",
    size: (Vertex_STRIDE * 6) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  const particleBuffer = hostDevice.createBuffer({
    label: "confetti-particles",
    size: (Particle_STRIDE * PARTICLE_COUNT) as u64,
    usage: GPUBufferUsage.STORAGE + GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  using queue = hostDevice.queue();
  queue.writeBuffer(vertexBuffer, 0, Context.bytesOf<FixedArray<Vertex, 6>>(vertices));
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

  hostDevice.pushErrorScope("validation");
  const computePipeline = createComputePipelineHost(
    hostDevice,
    confettiUpdate_WGSL,
    confettiUpdate_ENTRY,
    [confettiUpdate_LAYOUT0],
    [64, 1, 1],
  );
  const renderPipeline = createRenderPipelineHost(
    hostDevice,
    confettiRender_WGSL,
    confettiRender_VERTEX_ENTRY,
    confettiRender_FRAGMENT_ENTRY,
    [],
    [confettiRender_VERTEX_LAYOUT0, confettiRender_VERTEX_LAYOUT1],
    confettiRender,
  );
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    renderPipeline.dispose();
    computePipeline.dispose();
    particleBuffer.dispose();
    vertexBuffer.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  using computeBindLayout = computePipeline.bindGroupLayout(0);
  const group = hostDevice.createBindGroup({
    layout: computeBindLayout,
    entries: [{
      binding: confettiUpdate_LAYOUT0.entries[0].binding,
      buffer: particleBuffer,
      size: (Particle_STRIDE * PARTICLE_COUNT) as u64,
    }],
  });
  activeDevice = hostDevice;
  activeCompute = computePipeline;
  activeRender = renderPipeline;
  activeGroup = group;
  activeVertices = vertexBuffer;
  activeParticles = particleBuffer;
}

export function frame(
  view: SubscriptTypegpuTextureView,
  width: u32,
  height: u32,
  key: u32,
): void {
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
  using encoder = device.createCommandEncoderDefault();
  computePipeline.dispatch(encoder, [group], PARTICLE_COUNT / 64, 1, 1);
  const target = new GPUTextureView(view);
  using renderPass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.015, g: 0.02, b: 0.04, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  renderPass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  renderPass.setScissorRect(0, 0, width, height);
  renderPipeline.bind(renderPass, [], [vertices, particles]);
  renderPass.draw(6, PARTICLE_COUNT);
  renderPass.end();
  using command = encoder.finishDefault();
  using queue = device.queue();
  queue.submit([command]);
}

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
