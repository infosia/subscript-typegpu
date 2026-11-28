// program: x31-live-matrix-columns
// purpose: compare device and host results for each matrix column
// exercises: K10, K36
// questions: none

import {
  Buffer,
  ComputePipelineSpec,
  bufferResource,
  computePipeline,
  createBindGroup,
  createBuffer,
  createComputePipeline,
  readBuffer,
  simulateCompute,
} from "./typegpu";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice, GPUMapMode } from "./webgpu";
import {
  words_ENTRY,
  words_HOST_RUNNABLE,
  words_LAYOUT0,
  words_WGSL,
  words_WORKGROUP_X,
  words_WORKGROUP_Y,
  words_WORKGROUP_Z,
} from "./x31-live-matrix-columns.typegpu";

import { hostLayout, writeWords } from "./m34-matrix-columns";

const words: ComputePipelineSpec = computePipeline(writeWords, { name: "words", workgroupSize: [1, 1, 1] });

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult == null) { print("FAIL adapter"); gpu.dispose(); return; }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult == null) { print("FAIL device"); adapterResult.dispose(); gpu.dispose(); return; }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const host = hostLayout();
    simulateCompute(writeWords, host, words, [1, 1, 1], words_HOST_RUNNABLE);
    using output: Buffer<f32> = createBuffer(
      device,
      4,
      9,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC,
      "x31-output",
    );
    using readback: Buffer<f32> = createBuffer(
      device,
      4,
      9,
      GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
      "x31-readback",
    );
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      words_WGSL,
      words_ENTRY,
      [words_LAYOUT0],
      [words_WORKGROUP_X, words_WORKGROUP_Y, words_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError != null) {
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
    print("pipeline:created");
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(device, nativeLayout, words_LAYOUT0, [bufferResource(output.handle())]);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatch(encoder, [bindGroup], 1, 1, 1);
    output.copyTo(encoder, readback, 0, 9);
    using command = encoder.finishDefault();
    device.queue.submit([command]);
    print("dispatch:submitted");
    const mapped: boolean = await readback.handle().mapAsync(GPUMapMode.READ, 0, (4 * 9) as u64);
    if (!mapped) { print("FAIL map"); return; }
    const result: FixedArray<f32, 9> = Context.fromBytes<FixedArray<f32, 9>>(readBuffer(readback, 0, 9), 0);
    readback.handle().unmap();
    let index: u32 = 0;
    while (index < 9) {
      const expected: f32 = host.words[index];
      const actual: f32 = result[index as i32];
      if (actual != expected) {
        print(`FAIL ${index} expected=${expected} got=${actual}`);
        return;
      }
      index++;
    }
  }
  gpu.dispose();
  print("PASS");
}
