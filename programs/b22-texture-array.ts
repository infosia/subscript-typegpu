// program: b22-texture-array
// purpose: prove sampled, read-storage, and write-storage 2d-array bindings and host layers
// exercises: CL1, CL3, CL4, CL5, PI14, TX4, TX5, TX13, TX14
// questions: none

import {
  ComputeInvocation,
  ComputePipelineSpec,
  ReadStorageTexture2dArray,
  Rgba16float,
  Texture2dArray,
  WriteStorageTexture2dArray,
  computePipeline,
  createBindGroup,
  createComputePipeline,
  simulateComputeThreads,
  textureResource,
} from "./typegpu";
import { Vec2i, Vec4f } from "./typegpu-types";
import {
  gpu,
  GPUAdapter,
  GPUDevice,
  GPUTextureUsage,
} from "./webgpu";
import {
  textureArray_ENTRY,
  textureArray_HOST_RUNNABLE,
  textureArray_LAYOUT0,
  textureArray_WGSL,
  textureArray_WORKGROUP_X,
  textureArray_WORKGROUP_Y,
  textureArray_WORKGROUP_Z,
} from "./b22-texture-array.typegpu";

const WIDTH: u32 = 2;
const HEIGHT: u32 = 1;
const LAYERS: u32 = 2;

class TextureArrayLayout {
  sampled!: Texture2dArray<f32>;
  source!: ReadStorageTexture2dArray<Rgba16float>;
  target!: WriteStorageTexture2dArray<Rgba16float>;
}

function textureArrayKernel(res: TextureArrayLayout, ctx: ComputeInvocation): void {
  if (ctx.globalId.x >= WIDTH || ctx.globalId.y >= HEIGHT || ctx.globalId.z >= LAYERS) return;
  const coords = new Vec2i(ctx.globalId.x as i32, ctx.globalId.y as i32);
  const layer: i32 = ctx.globalId.z as i32;
  const sampled: Vec4f = res.sampled.load(coords, layer, 0);
  const stored: Vec4f = res.source.load(coords, layer);
  res.target.store(coords, layer, sampled.add(stored));
}

export const textureArray: ComputePipelineSpec = computePipeline<TextureArrayLayout>(
  textureArrayKernel,
  { name: "textureArray", workgroupSize: [2, 1, 1] },
);

function sampledPixels(): Vec4f[] {
  return [
    new Vec4f(1.0, 0.0, 0.0, 1.0),
    new Vec4f(2.0, 0.0, 0.0, 1.0),
    new Vec4f(3.0, 0.0, 0.0, 1.0),
    new Vec4f(4.0, 0.0, 0.0, 1.0),
  ];
}

function storagePixels(): Vec4f[] {
  return [
    new Vec4f(10.0, 0.0, 0.0, 0.0),
    new Vec4f(20.0, 0.0, 0.0, 0.0),
    new Vec4f(30.0, 0.0, 0.0, 0.0),
    new Vec4f(40.0, 0.0, 0.0, 0.0),
  ];
}

function zeroPixels(): Vec4f[] {
  return [
    new Vec4f(0.0, 0.0, 0.0, 0.0),
    new Vec4f(0.0, 0.0, 0.0, 0.0),
    new Vec4f(0.0, 0.0, 0.0, 0.0),
    new Vec4f(0.0, 0.0, 0.0, 0.0),
  ];
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
    using sampledTexture = device.createTexture({
      label: "b22-sampled",
      size: { width: WIDTH, height: HEIGHT, depthOrArrayLayers: LAYERS },
      format: "rgba16float",
      usage: GPUTextureUsage.TEXTURE_BINDING,
    });
    using sourceTexture = device.createTexture({
      label: "b22-source",
      size: { width: WIDTH, height: HEIGHT, depthOrArrayLayers: LAYERS },
      format: "rgba16float",
      usage: GPUTextureUsage.STORAGE_BINDING,
    });
    using targetTexture = device.createTexture({
      label: "b22-target",
      size: { width: WIDTH, height: HEIGHT, depthOrArrayLayers: LAYERS },
      format: "rgba16float",
      usage: GPUTextureUsage.STORAGE_BINDING,
    });
    using sampledView = sampledTexture.createView({
      dimension: "2d-array",
      mipLevelCount: 1,
      arrayLayerCount: LAYERS,
    });
    using sourceView = sourceTexture.createView({
      dimension: "2d-array",
      mipLevelCount: 1,
      arrayLayerCount: LAYERS,
    });
    using targetView = targetTexture.createView({
      dimension: "2d-array",
      mipLevelCount: 1,
      arrayLayerCount: LAYERS,
    });
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      textureArray_WGSL,
      textureArray_ENTRY,
      [textureArray_LAYOUT0],
      [textureArray_WORKGROUP_X, textureArray_WORKGROUP_Y, textureArray_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
    using group = createBindGroup(
      device,
      pipeline.bindGroupLayout(0),
      textureArray_LAYOUT0,
      [textureResource(sampledView), textureResource(sourceView), textureResource(targetView)],
    );
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [group], WIDTH, HEIGHT, LAYERS);
    using command = encoder.finishDefault();
    device.queue.submit([command]);

    const hostSampled: Vec4f[] = sampledPixels();
    const hostSource: Vec4f[] = storagePixels();
    const hostTarget: Vec4f[] = zeroPixels();
    const host = new TextureArrayLayout();
    host.sampled = new Texture2dArray<f32>(hostSampled, WIDTH, HEIGHT, LAYERS);
    host.source = new ReadStorageTexture2dArray<Rgba16float>(hostSource, WIDTH, HEIGHT, LAYERS);
    host.target = new WriteStorageTexture2dArray<Rgba16float>(hostTarget, WIDTH, HEIGHT, LAYERS);
    simulateComputeThreads<TextureArrayLayout>(
      textureArrayKernel,
      host,
      textureArray,
      WIDTH,
      HEIGHT,
      LAYERS,
      textureArray_HOST_RUNNABLE,
    );
    print(`sampled.kind=${textureArray_LAYOUT0.entries[0].kind}`);
    print(`source.kind=${textureArray_LAYOUT0.entries[1].kind}`);
    print(`source.access=${textureArray_LAYOUT0.entries[1].access}`);
    print(`target.kind=${textureArray_LAYOUT0.entries[2].kind}`);
    print(`target.access=${textureArray_LAYOUT0.entries[2].access}`);
    print(`textureArray_WGSL_LINES=${textureArray_WGSL.split("\n").length}`);
    print(`host:out=${hostTarget[0].x},${hostTarget[3].x}`);
  }
  gpu.dispose();
  print("PASS");
}
