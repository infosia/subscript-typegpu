// example: prefix-scan
// Checks an exclusive f32 addition scan at three lengths against the module's host oracle.
// Values are padded to 256 with the 0.0 identity. The upstream operator slot is committed
// to addition, and its GPU-on-CPU expected path becomes a real host implementation.
// Ported from TypeGPU's prefix-scan example (https://github.com/software-mansion/TypeGPU).

import {
  Buffer,
  ComputePipeline,
  ComputePipelineSpec,
  bufferResource,
  computePipeline,
  createBindGroup,
  createBuffer,
  createComputePipeline,
} from "./typegpu";
import {
  RandomF32,
  randF32,
  randSeed,
} from "./typegpu-noise";
import {
  PrefixScanApplyResources,
  PrefixScanBlockResources,
  PrefixScanPlanF32,
  prefixScanApplyF32,
  prefixScanBlockF32,
  prefixScanHostF32,
  prefixScanPlanF32,
  runPrefixScanF32,
} from "./typegpu-sort";
import {
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  gpu,
} from "./webgpu";
import {
  prefixScanApplyPipeline_ENTRY,
  prefixScanApplyPipeline_LAYOUT0,
  prefixScanApplyPipeline_WGSL,
  prefixScanBlockPipeline_ENTRY,
  prefixScanBlockPipeline_LAYOUT0,
  prefixScanBlockPipeline_WGSL,
} from "./main.typegpu";

// The block kernel comes from the sort module, and this file declares its pipeline. A
// workgroup of 256 threads matches the block width the kernel and the host oracle assume.
export const prefixScanBlockPipeline: ComputePipelineSpec = computePipeline<
  PrefixScanBlockResources
>(prefixScanBlockF32, {
  name: "prefixScanBlockPipeline",
  workgroupSize: [256, 1, 1],
});

// The second kernel adds one block offset to every value. Both pipelines share the workgroup
// size, so a value and its offset stay in the same block.
export const prefixScanApplyPipeline: ComputePipelineSpec = computePipeline<
  PrefixScanApplyResources
>(prefixScanApplyF32, {
  name: "prefixScanApplyPipeline",
  workgroupSize: [256, 1, 1],
});

// One fixed seed drives every length, so a failure repeats on the next run.
let randomState: u32 = randSeed(7);

function appendScanBytes(target: u8[], source: u8[]): void {
  let index: i32 = 0;
  while (index < source.length) {
    target.push(source[index]);
    index += 1;
  }
}

// The comparison works on bytes, so the input and the expected result pass through the same
// f32 encoding the buffers hold.
function f32Bytes(values: f32[]): u8[] {
  const bytes: u8[] = [];
  let index: i32 = 0;
  while (index < values.length) {
    appendScanBytes(bytes, Context.bytesOf<FixedArray<f32, 1>>([values[index]]));
    index += 1;
  }
  return bytes;
}

function equalBytes(left: u8[], right: u8[]): boolean {
  if (left.length !== right.length) return false;
  let index: i32 = 0;
  while (index < left.length) {
    if (left[index] !== right[index]) return false;
    index += 1;
  }
  return true;
}

// The xorshift chain from the noise module feeds every length deterministically.
function randomValues(length: u32): f32[] {
  const values: f32[] = [];
  let index: u32 = 0;
  while (index < length) {
    const sample: RandomF32 = randF32(randomState);
    randomState = sample.state;
    values.push(sample.value);
    index += 1;
  }
  return values;
}

// The tail beyond the input takes 0.0, the identity of addition. It adds nothing to any
// prefix, so the padded run and the exact run agree.
function paddedValues(values: f32[], paddedLength: u32): f32[] {
  const padded: f32[] = [];
  let index: u32 = 0;
  while (index < paddedLength) {
    padded.push(index < (values.length as u32) ? values[index as i32] : 0.0);
    index += 1;
  }
  return padded;
}

