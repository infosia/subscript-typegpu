// program: b11-texture
// purpose: prove sampled, sampler, storage-texture, and second-group uniform bindings
// exercises: TX1, TX2, TX3, TX4, TX5, TX6, PI1, PI2, PI3, PI5, PI8, PI9, K14, K15, K16
// questions: none

import {
  ComputeInvocation,
  ComputePipelineSpec,
  Rgba8unorm,
  Sampler,
  StorageTexture2d,
  Texture2d,
  Uniform,
  bufferResource,
  computePipeline2,
  createBindGroup,
  createComputePipeline,
  samplerResource,
  textureResource,
} from "./typegpu";
import { Vec2f, Vec2i, Vec4f } from "./typegpu-types";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  GPUTextureUsage,
} from "./webgpu";
import {
  SampleParams_SIZE,
  texturePass_ENTRY,
  texturePass_LAYOUT0,
  texturePass_LAYOUT1,
  texturePass_WGSL,
  texturePass_WORKGROUP_X,
  texturePass_WORKGROUP_Y,
  texturePass_WORKGROUP_Z,
} from "./b11-texture.typegpu";

@CStruct
class SampleParams {
  width: u32;
  height: u32;

  constructor(width: u32, height: u32) {
    this.width = width;
    this.height = height;
  }
}

class TextureLayout {
  source!: Texture2d<f32>;
  linear!: Sampler;
  target!: StorageTexture2d<Rgba8unorm>;
}

class ParamsLayout {
  params!: Uniform<SampleParams>;
}

function textureKernel(
  textures: TextureLayout,
  settings: ParamsLayout,
  ctx: ComputeInvocation,
): void {
  const params: SampleParams = settings.params.get();
  if (ctx.globalId.x >= params.width || ctx.globalId.y >= params.height) return;
  const coords = new Vec2i(ctx.globalId.x as i32, ctx.globalId.y as i32);
  const loaded: Vec4f = textures.source.load(coords, 0);
  const uv = new Vec2f(
    ((ctx.globalId.x as f32) + 0.5) / (params.width as f32),
    ((ctx.globalId.y as f32) + 0.5) / (params.height as f32),
  );
  const sampled: Vec4f = textures.source.sampleLevel(textures.linear, uv, 0.0);
  textures.target.store(coords, loaded.add(sampled).scale(0.5));
}

export const texturePass: ComputePipelineSpec = computePipeline2<TextureLayout, ParamsLayout>(
  textureKernel,
  { workgroupSize: [4, 4, 1] },
);

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) { print("FAIL adapter"); gpu.dispose(); return; }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) { print("FAIL device"); adapterResult.dispose(); gpu.dispose(); return; }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    using source = device.createTexture({
      label: "b11-source",
      size: { width: 4, height: 4, depthOrArrayLayers: 1 },
      format: "rgba8unorm",
      usage: GPUTextureUsage.TEXTURE_BINDING + GPUTextureUsage.COPY_DST,
    });
    using target = device.createTexture({
      label: "b11-target",
      size: { width: 4, height: 4, depthOrArrayLayers: 1 },
      format: "rgba8unorm",
      usage: GPUTextureUsage.STORAGE_BINDING + GPUTextureUsage.COPY_SRC,
    });
    using sourceView = source.createView();
    using targetView = target.createView();
    using linear = device.createSampler({ minFilter: "linear", magFilter: "linear" });
    using params = device.createBuffer({
      label: "b11-params",
      size: SampleParams_SIZE as u64,
      usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
    });
    using pipeline = createComputePipeline(
      device,
      texturePass_WGSL,
      texturePass_ENTRY,
      [texturePass_LAYOUT0, texturePass_LAYOUT1],
      [texturePass_WORKGROUP_X, texturePass_WORKGROUP_Y, texturePass_WORKGROUP_Z],
    );
    using nativeLayout0 = pipeline.bindGroupLayout(0);
    using nativeLayout1 = pipeline.bindGroupLayout(1);
    using group0 = createBindGroup(device, nativeLayout0, texturePass_LAYOUT0, [
      textureResource(sourceView), samplerResource(linear), textureResource(targetView),
    ]);
    using group1 = createBindGroup(device, nativeLayout1, texturePass_LAYOUT1, [
      bufferResource(params),
    ]);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [group0, group1], 4, 4, 1);
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    print(`source.kind=${texturePass_LAYOUT0.entries[0].kind}`);
    print(`linear.kind=${texturePass_LAYOUT0.entries[1].kind}`);
    print(`target.kind=${texturePass_LAYOUT0.entries[2].kind}`);
    print(`params.kind=${texturePass_LAYOUT1.entries[0].kind}`);
    print(`texturePass_WGSL_LINES=${texturePass_WGSL.split("\n").length}`);
  }
  gpu.dispose();
  print("PASS");
}
