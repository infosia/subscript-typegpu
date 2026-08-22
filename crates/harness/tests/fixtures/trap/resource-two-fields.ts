// program: resource-two-fields
// purpose: prove a BindingResource with two populated fields traps
// exercises: TX4
// questions: none

import { BindGroupLayoutSpec, COMPUTE_VISIBILITY, createBindGroup } from "./typegpu";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice } from "./webgpu";

export async function main(): Promise<void> {
  const adapter: GPUAdapter | null = await gpu.requestAdapter();
  if (adapter === null) { print("FAIL adapter"); return; }
  const device: GPUDevice | null = await adapter.requestDevice();
  if (device === null) { print("FAIL device"); return; }
  const spec: BindGroupLayoutSpec = { entries: [{
    binding: 1,
    visibility: COMPUTE_VISIBILITY,
    kind: "uniform",
    minBindingSize: 4,
  }] };
  using layout = device.createBindGroupLayout({ entries: [{
    binding: 1,
    visibility: COMPUTE_VISIBILITY,
    buffer: { type: "uniform", minBindingSize: 4 },
  }] });
  using buffer = device.createBuffer({
    label: "tx4-buffer",
    size: 4,
    usage: GPUBufferUsage.UNIFORM,
  });
  using sampler = device.createSampler({ minFilter: "nearest", magFilter: "nearest" });
  using group = createBindGroup(device, layout, spec, [{
    buffer,
    textureView: null,
    sampler,
  }]);
}