// One length per call. It seeds the buffers, records the three passes, and compares the
// readback with the sort module's host oracle.
async function checkLength(
  device: GPUDevice,
  blockPipeline: ComputePipeline,
  applyPipeline: ComputePipeline,
  length: u32,
): Promise<string> {
  // The host oracle repeats both GPU scan levels in the same f32 addition order, so the
  // comparison is byte for byte with no tolerance.
  const input: f32[] = randomValues(length);
  const expected: f32[] = prefixScanHostF32(input);
  const plan: PrefixScanPlanF32 = prefixScanPlanF32(length);
  const padded: f32[] = paddedValues(input, plan.paddedLength);
  // The second pass scans one full block of 256 block sums, so every unused slot starts at
  // zero and contributes nothing.
  const zeroSums: f32[] = paddedValues([], 256);
  // Every buffer is read and written by the kernels and copied back, so all three carry the
  // same three usage flags.
  const usage: u64 = GPUBufferUsage.STORAGE
    + GPUBufferUsage.COPY_DST
    + GPUBufferUsage.COPY_SRC;
  // An f32 element is 4 bytes. The values buffer holds the padded length, and the sums buffer
  // holds the 256 slots the second pass covers.
  using values: Buffer<f32> = createBuffer<f32>(
    device,
    4,
    plan.paddedLength,
    usage,
    `prefix-scan-values-${length}`,
  );
  using sums: Buffer<f32> = createBuffer<f32>(
    device,
    4,
    256,
    usage,
    `prefix-scan-sums-${length}`,
  );
  using total: Buffer<f32> = createBuffer<f32>(
    device,
    4,
    1,
    usage,
    `prefix-scan-total-${length}`,
  );
  // The three writes seed the device before the dispatches, because the queue keeps its work
  // in the order it receives it.
  values.write(device.queue, 0, f32Bytes(padded));
  sums.write(device.queue, 0, f32Bytes(zeroSums));
  total.write(device.queue, 0, f32Bytes([0.0]));

  // The block pipeline runs twice with two different bind groups: values into sums, then sums
  // into total. One layout serves both, because the two binding shapes agree.
  using blockLayout = blockPipeline.bindGroupLayout(0);
  using valuesGroup = createBindGroup(
    device,
    blockLayout,
    prefixScanBlockPipeline_LAYOUT0,
    [bufferResource(values.handle()), bufferResource(sums.handle())],
  );
  using sumsGroup = createBindGroup(
    device,
    blockLayout,
    prefixScanBlockPipeline_LAYOUT0,
    [bufferResource(sums.handle()), bufferResource(total.handle())],
  );
  using applyLayout = applyPipeline.bindGroupLayout(0);
  using applyGroup = createBindGroup(
    device,
    applyLayout,
    prefixScanApplyPipeline_LAYOUT0,
    [bufferResource(values.handle()), bufferResource(sums.handle())],
  );
  // One encoder carries all three passes. The order matters, because each pass reads what the
  // pass before it wrote.
  using encoder = device.createCommandEncoderDefault();
  runPrefixScanF32(
    encoder,
    blockPipeline,
    valuesGroup,
    sumsGroup,
    applyPipeline,
    applyGroup,
    length,
  );
  using command = encoder.finishDefault();
  device.queue.submit([command]);
  // The read covers the original length only, so the padded tail never reaches the
  // comparison.
  const actualBytes: u8[] = await values.read(device, 0, length);
  const inputBytes: u8[] = f32Bytes(input);
  // A backend that ran no compute returns the written input, the Noop signature.
  if (equalBytes(actualBytes, inputBytes)) return "noop";
  return equalBytes(actualBytes, f32Bytes(expected)) ? "pass" : "fail";
}

export async function main(): Promise<void> {
  // The API layer polls the future itself, so the script never pumps the event loop. A null
  // adapter reports the failure by value, because the layers carry no exceptions.
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) {
    gpu.dispose();
    print("check:scan8 fail");
    print("check:scan123 fail");
    print("check:scan4096 fail");
    return;
  }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) {
    adapterResult.dispose();
    gpu.dispose();
    print("check:scan8 fail");
    print("check:scan123 fail");
    print("check:scan4096 fail");
    return;
  }
  let scan8: string = "fail";
  let scan123: string = "fail";
  let scan4096: string = "fail";
  {
    // Every GPU handle lives in this block, and the script releases each one at the end.
    // TypeGPU leaves the same handles to `root.destroy` and the collector.
    using adapter = adapterResult;
    using device = deviceResult;
    // Both pipelines come from generated constants, and both outlive the three lengths. The
    // scope catches a validation error from either creation.
    device.pushErrorScope("validation");
    using blockPipeline = createComputePipeline(
      device,
      prefixScanBlockPipeline_WGSL,
      prefixScanBlockPipeline_ENTRY,
      [prefixScanBlockPipeline_LAYOUT0],
      [256, 1, 1],
    );
    using applyPipeline = createComputePipeline(
      device,
      prefixScanApplyPipeline_WGSL,
      prefixScanApplyPipeline_ENTRY,
      [prefixScanApplyPipeline_LAYOUT0],
      [256, 1, 1],
    );
    const validationError = await device.popErrorScope();
    if (validationError === null) {
      // Three lengths. 8 and 123 fit inside one padded block, and 4096 spans 16 blocks, so
      // the second and third passes carry real work.
      scan8 = await checkLength(device, blockPipeline, applyPipeline, 8);
      scan123 = await checkLength(device, blockPipeline, applyPipeline, 123);
      scan4096 = await checkLength(device, blockPipeline, applyPipeline, 4096);
    }
  }
  gpu.dispose();
  // One check line per length reports the outcome, so a reader who runs the example needs no
  // golden.
  print(`check:scan8 ${scan8}`);
  print(`check:scan123 ${scan123}`);
  print(`check:scan4096 ${scan4096}`);
}
