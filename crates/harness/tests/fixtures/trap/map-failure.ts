// program: map-failure
// purpose: prove that Buffer.read traps when the backend refuses the one-byte staging map
// exercises: BF9
// questions: none
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
  using buffer: Buffer<u8> = createBuffer<u8>(
    device,
    1,
    1,
    GPUBufferUsage.COPY_SRC,
    "trap-map-failure",
  );
  await buffer.read(device, 0, 1);
}
