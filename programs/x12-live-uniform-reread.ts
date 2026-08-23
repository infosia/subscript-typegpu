// program: x12-live-uniform-reread
// purpose: prove a local shadow cannot redirect a later uniform binding read
// exercises: K14, K15, K16, PI5, PI6, PI8, PI9, PI12, T4, T15
// questions: none

import {
  Buffer,
  ComputeInvocation,
  ComputePipelineSpec,
  MutStorage,
  Uniform,
  bufferResource,
  computePipeline,
  createBindGroup,
  createBuffer,
  createComputePipeline,
  readBuffer,
} from "./typegpu";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice, GPUMapMode } from "./webgpu";
import {
  Params_STRIDE,
  ShadowResult_STRIDE,
  uniformReread_ENTRY,
  uniformReread_LAYOUT0,
  uniformReread_WGSL,
  uniformReread_WORKGROUP_X,
  uniformReread_WORKGROUP_Y,
  uniformReread_WORKGROUP_Z,
} from "./x12-live-uniform-reread.typegpu";

@CStruct
class Params {
  value: u32;

  constructor(value: u32) {
    this.value = value;
  }
}

@CStruct
class ShadowResult {
  local: u32;
  reread: u32;

  constructor(local: u32, reread: u32) {
    this.local = local;
    this.reread = reread;
  }
}

class ShadowLayout {
  params!: Uniform<Params>;
  output!: MutStorage<ShadowResult>;
}

function shadowKernel(res: ShadowLayout, ctx: ComputeInvocation): void {
  let params: Params = res.params.get();
  params.value = params.value + 7;
  const reread: Params = res.params.get();
  res.output[0] = new ShadowResult(params.value, reread.value);
}

export const uniformReread: ComputePipelineSpec = computePipeline<ShadowLayout>(shadowKernel, {
  name: "uniformReread",
  workgroupSize: [1, 1, 1],
});

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) { print("FAIL adapter"); gpu.dispose(); return; }
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
    using params: Buffer<Params> = createBuffer<Params>(
      device, Params_STRIDE, 1,
      GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
      "x12-params",
    );
    using output: Buffer<ShadowResult> = createBuffer<ShadowResult>(
      device, ShadowResult_STRIDE, 1,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC,
      "x12-output",
    );
    using readback: Buffer<ShadowResult> = createBuffer<ShadowResult>(
      device, ShadowResult_STRIDE, 1,
      GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
      "x12-readback",
    );
    const queue = device.queue();
    params.writeOne(queue, 0, Context.bytesOf<Params>(new Params(5)));
    print("inputs:written");
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      uniformReread_WGSL,
      uniformReread_ENTRY,
      [uniformReread_LAYOUT0],
      [uniformReread_WORKGROUP_X, uniformReread_WORKGROUP_Y, uniformReread_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(device, nativeLayout, uniformReread_LAYOUT0, [
      bufferResource(params.handle()),
      bufferResource(output.handle()),
    ]);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatch(encoder, [bindGroup], 1, 1, 1);
    output.copyTo(encoder, readback, 0, 1);
    using command = encoder.finishDefault();
    queue.submit([command]);
    print("dispatch:submitted");
    if (!await readback.handle().mapAsync(GPUMapMode.READ, 0, ShadowResult_STRIDE as u64)) {
      print("FAIL map");
      return;
    }
    const result: FixedArray<ShadowResult, 1> = Context.fromBytes<FixedArray<ShadowResult, 1>>(
      readBuffer<ShadowResult>(readback, 0, 1),
      0,
    );
    print("readback:mapped");
    if (result[0].local !== 12 || result[0].reread !== 5) {
      print(`FAIL local=${result[0].local} reread=${result[0].reread}`);
      return;
    }
    readback.handle().unmap();
  }
  gpu.dispose();
  print("PASS");
}
