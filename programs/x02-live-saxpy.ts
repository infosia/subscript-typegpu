// program: x02-live-saxpy
// purpose: compute SAXPY with a uniform block and compare every result
// exercises: BF7, PI12, T4, T15, uniform and storage bindings, readback
// questions: none

import {
  Buffer,
  createBuffer,
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
  Item_STRIDE,
  SaxpyParams_STRIDE,
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
    const aValue: f32 = 1.5;
    const itemBytes: u64 = (Item_STRIDE as u64) * (count as u64);
    const xValues: FixedArray<Item, 64> = itemArray();
    const yValues: FixedArray<Item, 64> = itemArray();
    const expected: FixedArray<Item, 64> = itemArray();
    let index: i32 = 0;
    while (index < 64) {
      const xValue: f32 = (index as f32) * 0.125;
      const yValue: f32 = (((index as u32) % 7) as f32) * 0.25;
      xValues[index] = new Item(xValue);
      yValues[index] = new Item(yValue);
      expected[index] = new Item(aValue * xValue + yValue);
      index = index + 1;
    }
    using params: Buffer<SaxpyParams> = createBuffer<SaxpyParams>(
      device,
      SaxpyParams_STRIDE,
      1,
      GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
      "x02-params",
    );
    using x: Buffer<Item> = createBuffer<Item>(
      device,
      Item_STRIDE,
      count,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
      "x02-x",
    );
    using y: Buffer<Item> = createBuffer<Item>(
      device,
      Item_STRIDE,
      count,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC,
      "x02-y",
    );
    using readback: Buffer<Item> = createBuffer<Item>(
      device,
      Item_STRIDE,
      count,
      GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
      "x02-readback",
    );
    const queue = device.queue();
    params.writeOne(queue, 0, Context.bytesOf<SaxpyParams>(new SaxpyParams(aValue, count)));
    x.write(queue, 0, Context.bytesOf<FixedArray<Item, 64>>(xValues));
    y.write(queue, 0, Context.bytesOf<FixedArray<Item, 64>>(yValues));
    print("inputs:written");
    using pipeline = createComputePipeline(
      device,
      saxpy_WGSL,
      saxpy_ENTRY,
      [saxpy_LAYOUT0],
      [saxpy_WORKGROUP_X, saxpy_WORKGROUP_Y, saxpy_WORKGROUP_Z],
    );
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(
      device,
      nativeLayout,
      saxpy_LAYOUT0,
      [params.handle(), x.handle(), y.handle()],
    );
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], count, 1, 1);
    y.copyTo(encoder, readback, 0, count);
    using command = encoder.finishDefault();
    queue.submit([command]);
    print("dispatch:submitted");
    const mapped: boolean = await readback.handle().mapAsync(GPUMapMode.READ, 0, itemBytes);
    if (!mapped) {
      print("FAIL map");
      return;
    }
    const result: FixedArray<Item, 64> = Context.fromBytes<FixedArray<Item, 64>>(
      y.read(readback, 0, count),
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
