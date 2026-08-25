// program: b15-guarded-dispatch
// purpose: prove the generated hidden guard and host thread fence
// exercises: CL1, PI14, PI15
// questions: none

import {
  ComputeInvocation,
  ComputePipelineSpec,
  computePipeline,
  createComputePipeline,
  MutStorage,
  simulateComputeThreads,
} from "./typegpu";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
} from "./webgpu";
import {
  createGuardedLayoutResources,
  createGuardedPipelineBindGroup0,
  guardedPipeline_ENTRY,
  guardedPipeline_HOST_RUNNABLE,
  guardedPipeline_LAYOUT0,
  guardedPipeline_WGSL,
  guardedPipeline_WORKGROUP_X,
  guardedPipeline_WORKGROUP_Y,
  guardedPipeline_WORKGROUP_Z,
} from "./b15-guarded-dispatch.typegpu";

class GuardedLayout {
  output!: MutStorage<u32>;
}

function guardedKernel(res: GuardedLayout, ctx: ComputeInvocation): void {
  res.output[ctx.globalId.x] = ctx.globalId.x + 100;
}

export const guardedPipeline: ComputePipelineSpec = computePipeline<GuardedLayout>(
  guardedKernel,
  {
    name: "guardedPipeline",
    workgroupSize: [4, 1, 1],
    guarded: true,
  },
);

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
    using output = device.createBuffer({
      label: "b15-output",
      size: 32,
      usage: GPUBufferUsage.STORAGE,
    });
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      guardedPipeline_WGSL,
      guardedPipeline_ENTRY,
      [guardedPipeline_LAYOUT0],
      [
        guardedPipeline_WORKGROUP_X,
        guardedPipeline_WORKGROUP_Y,
        guardedPipeline_WORKGROUP_Z,
      ],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print("pipeline:invalid");
      print("FAIL");
      return;
    }
    const resources = createGuardedLayoutResources(output);
    using bindGroup = createGuardedPipelineBindGroup0(
      device,
      pipeline,
      resources,
    );
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], 6, 1, 1);
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    const host = new GuardedLayout();
    host.output = new MutStorage<u32>([999, 999, 999, 999, 999, 999, 999, 999]);
    simulateComputeThreads<GuardedLayout>(
      guardedKernel,
      host,
      guardedPipeline,
      6,
      1,
      1,
      guardedPipeline_HOST_RUNNABLE,
    );
    const guard = guardedPipeline_LAYOUT0.entries[1];
    const guardGroup: u32 = 0;
    print(`guard.group=${guardGroup}`);
    print(`guard.binding=${guard.binding}`);
    print(`layout.entries=${guardedPipeline_LAYOUT0.entries.length}`);
    print(`host:out=${host.output[0]},${host.output[5]},${host.output[6]},${host.output[7]}`);
  }
  gpu.dispose();
  print("PASS");
}
