// program: a01-smoke
// purpose: prove API-layer buffer transfer and explicit ownership on both tiers
// exercises: adapter, device, queue, buffers, copy, submit, map, readback, disposal
// questions: none

import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  GPUMapMode,
} from "./webgpu";

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) {
    print("FAIL adapter");
    gpu.dispose();
    return;
  }
  print("adapter:ready");

  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) {
    print("FAIL device");
    adapterResult.dispose();
    gpu.dispose();
    return;
  }
  print("device:ready");

  {
    using adapter = adapterResult;
    using device = deviceResult;
    device.pushErrorScope("validation");
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print("pipeline:invalid");
      print("FAIL");
      return;
    }
    const queue = device.queue();
    using source = device.createBuffer({
      label: "a01-source",
      size: 16,
      usage: GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC,
    });
    using readback = device.createBuffer({
      label: "a01-readback",
      size: 16,
      usage: GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
    });
    print("buffers:size=16");

    const written: u8[] = [
      1, 2, 3, 4,
      5, 6, 7, 8,
      9, 10, 11, 12,
      13, 14, 15, 16,
    ];
    queue.writeBuffer(source, 0, written);
    print("write:bytes=16");

    using encoder = device.createCommandEncoder();
    encoder.copyBufferToBuffer(source, 0, readback, 0, 16);
    print("copy:size=16");
    using command = encoder.finish();
    queue.submit([command]);
    print("submit:commands=1");

    const submitted: boolean = await queue.onSubmittedWorkDone();
    if (submitted) {
      print("queue:completed");
    } else {
      print("FAIL submission");
    }

    const mapped: boolean = await readback.mapAsync(GPUMapMode.READ, 0, 16);
    if (mapped) {
      print("map:range=0+16");
    } else {
      print("FAIL map");
    }
    const observed: u8[] = readback.readMappedRange(0, 16);
    let mismatch: i32 = -1;
    let index: i32 = 0;
    while (index < written.length && index < observed.length) {
      if (mismatch === -1 && observed[index] !== written[index]) {
        mismatch = index;
      }
      index = index + 1;
    }
    if (mismatch === -1 && observed.length !== written.length) {
      mismatch = index;
    }
    if (mismatch === -1) {
      print("read:match");
    } else {
      print(`FAIL read-mismatch=${mismatch}`);
    }
    readback.unmap();
    print("buffers:unmapped");
  }

  gpu.dispose();
  print("PASS");
}
