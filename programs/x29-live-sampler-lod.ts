// program: x29-live-sampler-lod
// purpose: compare nearest samples with the default and clamped LOD ranges
// exercises: CL5
// questions: none

import {
  Buffer,
  ComputeInvocation,
  ComputePipelineSpec,
  MutStorage,
  Sampler,
  Texture2d,
  bufferResource,
  computePipeline,
  createBindGroup,
  createBuffer,
  createComputePipeline,
  samplerFromDescriptor,
  samplerResource,
  simulateCompute,
  textureResource,
  writeTexturePixels,
} from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice, GPUSamplerDescriptor, GPUTextureUsage } from "./webgpu";
import {
  samplerLod_ENTRY,
  samplerLod_HOST_RUNNABLE,
  samplerLod_LAYOUT0,
  samplerLod_WGSL,
  samplerLod_WORKGROUP_X,
  samplerLod_WORKGROUP_Y,
  samplerLod_WORKGROUP_Z,
} from "./x29-live-sampler-lod.typegpu";

class SamplerLodLayout {
  constructor(
    public source: Texture2d<f32>,
    public nearest: Sampler,
    public clamped: Sampler,
    public output: MutStorage<Vec4f>,
  ) {}
}

function sampleKernel(res: SamplerLodLayout, ctx: ComputeInvocation): void {
  const uv = new Vec2f(0.75, 0.5);
  res.output[0] = res.source.sampleLevel(res.nearest, uv, 0.0);
  res.output[1] = res.source.sampleLevel(res.clamped, uv, 0.0);
}

const samplerLod: ComputePipelineSpec = computePipeline(sampleKernel, {
  name: "samplerLod", workgroupSize: [1, 1, 1],
});

function rgba(color: Vec4f): string {
  return `${color.x},${color.y},${color.z},${color.w}`;
}

function same(a: Vec4f, b: Vec4f): boolean {
  return a.x == b.x && a.y == b.y && a.z == b.z && a.w == b.w;
}

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult == null) { print("FAIL adapter"); gpu.dispose(); return; }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult == null) { print("FAIL device"); adapterResult.dispose(); gpu.dispose(); return; }
  let passed: boolean = true;
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const defaultDescriptor: GPUSamplerDescriptor = {};
    const clampedDescriptor: GPUSamplerDescriptor = { lodMinClamp: 1, lodMaxClamp: 1 };
    device.pushErrorScope("validation");
    using nearest = device.createSampler(defaultDescriptor);
    using clamped = device.createSampler(clampedDescriptor);
    using pipeline = createComputePipeline(
      device, samplerLod_WGSL, samplerLod_ENTRY, [samplerLod_LAYOUT0],
      [samplerLod_WORKGROUP_X, samplerLod_WORKGROUP_Y, samplerLod_WORKGROUP_Z],
    );
    using layout = pipeline.bindGroupLayout(0);
    const red = new Vec4f(1, 0, 0, 1);
    const cyan = new Vec4f(0, 1, 1, 1);
    for (let levels: u32 = 1; levels <= 2; levels += 1) {
      using source = device.createTexture({
        label: "x29-source", size: { width: 2, height: 1 }, mipLevelCount: levels,
        format: "rgba8unorm", usage: GPUTextureUsage.TEXTURE_BINDING + GPUTextureUsage.COPY_DST,
      });
      const pixels: Vec4f[] = [red, levels == 1 ? cyan : red];
      writeTexturePixels(device.queue, source, pixels, 2, 1);
      if (levels == 2) {
        const blue: u8[] = [0, 0, 255, 255];
        device.queue.writeTexture(
          { texture: source, mipLevel: 1 }, blue,
          { bytesPerRow: 4, rowsPerImage: 1 }, { width: 1, height: 1 },
        );
      }
      using view = source.createView({ mipLevelCount: levels, arrayLayerCount: 1 });
      using output: Buffer<Vec4f> = createBuffer(
        device, 16, 2, GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC, "x29-output",
      );
      using group = createBindGroup(device, layout, samplerLod_LAYOUT0, [
        textureResource(view), samplerResource(nearest), samplerResource(clamped), bufferResource(output.handle()),
      ]);
      using encoder = device.createCommandEncoderDefault();
      pipeline.dispatch(encoder, [group], 1, 1, 1);
      using command = encoder.finishDefault();
      device.queue.submit([command]);
      const bytes: u8[] = await output.read(device, 0, 2);
      const samples: FixedArray<Vec4f, 2> = Context.fromBytes<FixedArray<Vec4f, 2>>(bytes, 0);
      if (levels == 1) {
        const hostSamples: Vec4f[] = [new Vec4f(0, 0, 0, 0), new Vec4f(0, 0, 0, 0)];
        const host = new SamplerLodLayout(
          new Texture2d<f32>(pixels, 2, 1),
          samplerFromDescriptor(defaultDescriptor),
          samplerFromDescriptor(clampedDescriptor),
          new MutStorage(hostSamples),
        );
        simulateCompute(sampleKernel, host, samplerLod, [1, 1, 1], samplerLod_HOST_RUNNABLE);
        print(`A default host=${rgba(hostSamples[0])} device=${rgba(samples[0])}`);
        print(`A clamped host=${rgba(hostSamples[1])} device=${rgba(samples[1])}`);
        passed = same(samples[0], hostSamples[0]) && same(samples[1], hostSamples[1]);
      } else {
        print(`B default device=${rgba(samples[0])} clamped device=${rgba(samples[1])}`);
        passed = passed && same(samples[0], red) && same(samples[1], new Vec4f(0, 0, 1, 1));
      }
    }
    const error = await device.popErrorScope();
    if (error != null) {
      print(`validation: ${error.message.split("\n")[0]}`);
      passed = false;
    }
  }
  gpu.dispose();
  print(passed ? "PASS" : "FAIL sampler LOD");
}
