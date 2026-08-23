// program: x13-live-rejection
// purpose: prove that a backend shader rejection is visible through a validation error scope
// exercises: PI14, T15
// questions: none

import {
  gpu,
  GPUAdapter,
  GPUComputePipeline,
  GPUDevice,
  GPUShaderModule,
} from "./webgpu";

const NON_UNIFORM_BARRIER: string = `
@compute @workgroup_size(64)
fn rejected(@builtin(local_invocation_index) local: u32) {
  if (local > 0u) {
    return;
  }
  workgroupBarrier();
}
`;

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) {
    print("FAIL adapter");
    gpu.dispose();
    return;
  }
  print("adapter:ready");
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) {
    print("FAIL device");
    adapterResult.dispose();
    gpu.dispose();
    return;
  }
  print("device:ready");
  {
    using adapter = adapterResult;
    using device = deviceResult;
    device.pushErrorScope("validation");
    using shader: GPUShaderModule = device.createShaderModule({
      label: "x13-non-uniform-barrier",
      code: NON_UNIFORM_BARRIER,
    });
    using pipeline: GPUComputePipeline = device.createComputePipeline({
      label: "x13-rejected-pipeline",
      layout: null,
      compute: { module: shader, entryPoint: "rejected" },
    });
    const validationError = await device.popErrorScope();
    if (validationError === null) {
      print("FAIL validation missing uniform-control-flow rejection");
      return;
    }
    const firstLine: string = validationError.message.split("\n")[0];
    if (!firstLine.includes("uniform control flow")) {
      print(`FAIL validation ${firstLine}`);
      return;
    }
  }
  gpu.dispose();
  print("PASS");
}
