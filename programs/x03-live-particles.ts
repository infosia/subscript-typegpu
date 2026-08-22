// program: x03-live-particles
// purpose: integrate particles for four GPU steps and compare all components
// exercises: PI12, T4, T15, vector schema, helper function, repeated dispatch
// questions: none

import {
  createComputePipeline,
  createBindGroup,
  ComputeInvocation,
  computePipeline,
  ComputePipelineSpec,
  MutStorage,
  Uniform,
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
  Particle_SIZE,
  SimParams_SIZE,
  particles_ENTRY,
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
  const settings: SimParams = res.params.get();
  const i: u32 = ctx.globalId.x;
  if (i < settings.count) {
    res.particles[i] = integrate(res.particles[i], settings.dt);
  }
}

export const particles: ComputePipelineSpec = computePipeline<ParticleLayout>(particleKernel, {
  workgroupSize: [64, 1, 1],
});

function appendU32(bytes: u8[], value: u32): void {
  bytes.push((value & 255) as u8);
  bytes.push(((value >> 8) & 255) as u8);
  bytes.push(((value >> 16) & 255) as u8);
  bytes.push(((value >> 24) & 255) as u8);
}

function appendF32(bytes: u8[], value: f32): void {
  appendU32(bytes, Math.f32ToBits(value as f64));
}

function appendParticle(bytes: u8[], particle: Particle): void {
  appendF32(bytes, particle.pos.x);
  appendF32(bytes, particle.pos.y);
  appendF32(bytes, particle.pos.z);
  appendF32(bytes, 0.0);
  appendF32(bytes, particle.vel.x);
  appendF32(bytes, particle.vel.y);
  appendF32(bytes, particle.vel.z);
  appendF32(bytes, 0.0);
}

function readF32(bytes: u8[], offset: u32): f32 {
  const index: i32 = offset as i32;
  const bits: u32 =
    (bytes[index] as u32) |
    ((bytes[index + 1] as u32) << 8) |
    ((bytes[index + 2] as u32) << 16) |
    ((bytes[index + 3] as u32) << 24);
  return Math.f32FromBits(bits) as f32;
}

function equalF32(left: f32, right: f32): boolean {
  return Math.f32ToBits(left as f64) === Math.f32ToBits(right as f64);
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
    const paramsBytes: u8[] = [];
    appendF32(paramsBytes, dt);
    appendU32(paramsBytes, count);
    const particleBytes: u8[] = [];
    const expected: Particle[] = [];
    let i: u32 = 0;
    while (i < count) {
      const particle = new Particle(
        new Vec3f(i as f32, (i as f32) * 0.5, (i as f32) * -0.25),
        new Vec3f(0.5, -0.25, 0.125),
      );
      expected.push(particle);
      appendParticle(particleBytes, particle);
      i = i + 1;
    }
    let step: u32 = 0;
    while (step < steps) {
      i = 0;
      while (i < count) {
        expected[i as i32] = integrate(expected[i as i32], dt);
        i = i + 1;
      }
      step = step + 1;
    }
    const size: u64 = (Particle_SIZE * count) as u64;
    using params = device.createBuffer({
      label: "x03-params",
      size: SimParams_SIZE as u64,
      usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
    });
    using particlesBuffer = device.createBuffer({
      label: "x03-particles",
      size,
      usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC,
    });
    using readback = device.createBuffer({
      label: "x03-readback",
      size,
      usage: GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
    });
    const queue = device.queue();
    queue.writeBuffer(params, 0, paramsBytes);
    queue.writeBuffer(particlesBuffer, 0, particleBytes);
    print("inputs:written");
    using pipeline = createComputePipeline(
      device,
      particles_WGSL,
      particles_ENTRY,
      [particles_LAYOUT0],
      [particles_WORKGROUP_X, particles_WORKGROUP_Y, particles_WORKGROUP_Z],
    );
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(
      device,
      nativeLayout,
      particles_LAYOUT0,
      [params, particlesBuffer],
    );
    using encoder = device.createCommandEncoderDefault();
    step = 0;
    while (step < steps) {
      pipeline.dispatchThreads(encoder, [bindGroup], count, 1, 1);
      step = step + 1;
    }
    encoder.copyBufferToBuffer(particlesBuffer, 0, readback, 0, size);
    using command = encoder.finishDefault();
    queue.submit([command]);
    print("dispatch:steps=4");
    const mapped: boolean = await readback.mapAsync(GPUMapMode.READ, 0, size);
    if (!mapped) {
      print("FAIL map");
      return;
    }
    const result: u8[] = readback.readMappedRange(0, size);
    print("readback:mapped");
    i = 0;
    while (i < count) {
      const base: u32 = i * Particle_SIZE;
      const expectedParticle: Particle = expected[i as i32];
      const got0: f32 = readF32(result, base);
      const got1: f32 = readF32(result, base + 4);
      const got2: f32 = readF32(result, base + 8);
      const got3: f32 = readF32(result, base + 16);
      const got4: f32 = readF32(result, base + 20);
      const got5: f32 = readF32(result, base + 24);
      if (!equalF32(got0, expectedParticle.pos.x)) {
        print(`FAIL ${i * 6} expected=${expectedParticle.pos.x} got=${got0}`);
        return;
      }
      if (!equalF32(got1, expectedParticle.pos.y)) {
        print(`FAIL ${i * 6 + 1} expected=${expectedParticle.pos.y} got=${got1}`);
        return;
      }
      if (!equalF32(got2, expectedParticle.pos.z)) {
        print(`FAIL ${i * 6 + 2} expected=${expectedParticle.pos.z} got=${got2}`);
        return;
      }
      if (!equalF32(got3, expectedParticle.vel.x)) {
        print(`FAIL ${i * 6 + 3} expected=${expectedParticle.vel.x} got=${got3}`);
        return;
      }
      if (!equalF32(got4, expectedParticle.vel.y)) {
        print(`FAIL ${i * 6 + 4} expected=${expectedParticle.vel.y} got=${got4}`);
        return;
      }
      if (!equalF32(got5, expectedParticle.vel.z)) {
        print(`FAIL ${i * 6 + 5} expected=${expectedParticle.vel.z} got=${got5}`);
        return;
      }
      i = i + 1;
    }
    readback.unmap();
  }
  gpu.dispose();
  print("PASS");
}
