// program: patch-unaligned
// purpose: prove that Buffer.patch rejects an unaligned field write
// exercises: BF2
// questions: none
// expected-rule: BF2

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
    "trap-patch-unaligned",
  );
  buffer.patch(device.queue, 0, 2, [0, 0]);
}
