// program: x03-live-particles
// purpose: integrate particles for four GPU steps and compare all components
// exercises: BF7, CL1, CL3, CL4, PI12, T4, T15, vector schema, helper function, repeated dispatch
// questions: none

import {
  Buffer,
  createBuffer,
  readBuffer,
  createComputePipeline,
  createBindGroup,
  ComputeInvocation,
  computePipeline,
  ComputePipelineSpec,
  MutStorage,
  simulateComputeThreads,
  Uniform,
  bufferResource,
} from "./typegpu";
import { Vec3f } from "./typegpu-types";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  GPUMapMode,
} from "./webgpu";
import {
  Particle_STRIDE,
  SimParams_STRIDE,
  particles_ENTRY,
  particles_HOST_RUNNABLE,
  particles_LAYOUT0,
  particles_WGSL,
  particles_WORKGROUP_X,
  particles_WORKGROUP_Y,
  particles_WORKGROUP_Z,
} from "./x03-live-particles.typegpu";

@CStruct
class Particle {
  pos: Vec3f;
  vel: Vec3f;

  constructor(pos: Vec3f, vel: Vec3f) {
    this.pos = pos;
    this.vel = vel;
  }
}

@CStruct
class SimParams {
  dt: f32;
  count: u32;

  constructor(dt: f32, count: u32) {
    this.dt = dt;
    this.count = count;
  }
}

class ParticleLayout {
  params!: Uniform<SimParams>;
  particles!: MutStorage<Particle>;
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

export const particles: ComputePipelineSpec = computePipeline<ParticleLayout>(particleKernel, {
  name: "particles",
  workgroupSize: [64, 1, 1],
});

function particleArray(): FixedArray<Particle, 64> {
  const zero = new Particle(new Vec3f(0.0, 0.0, 0.0), new Vec3f(0.0, 0.0, 0.0));
  return [
    zero, zero, zero, zero, zero, zero, zero, zero,
    zero, zero, zero, zero, zero, zero, zero, zero,
    zero, zero, zero, zero, zero, zero, zero, zero,
    zero, zero, zero, zero, zero, zero, zero, zero,
    zero, zero, zero, zero, zero, zero, zero, zero,
    zero, zero, zero, zero, zero, zero, zero, zero,
    zero, zero, zero, zero, zero, zero, zero, zero,
    zero, zero, zero, zero, zero, zero, zero, zero,
  ];
}

function equalParticle(left: Particle, right: Particle): boolean {
  return left.pos.x === right.pos.x
    && left.pos.y === right.pos.y
    && left.pos.z === right.pos.z
    && left.vel.x === right.vel.x
    && left.vel.y === right.vel.y
    && left.vel.z === right.vel.z;
}

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) {
    print("FAIL adapter");
    gpu.dispose();
    return;
  }
  print("adapter:ready");
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) {
    print("FAIL device");
    adapterResult.dispose();
    gpu.dispose();
    return;
  }
  print("device:ready");
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const count: u32 = 64;
    const dt: f32 = 0.25;
    const steps: u32 = 4;
    const size: u64 = (Particle_STRIDE as u64) * (count as u64);
    const particleValues: FixedArray<Particle, 64> = particleArray();
    const hostParticles: Particle[] = [];
    let index: i32 = 0;
    while (index < 64) {
      const particle = new Particle(
        new Vec3f(index as f32, (index as f32) * 0.5, (index as f32) * -0.25),
        new Vec3f(0.5, -0.25, 0.125),
      );
      particleValues[index] = particle;
      hostParticles.push(particle);
      index = index + 1;
    }
    const hostLayout = new ParticleLayout();
    hostLayout.params = new Uniform<SimParams>(new SimParams(dt, count));
    hostLayout.particles = new MutStorage<Particle>(hostParticles);
    let step: u32 = 0;
    while (step < steps) {
      simulateComputeThreads<ParticleLayout>(
        particleKernel,
        hostLayout,
        particles,
        count,
        1,
        1,
        particles_HOST_RUNNABLE,
      );
      step = step + 1;
    }
    using params: Buffer<SimParams> = createBuffer<SimParams>(
      device,
      SimParams_STRIDE,
      1,
      GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
      "x03-params",
    );
    using particlesBuffer: Buffer<Particle> = createBuffer<Particle>(
      device,
      Particle_STRIDE,
      count,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC,
      "x03-particles",
    );
    using readback: Buffer<Particle> = createBuffer<Particle>(
      device,
      Particle_STRIDE,
      count,
      GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
      "x03-readback",
    );
    const queue = device.queue();
    params.writeOne(queue, 0, Context.bytesOf<SimParams>(new SimParams(dt, count)));
    particlesBuffer.write(
      queue,
      0,
      Context.bytesOf<FixedArray<Particle, 64>>(particleValues),
    );
    print("inputs:written");
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      particles_WGSL,
      particles_ENTRY,
      [particles_LAYOUT0],
      [particles_WORKGROUP_X, particles_WORKGROUP_Y, particles_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(
      device,
      nativeLayout,
      particles_LAYOUT0,
      [bufferResource(params.handle()), bufferResource(particlesBuffer.handle())],
    );
    using encoder = device.createCommandEncoderDefault();
    step = 0;
    while (step < steps) {
      pipeline.dispatchThreads(encoder, [bindGroup], count, 1, 1);
      step = step + 1;
    }
    particlesBuffer.copyTo(encoder, readback, 0, count);
    using command = encoder.finishDefault();
    queue.submit([command]);
    print("dispatch:steps=4");
    const mapped: boolean = await readback.handle().mapAsync(GPUMapMode.READ, 0, size);
    if (!mapped) {
      print("FAIL map");
      return;
    }
    const result: FixedArray<Particle, 64> = Context.fromBytes<FixedArray<Particle, 64>>(
      readBuffer<Particle>(readback, 0, count),
      0,
    );
    print("readback:mapped");
    index = 0;
    while (index < 64) {
      if (!equalParticle(result[index], hostLayout.particles[index as u32])) {
        print(`FAIL ${index}`);
        return;
      }
      index = index + 1;
    }
    readback.handle().unmap();
  }
  gpu.dispose();
  print("PASS");
}
