// program: b10-workgroup
// purpose: prove private and workgroup variables, barriers, and storage atomics
// exercises: K20, K21, K22, K23
// questions: none

import {
  ComputeInvocation,
  ComputePipelineSpec,
  MutStorage,
  PrivateVar,
  WorkgroupArray,
  WorkgroupVar,
  computePipeline,
  createBindGroup,
  createComputePipeline,
  privateVar,
  workgroupArray,
  workgroupBarrier,
  workgroupVar,
  bufferResource,
} from "./typegpu";
import { AtomicU32 } from "./typegpu-types";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice } from "./webgpu";
import {
  WorkCounter_SIZE,
  workgroup_ENTRY,
  workgroup_LAYOUT0,
  workgroup_WGSL,
  workgroup_WORKGROUP_X,
  workgroup_WORKGROUP_Y,
  workgroup_WORKGROUP_Z,
} from "./b10-workgroup.typegpu";

@CStruct
class WorkCounter {
  total: AtomicU32;
  constructor(total: AtomicU32) { this.total = total; }
}

class WorkgroupLayout {
  counters!: MutStorage<WorkCounter>;
}

const privateOffset: PrivateVar<u32> = privateVar<u32>(3);
const sharedValues: WorkgroupArray<u32> = workgroupArray<u32>(4);
const sharedCounter: WorkgroupVar<AtomicU32> = workgroupVar<AtomicU32>();

function workgroupKernel(res: WorkgroupLayout, ctx: ComputeInvocation): void {
  privateOffset.set(privateOffset.get() + 1);
  sharedValues[ctx.localIndex] = ctx.localIndex + privateOffset.get();
  if (ctx.localIndex === 0) {
    sharedCounter.get().store(0);
  }
  workgroupBarrier();
  sharedCounter.get().add(sharedValues[ctx.localIndex]);
  workgroupBarrier();
  if (ctx.localIndex === 0) {
    res.counters[ctx.workgroupId.x].total.add(sharedCounter.get().load());
  }
}

export const workgroup: ComputePipelineSpec = computePipeline<WorkgroupLayout>(workgroupKernel, {
  name: "workgroup",
  workgroupSize: [4, 1, 1],
});

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) { print("FAIL adapter"); gpu.dispose(); return; }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) { print("FAIL device"); adapterResult.dispose(); gpu.dispose(); return; }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const count: u32 = 2;
    using counters = device.createBuffer({
      label: "b10-counters",
      size: (WorkCounter_SIZE * count) as u64,
      usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC,
    });
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      workgroup_WGSL,
      workgroup_ENTRY,
      [workgroup_LAYOUT0],
      [workgroup_WORKGROUP_X, workgroup_WORKGROUP_Y, workgroup_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print("pipeline:invalid");
      print("FAIL");
      return;
    }
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(device, nativeLayout, workgroup_LAYOUT0, [bufferResource(counters)]);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], count * 4, 1, 1);
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    print("pipeline:created");
    print(`WorkCounter_SIZE=${WorkCounter_SIZE}`);
    print(`workgroup_WORKGROUP_X=${workgroup_WORKGROUP_X}`);
    print(`workgroup_WORKGROUP_Y=${workgroup_WORKGROUP_Y}`);
    print(`workgroup_WORKGROUP_Z=${workgroup_WORKGROUP_Z}`);
    print(`workgroup_WGSL_LINES=${workgroup_WGSL.split("\n").length}`);
    print("dispatch:submitted");
  }
  gpu.dispose();
  print("PASS");
}
