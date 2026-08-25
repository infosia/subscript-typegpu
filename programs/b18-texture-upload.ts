// program: b18-texture-upload
// purpose: prove typed and raw texture uploads share the fixed rgba8unorm encoding
// exercises: CL1, CL3, CL4, CL5, PI14, TX1, TX3, TX4, TX5, TX9, TX10
// questions: none

import {
  bufferResource,
  ComputeInvocation,
  ComputePipelineSpec,
  computePipeline,
  createBindGroup,
  createComputePipeline,
  MutStorage,
  Sampler,
  samplerFromDescriptor,
  samplerResource,
  simulateComputeThreads,
  Texture2d,
  textureResource,
  writeTextureBytes,
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
  uploadPass_ENTRY,
  uploadPass_HOST_RUNNABLE,
  uploadPass_LAYOUT0,
  uploadPass_WGSL,
  uploadPass_WORKGROUP_X,
  uploadPass_WORKGROUP_Y,
  uploadPass_WORKGROUP_Z,
} from "./b18-texture-upload.typegpu";

const WIDTH: u32 = 64;
const HEIGHT: u32 = 2;
const PIXEL_COUNT: u32 = WIDTH * HEIGHT;

class UploadLayout {
  source!: Texture2d<f32>;
  nearest!: Sampler;
  output!: MutStorage<Vec4f>;
}

function uploadKernel(res: UploadLayout, ctx: ComputeInvocation): void {
  if (ctx.globalId.x >= WIDTH || ctx.globalId.y >= HEIGHT) return;
  const index: u32 = ctx.globalId.y * WIDTH + ctx.globalId.x;
  const uv = new Vec2f(
    ((ctx.globalId.x as f32) + 0.5) / (WIDTH as f32),
    ((ctx.globalId.y as f32) + 0.5) / (HEIGHT as f32),
  );
  res.output[index] = res.source.sampleLevel(res.nearest, uv, 0.0);
}

export const uploadPass: ComputePipelineSpec = computePipeline<UploadLayout>(
  uploadKernel,
  { name: "uploadPass", workgroupSize: [8, 1, 1] },
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

function gradientBytes(): u8[] {
  const bytes: u8[] = [];
  let y: u32 = 0;
  while (y < HEIGHT) {
    let x: u32 = 0;
    while (x < WIDTH) {
      bytes.push((x * 4) as u8);
      bytes.push((y * 255) as u8);
      bytes.push(128);
      bytes.push(255);
      x += 1;
    }
    y += 1;
  }
  return bytes;
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
    const pixels: Vec4f[] = gradientPixels();
    const bytes: u8[] = gradientBytes();
    using typedTexture = device.createTexture({
      label: "b18-typed-upload",
      size: { width: WIDTH, height: HEIGHT },
      format: "rgba8unorm",
      usage: GPUTextureUsage.TEXTURE_BINDING + GPUTextureUsage.COPY_DST,
    });
    using rawTexture = device.createTexture({
      label: "b18-raw-upload",
      size: { width: WIDTH, height: HEIGHT },
      format: "rgba8unorm",
      usage: GPUTextureUsage.COPY_DST,
    });
    writeTexturePixels(device.queue(), typedTexture, pixels, WIDTH, HEIGHT);
    writeTextureBytes(device.queue(), rawTexture, bytes, 256, WIDTH, HEIGHT);
    using sourceView = typedTexture.createView();
    const nearestDescriptor: GPUSamplerDescriptor = {
      minFilter: "nearest",
      magFilter: "nearest",
    };
    using nearest = device.createSampler(nearestDescriptor);
    using output = device.createBuffer({
      label: "b18-output",
      size: (PIXEL_COUNT * 16) as u64,
      usage: GPUBufferUsage.STORAGE,
    });
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      uploadPass_WGSL,
      uploadPass_ENTRY,
      [uploadPass_LAYOUT0],
      [uploadPass_WORKGROUP_X, uploadPass_WORKGROUP_Y, uploadPass_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print("pipeline:invalid");
      print("FAIL");
      return;
    }
    using group = createBindGroup(
      device,
      pipeline.bindGroupLayout(0),
      uploadPass_LAYOUT0,
      [textureResource(sourceView), samplerResource(nearest), bufferResource(output)],
    );
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [group], WIDTH, HEIGHT, 1);
    using command = encoder.finishDefault();
    device.queue().submit([command]);

    const host = new UploadLayout();
    host.source = new Texture2d<f32>(pixels, WIDTH, HEIGHT);
    host.nearest = samplerFromDescriptor(nearestDescriptor);
    const hostOutput: Vec4f[] = zeroPixels();
    host.output = new MutStorage<Vec4f>(hostOutput);
    simulateComputeThreads<UploadLayout>(
      uploadKernel,
      host,
      uploadPass,
      WIDTH,
      HEIGHT,
      1,
      uploadPass_HOST_RUNNABLE,
    );
    print(`encoded:first=${bytes[0]},${bytes[1]},${bytes[2]},${bytes[3]}`);
    print(`encoded:last=${bytes[508]},${bytes[509]},${bytes[510]},${bytes[511]}`);
    print(`uploadPass_WGSL_LINES=${uploadPass_WGSL.split("\n").length}`);
    print(`host:out=${(hostOutput[0].x * 255.0) as u32},${(hostOutput[127].x * 255.0) as u32}`);
  }
  gpu.dispose();
  print("PASS");
}
