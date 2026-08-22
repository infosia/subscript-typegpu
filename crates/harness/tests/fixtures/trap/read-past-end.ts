// program: read-past-end
// purpose: prove that Buffer.read traps before crossing the mapped range
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
  using source: Buffer<u32> = createBuffer<u32>(
    device,
    4,
    2,
    GPUBufferUsage.COPY_SRC,
    "trap-read-source",
  );
  using readback: Buffer<u32> = createBuffer<u32>(
    device,
    4,
    2,
    GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
    "trap-readback",
  );
  source.read(readback, 1, 2);
}
