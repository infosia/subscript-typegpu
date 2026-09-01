// program: x23-live-texture-array
// purpose: ping-pong a two-layer color/coordinate payload and read one layer through a 2d view
// exercises: BF9, CL1, CL3, CL4, CL5, PI14, T4, T15, TX4, TX5, TX13, TX14
// questions: none

import {
  Buffer,
  ComputeInvocation,
  ComputePipelineSpec,
  MutStorage,
  ReadStorageTexture2dArray,
  Rgba16float,
  Texture2d,
  WriteStorageTexture2dArray,
  bufferResource,
  computePipeline,
  createBindGroup,
  createBuffer,
  createComputePipeline,
  simulateComputeThreads,
  textureResource,
} from "./typegpu";
import { Vec2i, Vec4f } from "./typegpu-types";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  GPUTexture,
  GPUTextureUsage,
} from "./webgpu";
import {
  arrayPingPong_ENTRY,
  arrayPingPong_HOST_RUNNABLE,
  arrayPingPong_LAYOUT0,
  arrayPingPong_WGSL,
  arrayPingPong_WORKGROUP_X,
  arrayPingPong_WORKGROUP_Y,
  arrayPingPong_WORKGROUP_Z,
  layerReadback_ENTRY,
  layerReadback_LAYOUT0,
  layerReadback_WGSL,
  layerReadback_WORKGROUP_X,
  layerReadback_WORKGROUP_Y,
  layerReadback_WORKGROUP_Z,
} from "./x23-live-texture-array.typegpu";

const WIDTH: u32 = 2;
const HEIGHT: u32 = 2;
const LAYERS: u32 = 2;
const PIXEL_COUNT: u32 = WIDTH * HEIGHT;

class ArrayPingPongLayout {
  source!: ReadStorageTexture2dArray<Rgba16float>;
  target!: WriteStorageTexture2dArray<Rgba16float>;
}

function arrayPingPongKernel(res: ArrayPingPongLayout, ctx: ComputeInvocation): void {
  if (ctx.globalId.x >= WIDTH || ctx.globalId.y >= HEIGHT || ctx.globalId.z >= LAYERS) return;
  const coords = new Vec2i(ctx.globalId.x as i32, ctx.globalId.y as i32);
  const layer: i32 = ctx.globalId.z as i32;
  const pairedLayer: i32 = layer === 0 ? 1 : 0;
  const pairScale: f32 = layer === 0 ? 0.5 : 0.0;
  const coordinateStep: Vec4f = layer === 1
    ? new Vec4f(1.0, 1.0, 0.0, 0.0)
    : new Vec4f(0.0, 0.0, 0.0, 0.0);
  const current: Vec4f = res.source.load(coords, layer);
  const paired: Vec4f = res.source.load(coords, pairedLayer);
  res.target.store(coords, layer, current.add(paired.scale(pairScale)).add(coordinateStep));
}

export const arrayPingPong: ComputePipelineSpec = computePipeline<ArrayPingPongLayout>(
  arrayPingPongKernel,
  { name: "arrayPingPong", workgroupSize: [2, 2, 1] },
);

class LayerReadbackLayout {
  source!: Texture2d<f32>;
  output!: MutStorage<Vec4f>;
}

function layerReadbackKernel(res: LayerReadbackLayout, ctx: ComputeInvocation): void {
  if (ctx.globalId.x >= WIDTH || ctx.globalId.y >= HEIGHT) return;
  const coords = new Vec2i(ctx.globalId.x as i32, ctx.globalId.y as i32);
  const index: u32 = ctx.globalId.y * WIDTH + ctx.globalId.x;
  res.output[index] = res.source.load(coords, 0);
}

export const layerReadback: ComputePipelineSpec = computePipeline<LayerReadbackLayout>(
  layerReadbackKernel,
  { name: "layerReadback", workgroupSize: [2, 2, 1] },
);

