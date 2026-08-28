// program: a06-map-async-cost
// purpose: measure one mapAsync host drain and verify its mapped bytes
// exercises: adapter, device, queue, buffer copy, mapAsync, readback, disposal
// questions: none

import {
  gpu,
  GPUAdapter,
  GPUBuffer,
  GPUBufferUsage,
  GPUDevice,
  GPUMapMode,
} from "./webgpu";

let activeAdapter: GPUAdapter | null = null;
let activeDevice: GPUDevice | null = null;
let activeReadback: GPUBuffer | null = null;
let mapPassed: boolean = false;
// The ship tier's subscript_kick_async_exports runs each async phase again after main.
// This flag makes those second phase calls return without work.
let measurementEnabled: boolean = false;

export function enableMapAsyncMeasurement(): void {
  measurementEnabled = true;
}

export async function prepareMapAsync(): Promise<void> {
  if (!measurementEnabled) return;
  const adapter = await gpu.requestAdapter();
  if (adapter === null) {
    print("FAIL adapter");
    return;
  }
  const device = await adapter.requestDevice();
  if (device === null) {
    print("FAIL device");
    adapter.dispose();
    return;
  }
  using source = device.createBuffer({
    label: "a06-map-async-source",
    size: 8,
    usage: GPUBufferUsage.COPY_SRC + GPUBufferUsage.COPY_DST,
  });
  const readback = device.createBuffer({
    label: "a06-map-async-readback",
    size: 8,
    usage: GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
  });
  device.queue.writeBuffer(source, 0, [3, 1, 4, 1, 5, 9, 2, 6]);
  using encoder = device.createCommandEncoder();
  encoder.copyBufferToBuffer(source, 0, readback, 0, 8);
  using command = encoder.finish();
  device.queue.submit([command]);
  if (!await device.queue.onSubmittedWorkDone()) {
    print("FAIL submit");
    readback.dispose();
    device.dispose();
    adapter.dispose();
    return;
  }
  activeAdapter = adapter;
  activeDevice = device;
  activeReadback = readback;
}

export async function measureMapAsync(): Promise<void> {
  if (!measurementEnabled) return;
  const readback = activeReadback;
  if (readback === null) {
    print("FAIL prepare");
    return;
  }
  if (!await readback.mapAsync(GPUMapMode.READ, 0, 8)) {
    print("FAIL map");
    return;
  }
  const bytes = readback.readMappedRange(0, 8);
  readback.unmap();
  const expected: FixedArray<u8, 8> = [3, 1, 4, 1, 5, 9, 2, 6];
  let index: i32 = 0;
  while (index < expected.length && index < bytes.length) {
    if (bytes[index] !== expected[index]) {
      print(`FAIL byte ${index}`);
      return;
    }
    index += 1;
  }
  if (bytes.length !== expected.length) {
    print(`FAIL length ${bytes.length}`);
    return;
  }
  mapPassed = true;
  print("mapAsync:readback-match");
}

export function cleanupMapAsync(): void {
  if (!measurementEnabled) return;
  const readback = activeReadback;
  const device = activeDevice;
  const adapter = activeAdapter;
  activeReadback = null;
  activeDevice = null;
  activeAdapter = null;
  if (readback !== null) readback.dispose();
  if (device !== null) device.dispose();
  if (adapter !== null) adapter.dispose();
  gpu.dispose();
  if (mapPassed) print("PASS");
  measurementEnabled = false;
}

export async function main(): Promise<void> {
  enableMapAsyncMeasurement();
  await prepareMapAsync();
  await measureMapAsync();
  cleanupMapAsync();
}
