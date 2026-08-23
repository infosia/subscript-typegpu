// program: b12-readback
// purpose: round-trip typed values through a buffer-owned staging allocation
// exercises: BF9-BF11, PI14, SC1
// questions: none

import { Buffer, createBuffer } from "./typegpu";
import { Vec3f } from "./typegpu-types";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice } from "./webgpu";
import { Particle_STRIDE } from "./b12-readback.typegpu";

@CStruct
class Particle {
  mass: f32;
  pos: Vec3f;

  constructor(mass: f32, pos: Vec3f) {
    this.mass = mass;
    this.pos = pos;
  }
}

function equalParticle(left: Particle, right: Particle): boolean {
  return left.pos.x === right.pos.x
    && left.pos.y === right.pos.y
    && left.pos.z === right.pos.z
    && left.mass === right.mass;
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
    device.pushErrorScope("validation");
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print("pipeline:invalid");
      print("FAIL");
      return;
    }
    const particles: FixedArray<Particle, 4> = [
      new Particle(0.5, new Vec3f(1.0, 2.0, 3.0)),
      new Particle(1.5, new Vec3f(4.0, 5.0, 6.0)),
      new Particle(2.5, new Vec3f(-2.0, 8.0, 0.25)),
      new Particle(3.5, new Vec3f(9.0, -3.0, 7.0)),
    ];
    using buffer: Buffer<Particle> = createBuffer<Particle>(
      device,
      Particle_STRIDE,
      4,
      GPUBufferUsage.COPY_SRC + GPUBufferUsage.COPY_DST,
      "b12-readback",
    );
    buffer.write(device.queue(), 0, Context.bytesOf<FixedArray<Particle, 4>>(particles));
    const bytes: u8[] = await buffer.read(device, 0, 4);
    const decoded: FixedArray<Particle, 4> = Context.fromBytes<FixedArray<Particle, 4>>(bytes, 0);
    let index: i32 = 0;
    while (index < 4) {
      if (!equalParticle(decoded[index], particles[index])) {
        print(`FAIL particle ${index}`);
        return;
      }
      index = index + 1;
    }
    print("roundtrip:match");
    const oneBytes: u8[] = await buffer.readOne(device, 2);
    const particle2: Particle = Context.fromBytes<Particle>(oneBytes, 0);
    print(`particle2=${particle2.pos.x},${particle2.pos.y},${particle2.pos.z},${particle2.mass}`);
  }
  gpu.dispose();
  print("PASS");
}
