// program: a05-timestamps
// purpose: prove opt-in compute pass timestamp resolution and never print timestamp values
// exercises: EG6
// questions: none

import {
  Buffer,
  ComputePipeline,
  TimestampPair,
  createBuffer,
  createTimestampPair,
  readOne,
} from "./typegpu";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUComputePipeline,
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
  if (!adapterResult.hasFeature("timestamp-query")) {
    adapterResult.dispose();
    gpu.dispose();
    print("timestamps:unsupported");
    return;
  }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice({
    requiredFeatures: ["timestamp-query"],
  });
  if (deviceResult === null) {
    adapterResult.dispose();
    gpu.dispose();
    print("timestamps:unsupported");
    return;
  }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const pairResult: TimestampPair | null = createTimestampPair(device);
    if (pairResult === null) {
      print("timestamps:unsupported");
      return;
    }
    using pair = pairResult;
    using shader = device.createShaderModule({
      label: "a05-shader",
      code: "@compute @workgroup_size(1) fn main() {}",
    });
    const nativePipeline: GPUComputePipeline = device.createComputePipeline({
      label: "a05-pipeline",
      layout: null,
      compute: { module: shader, entryPoint: "main" },
    });
    using pipeline = new ComputePipeline(nativePipeline, [1, 1, 1]);
    using readback: Buffer<FixedArray<u64, 2>> = createBuffer<FixedArray<u64, 2>>(
      device,
      16,
      1,
      GPUBufferUsage.COPY_DST + GPUBufferUsage.MAP_READ,
      "a05-readback",
    );
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchTimed(encoder, [], 1, 1, 1, pair);
    pair.resolve(encoder);
    pair.copyTo(encoder, readback);
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    const mapped: boolean = await readback.handle().mapAsync(GPUMapMode.READ, 0, 16);
    if (!mapped) {
      print("FAIL map");
      return;
    }
    const values: FixedArray<u64, 2> = Context.fromBytes<FixedArray<u64, 2>>(
      readOne<FixedArray<u64, 2>>(readback, 0),
      0,
    );
    readback.handle().unmap();
    print("timestamps:resolved");
  }
  gpu.dispose();
}
