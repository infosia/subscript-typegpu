// program: b03-saxpy-uniform
// purpose: prove a uniform parameter block with storage input and mutable output
// exercises: CL1, CL3, CL4, K1-K16, PI1-PI11, LY11
// questions: none

import {
  createComputePipeline,
  createBindGroup,
  ComputeInvocation,
  computePipeline,
  ComputePipelineSpec,
  MutStorage,
  simulateCompute,
  Storage,
  Uniform,
  bufferResource,
} from "./typegpu";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice } from "./webgpu";
import {
  Item_SIZE,
  SaxpyParams_SIZE,
  saxpy_ENTRY,
  saxpy_HOST_RUNNABLE,
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
  const settings: SaxpyParams = res.params.$;
  const i: u32 = ctx.globalId.x;
  if (i < settings.count) {
    const xItem: Item = res.x[i];
    const yItem: Item = res.y[i];
    res.y[i] = new Item(settings.a * xItem.value + yItem.value);
  }
}

export const saxpy: ComputePipelineSpec = computePipeline<SaxpyLayout>(saxpyKernel, {
  name: "saxpy",
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
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      saxpy_WGSL,
      saxpy_ENTRY,
      [saxpy_LAYOUT0],
      [saxpy_WORKGROUP_X, saxpy_WORKGROUP_Y, saxpy_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print("pipeline:invalid");
      print("FAIL");
      return;
    }
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(device, nativeLayout, saxpy_LAYOUT0, [bufferResource(params), bufferResource(x), bufferResource(y)]);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], count, 1, 1);
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    const hostLayout = new SaxpyLayout();
    hostLayout.params = new Uniform<SaxpyParams>(new SaxpyParams(2.0, 2));
    hostLayout.x = new Storage<Item>([new Item(3.0), new Item(4.0)]);
    hostLayout.y = new MutStorage<Item>([new Item(1.0), new Item(2.0)]);
    simulateCompute<SaxpyLayout>(
      saxpyKernel,
      hostLayout,
      saxpy,
      [1, 1, 1],
      saxpy_HOST_RUNNABLE,
    );
    print("pipeline:created");
    print(`Item_SIZE=${Item_SIZE}`);
    print(`SaxpyParams_SIZE=${SaxpyParams_SIZE}`);
    print(`saxpy_WORKGROUP_X=${saxpy_WORKGROUP_X}`);
    print(`saxpy_WORKGROUP_Y=${saxpy_WORKGROUP_Y}`);
    print(`saxpy_WORKGROUP_Z=${saxpy_WORKGROUP_Z}`);
    print(`saxpy_WGSL_LINES=${saxpy_WGSL.split("\n").length}`);
    print("dispatch:submitted");
    print(`host:out=${hostLayout.y[0].value},${hostLayout.y[1].value}`);
  }
  gpu.dispose();
  print("PASS");
}
