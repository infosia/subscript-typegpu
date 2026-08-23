// program: x16-live-guarded
// purpose: prove guarded dispatchThreads preserves slots beyond the exact thread count
// exercises: BF9, CL1, PI14, PI15
// questions: none

import {
  Buffer,
  bufferResource,
  ComputeInvocation,
  ComputePipelineSpec,
  computePipeline,
  createBindGroup,
  createBuffer,
  createComputePipeline,
  MutStorage,
} from "./typegpu";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
} from "./webgpu";
import {
  guardedPipeline_ENTRY,
  guardedPipeline_LAYOUT0,
  guardedPipeline_WGSL,
  guardedPipeline_WORKGROUP_X,
  guardedPipeline_WORKGROUP_Y,
  guardedPipeline_WORKGROUP_Z,
} from "./x16-live-guarded.typegpu";

class GuardedLayout {
  output!: MutStorage<u32>;
}

function guardedKernel(res: GuardedLayout, ctx: ComputeInvocation): void {
  res.output.set(ctx.globalId.x, ctx.globalId.x + 100);
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
    using output: Buffer<u32> = createBuffer<u32>(
      device,
      4,
      8,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC,
      "x16-output",
    );
    output.write(
      device.queue(),
      0,
      Context.bytesOf<FixedArray<u32, 8>>([
        999,
        999,
        999,
        999,
        999,
        999,
        999,
        999,
      ]),
    );
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
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
    using bindGroup = createBindGroup(
      device,
      pipeline.bindGroupLayout(0),
      guardedPipeline_LAYOUT0,
      [bufferResource(output.handle())],
      pipeline.guardBuffer(0),
    );
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], 6, 1, 1);
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    const outputBytes: u8[] = await output.read(device, 0, 8);
    const values: FixedArray<u32, 8> = Context.fromBytes<FixedArray<u32, 8>>(
      outputBytes,
      0,
    );
    for (let i: i32 = 0; i < 6; i += 1) {
      if (values[i] !== (i as u32) + 100) {
        print(`FAIL slot=${i} value=${values[i]}`);
        return;
      }
    }
    if (values[6] !== 999 || values[7] !== 999) {
      print(`FAIL sentinel=${values[6]},${values[7]}`);
      return;
    }
  }
  gpu.dispose();
  print("PASS");
}
