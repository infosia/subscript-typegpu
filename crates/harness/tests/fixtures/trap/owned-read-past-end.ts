// program: owned-read-past-end
// purpose: prove that Buffer.read checks bounds before it creates the staging buffer
// exercises: BF9
// questions: none
// expected-rule: BF9

import { Buffer, createBuffer } from "./typegpu";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice } from "./webgpu";

export async function main(): Promise<void> {
  const adapter: GPUAdapter | null = await gpu.requestAdapter();
  if (adapter === null) { print("FAIL adapter"); return; }
  const device: GPUDevice | null = await adapter.requestDevice();
  if (device === null) { print("FAIL device"); return; }
  using buffer: Buffer<u32> = createBuffer<u32>(device, 4, 1, GPUBufferUsage.COPY_SRC, "trap-owned-read-range");
  await buffer.read(device, 1, 1);
}
