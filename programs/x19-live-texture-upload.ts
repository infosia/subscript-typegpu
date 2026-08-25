// program: x19-live-texture-upload
// purpose: compare a typed texture upload and GPU sampling with the kernel host lane
// exercises: BF9, CL1, CL3, CL4, CL5, PI14, T4, T15, TX1, TX3, TX4, TX5, TX9, TX10
// questions: none

import {
  Buffer,
  bufferResource,
  ComputeInvocation,
  ComputePipelineSpec,
  computePipeline,
  createBindGroup,
  createBuffer,
  createComputePipeline,
  MutStorage,
  Sampler,
  samplerFromDescriptor,
  samplerResource,
  simulateComputeThreads,
  Texture2d,
  textureResource,
  writeTexturePixels,
} from "./typegpu";
import {
  Vec2f,
  Vec4f,
} from "./typegpu-types";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  GPUSamplerDescriptor,
  GPUTextureUsage,
} from "./webgpu";
import {
  uploadLive_ENTRY,
  uploadLive_HOST_RUNNABLE,
  uploadLive_LAYOUT0,
  uploadLive_WGSL,
  uploadLive_WORKGROUP_X,
  uploadLive_WORKGROUP_Y,
  uploadLive_WORKGROUP_Z,
} from "./x19-live-texture-upload.typegpu";

const WIDTH: u32 = 64;
const HEIGHT: u32 = 2;
const PIXEL_COUNT: u32 = WIDTH * HEIGHT;

class UploadLiveLayout {
  source!: Texture2d<f32>;
  nearest!: Sampler;
  output!: MutStorage<Vec4f>;
}

function uploadLiveKernel(res: UploadLiveLayout, ctx: ComputeInvocation): void {
  if (ctx.globalId.x >= WIDTH || ctx.globalId.y >= HEIGHT) return;
  const index: u32 = ctx.globalId.y * WIDTH + ctx.globalId.x;
  const uv = new Vec2f(
    ((ctx.globalId.x as f32) + 0.5) / (WIDTH as f32),
    ((ctx.globalId.y as f32) + 0.5) / (HEIGHT as f32),
  );
  res.output[index] = res.source.sampleLevel(res.nearest, uv, 0.0);
}

export const uploadLive: ComputePipelineSpec = computePipeline<UploadLiveLayout>(
  uploadLiveKernel,
  { name: "uploadLive", workgroupSize: [8, 1, 1] },
);

function gradientPixels(): Vec4f[] {
  const pixels: Vec4f[] = [];
  let y: u32 = 0;
  while (y < HEIGHT) {
    let x: u32 = 0;
    while (x < WIDTH) {
      pixels.push(new Vec4f(
        ((x * 4) as f32) / 255.0,
        (y as f32),
        128.0 / 255.0,
        1.0,
      ));
      x += 1;
    }
    y += 1;
  }
  return pixels;
}

function zeroPixels(): Vec4f[] {
  const pixels: Vec4f[] = [];
  let index: u32 = 0;
  while (index < PIXEL_COUNT) {
    pixels.push(new Vec4f(0.0, 0.0, 0.0, 0.0));
    index += 1;
  }
  return pixels;
}

function same(left: Vec4f, right: Vec4f): boolean {
  return left.x === right.x
    && left.y === right.y
    && left.z === right.z
    && left.w === right.w;
}

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
    const pixels: Vec4f[] = gradientPixels();
    using source = device.createTexture({
      label: "x19-source",
      size: { width: WIDTH, height: HEIGHT },
      format: "rgba8unorm",
      usage: GPUTextureUsage.TEXTURE_BINDING + GPUTextureUsage.COPY_DST,
    });
    writeTexturePixels(device.queue, source, pixels, WIDTH, HEIGHT);
    using sourceView = source.createView();
    const nearestDescriptor: GPUSamplerDescriptor = {
      minFilter: "nearest",
      magFilter: "nearest",
    };
    using nearest = device.createSampler(nearestDescriptor);
    using output: Buffer<Vec4f> = createBuffer<Vec4f>(
      device,
      16,
      PIXEL_COUNT,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC,
      "x19-output",
    );
    print("inputs:written");
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      uploadLive_WGSL,
      uploadLive_ENTRY,
      [uploadLive_LAYOUT0],
      [uploadLive_WORKGROUP_X, uploadLive_WORKGROUP_Y, uploadLive_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
    print("pipeline:created");
    using group = createBindGroup(
      device,
      pipeline.bindGroupLayout(0),
      uploadLive_LAYOUT0,
      [textureResource(sourceView), samplerResource(nearest), bufferResource(output.handle())],
    );
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [group], WIDTH, HEIGHT, 1);
    using command = encoder.finishDefault();
    device.queue.submit([command]);
    if (!await device.queue.onSubmittedWorkDone()) {
      print("FAIL submit");
      return;
    }
    print("dispatch:submitted");

    const host = new UploadLiveLayout();
    host.source = new Texture2d<f32>(pixels, WIDTH, HEIGHT);
    host.nearest = samplerFromDescriptor(nearestDescriptor);
    const hostOutput: Vec4f[] = zeroPixels();
    host.output = new MutStorage<Vec4f>(hostOutput);
    simulateComputeThreads<UploadLiveLayout>(
      uploadLiveKernel,
      host,
      uploadLive,
      WIDTH,
      HEIGHT,
      1,
      uploadLive_HOST_RUNNABLE,
    );
    const outputBytes: u8[] = await output.read(device, 0, PIXEL_COUNT);
    const gpuOutput: FixedArray<Vec4f, 128> = Context.fromBytes<FixedArray<Vec4f, 128>>(
      outputBytes,
      0,
    );
    print("readback:mapped");
    let index: u32 = 0;
    while (index < PIXEL_COUNT) {
      if (!same(gpuOutput[index as i32], hostOutput[index as i32])) {
        print(`FAIL index=${index}`);
        return;
      }
      index += 1;
    }
  }
  gpu.dispose();
  print("PASS");
}
