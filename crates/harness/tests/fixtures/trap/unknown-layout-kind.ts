// program: unknown-layout-kind
// purpose: prove an unknown layout entry kind traps instead of becoming a sampler
// exercises: TX5
// questions: none
// expected-rule: TX5

import { BindGroupLayoutSpec, COMPUTE_VISIBILITY, createComputePipeline } from "./typegpu";
import { gpu, GPUAdapter, GPUDevice } from "./webgpu";

export async function main(): Promise<void> {
  const adapter: GPUAdapter | null = await gpu.requestAdapter();
  if (adapter === null) { print("FAIL adapter"); return; }
  const device: GPUDevice | null = await adapter.requestDevice();
  if (device === null) { print("FAIL device"); return; }
  const spec: BindGroupLayoutSpec = { entries: [{
    binding: 4,
    visibility: COMPUTE_VISIBILITY,
    kind: "mystery",
    minBindingSize: 0,
  }] };
  using pipeline = createComputePipeline(
    device,
    "@compute @workgroup_size(1) fn main() {}",
    "main",
    [spec],
    [1, 1, 1],
  );
}
