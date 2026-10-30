// program: x25-live-kernel-forms
// purpose: compare kernel language forms with host results on a live backend
// exercises: CL1, CL3, CL4, K7, K11, K12, K32, K33, T15
// questions: none

import {
  Buffer,
  ComputeInvocation,
  ComputePipelineSpec,
  MutStorage,
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
  FormsItem_STRIDE,
  kernelForms_ENTRY,
  kernelForms_HOST_RUNNABLE,
  kernelForms_LAYOUT0,
  kernelForms_WGSL,
  kernelForms_WORKGROUP_X,
  kernelForms_WORKGROUP_Y,
  kernelForms_WORKGROUP_Z,
} from "./x25-live-kernel-forms.typegpu";

interface Weighted {
  weight(): u32;
}

@ValueType
class FormsItem implements Weighted {
  constructor(public value: u32) {}
  weight(): u32 { return this.value * 2; }
}

class FormsLayout {
  constructor(public output: MutStorage<FormsItem>) {}
}

enum Mode { Bias = 3, Scale = 5 }

function identity<T>(value: T): T { return value; }

function weighted<T extends Weighted>(value: T): u32 {
  return value.weight();
}

function modeValue(mode: Mode): u32 {
  switch (mode) {
    case Mode.Bias: return 3;
    case Mode.Scale: return 5;
  }
}

function localScore(value: u32): u32 {
  function plusOne(value: u32): u32 { return value + 1; }
  return plusOne(value);
}

function formsKernel(res: FormsLayout, ctx: ComputeInvocation): void {
  let iteration: u32 = 0;
  let total: u32 = 0;
  do {
    iteration++;
    if (iteration == 2) { continue; }
    total += iteration;
  } while (iteration < 4);
  let selected: boolean = ctx.globalId.x == 0;
  selected &&= total == 8;
  selected ||= false;
  const base: f32 = total as f32;
  const maximum: f32 = Math.max(base as f64, (base + 2.0) as f64, (base + 1.0) as f64) as f32;
  const mode: Mode = Mode.Scale;
  const item = new FormsItem(identity<u32>(3));
  let result: u32 = total + (identity<f32>(maximum) as u32) + modeValue(mode)
    + item.value + item.weight() + weighted(item) + localScore(2);
  if (selected) { result += 1; }
  res.output[ctx.globalId.x] = new FormsItem(result);
}

const kernelForms: ComputePipelineSpec = computePipeline(formsKernel, {
  name: "kernelForms",
  workgroupSize: [2, 1, 1],
});

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult == null) { print("FAIL adapter"); gpu.dispose(); return; }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult == null) { print("FAIL device"); adapterResult.dispose(); gpu.dispose(); return; }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const hostLayout = new FormsLayout(new MutStorage([new FormsItem(0), new FormsItem(0)]));
    simulateCompute(formsKernel, hostLayout, kernelForms, [1, 1, 1], kernelForms_HOST_RUNNABLE);
    using output: Buffer<FormsItem> = createBuffer<FormsItem, GPUDevice>(
      device,
      FormsItem_STRIDE,
      2,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC + GPUBufferUsage.COPY_DST,
      "x25-output",
    );
    using readback: Buffer<FormsItem> = createBuffer<FormsItem, GPUDevice>(
      device,
      FormsItem_STRIDE,
      2,
      GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
      "x25-readback",
    );
    const zeros: FixedArray<FormsItem, 2> = [new FormsItem(0), new FormsItem(0)];
    output.write(device.queue, 0, Context.bytesOf<FixedArray<FormsItem, 2>>(zeros));
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      kernelForms_WGSL,
      kernelForms_ENTRY,
      [kernelForms_LAYOUT0],
      [kernelForms_WORKGROUP_X, kernelForms_WORKGROUP_Y, kernelForms_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError != null) {
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
    print("pipeline:created");
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(device, nativeLayout, kernelForms_LAYOUT0, [bufferResource(output.handle())]);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatch(encoder, [bindGroup], 1, 1, 1);
    output.copyTo(encoder, readback, 0, 2);
    using command = encoder.finishDefault();
    device.queue.submit([command]);
    print("dispatch:submitted");
    const mapped: boolean = await readback.handle().mapAsync(GPUMapMode.READ, 0, (FormsItem_STRIDE * 2) as u64);
    if (!mapped) { print("FAIL map"); return; }
    const result: FixedArray<FormsItem, 2> = Context.fromBytes<FixedArray<FormsItem, 2>>(readBuffer(readback, 0, 2), 0);
    readback.handle().unmap();
    let index: u32 = 0;
    while (index < 2) {
      const expected: u32 = hostLayout.output[index].value;
      const actual: u32 = result[index as i32].value;
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
