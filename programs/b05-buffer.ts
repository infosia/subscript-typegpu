// program: b05-buffer
// purpose: round-trip a fixed array through typed GPU buffers and mapped bytes
// exercises: BF1-BF6, SC1, SC11, LY16, R34
// questions: none

import { Buffer, createBuffer, readOne } from "./typegpu";
import { Vec3f } from "./typegpu-types";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  GPUMapMode,
} from "./webgpu";
import { Particle_OFFSET_vel, Particle_STRIDE } from "./b05-buffer.typegpu";

@CStruct
class Particle {
  pos: Vec3f;
  vel: Vec3f;

  constructor(pos: Vec3f, vel: Vec3f) {
    this.pos = pos;
    this.vel = vel;
  }
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
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) {
    print("FAIL device");
    adapterResult.dispose();
    gpu.dispose();
    return;
  }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const particles: FixedArray<Particle, 4> = [
      new Particle(new Vec3f(1.0, 2.0, 3.0), new Vec3f(0.5, -0.25, 0.125)),
      new Particle(new Vec3f(4.0, 5.0, 6.0), new Vec3f(-1.0, 0.75, 0.25)),
      new Particle(new Vec3f(-2.0, 8.0, 0.5), new Vec3f(2.0, 1.0, -0.5)),
      new Particle(new Vec3f(9.0, -3.0, 7.0), new Vec3f(0.0, 0.25, 1.5)),
    ];
    const bytes: u8[] = Context.bytesOf<FixedArray<Particle, 4>>(particles);
    using source: Buffer<Particle> = createBuffer<Particle>(
      device,
      Particle_STRIDE,
      4,
      GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC,
      "b05-source",
    );
    using readback: Buffer<Particle> = createBuffer<Particle>(
      device,
      Particle_STRIDE,
      4,
      GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
      "b05-readback",
    );
    const queue = device.queue();
    source.write(queue, 0, bytes);
    const patchedVelocity = new Vec3f(3.0, -2.0, 1.0);
    source.patch(
      queue,
      1,
      Particle_OFFSET_vel,
      Context.bytesOf<Vec3f>(patchedVelocity),
    );
    particles[1].vel = patchedVelocity;
    using encoder = device.createCommandEncoderDefault();
    source.copyTo(encoder, readback, 0, 4);
    using command = encoder.finishDefault();
    queue.submit([command]);
    if (!await queue.onSubmittedWorkDone()) {
      print("FAIL submit");
      return;
    }
    const byteLength: u64 = bytes.length as u64;
    if (!await readback.handle().mapAsync(GPUMapMode.READ, 0, byteLength)) {
      print("FAIL map");
      return;
    }
    print(`bytes=${bytes.length}`);
    let index: i32 = 0;
    while (index < 4) {
      const value: Particle = Context.fromBytes<Particle>(
        readOne<Particle>(readback, index as u32),
        0,
      );
      if (!equalParticle(value, particles[index])) {
        print(`FAIL particle ${index}`);
        return;
      }
      print(
        `particle${index}=${value.pos.x},${value.pos.y},${value.pos.z},${value.vel.x},${value.vel.y},${value.vel.z}`,
      );
      index = index + 1;
    }
    print("roundtrip:match");
    readback.handle().unmap();
  }
  gpu.dispose();
  print("PASS");
}
