// program: b12-readback
// purpose: decode typed values from a staging buffer and a mapped element range
// exercises: BF3, BF9-BF11, SC1
// questions: none

import { Buffer, createBuffer, readBuffer, readOne } from "./typegpu";
import { Vec3f } from "./typegpu-types";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice, GPUMapMode } from "./webgpu";
import { Particle_STRIDE } from "./b12-readback.typegpu";

@ValueType
class Particle {
  constructor(public mass: f32, public pos: Vec3f) {}
}

function equalParticle(left: Particle, right: Particle): boolean {
  return left.pos.x == right.pos.x
    && left.pos.y == right.pos.y
    && left.pos.z == right.pos.z
    && left.mass == right.mass;
}

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
    const particles: FixedArray<Particle, 4> = [
      new Particle(0.5, new Vec3f(1.0, 2.0, 3.0)),
      new Particle(1.5, new Vec3f(4.0, 5.0, 6.0)),
      new Particle(2.5, new Vec3f(-2.0, 8.0, 0.25)),
      new Particle(3.5, new Vec3f(9.0, -3.0, 7.0)),
    ];
    using buffer: Buffer<Particle> = createBuffer(
      device,
      Particle_STRIDE,
      4,
      GPUBufferUsage.COPY_SRC + GPUBufferUsage.COPY_DST,
      "b12-readback",
    );
    buffer.write(device.queue, 0, Context.bytesOf<FixedArray<Particle, 4>>(particles));
    const bytes: u8[] = await buffer.read(device, 0, 4);
    const decoded: FixedArray<Particle, 4> = Context.fromBytes<FixedArray<Particle, 4>>(bytes, 0);
    let index: i32 = 0;
    while (index < 4) {
      if (!equalParticle(decoded[index], particles[index])) {
        print(`FAIL particle ${index}`);
        return;
      }
      index += 1;
    }
    print("roundtrip:match");
    const oneBytes: u8[] = await buffer.readOne(device, 2);
    const particle2: Particle = Context.fromBytes<Particle>(oneBytes, 0);
    print(`particle2=${particle2.pos.x},${particle2.pos.y},${particle2.pos.z},${particle2.mass}`);
    const words: FixedArray<u32, 4> = [11, 22, 33, 44];
    using mapped: Buffer<u32> = createBuffer(device, 4, 4,
      GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST, "b12-words");
    mapped.write(device.queue, 0, Context.bytesOf<FixedArray<u32, 4>>(words));
    if (!await mapped.handle().mapAsync(GPUMapMode.READ, 0, 16)) {
      print("FAIL map");
      return;
    }
    let complete = true;
    for (let i: u32 = 0; i < 4; i++) {
      const word = readOne(mapped, i);
      if (word.length == 4) {
        const decodedWord = Context.fromBytes<u32>(word, 0);
        print(`word${i}=${decodedWord}`);
      } else {
        complete = false;
        print(`FAIL word${i} bytes=${word.length}`);
      }
    }
    const middle = readBuffer(mapped, 1, 2);
    if (middle.length == 8) {
      const pair = Context.fromBytes<FixedArray<u32, 2>>(middle, 0);
      print(`middle=${pair[0]},${pair[1]}`);
    } else {
      complete = false;
      print(`FAIL middle bytes=${middle.length}`);
    }
    mapped.handle().unmap();
    if (!complete) return;
    if (!await mapped.handle().mapAsync(GPUMapMode.READ, 8, 8)) {
      print("FAIL subrange map");
      return;
    }
    const last = readOne(mapped, 3);
    const empty = readBuffer(mapped, 4, 0);
    mapped.handle().unmap();
    if (last.length != 4 || empty.length != 0) {
      print(`FAIL subrange bytes=${last.length} empty bytes=${empty.length}`);
      return;
    }
    const lastWord = Context.fromBytes<u32>(last, 0);
    if (lastWord != words[3]) {
      print("FAIL subrange word");
      return;
    }
    print(`subrange word3=${words[3]}`);
    print(`empty bytes=${empty.length}`);
  }
  gpu.dispose();
  print("PASS");
}
