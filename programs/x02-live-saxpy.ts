// program: x02-live-saxpy
// purpose: compute SAXPY with a uniform block and compare every result
// exercises: PI12, T4, T15, uniform and storage bindings, readback
// questions: none

import {
  createComputePipeline,
  createBindGroup,
  ComputeInvocation,
  computePipeline,
  ComputePipelineSpec,
  MutStorage,
  Storage,
  Uniform,
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
  SaxpyParams_SIZE,
  saxpy_ENTRY,
  saxpy_LAYOUT0,
  saxpy_WGSL,
  saxpy_WORKGROUP_X,
  saxpy_WORKGROUP_Y,
  saxpy_WORKGROUP_Z,
} from "./x02-live-saxpy.typegpu";

@CStruct
class Item {
  value: f32;

  constructor(value: f32) {
    this.value = value;
  }
}

@CStruct
class SaxpyParams {
  a: f32;
  count: u32;

  constructor(a: f32, count: u32) {
    this.a = a;
    this.count = count;
  }
}

class SaxpyLayout {
  params!: Uniform<SaxpyParams>;
  x!: Storage<Item>;
  y!: MutStorage<Item>;
}

function saxpyKernel(res: SaxpyLayout, ctx: ComputeInvocation): void {
  const settings: SaxpyParams = res.params.get();
  const i: u32 = ctx.globalId.x;
  if (i < settings.count) {
    const xItem: Item = res.x[i];
    const yItem: Item = res.y[i];
    res.y[i] = new Item(settings.a * xItem.value + yItem.value);
  }
}

export const saxpy: ComputePipelineSpec = computePipeline<SaxpyLayout>(saxpyKernel, {
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
    const aValue: f32 = 1.5;
    const itemBytes: u64 = (Item_SIZE * count) as u64;
    const paramsBytes: u8[] = [];
    appendF32(paramsBytes, aValue);
    appendU32(paramsBytes, count);
    const xBytes: u8[] = [];
    const yBytes: u8[] = [];
    const expected: f32[] = [];
    let i: u32 = 0;
    while (i < count) {
      const xValue: f32 = (i as f32) * 0.125;
      const yValue: f32 = ((i % 7) as f32) * 0.25;
      appendF32(xBytes, xValue);
      appendF32(yBytes, yValue);
      expected.push(aValue * xValue + yValue);
      i = i + 1;
    }
    using params = device.createBuffer({
      label: "x02-params",
      size: SaxpyParams_SIZE as u64,
      usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
    });
    using x = device.createBuffer({
      label: "x02-x",
      size: itemBytes,
      usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
    });
    using y = device.createBuffer({
      label: "x02-y",
      size: itemBytes,
      usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC,
    });
    using readback = device.createBuffer({
      label: "x02-readback",
      size: itemBytes,
      usage: GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
    });
    const queue = device.queue();
    queue.writeBuffer(params, 0, paramsBytes);
    queue.writeBuffer(x, 0, xBytes);
    queue.writeBuffer(y, 0, yBytes);
    print("inputs:written");
    using pipeline = createComputePipeline(
      device,
      saxpy_WGSL,
      saxpy_ENTRY,
      [saxpy_LAYOUT0],
      [saxpy_WORKGROUP_X, saxpy_WORKGROUP_Y, saxpy_WORKGROUP_Z],
    );
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(device, nativeLayout, saxpy_LAYOUT0, [params, x, y]);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], count, 1, 1);
    encoder.copyBufferToBuffer(y, 0, readback, 0, itemBytes);
    using command = encoder.finishDefault();
    queue.submit([command]);
    print("dispatch:submitted");
    const mapped: boolean = await readback.mapAsync(GPUMapMode.READ, 0, itemBytes);
    if (!mapped) {
      print("FAIL map");
      return;
    }
    const result: u8[] = readback.readMappedRange(0, itemBytes);
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