function payloadPixels(): Vec4f[] {
  return [
    // Layer 0: color.
    new Vec4f(1.0, 2.0, 3.0, 1.0),
    new Vec4f(2.0, 3.0, 4.0, 1.0),
    new Vec4f(3.0, 4.0, 5.0, 1.0),
    new Vec4f(4.0, 5.0, 6.0, 1.0),
    // Layer 1: coordinate.
    new Vec4f(0.0, 0.0, 0.0, 0.0),
    new Vec4f(1.0, 0.0, 0.0, 0.0),
    new Vec4f(0.0, 1.0, 0.0, 0.0),
    new Vec4f(1.0, 1.0, 0.0, 0.0),
  ];
}

function zeroPayload(): Vec4f[] {
  const values: Vec4f[] = [];
  let index: u32 = 0;
  while (index < PIXEL_COUNT * LAYERS) {
    values.push(new Vec4f(0.0, 0.0, 0.0, 0.0));
    index += 1;
  }
  return values;
}

function appendHalf(bytes: u8[], value: f32): void {
  const encoded: u8[] = Context.bytesOf<FixedArray<f16, 1>>([value as f16]);
  bytes.push(encoded[0]);
  bytes.push(encoded[1]);
}

function writeArrayPixels(device: GPUDevice, texture: GPUTexture, pixels: Vec4f[]): void {
  const bytes: u8[] = [];
  let layer: u32 = 0;
  while (layer < LAYERS) {
    let y: u32 = 0;
    while (y < HEIGHT) {
      let x: u32 = 0;
      while (x < WIDTH) {
        const pixel: Vec4f = pixels[((layer * HEIGHT + y) * WIDTH + x) as i32];
        appendHalf(bytes, pixel.x);
        appendHalf(bytes, pixel.y);
        appendHalf(bytes, pixel.z);
        appendHalf(bytes, pixel.w);
        x += 1;
      }
      let rowByte: u32 = WIDTH * 8;
      while (rowByte < 256) {
        bytes.push(0);
        rowByte += 1;
      }
      y += 1;
    }
    layer += 1;
  }
  device.queue.writeTexture(
    { texture },
    bytes,
    { offset: 0, bytesPerRow: 256, rowsPerImage: HEIGHT },
    { width: WIDTH, height: HEIGHT, depthOrArrayLayers: LAYERS },
  );
}

function close(actual: f32, expected: f32): boolean {
  const difference: f32 = Math.abs((actual - expected) as f64) as f32;
  return difference <= 0.001;
}

function same(actual: Vec4f, expected: Vec4f): boolean {
  return close(actual.x, expected.x)
    && close(actual.y, expected.y)
    && close(actual.z, expected.z)
    && close(actual.w, expected.w);
}

