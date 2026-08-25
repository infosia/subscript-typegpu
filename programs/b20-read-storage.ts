// program: b20-read-storage
// purpose: prove read-only and read-write storage textures emit their access modes
// exercises: CL1, CL3, CL4, CL5, PI14, TX1, TX4, TX5, TX11, TX12
// questions: none

import {
  ComputeInvocation,
  ComputePipelineSpec,
  R32float,
  ReadStorageTexture2d,
  ReadWriteStorageTexture2d,
  computePipeline,
  createBindGroup,
  createComputePipeline,
  simulateComputeThreads,
  textureResource,
} from "./typegpu";
import {
  Vec2i,
  Vec4f,
} from "./typegpu-types";
import {
  gpu,
  GPUAdapter,
  GPUDevice,
  GPUTextureUsage,
} from "./webgpu";
import {
  readStorage_ENTRY,
  readStorage_HOST_RUNNABLE,
  readStorage_LAYOUT0,
  readStorage_WGSL,
  readStorage_WORKGROUP_X,
  readStorage_WORKGROUP_Y,
  readStorage_WORKGROUP_Z,
} from "./b20-read-storage.typegpu";

const WIDTH: u32 = 2;
const HEIGHT: u32 = 2;

class ReadStorageLayout {
  source!: ReadStorageTexture2d<R32float>;
  target!: ReadWriteStorageTexture2d<R32float>;
}

function readStorageKernel(res: ReadStorageLayout, ctx: ComputeInvocation): void {
  const size = res.source.dimensions();
  if (ctx.globalId.x >= size.x || ctx.globalId.y >= size.y) return;
  const coords = new Vec2i(ctx.globalId.x as i32, ctx.globalId.y as i32);
  const source: Vec4f = res.source.load(coords);
  const target: Vec4f = res.target.load(coords);
  res.target.store(coords, source.add(target));
}

export const readStorage: ComputePipelineSpec = computePipeline<ReadStorageLayout>(
  readStorageKernel,
  { name: "readStorage", workgroupSize: [2, 2, 1] },
);

function sourcePixels(): Vec4f[] {
  return [
    new Vec4f(1.0, 0.0, 0.0, 1.0),
    new Vec4f(2.0, 0.0, 0.0, 1.0),
    new Vec4f(3.0, 0.0, 0.0, 1.0),
    new Vec4f(4.0, 0.0, 0.0, 1.0),
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
    using source = device.createTexture({
      label: "b20-source",
      size: { width: WIDTH, height: HEIGHT },
      format: "r32float",
      usage: GPUTextureUsage.STORAGE_BINDING,
    });
    using target = device.createTexture({
      label: "b20-target",
      size: { width: WIDTH, height: HEIGHT },
      format: "r32float",
      usage: GPUTextureUsage.STORAGE_BINDING,
    });
    using sourceView = source.createView();
    using targetView = target.createView();
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      readStorage_WGSL,
      readStorage_ENTRY,
      [readStorage_LAYOUT0],
      [readStorage_WORKGROUP_X, readStorage_WORKGROUP_Y, readStorage_WORKGROUP_Z],
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
      readStorage_LAYOUT0,
      [textureResource(sourceView), textureResource(targetView)],
    );
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [group], WIDTH, HEIGHT, 1);
    using command = encoder.finishDefault();
    device.queue.submit([command]);

    const hostSource: Vec4f[] = sourcePixels();
    const hostTarget: Vec4f[] = zeroPixels();
    const host = new ReadStorageLayout();
    host.source = new ReadStorageTexture2d<R32float>(hostSource, WIDTH, HEIGHT);
    host.target = new ReadWriteStorageTexture2d<R32float>(hostTarget, WIDTH, HEIGHT);
    simulateComputeThreads<ReadStorageLayout>(
      readStorageKernel,
      host,
      readStorage,
      WIDTH,
      HEIGHT,
      1,
      readStorage_HOST_RUNNABLE,
    );
    print(`source.kind=${readStorage_LAYOUT0.entries[0].kind}`);
    print(`source.access=${readStorage_LAYOUT0.entries[0].access}`);
    print(`target.kind=${readStorage_LAYOUT0.entries[1].kind}`);
    print(`target.access=${readStorage_LAYOUT0.entries[1].access}`);
    print(`readStorage_WGSL_LINES=${readStorage_WGSL.split("\n").length}`);
    print(`host:out=${hostTarget[0].x},${hostTarget[3].x}`);
  }
  gpu.dispose();
  print("PASS");
}
