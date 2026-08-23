// program: write-unaligned
// purpose: prove that Buffer.write rejects a six-byte WebGPU write
// exercises: BF2
// questions: none
// expected-rule: BF2

import {
  Buffer,
  createBuffer,
} from "./typegpu";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
} from "./webgpu";

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
  using buffer: Buffer<u16> = createBuffer<u16>(
    device,
    2,
    4,
    GPUBufferUsage.COPY_DST,
    "trap-write-unaligned",
  );
  buffer.write(device.queue(), 0, [0, 0, 1, 0, 2, 0]);
}
