// program: x28-live-evaluation-order
// purpose: compare device and host results for operand order and short-circuit conditions
// exercises: K35
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
  evaluation_ENTRY,
  evaluation_HOST_RUNNABLE,
  evaluation_LAYOUT0,
  evaluation_WGSL,
  evaluation_WORKGROUP_X,
  evaluation_WORKGROUP_Y,
  evaluation_WORKGROUP_Z,
} from "./x28-live-evaluation-order.typegpu";

import { hostLayout, orderKernel } from "./m32-evaluation-order";

const evaluation: ComputePipelineSpec = computePipeline(orderKernel, { name: "evaluation", workgroupSize: [1, 1, 1] });

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult == null) { print("FAIL adapter"); gpu.dispose(); return; }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult == null) { print("FAIL device"); adapterResult.dispose(); gpu.dispose(); return; }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const host = hostLayout();
    simulateCompute(orderKernel, host, evaluation, [1, 1, 1], evaluation_HOST_RUNNABLE);
    using output: Buffer<u32> = createBuffer(
      device,
      4,
      19,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC,
      "x28-output",
    );
    using readback: Buffer<u32> = createBuffer(
      device,
      4,
      18,
      GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
      "x28-readback",
    );
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      evaluation_WGSL,
      evaluation_ENTRY,
      [evaluation_LAYOUT0],
      [evaluation_WORKGROUP_X, evaluation_WORKGROUP_Y, evaluation_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError != null) {
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
    print("pipeline:created");
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(device, nativeLayout, evaluation_LAYOUT0, [bufferResource(output.handle())]);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatch(encoder, [bindGroup], 1, 1, 1);
    output.copyTo(encoder, readback, 0, 18);
    using command = encoder.finishDefault();
    device.queue.submit([command]);
    print("dispatch:submitted");
    const mapped: boolean = await readback.handle().mapAsync(GPUMapMode.READ, 0, (4 * 18) as u64);
    if (!mapped) { print("FAIL map"); return; }
    const result: FixedArray<u32, 18> = Context.fromBytes<FixedArray<u32, 18>>(readBuffer(readback, 0, 18), 0);
    readback.handle().unmap();
    let index: u32 = 0;
    while (index < 18) {
      const expected: u32 = host.words[index];
      const actual: u32 = result[index as i32];
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
