// program: b25-kernel-forms
// purpose: prove kernel language forms through WGSL structure and host execution
// exercises: CL1, CL4, K7, K11, K12, K32, K33
// questions: none

import {
  ComputeInvocation,
  ComputePipelineSpec,
  MutStorage,
  bufferResource,
  computePipeline,
  createBindGroup,
  createComputePipeline,
  simulateCompute,
} from "./typegpu";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice } from "./webgpu";
import {
  FormsItem_STRIDE,
  kernelForms_ENTRY,
  kernelForms_HOST_RUNNABLE,
  kernelForms_LAYOUT0,
  kernelForms_WGSL,
  kernelForms_WORKGROUP_X,
  kernelForms_WORKGROUP_Y,
  kernelForms_WORKGROUP_Z,
} from "./b25-kernel-forms.typegpu";

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
    using output = device.createBuffer({
      label: "b25-output",
      size: (FormsItem_STRIDE * 2) as u64,
      usage: GPUBufferUsage.STORAGE,
    });
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      kernelForms_WGSL,
      kernelForms_ENTRY,
      [kernelForms_LAYOUT0],
      [kernelForms_WORKGROUP_X, kernelForms_WORKGROUP_Y, kernelForms_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError != null) { print("FAIL pipeline"); return; }
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(device, nativeLayout, kernelForms_LAYOUT0, [bufferResource(output)]);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatch(encoder, [bindGroup], 1, 1, 1);
    using command = encoder.finishDefault();
    device.queue.submit([command]);
    const hostLayout = new FormsLayout(new MutStorage([new FormsItem(0), new FormsItem(0)]));
    simulateCompute(formsKernel, hostLayout, kernelForms, [1, 1, 1], kernelForms_HOST_RUNNABLE);
    print("pipeline:created");
    print(`FormsItem_STRIDE=${FormsItem_STRIDE}`);
    print(`kernelForms_WORKGROUP_X=${kernelForms_WORKGROUP_X}`);
    print(`kernelForms_WORKGROUP_Y=${kernelForms_WORKGROUP_Y}`);
    print(`kernelForms_WORKGROUP_Z=${kernelForms_WORKGROUP_Z}`);
    print(`kernelForms_WGSL_LINES=${kernelForms_WGSL.split("\n").length}`);
    print("dispatch:submitted");
    print(`host:out=${hostLayout.output[0].value},${hostLayout.output[1].value}`);
  }
  gpu.dispose();
  print("PASS");
}
