// expected-rule: PI16
// purpose: prove a guarded pipeline rejects indirect dispatch

import { BindGroupLayoutSpec, ComputePipeline, ComputePipelineSpec, createComputePipeline } from "./typegpu";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice, GPUShaderStage } from "./webgpu";

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) { print("FAIL adapter"); gpu.dispose(); return; }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) { print("FAIL device"); adapterResult.dispose(); gpu.dispose(); return; }
  {
    using adapter = adapterResult; using device = deviceResult;
    const guardLayout: BindGroupLayoutSpec = { entries: [{ binding: 0, visibility: GPUShaderStage.COMPUTE, kind: "guard", minBindingSize: 16 }] };
    using pipeline: ComputePipeline = createComputePipeline(device, "@compute @workgroup_size(1) fn main() {}", "main", [guardLayout], [1,1,1]);
    using indirect = device.createBuffer({ label: "guarded-indirect", size: 12, usage: GPUBufferUsage.INDIRECT });
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchIndirect(encoder, [], indirect, 0);
  }
  gpu.dispose();
}
