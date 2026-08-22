// program: write-past-end
// purpose: prove that Buffer.write traps before crossing the element range
// exercises: BF8
// questions: none

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
    2,
    GPUBufferUsage.COPY_DST,
    "trap-write-past-end",
  );
  const bytes: u8[] = [0, 0, 0, 0, 0, 0, 0, 0];
  buffer.write(device.queue(), 1, bytes);
}
