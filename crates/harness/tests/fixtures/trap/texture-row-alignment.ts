// program: texture-row-alignment
// purpose: prove a multi-row raw texture upload rejects a row stride below 256 bytes
// exercises: TX9
// questions: none
// expected-rule: TX9

import { writeTextureBytes } from "./typegpu";
import {
  gpu,
  GPUAdapter,
  GPUDevice,
  GPUTextureUsage,
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
  using texture = device.createTexture({
    label: "trap-texture-row-alignment",
    size: { width: 2, height: 2 },
    format: "rgba8unorm",
    usage: GPUTextureUsage.COPY_DST,
  });
  writeTextureBytes(
    device.queue,
    texture,
    [
      0, 0, 0, 255, 255, 255, 255, 255,
      0, 0, 0, 255, 255, 255, 255, 255,
    ],
    8,
    2,
    2,
  );
}
