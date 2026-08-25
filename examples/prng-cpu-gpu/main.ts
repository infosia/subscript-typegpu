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

const SAMPLE_COUNT: u32 = 64;
const ROUND_COUNT: u32 = 5;

@CStruct
class RandomValue {
  state: u32;
  value: f32;

  constructor(state: u32, value: f32) {
    this.state = state;
    this.value = value;
  }
}

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

export const randomFill: ComputePipelineSpec = computePipeline<RandomLayout>(randomKernel, {
  name: "randomFill",
  workgroupSize: [32, 1, 1],
});

function makeHostLayout(): RandomLayout {
  const values: RandomValue[] = [];
  for (let index: u32 = 0; index < SAMPLE_COUNT; index += 1) {
    values.push(new RandomValue(0, 0.0));
  }
  const layout = new RandomLayout();
  layout.output = new MutStorage<RandomValue>(values);
  return layout;
}

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
    using adapter = adapterResult;
    using device = deviceResult;
    using output: Buffer<RandomValue> = createBuffer<RandomValue>(
      device,
      RandomValue_STRIDE,
      SAMPLE_COUNT,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC,
      "prng-output",
    );
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
      using nativeLayout = pipeline.bindGroupLayout(0);
      using group = createBindGroup(
        device,
        nativeLayout,
        randomFill_LAYOUT0,
        [bufferResource(output.handle())],
      );
      using encoder = device.createCommandEncoderDefault();
      pipeline.dispatchThreads(encoder, [group], SAMPLE_COUNT, 1, 1);
      using command = encoder.finishDefault();
      device.queue().submit([command]);

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
  print(`check:prng ${state}`);
}
