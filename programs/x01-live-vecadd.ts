// program: x01-live-vecadd
// purpose: compute vector addition on a real adapter and compare every result
// exercises: PI12, T4, T15, storage bindings, readback
// questions: none

import {
  createComputePipeline,
  createBindGroup,
  ComputeInvocation,
  computePipeline,
  ComputePipelineSpec,
  MutStorage,
  Storage,
} from "./typegpu";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  GPUMapMode,
} from "./webgpu";
import {
  Item_SIZE,
  vecAdd_ENTRY,
  vecAdd_LAYOUT0,
  vecAdd_WGSL,
  vecAdd_WORKGROUP_X,
  vecAdd_WORKGROUP_Y,
  vecAdd_WORKGROUP_Z,
} from "./x01-live-vecadd.typegpu";

@CStruct
class Item {
  value: f32;

  constructor(value: f32) {
    this.value = value;
  }
}

class VecAddLayout {
  a!: Storage<Item>;
  b!: Storage<Item>;
  out!: MutStorage<Item>;
}

function vecAddKernel(res: VecAddLayout, ctx: ComputeInvocation): void {
  const i: u32 = ctx.globalId.x;
  if (i < res.out.length()) {
    const left: Item = res.a[i];
    const right: Item = res.b[i];
    res.out[i] = new Item(left.value + right.value);
  }
}

export const vecAdd: ComputePipelineSpec = computePipeline<VecAddLayout>(vecAddKernel, {
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

function readF32(bytes: u8[], offset: u32): f32 {
  const index: i32 = offset as i32;
  const bits: u32 =
    (bytes[index] as u32) |
    ((bytes[index + 1] as u32) << 8) |
    ((bytes[index + 2] as u32) << 16) |
    ((bytes[index + 3] as u32) << 24);
  return Math.f32FromBits(bits) as f32;
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
    const size: u64 = (Item_SIZE * count) as u64;
    const aBytes: u8[] = [];
    const bBytes: u8[] = [];
    const expected: f32[] = [];
    let i: u32 = 0;
    while (i < count) {
      const a: f32 = (i as f32) * 0.5;
      const b: f32 = ((count - i) as f32) * 0.25;
      appendF32(aBytes, a);
      appendF32(bBytes, b);
      expected.push(a + b);
      i = i + 1;
    }
    using a = device.createBuffer({
      label: "x01-a",
      size,
      usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
    });
    using b = device.createBuffer({
      label: "x01-b",
      size,
      usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
    });
    using out = device.createBuffer({
      label: "x01-out",
      size,
      usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC,
    });
    using readback = device.createBuffer({
      label: "x01-readback",
      size,
      usage: GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
    });
    const queue = device.queue();
    queue.writeBuffer(a, 0, aBytes);
    queue.writeBuffer(b, 0, bBytes);
    print("inputs:written");
    using pipeline = createComputePipeline(
      device,
      vecAdd_WGSL,
      vecAdd_ENTRY,
      [vecAdd_LAYOUT0],
      [vecAdd_WORKGROUP_X, vecAdd_WORKGROUP_Y, vecAdd_WORKGROUP_Z],
    );
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(device, nativeLayout, vecAdd_LAYOUT0, [a, b, out]);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], count, 1, 1);
    encoder.copyBufferToBuffer(out, 0, readback, 0, size);
    using command = encoder.finishDefault();
    queue.submit([command]);
    print("dispatch:submitted");
    const mapped: boolean = await readback.mapAsync(GPUMapMode.READ, 0, size);
    if (!mapped) {
      print("FAIL map");
      return;
    }
    const result: u8[] = readback.readMappedRange(0, size);
    print("readback:mapped");
    i = 0;
    while (i < count) {
      const got: f32 = readF32(result, i * Item_SIZE);
      if (Math.f32ToBits(got as f64) !== Math.f32ToBits(expected[i as i32] as f64)) {
        print(`FAIL ${i} expected=${expected[i as i32]} got=${got}`);
        return;
      }
      i = i + 1;
    }
    readback.unmap();
  }
  gpu.dispose();
  print("PASS");
}
