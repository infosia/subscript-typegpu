// program: read-without-copy-src
// purpose: prove that Buffer.read requires COPY_SRC before allocating staging storage
// exercises: BF10
// questions: none
// expected-rule: BF10

import { Buffer, createBuffer } from "./typegpu";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice } from "./webgpu";

export async function main(): Promise<void> {
  const adapter: GPUAdapter | null = await gpu.requestAdapter();
  if (adapter === null) {
    print("FAIL adapter");
    return;
  }
  const device: GPUDevice | null = await adapter.requestDevice();
  if (device === null) {
    print("FAIL device");
    return;
  }
  using buffer: Buffer<u32> = createBuffer<u32>(
    device,
    4,
    1,
    GPUBufferUsage.COPY_DST,
    "trap-read-without-copy-src",
  );
  await buffer.read(device, 0, 1);
}
