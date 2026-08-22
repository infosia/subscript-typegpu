// program: a04-errors
// purpose: cover adapter capabilities, device reports, labels, and error scopes
// exercises: EG4, G1-G4, H2-H3
// questions: none

import { gpu, GPUAdapter, GPUDevice, GPUError } from "./webgpu";

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) {
    print("FAIL adapter");
    gpu.dispose();
    return;
  }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) {
    print("FAIL device");
    adapterResult.dispose();
    gpu.dispose();
    return;
  }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const adapterFeature: boolean = adapter.hasFeature("timestamp-query");
    adapter.limits();
    adapter.info();
    print(`adapter:capabilities=${adapterFeature}`);

    const deviceFeature: boolean = device.hasFeature("timestamp-query");
    device.limits();
    device.adapterInfo();
    device.label("a04-device");
    device.queue().label("a04-queue");
    print(`device:capabilities=${deviceFeature}`);

    device.pushErrorScope("validation");
    const scoped: GPUError | null = await device.popErrorScope();
    device.nextUncapturedError();
    device.deviceLostInfo();
    if (scoped === null) {
      print("errors:empty");
    } else {
      print("errors:reported");
    }
    device.destroy();
  }
  gpu.dispose();
  print("PASS");
}
