// program: resource-count-mismatch
// purpose: prove createBindGroup rejects a resource-count mismatch
// exercises: PI9
// questions: none
// expected-rule: PI9

import { BindGroupLayoutSpec, COMPUTE_VISIBILITY, createBindGroup } from "./typegpu";
import { gpu, GPUAdapter, GPUDevice } from "./webgpu";

export async function main(): Promise<void> {
  const adapter: GPUAdapter | null = await gpu.requestAdapter();
  if (adapter === null) { print("FAIL adapter"); return; }
  const device: GPUDevice | null = await adapter.requestDevice();
  if (device === null) { print("FAIL device"); return; }
  const spec: BindGroupLayoutSpec = { entries: [{
    binding: 0,
    visibility: COMPUTE_VISIBILITY,
    kind: "uniform",
    minBindingSize: 4,
  }] };
  using layout = device.createBindGroupLayout({ entries: [{
    binding: 0,
    visibility: COMPUTE_VISIBILITY,
    buffer: { type: "uniform", minBindingSize: 4 },
  }] });
  using group = createBindGroup(device, layout, spec, []);
}
