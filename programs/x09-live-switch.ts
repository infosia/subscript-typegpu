// program: x09-live-switch
// purpose: compare switch break and loop continue decisions for every lane
// exercises: CL1, CL3, CL4, K18, K23, T15
// questions: none

import {
  Buffer,
  ComputeInvocation,
  ComputePipelineSpec,
  MutStorage,
  computePipeline,
  createBindGroup,
  createBuffer,
  createComputePipeline,
  readBuffer,
  simulateComputeThreads,
  bufferResource,
} from "./typegpu";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice, GPUMapMode } from "./webgpu";
import {
  SwitchValue_STRIDE,
  liveSwitch_ENTRY,
  liveSwitch_HOST_RUNNABLE,
  liveSwitch_LAYOUT0,
  liveSwitch_WGSL,
  liveSwitch_WORKGROUP_X,
  liveSwitch_WORKGROUP_Y,
  liveSwitch_WORKGROUP_Z,
} from "./x09-live-switch.typegpu";

@CStruct
class SwitchValue {
  value: u32;
  constructor(value: u32) { this.value = value; }
}

class SwitchLayout {
  output!: MutStorage<SwitchValue>;
}

function liveSwitchKernel(res: SwitchLayout, ctx: ComputeInvocation): void {
  const mode: u32 = ctx.globalId.x % 4;
  let iteration: u32 = 0;
  let value: u32 = 0;
  while (iteration < 4) {
    switch (mode) {
      case 0: value += 1; break;
      case 1: { value += 2; iteration += 1; continue; }
      case 2:
      case 3: value += 3; break;
      default: return;
    }
    iteration += 1;
  }
  res.output[ctx.globalId.x] = new SwitchValue(value);
}

export const liveSwitch: ComputePipelineSpec = computePipeline<SwitchLayout>(liveSwitchKernel, {
  name: "liveSwitch",
  workgroupSize: [16, 1, 1],
});

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) { print("FAIL adapter"); gpu.dispose(); return; }
  print("adapter:ready");
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) { print("FAIL device"); adapterResult.dispose(); gpu.dispose(); return; }
  print("device:ready");
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const count: u32 = 16;
    const hostValues: SwitchValue[] = [];
    let index: u32 = 0;
    while (index < count) {
      hostValues.push(new SwitchValue(0));
      index += 1;
    }
    const hostLayout = new SwitchLayout();
    hostLayout.output = new MutStorage<SwitchValue>(hostValues);
    simulateComputeThreads<SwitchLayout>(
      liveSwitchKernel,
      hostLayout,
      liveSwitch,
      count,
      1,
      1,
      liveSwitch_HOST_RUNNABLE,
    );
    using output: Buffer<SwitchValue> = createBuffer<SwitchValue>(
      device,
      SwitchValue_STRIDE,
      count,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC,
      "x09-output",
    );
    using readback: Buffer<SwitchValue> = createBuffer<SwitchValue>(
      device,
      SwitchValue_STRIDE,
      count,
      GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
      "x09-readback",
    );
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      liveSwitch_WGSL,
      liveSwitch_ENTRY,
      [liveSwitch_LAYOUT0],
      [liveSwitch_WORKGROUP_X, liveSwitch_WORKGROUP_Y, liveSwitch_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
    print("pipeline:created");
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(device, nativeLayout, liveSwitch_LAYOUT0, [bufferResource(output.handle())]);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], count, 1, 1);
    output.copyTo(encoder, readback, 0, count);
    using command = encoder.finishDefault();
    device.queue.submit([command]);
    print("dispatch:submitted");
    const mapped: boolean = await readback.handle().mapAsync(
      GPUMapMode.READ,
      0,
      (SwitchValue_STRIDE * count) as u64,
    );
    if (!mapped) { print("FAIL map"); return; }
    const result: FixedArray<SwitchValue, 16> = Context.fromBytes<FixedArray<SwitchValue, 16>>(
      readBuffer<SwitchValue>(readback, 0, count),
      0,
    );
    print("readback:mapped");
    index = 0;
    while (index < count) {
      const expected: u32 = hostLayout.output[index].value;
      if (result[index as i32].value !== expected) {
        print(`FAIL ${index} expected=${expected} got=${result[index as i32].value}`);
        return;
      }
      index += 1;
    }
    readback.handle().unmap();
  }
  gpu.dispose();
  print("PASS");
}
