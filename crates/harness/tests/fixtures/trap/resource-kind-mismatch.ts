// program: resource-kind-mismatch
// purpose: prove createBindGroup rejects a resource whose kind differs from the layout entry
// exercises: TX4, TX8
// questions: none

import {
  BindGroupLayoutSpec,
  COMPUTE_VISIBILITY,
  bufferResource,
  createBindGroup,
} from "./typegpu";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice } from "./webgpu";

export async function main(): Promise<void> {
  const adapter: GPUAdapter | null = await gpu.requestAdapter();
  if (adapter === null) { print("FAIL adapter"); return; }
  const device: GPUDevice | null = await adapter.requestDevice();
  if (device === null) { print("FAIL device"); return; }
  const spec: BindGroupLayoutSpec = { entries: [{
    binding: 3,
    visibility: COMPUTE_VISIBILITY,
    kind: "texture",
    minBindingSize: 0,
    sampleType: "float",
  }] };
  using layout = device.createBindGroupLayout({ entries: [{
    binding: 3,
    visibility: COMPUTE_VISIBILITY,
    texture: { sampleType: "float", viewDimension: "2d", multisampled: false },
  }] });
  using buffer = device.createBuffer({
    label: "tx8-buffer",
    size: 4,
    usage: GPUBufferUsage.STORAGE,
  });
  using group = createBindGroup(device, layout, spec, [bufferResource(buffer)]);
}
