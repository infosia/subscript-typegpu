// program: x01-live-vecadd
// purpose: compute vector addition on a real adapter and compare every result
// exercises: BF7, PI12, T4, T15, storage bindings, readback
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
  Storage,
  bufferResource,
} from "./typegpu";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  GPUMapMode,
} from "./webgpu";
import {
  Item_STRIDE,
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

function itemArray(): FixedArray<Item, 64> {
  return [
    new Item(0.0), new Item(0.0), new Item(0.0), new Item(0.0),
    new Item(0.0), new Item(0.0), new Item(0.0), new Item(0.0),
    new Item(0.0), new Item(0.0), new Item(0.0), new Item(0.0),
    new Item(0.0), new Item(0.0), new Item(0.0), new Item(0.0),
    new Item(0.0), new Item(0.0), new Item(0.0), new Item(0.0),
    new Item(0.0), new Item(0.0), new Item(0.0), new Item(0.0),
    new Item(0.0), new Item(0.0), new Item(0.0), new Item(0.0),
    new Item(0.0), new Item(0.0), new Item(0.0), new Item(0.0),
    new Item(0.0), new Item(0.0), new Item(0.0), new Item(0.0),
    new Item(0.0), new Item(0.0), new Item(0.0), new Item(0.0),
    new Item(0.0), new Item(0.0), new Item(0.0), new Item(0.0),
    new Item(0.0), new Item(0.0), new Item(0.0), new Item(0.0),
    new Item(0.0), new Item(0.0), new Item(0.0), new Item(0.0),
    new Item(0.0), new Item(0.0), new Item(0.0), new Item(0.0),
    new Item(0.0), new Item(0.0), new Item(0.0), new Item(0.0),
    new Item(0.0), new Item(0.0), new Item(0.0), new Item(0.0),
  ];
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
    const itemBytes: u64 = (Item_STRIDE as u64) * (count as u64);
    const aValues: FixedArray<Item, 64> = itemArray();
    const bValues: FixedArray<Item, 64> = itemArray();
    const expected: FixedArray<Item, 64> = itemArray();
    let index: i32 = 0;
    while (index < 64) {
      const aValue: f32 = (index as f32) * 0.5;
      const bValue: f32 = ((count - (index as u32)) as f32) * 0.25;
      aValues[index] = new Item(aValue);
      bValues[index] = new Item(bValue);
      expected[index] = new Item(aValue + bValue);
      index = index + 1;
    }
    using a: Buffer<Item> = createBuffer<Item>(
      device,
      Item_STRIDE,
      count,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
      "x01-a",
    );
    using b: Buffer<Item> = createBuffer<Item>(
      device,
      Item_STRIDE,
      count,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
      "x01-b",
    );
    using out: Buffer<Item> = createBuffer<Item>(
      device,
      Item_STRIDE,
      count,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC,
      "x01-out",
    );
    using readback: Buffer<Item> = createBuffer<Item>(
      device,
      Item_STRIDE,
      count,
      GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
      "x01-readback",
    );
    const queue = device.queue();
    a.write(queue, 0, Context.bytesOf<FixedArray<Item, 64>>(aValues));
    b.write(queue, 0, Context.bytesOf<FixedArray<Item, 64>>(bValues));
    print("inputs:written");
    using pipeline = createComputePipeline(
      device,
      vecAdd_WGSL,
      vecAdd_ENTRY,
      [vecAdd_LAYOUT0],
      [vecAdd_WORKGROUP_X, vecAdd_WORKGROUP_Y, vecAdd_WORKGROUP_Z],
    );
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(
      device,
      nativeLayout,
      vecAdd_LAYOUT0,
      [bufferResource(a.handle()), bufferResource(b.handle()), bufferResource(out.handle())],
    );
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], count, 1, 1);
    out.copyTo(encoder, readback, 0, count);
    using command = encoder.finishDefault();
    queue.submit([command]);
    print("dispatch:submitted");
    const mapped: boolean = await readback.handle().mapAsync(GPUMapMode.READ, 0, itemBytes);
    if (!mapped) {
      print("FAIL map");
      return;
    }
    const result: FixedArray<Item, 64> = Context.fromBytes<FixedArray<Item, 64>>(
      readBuffer<Item>(readback, 0, count),
      0,
    );
    print("readback:mapped");
    index = 0;
    while (index < 64) {
      if (result[index].value !== expected[index].value) {
        print(`FAIL ${index} expected=${expected[index].value} got=${result[index].value}`);
        return;
      }
      index = index + 1;
    }
    readback.handle().unmap();
  }
  gpu.dispose();
  print("PASS");
}
