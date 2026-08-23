// program: b09-kernel-depth
// purpose: prove module constants and structured switch control flow
// exercises: CL1, CL3, CL4, K18, K19, K23
// questions: none

import {
  ComputeInvocation,
  ComputePipelineSpec,
  MutStorage,
  computePipeline,
  createBindGroup,
  createComputePipeline,
  bufferResource,
  simulateCompute,
} from "./typegpu";
import { Vec2u, v2u } from "./typegpu-types";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice } from "./webgpu";
import {
  DepthItem_SIZE,
  kernelDepth_ENTRY,
  kernelDepth_HOST_RUNNABLE,
  kernelDepth_LAYOUT0,
  kernelDepth_WGSL,
  kernelDepth_WORKGROUP_X,
  kernelDepth_WORKGROUP_Y,
  kernelDepth_WORKGROUP_Z,
} from "./b09-kernel-depth.typegpu";

@CStruct
class DepthItem {
  value: u32;
  constructor(value: u32) { this.value = value; }
}

class DepthLayout {
  output!: MutStorage<DepthItem>;
}

const ITERATIONS: u32 = 4;
const INCREMENTS: Vec2u = v2u(2, 3);

function depthKernel(res: DepthLayout, ctx: ComputeInvocation): void {
  let iteration: u32 = 0;
  let value: u32 = ctx.localIndex;
  while (iteration < ITERATIONS) {
    switch (ctx.globalId.x % 4) {
      case 0:
      case 1: value += INCREMENTS.x; break;
      case 2: { iteration += 1; continue; }
      default: value += INCREMENTS.y; break;
    }
    { value += 1; }
    iteration += 1;
  }
  res.output.set(ctx.globalId.x, new DepthItem(value));
}

export const kernelDepth: ComputePipelineSpec = computePipeline<DepthLayout>(depthKernel, {
  name: "kernelDepth",
  workgroupSize: [8, 1, 1],
});

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) { print("FAIL adapter"); gpu.dispose(); return; }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) { print("FAIL device"); adapterResult.dispose(); gpu.dispose(); return; }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const count: u32 = 8;
    using output = device.createBuffer({
      label: "b09-output",
      size: (DepthItem_SIZE * count) as u64,
      usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC,
    });
    using pipeline = createComputePipeline(
      device,
      kernelDepth_WGSL,
      kernelDepth_ENTRY,
      [kernelDepth_LAYOUT0],
      [kernelDepth_WORKGROUP_X, kernelDepth_WORKGROUP_Y, kernelDepth_WORKGROUP_Z],
    );
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(device, nativeLayout, kernelDepth_LAYOUT0, [bufferResource(output)]);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], count, 1, 1);
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    const hostValues: DepthItem[] = [];
    let hostIndex: u32 = 0;
    while (hostIndex < count) {
      hostValues.push(new DepthItem(0));
      hostIndex += 1;
    }
    const hostLayout = new DepthLayout();
    hostLayout.output = new MutStorage<DepthItem>(hostValues);
    simulateCompute<DepthLayout>(
      depthKernel,
      hostLayout,
      kernelDepth,
      [1, 1, 1],
      kernelDepth_HOST_RUNNABLE,
    );
    print("pipeline:created");
    print(`DepthItem_SIZE=${DepthItem_SIZE}`);
    print(`kernelDepth_WORKGROUP_X=${kernelDepth_WORKGROUP_X}`);
    print(`kernelDepth_WORKGROUP_Y=${kernelDepth_WORKGROUP_Y}`);
    print(`kernelDepth_WORKGROUP_Z=${kernelDepth_WORKGROUP_Z}`);
    print(`kernelDepth_WGSL_LINES=${kernelDepth_WGSL.split("\n").length}`);
    print("dispatch:submitted");
    print(`host:out=${hostLayout.output.get(0).value},${hostLayout.output.get(7).value}`);
  }
  gpu.dispose();
  print("PASS");
}
