// program: read-unaligned
// purpose: prove that Buffer.read rejects a two-byte WebGPU copy
// exercises: BF9
// questions: none
// expected-rule: BF9

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
    1,
    GPUBufferUsage.COPY_SRC,
    "trap-read-unaligned",
  );
  await buffer.read(device, 0, 1);
}
