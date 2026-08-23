// program: b02-vecadd
// purpose: prove one generated storage pipeline from typed HIR through dispatch
// exercises: CL1, CL3, CL4, K1-K16, PI1-PI11
// questions: none

import {
  createComputePipeline,
  ComputeInvocation,
  computePipeline,
  ComputePipelineSpec,
  MutStorage,
  simulateCompute,
  Storage,
} from "./typegpu";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
} from "./webgpu";
import {
  Item_SIZE,
  VecAddLayoutResources,
  createVecAddLayoutResources,
  createVecAddBindGroup0,
  vecAdd_ENTRY,
  vecAdd_HOST_RUNNABLE,
  vecAdd_LAYOUT0,
  vecAdd_WGSL,
  vecAdd_WORKGROUP_X,
  vecAdd_WORKGROUP_Y,
  vecAdd_WORKGROUP_Z,
} from "./b02-vecadd.typegpu";

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
    const left: Item = res.a.get(i);
    const right: Item = res.b.get(i);
    const sum: Item = new Item(left.value + right.value);
    res.out.set(i, sum);
  }
}

export const vecAdd: ComputePipelineSpec = computePipeline<VecAddLayout>(vecAddKernel, {
  name: "vecAdd",
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
    const bytes: u64 = (Item_SIZE * count) as u64;
    using a = device.createBuffer({
      label: "b02-a",
      size: bytes,
      usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
    });
    using b = device.createBuffer({
      label: "b02-b",
      size: bytes,
      usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
    });
    using out = device.createBuffer({
      label: "b02-out",
      size: bytes,
      usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC,
    });
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      vecAdd_WGSL,
      vecAdd_ENTRY,
      [vecAdd_LAYOUT0],
      [vecAdd_WORKGROUP_X, vecAdd_WORKGROUP_Y, vecAdd_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print("pipeline:invalid");
      print("FAIL");
      return;
    }
    const resources: VecAddLayoutResources = createVecAddLayoutResources(a, b, out);
    using bindGroup = createVecAddBindGroup0(device, pipeline, resources);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], count, 1, 1);
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    const hostLayout = new VecAddLayout();
    hostLayout.a = new Storage<Item>([new Item(1.0), new Item(2.0), new Item(3.0)]);
    hostLayout.b = new Storage<Item>([new Item(4.0), new Item(5.0), new Item(6.0)]);
    hostLayout.out = new MutStorage<Item>([new Item(0.0), new Item(0.0), new Item(0.0)]);
    simulateCompute<VecAddLayout>(
      vecAddKernel,
      hostLayout,
      vecAdd,
      [1, 1, 1],
      vecAdd_HOST_RUNNABLE,
    );
    print("pipeline:created");
    print(`Item_SIZE=${Item_SIZE}`);
    print(`vecAdd_WORKGROUP_X=${vecAdd_WORKGROUP_X}`);
    print(`vecAdd_WORKGROUP_Y=${vecAdd_WORKGROUP_Y}`);
    print(`vecAdd_WORKGROUP_Z=${vecAdd_WORKGROUP_Z}`);
    print(`vecAdd_WGSL_LINES=${vecAdd_WGSL.split("\n").length}`);
    print("dispatch:submitted");
    print(`host:out=${hostLayout.out.get(0).value},${hostLayout.out.get(1).value},${hostLayout.out.get(2).value}`);
  }
  gpu.dispose();
  print("PASS");
}
