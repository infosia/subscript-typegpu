// program: x16-live-guarded
// purpose: prove each guarded dispatch preserves its own bounds and untouched output slots
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
  simulateComputeThreads,
} from "./typegpu";
import {
  gpu,
  GPUBindGroup,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
} from "./webgpu";
import {
  guardedPipeline_ENTRY,
  guardedPipeline_HOST_RUNNABLE,
  guardedPipeline_LAYOUT0,
  guardedPipeline_WGSL,
  guardedPipeline_WORKGROUP_X,
  guardedPipeline_WORKGROUP_Y,
  guardedPipeline_WORKGROUP_Z,
} from "./x16-live-guarded.typegpu";

class GuardedLayout {
  constructor(public output: MutStorage<u32>) {}
}

function guardedKernel(res: GuardedLayout, ctx: ComputeInvocation): void {
  res.output[ctx.globalId.x] = ctx.globalId.x + 100;
}

const guardedPipeline: ComputePipelineSpec = computePipeline(
  guardedKernel,
  {
    name: "guardedPipeline",
    workgroupSize: [4, 1, 1],
    guarded: true,
  },
);

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult == null) {
    print("FAIL adapter");
    gpu.dispose();
    return;
  }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult == null) {
    print("FAIL device");
    adapterResult.dispose();
    gpu.dispose();
    return;
  }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    using output: Buffer<u32> = createBuffer(
      device,
      4,
      8,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC,
      "x16-output",
    );
    output.write(
      device.queue,
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
    if (validationError != null) {
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
    device.queue.submit([command]);
    const outputBytes: u8[] = await output.read(device, 0, 8);
    const values: FixedArray<u32, 8> = Context.fromBytes<FixedArray<u32, 8>>(
      outputBytes,
      0,
    );
    for (let i: i32 = 0; i < 6; i += 1) {
      if (values[i] != (i as u32) + 100) {
        print(`FAIL slot=${i} value=${values[i]}`);
        return;
      }
    }
    if (values[6] != 999 || values[7] != 999) {
      print(`FAIL sentinel=${values[6]},${values[7]}`);
      return;
    }
    for (let batch: i32 = 0; batch < 2; batch += 1) {
      const counts: u32[] = batch == 0 ? [1, 3, 6] : [2, 5];
      const outputs: Buffer<u32>[] = [];
      const groups: GPUBindGroup[] = [];
      using encoderA = device.createCommandEncoderDefault();
      using encoderB = device.createCommandEncoderDefault();
      for (let index: i32 = 0; index < counts.length; index += 1) {
        const dispatchOutput: Buffer<u32> = createBuffer(
          device,
          4,
          8,
          GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC,
          "x16-dispatch-output",
        );
        dispatchOutput.write(
          device.queue,
          0,
          Context.bytesOf<FixedArray<u32, 8>>([999, 999, 999, 999, 999, 999, 999, 999]),
        );
        using dispatchLayout = pipeline.bindGroupLayout(0);
        const dispatchGroup = createBindGroup(
          device,
          dispatchLayout,
          guardedPipeline_LAYOUT0,
          [bufferResource(dispatchOutput.handle())],
          pipeline.guardBuffer(0),
        );
        outputs.push(dispatchOutput);
        groups.push(dispatchGroup);
        if (batch == 0 && index == 1) {
          pipeline.dispatchThreads(encoderB, [dispatchGroup], counts[index], 1, 1);
        } else {
          pipeline.dispatchThreads(encoderA, [dispatchGroup], counts[index], 1, 1);
        }
      }
      using commandA = encoderA.finishDefault();
      using commandB = encoderB.finishDefault();
      device.queue.submit([commandA, commandB]);
      let matches: boolean = true;
      for (let index: i32 = 0; index < counts.length; index += 1) {
        const host = new GuardedLayout(new MutStorage([999, 999, 999, 999, 999, 999, 999, 999]));
        simulateComputeThreads(
          guardedKernel,
          host,
          guardedPipeline,
          counts[index],
          1,
          1,
          guardedPipeline_HOST_RUNNABLE,
        );
        const bytes: u8[] = await outputs[index].read(device, 0, 8);
        const actual: FixedArray<u32, 8> = Context.fromBytes<FixedArray<u32, 8>>(bytes, 0);
        for (let slot: i32 = 0; slot < 8; slot += 1) {
          if (actual[slot] != host.output[slot as u32]) {
            print(`FAIL batch=${batch} dispatch=${index} slot=${slot} value=${actual[slot]} expected=${host.output[slot as u32]}`);
            matches = false;
          }
        }
        groups[index].dispose();
        outputs[index].dispose();
      }
      if (!matches) return;
    }
  }
  gpu.dispose();
  print("PASS");
}
