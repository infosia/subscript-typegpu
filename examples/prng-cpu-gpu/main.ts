// example: prng-cpu-gpu
// Checks that one deterministic PRNG produces byte-identical CPU and GPU sequences.
// TypeGPU compares three generators and four seed functions inside a 1e-6 tolerance.
// This port keeps one Wang-seeded xorshift32 and one seed function, and compares raw bytes.
// Ported from TypeGPU's prng-cpu-gpu example (https://github.com/software-mansion/TypeGPU).

import {
  Buffer,
  ComputeInvocation,
  ComputePipelineSpec,
  MutStorage,
  bufferResource,
  computePipeline,
  createBindGroup,
  createBuffer,
  createComputePipeline,
  simulateComputeThreads,
} from "./typegpu";
import {
  RandomF32,
  randF32,
  randSeed,
} from "./typegpu-noise";
import {
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  gpu,
} from "./webgpu";
import {
  RandomValue_STRIDE,
  randomFill_ENTRY,
  randomFill_HOST_RUNNABLE,
  randomFill_LAYOUT0,
  randomFill_WGSL,
  randomFill_WORKGROUP_X,
  randomFill_WORKGROUP_Y,
  randomFill_WORKGROUP_Z,
} from "./main.typegpu";

// One sample per thread and five draws per sample. The count stays small, because the host
// compares every byte of the result.
const SAMPLE_COUNT: u32 = 64;
const ROUND_COUNT: u32 = 5;

// The output record: the generator state after the last draw, and that draw's value. The
// comparison covers both, so a state divergence cannot hide behind an equal value.
@CStruct
class RandomValue {
  state: u32;
  value: f32;

  constructor(state: u32, value: f32) {
    this.state = state;
    this.value = value;
  }
}

// The single mutable storage binding. TypeGPU allocates the same buffer with
// `root.createMutable`, and the generator emits `randomFill_LAYOUT0` from this class.
class RandomLayout {
  output!: MutStorage<RandomValue>;
}

// One thread per sample. Each thread seeds from its own index and advances the
// generator ROUND_COUNT times, so the host repeats the same walk.
function randomKernel(res: RandomLayout, ctx: ComputeInvocation): void {
  const index: u32 = ctx.globalId.x;
  if (index >= res.output.length()) return;
  let state: u32 = randSeed(index + 1);
  let value: f32 = 0.0;
  for (let round: u32 = 0; round < ROUND_COUNT; round += 1) {
    const sample: RandomF32 = randF32(state);
    state = sample.state;
    value = sample.value;
  }
  res.output[index] = new RandomValue(state, value);
}

// The declaration names the pipeline and fixes the workgroup size at 32 threads. The
// generator reads it and emits the WGSL and the constants of `main.typegpu`.
export const randomFill: ComputePipelineSpec = computePipeline<RandomLayout>(randomKernel, {
  name: "randomFill",
  workgroupSize: [32, 1, 1],
});

// The host lane takes the same layout class the kernel takes. The wrapper types hold plain
// script arrays, so one kernel body serves both lanes.
function makeHostLayout(): RandomLayout {
  const values: RandomValue[] = [];
  for (let index: u32 = 0; index < SAMPLE_COUNT; index += 1) {
    values.push(new RandomValue(0, 0.0));
  }
  const layout = new RandomLayout();
  layout.output = new MutStorage<RandomValue>(values);
  return layout;
}

// The host result becomes bytes through the same schema layout the device buffer holds, so
// the comparison needs no field-by-field code.
function hostBytes(layout: RandomLayout): u8[] {
  const bytes: u8[] = [];
  for (let index: u32 = 0; index < SAMPLE_COUNT; index += 1) {
    const valueBytes: u8[] = Context.bytesOf<RandomValue>(layout.output[index]);
    for (let byteIndex: i32 = 0; byteIndex < valueBytes.length; byteIndex += 1) {
      bytes.push(valueBytes[byteIndex]);
    }
  }
  return bytes;
}

export async function main(): Promise<void> {
  // The API layer polls the future itself, so the script never pumps the event loop. A null
  // adapter reports the failure by value, because the layers carry no exceptions.
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) {
    gpu.dispose();
    print("check:prng fail");
    return;
  }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) {
    adapterResult.dispose();
    gpu.dispose();
    print("check:prng fail");
    return;
  }
  let state: string = "fail";
  {
    // Every GPU handle lives in this block, and the script releases each one at the end.
    // TypeGPU leaves the same handles to `root.destroy` and the collector.
    using adapter = adapterResult;
    using device = deviceResult;
    // `RandomValue_STRIDE` is a generated constant, so the element pitch follows the schema
    // layout. STORAGE binds the buffer to the kernel, and COPY_SRC admits the readback copy.
    using output: Buffer<RandomValue> = createBuffer<RandomValue>(
      device,
      RandomValue_STRIDE,
      SAMPLE_COUNT,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC,
      "prng-output",
    );
    // The generated WGSL, entry name, layout, and workgroup size build the pipeline. The scope
    // catches a validation error from that call, and this device lane awaits the result.
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      randomFill_WGSL,
      randomFill_ENTRY,
      [randomFill_LAYOUT0],
      [randomFill_WORKGROUP_X, randomFill_WORKGROUP_Y, randomFill_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError === null) {
      // The bind group joins the output buffer to the generated layout. The resource order
      // follows the field order of `RandomLayout`.
      using nativeLayout = pipeline.bindGroupLayout(0);
      using group = createBindGroup(
        device,
        nativeLayout,
        randomFill_LAYOUT0,
        [bufferResource(output.handle())],
      );
      using encoder = device.createCommandEncoderDefault();
      // `dispatchThreads` takes a thread count and rounds it up to whole workgroups. 64 threads
      // fill two workgroups of 32 exactly, so the bounds check never ends an invocation.
      pipeline.dispatchThreads(encoder, [group], SAMPLE_COUNT, 1, 1);
      // The GPU runs nothing until the queue receives the command buffer.
      using command = encoder.finishDefault();
      device.queue.submit([command]);

      // The read copies into a staging buffer and awaits the map. TypeGPU's `buffer.read`
      // hides the same two steps.
      const gpuBytes: u8[] = await output.read(device, 0, SAMPLE_COUNT);
      // The Noop backend leaves the output at zero. The example reports `noop` for that
      // case, so an unexecuted kernel never reads as a passing comparison.
      let allZero: boolean = true;
      for (let index: i32 = 0; index < gpuBytes.length; index += 1) {
        allZero = allZero && gpuBytes[index] === 0;
      }
      if (allZero) {
        state = "noop";
      } else {
        // The host lane runs the same kernel over host storage. The thread count matches the
        // dispatch, so the example holds no second implementation of the generator.
        const hostLayout: RandomLayout = makeHostLayout();
        simulateComputeThreads<RandomLayout>(
          randomKernel,
          hostLayout,
          randomFill,
          SAMPLE_COUNT,
          1,
          1,
          randomFill_HOST_RUNNABLE,
        );
        const expected: u8[] = hostBytes(hostLayout);
        // The comparison is byte for byte. Both lanes advance the same integer state, so this
        // example needs none of the tolerance the upstream page reports.
        let equal: boolean = expected.length === gpuBytes.length;
        let byteIndex: i32 = 0;
        while (equal && byteIndex < expected.length) {
          equal = expected[byteIndex] === gpuBytes[byteIndex];
          byteIndex += 1;
        }
        if (equal) state = "pass";
      }
    }
  }
  gpu.dispose();
  // One check line reports the outcome, so a reader who runs the example needs no golden.
  print(`check:prng ${state}`);
}
