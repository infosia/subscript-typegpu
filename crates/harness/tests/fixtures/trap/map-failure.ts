// program: map-failure
// purpose: prove that Buffer.read traps when the backend refuses an aligned staging map
// exercises: BF9
// questions: none
// precondition: the backend refuses the aligned four-byte map
// expected-rule: BF9

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
    GPUBufferUsage.COPY_SRC,
    "trap-map-failure",
  );
  device.destroy();
  await buffer.read(device, 0, 1);
}