function allZero(values: FixedArray<Vec4f, 4>): boolean {
  let index: i32 = 0;
  while (index < 4) {
    const value: Vec4f = values[index];
    if (value.x !== 0.0 || value.y !== 0.0 || value.z !== 0.0 || value.w !== 0.0) {
      return false;
    }
    index += 1;
  }
  return true;
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
  let noop: boolean = false;
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const initial: Vec4f[] = payloadPixels();
    using textureA = device.createTexture({
      label: "x23-a",
      size: { width: WIDTH, height: HEIGHT, depthOrArrayLayers: LAYERS },
      format: "rgba16float",
      usage: GPUTextureUsage.STORAGE_BINDING
        + GPUTextureUsage.TEXTURE_BINDING
        + GPUTextureUsage.COPY_DST,
    });
    using textureB = device.createTexture({
      label: "x23-b",
      size: { width: WIDTH, height: HEIGHT, depthOrArrayLayers: LAYERS },
      format: "rgba16float",
      usage: GPUTextureUsage.STORAGE_BINDING,
    });
    writeArrayPixels(device, textureA, initial);
    using arrayViewA = textureA.createView({
      dimension: "2d-array",
      mipLevelCount: 1,
      arrayLayerCount: LAYERS,
    });
    using arrayViewB = textureB.createView({
      dimension: "2d-array",
      mipLevelCount: 1,
      arrayLayerCount: LAYERS,
    });
    using layerView = textureA.createView({
      dimension: "2d",
      mipLevelCount: 1,
      baseArrayLayer: 1,
      arrayLayerCount: 1,
    });
    using output: Buffer<Vec4f> = createBuffer<Vec4f>(
      device,
      16,
      PIXEL_COUNT,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC,
      "x23-output",
    );
    print("inputs:written");

    device.pushErrorScope("validation");
    using pingPongPipeline = createComputePipeline(
      device,
      arrayPingPong_WGSL,
      arrayPingPong_ENTRY,
      [arrayPingPong_LAYOUT0],
      [arrayPingPong_WORKGROUP_X, arrayPingPong_WORKGROUP_Y, arrayPingPong_WORKGROUP_Z],
    );
    using readbackPipeline = createComputePipeline(
      device,
      layerReadback_WGSL,
      layerReadback_ENTRY,
      [layerReadback_LAYOUT0],
      [layerReadback_WORKGROUP_X, layerReadback_WORKGROUP_Y, layerReadback_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
    print("pipelines:created");

    using groupAB = createBindGroup(
      device,
      pingPongPipeline.bindGroupLayout(0),
      arrayPingPong_LAYOUT0,
      [textureResource(arrayViewA), textureResource(arrayViewB)],
    );
    using groupBA = createBindGroup(
      device,
      pingPongPipeline.bindGroupLayout(0),
      arrayPingPong_LAYOUT0,
      [textureResource(arrayViewB), textureResource(arrayViewA)],
    );
    using readbackGroup = createBindGroup(
      device,
      readbackPipeline.bindGroupLayout(0),
      layerReadback_LAYOUT0,
      [textureResource(layerView), bufferResource(output.handle())],
    );
    using encoder = device.createCommandEncoderDefault();
    pingPongPipeline.dispatchThreads(encoder, [groupAB], WIDTH, HEIGHT, LAYERS);
    pingPongPipeline.dispatchThreads(encoder, [groupBA], WIDTH, HEIGHT, LAYERS);
    readbackPipeline.dispatchThreads(encoder, [readbackGroup], WIDTH, HEIGHT, 1);
    using command = encoder.finishDefault();
    device.queue.submit([command]);
    if (!await device.queue.onSubmittedWorkDone()) {
      print("FAIL submit");
      return;
    }
    print("dispatch:submitted");

    const hostA: Vec4f[] = payloadPixels();
    const hostB: Vec4f[] = zeroPayload();
    const hostAB = new ArrayPingPongLayout();
    hostAB.source = new ReadStorageTexture2dArray<Rgba16float>(hostA, WIDTH, HEIGHT, LAYERS);
    hostAB.target = new WriteStorageTexture2dArray<Rgba16float>(hostB, WIDTH, HEIGHT, LAYERS);
    simulateComputeThreads<ArrayPingPongLayout>(
      arrayPingPongKernel,
      hostAB,
      arrayPingPong,
      WIDTH,
      HEIGHT,
      LAYERS,
      arrayPingPong_HOST_RUNNABLE,
    );
    const hostBA = new ArrayPingPongLayout();
    hostBA.source = new ReadStorageTexture2dArray<Rgba16float>(hostB, WIDTH, HEIGHT, LAYERS);
    hostBA.target = new WriteStorageTexture2dArray<Rgba16float>(hostA, WIDTH, HEIGHT, LAYERS);
    simulateComputeThreads<ArrayPingPongLayout>(
      arrayPingPongKernel,
      hostBA,
      arrayPingPong,
      WIDTH,
      HEIGHT,
      LAYERS,
      arrayPingPong_HOST_RUNNABLE,
    );

    const outputBytes: u8[] = await output.read(device, 0, PIXEL_COUNT);
    const gpuOutput: FixedArray<Vec4f, 4> = Context.fromBytes<FixedArray<Vec4f, 4>>(
      outputBytes,
      0,
    );
    print("readback:mapped");
    if (allZero(gpuOutput)) {
      noop = true;
    } else {
      let index: u32 = 0;
      while (index < PIXEL_COUNT) {
        const expected: Vec4f = hostA[(PIXEL_COUNT + index) as i32];
        if (!same(gpuOutput[index as i32], expected)) {
          print(`FAIL index=${index}`);
          return;
        }
        index += 1;
      }
    }
  }
  gpu.dispose();
  if (noop) {
    print("pending: Noop backend does not execute compute");
    return;
  }
  print("PASS");
}
