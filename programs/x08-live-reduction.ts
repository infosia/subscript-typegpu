// program: x08-live-reduction
// purpose: reduce 1024 integral f32 values in four workgroups and compare the atomic sum
// exercises: K20, K21, K22, K23, T15
// questions: none

import {
  Buffer,
  ComputeInvocation,
  ComputePipelineSpec,
  MutStorage,
  Storage,
  WorkgroupArray,
  computePipeline,
  createBindGroup,
  createBuffer,
  createComputePipeline,
  readBuffer,
  workgroupArray,
  workgroupBarrier,
} from "./typegpu";
import { AtomicU32 } from "./typegpu-types";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice, GPUMapMode } from "./webgpu";
import {
  ReductionCounter_STRIDE,
  ReductionValue_STRIDE,
  reduction_ENTRY,
  reduction_LAYOUT0,
  reduction_WGSL,
  reduction_WORKGROUP_X,
  reduction_WORKGROUP_Y,
  reduction_WORKGROUP_Z,
} from "./x08-live-reduction.typegpu";

@CStruct
class ReductionValue {
  value: f32;
  constructor(value: f32) { this.value = value; }
}

@CStruct
class ReductionCounter {
  total: AtomicU32;
  constructor(total: AtomicU32) { this.total = total; }
}

class ReductionLayout {
  input!: Storage<ReductionValue>;
  output!: MutStorage<ReductionCounter>;
}

const partials: WorkgroupArray<f32> = workgroupArray<f32>(256);

function reductionKernel(res: ReductionLayout, ctx: ComputeInvocation): void {
  const global: u32 = ctx.globalId.x;
  const local: u32 = ctx.localIndex;
  partials[local] = global < 1024 ? res.input[global].value : 0.0;
  workgroupBarrier();
  let stride: u32 = 128;
  while (stride > 0) {
    if (local < stride) {
      partials[local] = partials[local] + partials[local + stride];
    }
    workgroupBarrier();
    stride = stride / 2;
  }
  if (local === 0) {
    res.output[0].total.add(partials[0] as u32);
  }
}

export const reduction: ComputePipelineSpec = computePipeline<ReductionLayout>(reductionKernel, {
  workgroupSize: [256, 1, 1],
});

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) { print("FAIL adapter"); gpu.dispose(); return; }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) { print("FAIL device"); adapterResult.dispose(); gpu.dispose(); return; }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const count: u32 = 1024;
    const inputBytes: u8[] = [];
    let byteIndex: u32 = 0;
    while (byteIndex < ReductionValue_STRIDE * count) {
      inputBytes.push(0);
      byteIndex += 1;
    }
    let index: u32 = 0;
    let expected: u32 = 0;
    while (index < count) {
      const value: u32 = (index % 8) + 1;
      Context.bytesInto<ReductionValue>(
        new ReductionValue(value as f32),
        inputBytes,
        index * ReductionValue_STRIDE,
      );
      expected += value;
      index += 1;
    }
    using input: Buffer<ReductionValue> = createBuffer<ReductionValue>(
      device,
      ReductionValue_STRIDE,
      count,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
      "x08-input",
    );
    using output: Buffer<ReductionCounter> = createBuffer<ReductionCounter>(
      device,
      ReductionCounter_STRIDE,
      1,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC,
      "x08-output",
    );
    using readback: Buffer<ReductionCounter> = createBuffer<ReductionCounter>(
      device,
      ReductionCounter_STRIDE,
      1,
      GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
      "x08-readback",
    );
    const queue = device.queue();
    input.write(queue, 0, inputBytes);
    output.writeOne(
      queue,
      0,
      Context.bytesOf<ReductionCounter>(new ReductionCounter(new AtomicU32(0))),
    );
    using pipeline = createComputePipeline(
      device,
      reduction_WGSL,
      reduction_ENTRY,
      [reduction_LAYOUT0],
      [reduction_WORKGROUP_X, reduction_WORKGROUP_Y, reduction_WORKGROUP_Z],
    );
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(
      device,
      nativeLayout,
      reduction_LAYOUT0,
      [input.handle(), output.handle()],
    );
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], count, 1, 1);
    output.copyTo(encoder, readback, 0, 1);
    using command = encoder.finishDefault();
    queue.submit([command]);
    const mapped: boolean = await readback.handle().mapAsync(
      GPUMapMode.READ,
      0,
      ReductionCounter_STRIDE as u64,
    );
    if (!mapped) { print("FAIL map"); return; }
    const result: ReductionCounter = Context.fromBytes<ReductionCounter>(
      readBuffer<ReductionCounter>(readback, 0, 1),
      0,
    );
    if (result.total.load() !== expected) {
      print(`FAIL expected=${expected} got=${result.total.load()}`);
      return;
    }
    readback.handle().unmap();
  }
  gpu.dispose();
  print("PASS");
}
