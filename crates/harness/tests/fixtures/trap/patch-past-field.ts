// program: patch-past-field
// purpose: prove Buffer.patch rejects bytes that cross the element boundary
// exercises: EG2
// questions: none
// expected-rule: EG2

import { Buffer, createBuffer } from "./typegpu";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice } from "./webgpu";

export async function main(): Promise<void> {
  const adapter: GPUAdapter | null = await gpu.requestAdapter();
  if (adapter === null) { print("FAIL adapter"); return; }
  const device: GPUDevice | null = await adapter.requestDevice();
  if (device === null) { print("FAIL device"); return; }
  using buffer: Buffer<u32> = createBuffer<u32>(
    device,
    4,
    1,
    GPUBufferUsage.COPY_DST,
    "trap-patch-past-field",
  );
  buffer.patch(device.queue, 0, 3, [1, 2]);
}
