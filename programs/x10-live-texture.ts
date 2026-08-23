// program: x10-live-texture
// purpose: compare nearest compute sampling and storage-texture writes texel by texel
// exercises: TX1, TX2, TX3, TX4, TX5, TX7, PI1, PI2, PI3, PI5, PI8, PI9, K14, K15, K16, T4, T15
// questions: none

import {
  ComputeInvocation,
  ComputePipelineSpec,
  Rgba8unorm,
  Sampler,
  StorageTexture2d,
  Texture2d,
  computePipeline,
  createBindGroup,
  createComputePipeline,
  samplerResource,
  samplerFromDescriptor,
  textureResource,
} from "./typegpu";
import { Vec2f, Vec2i, Vec4f } from "./typegpu-types";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  GPUMapMode,
  GPUSamplerDescriptor,
  GPUTextureUsage,
} from "./webgpu";
import {
  textureCopy_ENTRY,
  textureCopy_LAYOUT0,
  textureCopy_WGSL,
  textureCopy_WORKGROUP_X,
  textureCopy_WORKGROUP_Y,
  textureCopy_WORKGROUP_Z,
} from "./x10-live-texture.typegpu";

class TextureCopyLayout {
  source!: Texture2d<f32>;
  nearest!: Sampler;
  target!: StorageTexture2d<Rgba8unorm>;
}

function textureCopyKernel(res: TextureCopyLayout, ctx: ComputeInvocation): void {
  if (ctx.globalId.x >= 4 || ctx.globalId.y >= 4) return;
  const uv = new Vec2f(
    ((ctx.globalId.x as f32) + 0.25) / 4.0,
    ((ctx.globalId.y as f32) + 0.25) / 4.0,
  );
  const color: Vec4f = res.source.sampleLevel(res.nearest, uv, 0.0);
  res.target.store(new Vec2i(ctx.globalId.x as i32, ctx.globalId.y as i32), color);
}

export const textureCopy: ComputePipelineSpec = computePipeline<TextureCopyLayout>(
  textureCopyKernel,
  { name: "textureCopy", workgroupSize: [4, 4, 1] },
);

function checkerBytes(): u8[] {
  const values: u8[] = [];
  let y: i32 = 0;
  while (y < 4) {
    let x: i32 = 0;
    while (x < 4) {
      const value: u8 = ((x + y) % 2 === 0) ? 255 : 0;
      values.push(value);
      values.push(value);
      values.push(value);
      values.push(255);
      x = x + 1;
    }
    y = y + 1;
  }
  return values;
}

function checkerPixels(): Vec4f[] {
  const pixels: Vec4f[] = [];
  let y: i32 = 0;
  while (y < 4) {
    let x: i32 = 0;
    while (x < 4) {
      const value: f32 = ((x + y) % 2 === 0) ? 1.0 : 0.0;
      pixels.push(new Vec4f(value, value, value, 1.0));
      x = x + 1;
    }
    y = y + 1;
  }
  return pixels;
}

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) { print("FAIL adapter"); gpu.dispose(); return; }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) { print("FAIL device"); adapterResult.dispose(); gpu.dispose(); return; }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    using source = device.createTexture({
      label: "x10-source",
      size: { width: 4, height: 4, depthOrArrayLayers: 1 },
      format: "rgba8unorm",
      usage: GPUTextureUsage.TEXTURE_BINDING + GPUTextureUsage.COPY_DST,
    });
    using target = device.createTexture({
      label: "x10-target",
      size: { width: 4, height: 4, depthOrArrayLayers: 1 },
      format: "rgba8unorm",
      usage: GPUTextureUsage.STORAGE_BINDING + GPUTextureUsage.COPY_SRC,
    });
    using sourceView = source.createView();
    using targetView = target.createView();
    const nearestDescriptor: GPUSamplerDescriptor = {
      minFilter: "nearest",
      magFilter: "nearest",
    };
    using nearest = device.createSampler(nearestDescriptor);
    using readback = device.createBuffer({
      label: "x10-readback",
      size: 1024,
      usage: GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
    });
    device.queue().writeTexture(
      { texture: source },
      checkerBytes(),
      { offset: 0, bytesPerRow: 16, rowsPerImage: 4 },
      { width: 4, height: 4, depthOrArrayLayers: 1 },
    );
    print("inputs:written");
    using pipeline = createComputePipeline(
      device,
      textureCopy_WGSL,
      textureCopy_ENTRY,
      [textureCopy_LAYOUT0],
      [textureCopy_WORKGROUP_X, textureCopy_WORKGROUP_Y, textureCopy_WORKGROUP_Z],
    );
    print("pipeline:created");
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(device, nativeLayout, textureCopy_LAYOUT0, [
      textureResource(sourceView), samplerResource(nearest), textureResource(targetView),
    ]);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], 4, 4, 1);
    encoder.copyTextureToBuffer(
      { texture: target },
      { buffer: readback, offset: 0, bytesPerRow: 256, rowsPerImage: 4 },
      { width: 4, height: 4, depthOrArrayLayers: 1 },
    );
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    if (!await device.queue().onSubmittedWorkDone()) { print("FAIL submit"); return; }
    print("dispatch:submitted");
    if (!await readback.mapAsync(GPUMapMode.READ, 0, 1024)) { print("FAIL map"); return; }
    const pixels: u8[] = readback.readMappedRange(0, 1024);
    print("readback:mapped");
    const hostTexture = new Texture2d<f32>(checkerPixels(), 4, 4);
    const hostSampler = samplerFromDescriptor(nearestDescriptor);
    let y: i32 = 0;
    while (y < 4) {
      let x: i32 = 0;
      while (x < 4) {
        const expected: Vec4f = hostTexture.sample(
          hostSampler,
          new Vec2f(((x as f32) + 0.25) / 4.0, ((y as f32) + 0.25) / 4.0),
        );
        const offset: i32 = y * 256 + x * 4;
        const expectedR: u8 = (expected.x * 255.0) as u8;
        const expectedG: u8 = (expected.y * 255.0) as u8;
        const expectedB: u8 = (expected.z * 255.0) as u8;
        const expectedA: u8 = (expected.w * 255.0) as u8;
        if (pixels[offset] !== expectedR || pixels[offset + 1] !== expectedG
          || pixels[offset + 2] !== expectedB || pixels[offset + 3] !== expectedA) {
          print(`FAIL x=${x} y=${y} expected=${expectedR},${expectedG},${expectedB},${expectedA} got=${pixels[offset]},${pixels[offset + 1]},${pixels[offset + 2]},${pixels[offset + 3]}`);
          return;
        }
        x = x + 1;
      }
      y = y + 1;
    }
    readback.unmap();
  }
  gpu.dispose();
  print("PASS");
}
