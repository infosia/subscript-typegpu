// program: copy-unaligned
// purpose: prove that Buffer.copyTo rejects an unaligned buffer copy
// exercises: BF8
// questions: none
// expected-rule: BF8

import { Buffer, createBuffer } from "./typegpu";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice } from "./webgpu";

export async function main(): Promise<void> {
  const adapter: GPUAdapter | null = await gpu.requestAdapter();
  if (adapter === null) { print("FAIL adapter"); return; }
  const device: GPUDevice | null = await adapter.requestDevice();
  if (device === null) { print("FAIL device"); return; }
  using source: Buffer<u16> = createBuffer<u16>(
    device,
    2,
    1,
    GPUBufferUsage.COPY_SRC,
    "trap-copy-unaligned-source",
  );
  using target: Buffer<u16> = createBuffer<u16>(
    device,
    2,
    1,
    GPUBufferUsage.COPY_DST,
    "trap-copy-unaligned-target",
  );
  using encoder = device.createCommandEncoderDefault();
  source.copyTo(encoder, target, 0, 1);
}
