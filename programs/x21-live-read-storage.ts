// program: x21-live-read-storage
// purpose: compare two storage-texture blur dispatches with the host simulation
// exercises: CL1, CL3, CL4, CL5, PI14, T4, T15, TX1, TX4, TX5, TX9, TX11, TX12
// questions: none

import {
  ComputeInvocation,
  ComputePipelineSpec,
  R32float,
  ReadStorageTexture2d,
  StorageTexture2d,
  computePipeline,
  createBindGroup,
  createComputePipeline,
  simulateComputeThreads,
  textureResource,
  writeTexturePixels,
} from "./typegpu";
import {
  Vec2i,
  Vec4f,
} from "./typegpu-types";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  GPUMapMode,
  GPUTextureUsage,
} from "./webgpu";
import {
  blurPass_ENTRY,
  blurPass_HOST_RUNNABLE,
  blurPass_LAYOUT0,
  blurPass_WGSL,
  blurPass_WORKGROUP_X,
  blurPass_WORKGROUP_Y,
  blurPass_WORKGROUP_Z,
} from "./x21-live-read-storage.typegpu";

const WIDTH: u32 = 4;
const HEIGHT: u32 = 4;
const PIXEL_COUNT: u32 = WIDTH * HEIGHT;

class BlurLayout {
  source!: ReadStorageTexture2d<R32float>;
  target!: StorageTexture2d<R32float>;
}

function blurKernel(res: BlurLayout, ctx: ComputeInvocation): void {
  const size = res.source.dimensions();
  if (ctx.globalId.x >= size.x || ctx.globalId.y >= size.y) return;
  const x: i32 = ctx.globalId.x as i32;
  const y: i32 = ctx.globalId.y as i32;
  const width: i32 = size.x as i32;
  const height: i32 = size.y as i32;
  const left: i32 = x > 0 ? x - 1 : x;
  const right: i32 = x + 1 < width ? x + 1 : x;
  const down: i32 = y > 0 ? y - 1 : y;
  const up: i32 = y + 1 < height ? y + 1 : y;
  const value: f32 = (
    res.source.load(new Vec2i(x, y)).x
    + res.source.load(new Vec2i(left, y)).x
    + res.source.load(new Vec2i(right, y)).x
    + res.source.load(new Vec2i(x, down)).x
    + res.source.load(new Vec2i(x, up)).x
  ) * 0.2;
  res.target.store(new Vec2i(x, y), new Vec4f(value, 0.0, 0.0, 1.0));
}

export const blurPass: ComputePipelineSpec = computePipeline<BlurLayout>(
  blurKernel,
  { name: "blurPass", workgroupSize: [4, 4, 1] },
);

function sourcePixels(): Vec4f[] {
  const pixels: Vec4f[] = [];
  let index: u32 = 0;
  while (index < PIXEL_COUNT) {
    const value: f32 = ((index * 7 + 3) as f32) / 127.0;
    pixels.push(new Vec4f(value, 0.0, 0.0, 1.0));
    index += 1;
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

function close(actual: f32, expected: f32): boolean {
  const difference: f32 = Math.abs((actual - expected) as f64) as f32;
  const scale: f32 = Math.abs(expected as f64) as f32;
  return difference <= 0.000001 + scale * 0.000001;
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
    const initial: Vec4f[] = sourcePixels();
    using textureA = device.createTexture({
      label: "x21-a",
      size: { width: WIDTH, height: HEIGHT },
      format: "r32float",
      usage: GPUTextureUsage.STORAGE_BINDING
        + GPUTextureUsage.COPY_DST
        + GPUTextureUsage.COPY_SRC,
    });
    using textureB = device.createTexture({
      label: "x21-b",
      size: { width: WIDTH, height: HEIGHT },
      format: "r32float",
      usage: GPUTextureUsage.STORAGE_BINDING,
    });
    writeTexturePixels(device.queue, textureA, initial, WIDTH, HEIGHT);
    using viewA = textureA.createView();
    using viewB = textureB.createView();
    using readback = device.createBuffer({
      label: "x21-readback",
      size: 1024,
      usage: GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
    });
    print("inputs:written");
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      blurPass_WGSL,
      blurPass_ENTRY,
      [blurPass_LAYOUT0],
      [blurPass_WORKGROUP_X, blurPass_WORKGROUP_Y, blurPass_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
    print("pipeline:created");
    using groupAB = createBindGroup(
      device,
      pipeline.bindGroupLayout(0),
      blurPass_LAYOUT0,
      [textureResource(viewA), textureResource(viewB)],
    );
    using groupBA = createBindGroup(
      device,
      pipeline.bindGroupLayout(0),
      blurPass_LAYOUT0,
      [textureResource(viewB), textureResource(viewA)],
    );
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [groupAB], WIDTH, HEIGHT, 1);
    pipeline.dispatchThreads(encoder, [groupBA], WIDTH, HEIGHT, 1);
    encoder.copyTextureToBuffer(
      { texture: textureA },
      { buffer: readback, bytesPerRow: 256, rowsPerImage: HEIGHT },
      { width: WIDTH, height: HEIGHT, depthOrArrayLayers: 1 },
    );
    using command = encoder.finishDefault();
    device.queue.submit([command]);
    if (!await device.queue.onSubmittedWorkDone()) {
      print("FAIL submit");
      return;
    }
    print("dispatch:submitted");

    const hostA: Vec4f[] = sourcePixels();
    const hostB: Vec4f[] = zeroPixels();
    const hostAB = new BlurLayout();
    hostAB.source = new ReadStorageTexture2d<R32float>(hostA, WIDTH, HEIGHT);
    hostAB.target = new StorageTexture2d<R32float>(hostB, WIDTH, HEIGHT);
    simulateComputeThreads<BlurLayout>(
      blurKernel,
      hostAB,
      blurPass,
      WIDTH,
      HEIGHT,
      1,
      blurPass_HOST_RUNNABLE,
    );
    const hostBA = new BlurLayout();
    hostBA.source = new ReadStorageTexture2d<R32float>(hostB, WIDTH, HEIGHT);
    hostBA.target = new StorageTexture2d<R32float>(hostA, WIDTH, HEIGHT);
    simulateComputeThreads<BlurLayout>(
      blurKernel,
      hostBA,
      blurPass,
      WIDTH,
      HEIGHT,
      1,
      blurPass_HOST_RUNNABLE,
    );

    if (!await readback.mapAsync(GPUMapMode.READ, 0, 1024)) {
      print("FAIL map");
      return;
    }
    const bytes: u8[] = readback.readMappedRange(0, 1024);
    print("readback:mapped");
    let y: u32 = 0;
    while (y < HEIGHT) {
      let x: u32 = 0;
      while (x < WIDTH) {
        const actual: FixedArray<f32, 1> = Context.fromBytes<FixedArray<f32, 1>>(
          bytes,
          y * 256 + x * 4,
        );
        const expected: f32 = hostA[(y * WIDTH + x) as i32].x;
        if (!close(actual[0], expected)) {
          print(`FAIL x=${x} y=${y} expected=${expected} actual=${actual[0]}`);
          return;
        }
        x += 1;
      }
      y += 1;
    }
    readback.unmap();
  }
  gpu.dispose();
  print("PASS");
}
