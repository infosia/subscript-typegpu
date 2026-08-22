// program: b03-saxpy-uniform
// purpose: prove a uniform parameter block with storage input and mutable output
// exercises: K1-K16, PI1-PI11, LY11
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
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice } from "./webgpu";
import {
  Item_SIZE,
  SaxpyParams_SIZE,
  saxpy_ENTRY,
  saxpy_LAYOUT0,
  saxpy_WGSL,
  saxpy_WORKGROUP_X,
  saxpy_WORKGROUP_Y,
  saxpy_WORKGROUP_Z,
} from "./b03-saxpy-uniform.typegpu";

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
    const count: u32 = 128;
    const itemBytes: u64 = (Item_SIZE * count) as u64;
    using params = device.createBuffer({
      label: "b03-params",
      size: SaxpyParams_SIZE as u64,
      usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
    });
    using x = device.createBuffer({
      label: "b03-x",
      size: itemBytes,
      usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
    });
    using y = device.createBuffer({
      label: "b03-y",
      size: itemBytes,
      usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC,
    });
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
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    print("pipeline:created");
    print(`Item_SIZE=${Item_SIZE}`);
    print(`SaxpyParams_SIZE=${SaxpyParams_SIZE}`);
    print(`saxpy_WORKGROUP_X=${saxpy_WORKGROUP_X}`);
    print(`saxpy_WORKGROUP_Y=${saxpy_WORKGROUP_Y}`);
    print(`saxpy_WORKGROUP_Z=${saxpy_WORKGROUP_Z}`);
    print(`saxpy_WGSL_LINES=${saxpy_WGSL.split("\n").length}`);
    print("dispatch:submitted");
  }
  gpu.dispose();
  print("PASS");
}
