// program: b04-particles
// purpose: prove a schema with vectors and a kernel helper function
// exercises: CL1, CL3, CL4, K1-K16, PI1-PI11, LY3, LY11
// questions: none

import {
  createComputePipeline,
  ComputeInvocation,
  computePipeline,
  ComputePipelineSpec,
  MutStorage,
  simulateCompute,
  Uniform,
} from "./typegpu";
import { Vec3f } from "./typegpu-types";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice } from "./webgpu";
import {
  Particle_SIZE,
  ParticleLayoutResources,
  SimParams_SIZE,
  createParticleLayoutResources,
  createParticlesBindGroup0,
  particles_ENTRY,
  particles_HOST_RUNNABLE,
  particles_LAYOUT0,
  particles_WGSL,
  particles_WORKGROUP_X,
  particles_WORKGROUP_Y,
  particles_WORKGROUP_Z,
} from "./b04-particles.typegpu";

@ValueType
class Particle {
  constructor(public pos: Vec3f, public vel: Vec3f) {}
}

@ValueType
class SimParams {
  constructor(public dt: f32, public count: u32) {}
}

class ParticleLayout {
  constructor(public params: Uniform<SimParams>, public particles: MutStorage<Particle>) {}
}

function integrate(particle: Particle, dt: f32): Particle {
  const speed: f32 = particle.vel.length();
  if (speed > 0.0) {
    const pos: Vec3f = particle.pos.add(particle.vel.scale(dt));
    return new Particle(pos, particle.vel);
  }
  return particle;
}

function particleKernel(res: ParticleLayout, ctx: ComputeInvocation): void {
  const settings: SimParams = res.params.$;
  const i: u32 = ctx.globalId.x;
  if (i < settings.count) {
    res.particles[i] = integrate(res.particles[i], settings.dt);
  }
}

const particles: ComputePipelineSpec = computePipeline(particleKernel, {
  name: "particles",
  workgroupSize: [64, 1, 1],
});

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult == null) {
    print("FAIL adapter");
    gpu.dispose();
    return;
  }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult == null) {
    print("FAIL device");
    adapterResult.dispose();
    gpu.dispose();
    return;
  }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const count: u32 = 128;
    using params = device.createBuffer({
      label: "b04-params",
      size: SimParams_SIZE as u64,
      usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
    });
    using particlesBuffer = device.createBuffer({
      label: "b04-particles",
      size: (Particle_SIZE * count) as u64,
      usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC,
    });
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      particles_WGSL,
      particles_ENTRY,
      [particles_LAYOUT0],
      [particles_WORKGROUP_X, particles_WORKGROUP_Y, particles_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError != null) {
      print("pipeline:invalid");
      print("FAIL");
      return;
    }
    const resources: ParticleLayoutResources = createParticleLayoutResources(
      params,
      particlesBuffer,
    );
    using bindGroup = createParticlesBindGroup0(device, pipeline, resources);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], count, 1, 1);
    using command = encoder.finishDefault();
    device.queue.submit([command]);
    const hostLayout = new ParticleLayout(
      new Uniform(new SimParams(2.0, 1)),
      new MutStorage([
        new Particle(new Vec3f(1.0, 2.0, 3.0), new Vec3f(0.5, 0.0, 0.0)),
      ]),
    );
    simulateCompute(
      particleKernel,
      hostLayout,
      particles,
      [1, 1, 1],
      particles_HOST_RUNNABLE,
    );
    print("pipeline:created");
    print(`Particle_SIZE=${Particle_SIZE}`);
    print(`SimParams_SIZE=${SimParams_SIZE}`);
    print(`particles_WORKGROUP_X=${particles_WORKGROUP_X}`);
    print(`particles_WORKGROUP_Y=${particles_WORKGROUP_Y}`);
    print(`particles_WORKGROUP_Z=${particles_WORKGROUP_Z}`);
    print(`particles_WGSL_LINES=${particles_WGSL.split("\n").length}`);
    print("dispatch:submitted");
    print(`host:out=${hostLayout.particles[0].pos.x}`);
  }
  gpu.dispose();
  print("PASS");
}
