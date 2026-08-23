// program: guarded-second-dispatch
// purpose: prove that one encoder rejects a second guarded dispatch
// exercises: PI15
// questions: none
// expected-rule: PI15

import {
  BindGroupLayoutSpec,
  ComputePipeline,
  createComputePipeline,
} from "./typegpu";
import {
  gpu,
  GPUAdapter,
  GPUDevice,
  GPUShaderStage,
} from "./webgpu";

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) {
    print("FAIL adapter");
    return;
  }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) {
    print("FAIL device");
    return;
  }
  using device = deviceResult;
  const layout: BindGroupLayoutSpec = {
    entries: [{
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      kind: "guard",
      minBindingSize: 16,
    }],
  };
  using pipeline: ComputePipeline = createComputePipeline(
    device,
    "@group(0) @binding(0) var<uniform> guard: vec3<u32>; @compute @workgroup_size(1) fn main() {}",
    "main",
    [layout],
    [1, 1, 1],
  );
  using firstEncoder = device.createCommandEncoderDefault();
  pipeline.dispatch(firstEncoder, [], 1, 1, 1);
  using secondEncoder = device.createCommandEncoderDefault();
  pipeline.dispatch(secondEncoder, [], 1, 1, 1);
  pipeline.dispatch(secondEncoder, [], 2, 1, 1);
}
