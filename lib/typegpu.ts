/** Runtime bindings shared by generated TypeGPU support modules and programs. */

import { Vec2f, Vec2i, Vec2u, Vec3u, Vec4f } from "./typegpu-types";
import {
  GPUBindGroup,
  GPUBindGroupEntry,
  GPUBindGroupLayout,
  GPUBindGroupLayoutEntry,
  GPUBlendComponent,
  GPUBlendState,
  GPUBuffer,
  GPUCommandEncoder,
  GPUComputePipeline,
  GPUComputePipelineDescriptor,
  GPUDevice,
  GPUHostOwnedDevice,
  GPUPipelineLayout,
  GPUQuerySet,
  GPUQueue,
  GPURenderPassEncoder,
  GPURenderPipeline,
  GPURenderPipelineDescriptor,
  GPUSampler,
  GPUSamplerDescriptor,
  GPUShaderStage,
  GPUShaderModule,
  GPUBufferUsage,
  GPUMapMode,
  GPUTextureView,
  GPUTexture,
  GPUVertexAttribute,
  GPUVertexBufferLayout,
} from "./webgpu";

function authorTrap(rule: string, method: string, values: string): void {
  print(`${rule} ${method} ${values} (author)`);
  unreachable();
}

function appendBytes(target: u8[], source: u8[]): void {
  let index: i32 = 0;
  while (index < source.length) {
    target.push(source[index]);
    index += 1;
  }
}

function textureComponentBytes(format: GPUTextureFormat): u32 {
  if (format === "rgba8unorm") return 1;
  if (format === "rgba16float") return 2;
  if (format === "r32float" || format === "rgba32float") return 4;
  authorTrap("TX9", "writeTexturePixels", `format=${format} is not supported`);
  return 0;
}

function textureChannelCount(format: GPUTextureFormat): u32 {
  return format === "r32float" ? 1 : 4;
}

function appendTextureComponent(bytes: u8[], format: GPUTextureFormat, value: f32): void {
  if (format === "rgba8unorm") {
    const clamped: f32 = Math.min(1.0, Math.max(0.0, value as f64)) as f32;
    bytes.push(Math.floor((clamped * 255.0 + 0.5) as f64) as u8);
    return;
  }
  if (format === "rgba16float") {
    appendBytes(bytes, Context.bytesOf<FixedArray<f16, 1>>([value as f16]));
    return;
  }
  appendBytes(bytes, Context.bytesOf<FixedArray<f32, 1>>([value]));
}

/**
 * Uploads raw bytes into mip level 0 of a texture, over its full extent and one layer.
 * When a write covers more than one row, a `bytesPerRow` below 256 traps with TX9.
 */
export function writeTextureBytes(
  queue: GPUQueue,
  texture: GPUTexture,
  bytes: u8[],
  bytesPerRow: u32,
  width: u32,
  height: u32,
): void {
  if (height > 1 && bytesPerRow < 256) {
    authorTrap("TX9", "writeTextureBytes", `bytesPerRow=${bytesPerRow} height=${height}`);
  }
  queue.writeTexture(
    { texture },
    bytes,
    { offset: 0, bytesPerRow, rowsPerImage: height },
    { width, height, depthOrArrayLayers: 1 },
  );
}

/**
 * Encodes `pixels` in row-major order into the texture's format and uploads them.
 * `rgba8unorm` scales each channel to a byte, and a float format stores the channel in its own
 * float type.
 * A pixel count other than `width * height`, or a format outside that set, traps with TX9.
 */
export function writeTexturePixels(
  queue: GPUQueue,
  texture: GPUTexture,
  pixels: Vec4f[],
  width: u32,
  height: u32,
): void {
  const pixelCount: u32 = width * height;
  if ((pixels.length as u32) !== pixelCount) {
    authorTrap("TX9", "writeTexturePixels", `pixels=${pixels.length} width=${width} height=${height}`);
  }
  const format: GPUTextureFormat = texture.format;
  const componentBytes: u32 = textureComponentBytes(format);
  const channels: u32 = textureChannelCount(format);
  const rowBytes: u32 = width * channels * componentBytes;
  const bytesPerRow: u32 = height > 1 ? ((rowBytes + 255) / 256) * 256 : rowBytes;
  const bytes: u8[] = [];
  let y: u32 = 0;
  while (y < height) {
    let x: u32 = 0;
    while (x < width) {
      const pixel: Vec4f = pixels[(y * width + x) as i32];
      appendTextureComponent(bytes, format, pixel.x);
      if (channels === 4) {
        appendTextureComponent(bytes, format, pixel.y);
        appendTextureComponent(bytes, format, pixel.z);
        appendTextureComponent(bytes, format, pixel.w);
      }
      x += 1;
    }
    let padding: u32 = rowBytes;
    while (padding < bytesPerRow) {
      bytes.push(0);
      padding += 1;
    }
    y += 1;
  }
  writeTextureBytes(queue, texture, bytes, bytesPerRow, width, height);
}

/**
 * A sampled 2d texture binding. In WGSL it becomes `texture_2d<T>`, where `T` is `f32`.
 * The host body reads the `Vec4f[]` image the constructor takes, so a kernel also runs on the host.
 */
export class Texture2d<T> {
  // The generator reads this zero-length marker to recover T from the typed HIR.
  private values: T[];
  private pixels: Vec4f[];
  private width: u32;
  private height: u32;

  constructor(pixels: Vec4f[], width: u32, height: u32) {
    this.values = [];
    this.pixels = pixels;
    this.width = width;
    this.height = height;
  }

  dimensions(): Vec2u {
    return new Vec2u(this.width, this.height);
  }

  load(coords: Vec2i, level: u32): Vec4f {
    if (level !== 0) {
      authorTrap("TX3", "load", `level=${level} is not supported`);
    }
    if (coords.x < 0 || coords.y < 0
      || (coords.x as u32) >= this.width || (coords.y as u32) >= this.height) {
      return new Vec4f(0.0, 0.0, 0.0, 0.0);
    }
    const pixel: u32 = (coords.y as u32) * this.width + (coords.x as u32);
    return this.pixels[pixel as i32];
  }

  sampleLevel(sampler: Sampler, uv: Vec2f, level: f32): Vec4f {
    if (!sampler.isNearest()) {
      authorTrap("TX3", "sampleLevel", "filterMode is not nearest");
    }
    if (level !== 0.0) {
      authorTrap("TX3", "sampleLevel", `level=${level} is not supported`);
    }
    if (this.width === 0 || this.height === 0) {
      return new Vec4f(0.0, 0.0, 0.0, 0.0);
    }
    let x: i32 = Math.floor((uv.x * (this.width as f32)) as f64) as i32;
    let y: i32 = Math.floor((uv.y * (this.height as f32)) as f64) as i32;
    if (x < 0) x = 0;
    if (y < 0) y = 0;
    if ((x as u32) >= this.width) x = (this.width - 1) as i32;
    if ((y as u32) >= this.height) y = (this.height - 1) as i32;
    return this.load(new Vec2i(x, y), 0);
  }

  sample(sampler: Sampler, uv: Vec2f): Vec4f {
    if (!sampler.isNearest()) {
      authorTrap("TX3", "sample", "filterMode is not nearest");
    }
    return this.sampleLevel(sampler, uv, 0.0);
  }

  store(coords: Vec2i, value: Vec4f): void {
    authorTrap("TX3", "store", "is not legal on Texture2d");
  }
}

/**
 * A sampler binding. In WGSL it becomes `sampler`.
 * The host body carries the filter mode and implements `nearest` only, and any other mode
 * traps with TX3 inside a sample call.
 */
export class Sampler {
  private filterMode: string;

  constructor(filterMode: string) {
    this.filterMode = filterMode;
  }

  isNearest(): boolean {
    return this.filterMode === "nearest";
  }
}

/**
 * Builds the host `Sampler` that matches a GPU sampler descriptor, so both lanes filter alike.
 * A descriptor with `minFilter` and `magFilter` of `nearest` gives a nearest sampler.
 * Every other descriptor gives one that traps with TX3 on a sample call.
 */
export function samplerFromDescriptor(descriptor: GPUSamplerDescriptor): Sampler {
  if (descriptor.minFilter === "nearest" && descriptor.magFilter === "nearest") {
    return new Sampler("nearest");
  }
  return new Sampler("non-nearest");
}

/**
 * The storage-texture format markers. Each one names the WGSL format of a storage texture
 * wrapper through the wrapper's `F` type argument, and carries no value of its own.
 */
export class Rgba8unorm {}
export class Rgba16float {}
export class R32float {}
export class Rgba32float {}

/**
 * A write-only storage texture binding. In WGSL it becomes `texture_storage_2d<F, write>`.
 * Its layout entry access is `write-only`, so a `load` call traps with TX11, and a `store`
 * outside the extent writes nothing.
 */
export class StorageTexture2d<F> {
  private values: Vec4f[];
  // The generator reads this zero-length marker to recover F from the typed HIR.
  private formats: F[];
  private width: u32;
  private height: u32;

  constructor(values: Vec4f[], width: u32, height: u32) {
    this.values = values;
    this.formats = [];
    this.width = width;
    this.height = height;
  }

  load(coords: Vec2i): Vec4f {
    authorTrap("TX11", "StorageTexture2d.load", "access=write-only");
    return new Vec4f(0.0, 0.0, 0.0, 0.0);
  }

  store(coords: Vec2i, value: Vec4f): void {
    if (coords.x < 0 || coords.y < 0
      || (coords.x as u32) >= this.width || (coords.y as u32) >= this.height) {
      return;
    }
    const pixel: i32 = ((coords.y as u32) * this.width + (coords.x as u32)) as i32;
    if (pixel === this.values.length) {
      this.values.push(value);
    } else {
      this.values[pixel] = value;
    }
  }
}

/**
 * A read-only storage texture binding. In WGSL it becomes `texture_storage_2d<F, read>`.
 * Its layout entry access is `read-only`, it declares no `store`, and a `load` outside the
 * extent returns a zero `Vec4f`.
 */
export class ReadStorageTexture2d<F> {
  private values: Vec4f[];
  // The generator reads this zero-length marker to recover F from the typed HIR.
  private formats: F[];
  private width: u32;
  private height: u32;

  constructor(values: Vec4f[], width: u32, height: u32) {
    this.values = values;
    this.formats = [];
    this.width = width;
    this.height = height;
  }

  load(coords: Vec2i): Vec4f {
    if (coords.x < 0 || coords.y < 0
      || (coords.x as u32) >= this.width || (coords.y as u32) >= this.height) {
      return new Vec4f(0.0, 0.0, 0.0, 0.0);
    }
    const pixel: i32 = ((coords.y as u32) * this.width + (coords.x as u32)) as i32;
    return this.values[pixel];
  }

  dimensions(): Vec2u {
    return new Vec2u(this.width, this.height);
  }
}

/**
 * A read-write storage texture binding. In WGSL it becomes
 * `texture_storage_2d<F, read_write>`.
 * A format outside the r32 set needs the device feature `texture-formats-tier2`, and an access
 * outside the extent reads a zero `Vec4f` or writes nothing.
 */
export class ReadWriteStorageTexture2d<F> {
  private values: Vec4f[];
  // The generator reads this zero-length marker to recover F from the typed HIR.
  private formats: F[];
  private width: u32;
  private height: u32;

  constructor(values: Vec4f[], width: u32, height: u32) {
    this.values = values;
    this.formats = [];
    this.width = width;
    this.height = height;
  }

  load(coords: Vec2i): Vec4f {
    if (coords.x < 0 || coords.y < 0
      || (coords.x as u32) >= this.width || (coords.y as u32) >= this.height) {
      return new Vec4f(0.0, 0.0, 0.0, 0.0);
    }
    const pixel: i32 = ((coords.y as u32) * this.width + (coords.x as u32)) as i32;
    return this.values[pixel];
  }

  store(coords: Vec2i, value: Vec4f): void {
    if (coords.x < 0 || coords.y < 0
      || (coords.x as u32) >= this.width || (coords.y as u32) >= this.height) {
      return;
    }
    const pixel: i32 = ((coords.y as u32) * this.width + (coords.x as u32)) as i32;
    if (pixel === this.values.length) {
      this.values.push(value);
    } else {
      this.values[pixel] = value;
    }
  }

  dimensions(): Vec2u {
    return new Vec2u(this.width, this.height);
  }
}

/**
 * A sampled 2d array texture binding. In WGSL it becomes `texture_2d_array<T>`.
 * `load` takes a layer index, `dimensions()` reports the width and the height only, and
 * `sample`, `sampleLevel`, and `store` trap with TX3.
 */
export class Texture2dArray<T> {
  // The generator reads this zero-length marker to recover T from the typed HIR.
  private values: T[];
  private pixels: Vec4f[];
  private width: u32;
  private height: u32;
  private layers: u32;

  constructor(pixels: Vec4f[], width: u32, height: u32, layers: u32) {
    this.values = [];
    this.pixels = pixels;
    this.width = width;
    this.height = height;
    this.layers = layers;
  }

  load(coords: Vec2i, layer: i32, level: u32): Vec4f {
    if (level !== 0) {
      authorTrap("TX3", "Texture2dArray.load", `level=${level} is not supported`);
    }
    if (coords.x < 0 || coords.y < 0 || layer < 0
      || (coords.x as u32) >= this.width || (coords.y as u32) >= this.height
      || (layer as u32) >= this.layers) {
      return new Vec4f(0.0, 0.0, 0.0, 0.0);
    }
    const pixel: u32 = ((layer as u32) * this.height + (coords.y as u32)) * this.width
      + (coords.x as u32);
    return this.pixels[pixel as i32];
  }

  sampleLevel(sampler: Sampler, uv: Vec2f, level: f32): Vec4f {
    authorTrap("TX3", "Texture2dArray.sampleLevel", "is not legal on an array sampled texture");
    return new Vec4f(0.0, 0.0, 0.0, 0.0);
  }

  sample(sampler: Sampler, uv: Vec2f): Vec4f {
    authorTrap("TX3", "Texture2dArray.sample", "is not legal on an array sampled texture");
    return new Vec4f(0.0, 0.0, 0.0, 0.0);
  }

  store(coords: Vec2i, layer: i32, value: Vec4f): void {
    authorTrap("TX3", "Texture2dArray.store", "is not legal on a sampled texture");
  }

  dimensions(): Vec2u {
    return new Vec2u(this.width, this.height);
  }
}

/**
 * A read-only storage array texture binding. In WGSL it becomes
 * `texture_storage_2d_array<F, read>`.
 * `load` takes a layer index and returns a zero `Vec4f` outside the extent, and a `store` call
 * traps with TX13.
 */
export class ReadStorageTexture2dArray<F> {
  private values: Vec4f[];
  // The generator reads this zero-length marker to recover F from the typed HIR.
  private formats: F[];
  private width: u32;
  private height: u32;
  private layers: u32;

  constructor(values: Vec4f[], width: u32, height: u32, layers: u32) {
    this.values = values;
    this.formats = [];
    this.width = width;
    this.height = height;
    this.layers = layers;
  }

  load(coords: Vec2i, layer: i32): Vec4f {
    if (coords.x < 0 || coords.y < 0 || layer < 0
      || (coords.x as u32) >= this.width || (coords.y as u32) >= this.height
      || (layer as u32) >= this.layers) {
      return new Vec4f(0.0, 0.0, 0.0, 0.0);
    }
    const pixel: i32 = (
      ((layer as u32) * this.height + (coords.y as u32)) * this.width + (coords.x as u32)
    ) as i32;
    return this.values[pixel];
  }

  store(coords: Vec2i, layer: i32, value: Vec4f): void {
    authorTrap("TX13", "ReadStorageTexture2dArray.store", "access=read-only");
  }
}

/**
 * A write-only storage array texture binding. In WGSL it becomes
 * `texture_storage_2d_array<F, write>`.
 * `store` takes a layer index and writes nothing outside the extent, and a `load` call traps
 * with TX13.
 */
export class WriteStorageTexture2dArray<F> {
  private values: Vec4f[];
  // The generator reads this zero-length marker to recover F from the typed HIR.
  private formats: F[];
  private width: u32;
  private height: u32;
  private layers: u32;

  constructor(values: Vec4f[], width: u32, height: u32, layers: u32) {
    this.values = values;
    this.formats = [];
    this.width = width;
    this.height = height;
    this.layers = layers;
  }

  load(coords: Vec2i, layer: i32): Vec4f {
    authorTrap("TX13", "WriteStorageTexture2dArray.load", "access=write-only");
    return new Vec4f(0.0, 0.0, 0.0, 0.0);
  }

  store(coords: Vec2i, layer: i32, value: Vec4f): void {
    if (coords.x < 0 || coords.y < 0 || layer < 0
      || (coords.x as u32) >= this.width || (coords.y as u32) >= this.height
      || (layer as u32) >= this.layers) {
      return;
    }
    const pixel: i32 = (
      ((layer as u32) * this.height + (coords.y as u32)) * this.width + (coords.x as u32)
    ) as i32;
    if (pixel === this.values.length) {
      this.values.push(value);
    } else {
      this.values[pixel] = value;
    }
  }
}

/**
 * A typed GPU buffer whose indices and counts are elements, never bytes.
 * `elementSize` is the schema's stride, so element `i` starts at `i * elementSize`.
 * The class owns the `GPUBuffer`, and `dispose()` releases it.
 */
export class Buffer<T> {
  buffer: GPUBuffer;
  elementSize: u32;
  count: u32;
  usage: u64;

  constructor(buffer: GPUBuffer, elementSize: u32, count: u32, usage: u64) {
    this.buffer = buffer;
    this.elementSize = elementSize;
    this.count = count;
    this.usage = usage;
  }

  write(queue: GPUQueue, elementIndex: u32, bytes: u8[]): void {
    if ((this.usage & GPUBufferUsage.COPY_DST) === 0) {
      authorTrap("BF10", "Buffer.write", `usage=${this.usage}`);
    }
    const byteLength: u32 = bytes.length as u32;
    const remainder: u32 = byteLength % this.elementSize;
    if (remainder !== 0) {
      authorTrap("BF8", "Buffer.write", `byteLength=${byteLength} elementSize=${this.elementSize} remainder=${remainder}`);
    }
    const elementCount: u32 = byteLength / this.elementSize;
    if (elementIndex > this.count || elementCount > this.count - elementIndex) {
      authorTrap("BF8", "Buffer.write", `elementIndex=${elementIndex} elementCount=${elementCount} count=${this.count}`);
    }
    const byteOffset: u64 = (elementIndex as u64) * (this.elementSize as u64);
    if (byteOffset % 4 !== 0 || byteLength % 4 !== 0) {
      authorTrap("BF2", "Buffer.write", `byteOffset=${byteOffset} byteLength=${byteLength}`);
    }
    queue.writeBuffer(
      this.buffer,
      byteOffset,
      bytes,
    );
  }

  writeOne(queue: GPUQueue, elementIndex: u32, bytes: u8[]): void {
    if ((this.usage & GPUBufferUsage.COPY_DST) === 0) {
      authorTrap("BF10", "Buffer.writeOne", `usage=${this.usage}`);
    }
    const byteLength: u32 = bytes.length as u32;
    if (byteLength !== this.elementSize) {
      authorTrap("BF8", "Buffer.writeOne", `elementIndex=${elementIndex} byteLength=${byteLength} elementSize=${this.elementSize}`);
    }
    if (elementIndex >= this.count) {
      authorTrap("BF8", "Buffer.writeOne", `elementIndex=${elementIndex} elementCount=1 count=${this.count}`);
    }
    const byteOffset: u64 = (elementIndex as u64) * (this.elementSize as u64);
    if (byteOffset % 4 !== 0 || byteLength % 4 !== 0) {
      authorTrap("BF2", "Buffer.writeOne", `byteOffset=${byteOffset} byteLength=${byteLength}`);
    }
    queue.writeBuffer(
      this.buffer,
      byteOffset,
      bytes,
    );
  }

  patch(queue: GPUQueue, elementIndex: u32, fieldOffset: u32, bytes: u8[]): void {
    if ((this.usage & GPUBufferUsage.COPY_DST) === 0) {
      authorTrap("BF10", "Buffer.patch", `usage=${this.usage}`);
    }
    const byteLength: u32 = bytes.length as u32;
    if (elementIndex >= this.count) {
      authorTrap("EG2", "Buffer.patch", `elementIndex=${elementIndex} elementCount=1 count=${this.count}`);
    }
    if (fieldOffset > this.elementSize || byteLength > this.elementSize - fieldOffset) {
      authorTrap("EG2", "Buffer.patch", `fieldOffset=${fieldOffset} byteLength=${byteLength} elementSize=${this.elementSize}`);
    }
    const byteOffset: u64 = (elementIndex as u64) * (this.elementSize as u64) + (fieldOffset as u64);
    if (byteOffset % 4 !== 0 || byteLength % 4 !== 0) {
      authorTrap("BF2", "Buffer.patch", `byteOffset=${byteOffset} byteLength=${byteLength}`);
    }
    queue.writeBuffer(
      this.buffer,
      byteOffset,
      bytes,
    );
  }

  copyTo(
    encoder: GPUCommandEncoder,
    target: Buffer<T>,
    elementIndex: u32,
    elementCount: u32,
  ): void {
    if (elementIndex > this.count || elementCount > this.count - elementIndex) {
      authorTrap("BF8", "Buffer.copyTo", `elementIndex=${elementIndex} elementCount=${elementCount} count=${this.count}`);
    }
    if (elementCount > target.count) {
      authorTrap("BF8", "Buffer.copyTo", `targetCount=${target.count} elementCount=${elementCount} elementSize=${this.elementSize}`);
    }
    const byteOffset: u64 = (elementIndex as u64) * (this.elementSize as u64);
    const byteLength: u64 = (elementCount as u64) * (this.elementSize as u64);
    if (byteOffset % 4 !== 0 || byteLength % 4 !== 0) {
      authorTrap("BF8", "Buffer.copyTo", `byteOffset=${byteOffset} byteLength=${byteLength}`);
    }
    encoder.copyBufferToBuffer(
      this.buffer,
      byteOffset,
      target.buffer,
      0,
      byteLength,
    );
  }

  async read(device: GPUDevice, elementIndex: u32, elementCount: u32): Promise<u8[]> {
    if ((this.usage & GPUBufferUsage.COPY_SRC) === 0) {
      authorTrap("BF10", "Buffer.read", `usage=${this.usage}`);
    }
    if (elementIndex > this.count || elementCount > this.count - elementIndex) {
      authorTrap("BF9", "Buffer.read", `elementIndex=${elementIndex} elementCount=${elementCount} count=${this.count}`);
    }
    const byteOffset: u64 = (elementIndex as u64) * (this.elementSize as u64);
    const byteLength: u64 = (elementCount as u64) * (this.elementSize as u64);
    if (byteOffset % 4 !== 0 || byteLength % 4 !== 0) {
      authorTrap("BF9", "Buffer.read", `byteOffset=${byteOffset} byteLength=${byteLength}`);
    }
    const staging: Buffer<T> = createBuffer<T>(
      device,
      this.elementSize,
      elementCount,
      GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
      "typegpu-read-staging",
    );
    using encoder = device.createCommandEncoderDefault();
    this.copyTo(encoder, staging, elementIndex, elementCount);
    using command = encoder.finishDefault();
    device.queue.submit([command]);
    if (!await staging.handle().mapAsync(GPUMapMode.READ, 0, byteLength)) {
      staging.dispose();
      authorTrap("BF9", "Buffer.read", `elementIndex=${elementIndex} elementCount=${elementCount} count=${this.count}`);
    }
    const bytes: u8[] = readBuffer<T>(staging, 0, elementCount);
    staging.handle().unmap();
    staging.dispose();
    return bytes;
  }

  async readOne(device: GPUDevice, elementIndex: u32): Promise<u8[]> {
    if (elementIndex >= this.count) {
      authorTrap("BF9", "Buffer.readOne", `elementIndex=${elementIndex} elementCount=1 count=${this.count}`);
    }
    return await this.read(device, elementIndex, 1);
  }

  handle(): GPUBuffer {
    return this.buffer;
  }

  dispose(): void {
    this.buffer.dispose();
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

/**
 * Copies `elementCount` elements out of a buffer the caller already mapped with `mapAsync`.
 * A range past the end traps with BF8. Decode the bytes with `Context.fromBytes`.
 */
export function readBuffer<T>(
  readback: Buffer<T>,
  elementIndex: u32,
  elementCount: u32,
): u8[] {
  if (elementIndex > readback.count || elementCount > readback.count - elementIndex) {
    authorTrap("BF8", "readBuffer", `elementIndex=${elementIndex} elementCount=${elementCount} count=${readback.count}`);
  }
  return readback.buffer.readMappedRange(
    (elementIndex as u64) * (readback.elementSize as u64),
    (elementCount as u64) * (readback.elementSize as u64),
  );
}

/**
 * Copies one element out of a buffer the caller already mapped with `mapAsync`.
 * An `elementIndex` at or past the element count traps with BF8.
 */
export function readOne<T>(readback: Buffer<T>, elementIndex: u32): u8[] {
  if (elementIndex >= readback.count) {
    authorTrap("BF8", "readOne", `elementIndex=${elementIndex} elementCount=1 count=${readback.count}`);
  }
  return readback.buffer.readMappedRange(
    (elementIndex as u64) * (readback.elementSize as u64),
    readback.elementSize as u64,
  );
}

/**
 * Creates a `Buffer<T>` that holds `count` elements of `elementSize` bytes each.
 * Pass the schema's stride constant as `elementSize`, never its size constant.
 * The buffer keeps `usage`, and a method whose usage flag is absent traps with BF10.
 */
export function createBuffer<T>(
  device: GPUDevice,
  elementSize: u32,
  count: u32,
  usage: u64,
  label: string,
): Buffer<T> {
  return new Buffer<T>(
    device.createBuffer({
      size: (elementSize as u64) * (count as u64),
      usage,
      label,
    }),
    elementSize,
    count,
    usage,
  );
}

/**
 * Creates a `Buffer<T>` through a device the host application owns, as `createBuffer` does.
 * The buffer belongs to the script, so the script disposes it.
 */
export function createBufferHost<T>(
  device: GPUHostOwnedDevice,
  elementSize: u32,
  count: u32,
  usage: u64,
  label: string,
): Buffer<T> {
  return new Buffer<T>(
    device.createBuffer({
      size: (elementSize as u64) * (count as u64),
      usage,
      label,
    }),
    elementSize,
    count,
    usage,
  );
}

/**
 * Carries the compute builtins into a kernel.
 * The generator emits a `@builtin` parameter for each field the kernel reads and no other.
 * The host lane builds one per invocation, and a program never constructs one itself.
 */
export class ComputeInvocation {
  globalId: Vec3u;
  localId: Vec3u;
  workgroupId: Vec3u;
  numWorkgroups: Vec3u;
  localIndex: u32;

  constructor(
    globalId: Vec3u,
    localId: Vec3u,
    workgroupId: Vec3u,
    numWorkgroups: Vec3u,
    localIndex: u32,
  ) {
    this.globalId = globalId;
    this.localId = localId;
    this.workgroupId = workgroupId;
    this.numWorkgroups = numWorkgroups;
    this.localIndex = localIndex;
  }
}

/**
 * Carries the vertex builtins into a vertex kernel.
 * The generator emits `@builtin(vertex_index)` and `@builtin(instance_index)` for the fields
 * the kernel reads.
 */
export class VertexInvocation {
  vertexIndex: u32;
  instanceIndex: u32;

  constructor(vertexIndex: u32, instanceIndex: u32) {
    this.vertexIndex = vertexIndex;
    this.instanceIndex = instanceIndex;
  }
}

/**
 * Carries the fragment builtins into a fragment kernel, and `frontFacing` becomes
 * `@builtin(front_facing)`.
 * The fragment position arrives as the varyings' `position` field, never through this class.
 */
export class FragmentInvocation {
  frontFacing: boolean;

  constructor(frontFacing: boolean) {
    this.frontFacing = frontFacing;
  }
}

/**
 * A uniform buffer binding. In WGSL it becomes `var<uniform> name: T`.
 * A kernel reads the value through the accessor `$`, where `T` is a schema class, a library
 * vector or matrix, or a scalar.
 */
export class Uniform<T> {
  private values: T[];

  constructor(value: T) {
    this.values = [value];
  }

  get $(): T {
    return this.values[0];
  }
}

/**
 * A read-only storage buffer binding. In WGSL it becomes `var<storage, read> name: array<T>`.
 * A kernel reads an element as `items[i]` and the element count as `length()`.
 */
export class Storage<T> {
  readonly [index: u32]: T;
  private values: T[];

  constructor(values: T[]) {
    this.values = values;
  }

  get(index: u32): T {
    return this.values[index as i32];
  }

  length(): u32 {
    return this.values.length as u32;
  }
}

/**
 * A read-write storage buffer binding. In WGSL it becomes
 * `var<storage, read_write> name: array<T>`.
 * A kernel reads an element as `items[i]` and writes one as `items[i] = value`.
 */
export class MutStorage<T> {
  [index: u32]: T;
  private values: T[];

  constructor(values: T[]) {
    this.values = values;
  }

  get(index: u32): T {
    return this.values[index as i32];
  }

  set(index: u32, value: T): void {
    this.values[index as i32] = value;
  }

  length(): u32 {
    return this.values.length as u32;
  }
}

/**
 * A private module variable. In WGSL it becomes `var<private> x: T = init`.
 * A kernel reads and writes it as `x.$` and `x.$ = value`, and every invocation holds its own copy.
 */
export class PrivateVar<T> {
  private value: T;

  constructor(value: T) {
    this.value = value;
  }

  get $(): T {
    return this.value;
  }

  set $(value: T) {
    this.value = value;
  }
}

/**
 * A workgroup variable. In WGSL it becomes `var<workgroup> x: T` with no initializer.
 * A kernel reads and writes it as `x.$` and `x.$ = value`, and one workgroup shares one copy.
 */
export class WorkgroupVar<T> {
  private values: T[];

  constructor() {
    this.values = [];
  }

  get $(): T {
    return this.values[0];
  }

  set $(value: T) {
    if (this.values.length === 0) {
      this.values.push(value);
    } else {
      this.values[0] = value;
    }
  }
}

/**
 * A workgroup array. In WGSL it becomes `var<workgroup> x: array<T, n>` with no initializer.
 * A kernel reads and writes an element as `x[i]`, and `length()` reports the declared `n`, not
 * the count a host run wrote.
 */
export class WorkgroupArray<T> {
  [index: u32]: T;
  private values: T[];
  private count: u32;

  constructor(count: u32) {
    this.values = [];
    this.count = count;
  }

  get(index: u32): T {
    return this.values[index as i32];
  }

  set(index: u32, value: T): void {
    if (index === (this.values.length as u32)) {
      this.values.push(value);
    } else {
      this.values[index as i32] = value;
    }
  }

  length(): u32 {
    return this.count;
  }
}

/**
 * Declares a private module variable as the initializer of a module-level `const`.
 * The generator folds `init`, and an initializer it cannot fold is a K20 diagnostic.
 */
export function privateVar<T>(init: T): PrivateVar<T> {
  return new PrivateVar<T>(init);
}

/**
 * Declares a workgroup variable as the initializer of a module-level `const`.
 * WGSL gives a workgroup variable no initializer, so a kernel writes it before it reads it.
 */
export function workgroupVar<T>(): WorkgroupVar<T> {
  return new WorkgroupVar<T>();
}

/**
 * Declares a workgroup array of `n` elements as the initializer of a module-level `const`.
 * `n` must be a literal, because the generator writes it into the WGSL array type.
 */
export function workgroupArray<T>(n: u32): WorkgroupArray<T> {
  return new WorkgroupArray<T>(n);
}

/**
 * A workgroup execution and memory barrier.
 * The call is legal as a statement in a kernel body under uniform control flow, never in a helper.
 * The host body does nothing, so a kernel that reaches it is not host-runnable (CL2).
 */
export function workgroupBarrier(): void {}

/**
 * A storage memory barrier.
 * The call is legal as a statement in a kernel body under uniform control flow, never in a helper.
 * The host body does nothing, so a kernel that reaches it is not host-runnable (CL2).
 */
export function storageBarrier(): void {}

/**
 * The descriptor of a WGSL shell.
 * `body` holds the WGSL statements the generator inserts as the function body, and it must be
 * a string literal.
 */
@Descriptor
export class WgslShellSpec {
  body!: string;
}

/**
 * Marks a module-level function as a WGSL shell.
 * The generator writes the WGSL signature from the function's types and inserts `spec.body`,
 * and it never walks the subscript body.
 * That subscript body stays as the host implementation of the same function.
 */
export function wgslShell<F>(fn: F, spec: WgslShellSpec): WgslShellSpec {
  return spec;
}

/**
 * Adds raw WGSL text above the generated declarations of every module the program emits.
 * A program holds at most one call, at module level.
 * A lexical fence rejects `override`, a barrier, `@group`, `@binding`, and `var<`.
 */
export function wgslDeclarations(text: string): void {}

/**
 * The descriptor of a compute pipeline declaration.
 * `workgroupSize` holds the three literal workgroup dimensions.
 * `guarded` adds a hidden uniform binding that clips the kernel to the dispatched thread counts.
 */
@Descriptor
export class ComputePipelineSpec {
  workgroupSize!: FixedArray<u32, 3>;
  name?: string = "";
  guarded?: boolean = false;
}

/**
 * Declares a compute pipeline over one layout class, as a module-level `const`.
 * The generator reads the kernel from the function reference and the workgroup size from the
 * literal spec.
 * `guarded` is legal on this form only.
 */
export function computePipeline<L>(
  kernel: (res: L, ctx: ComputeInvocation) => void,
  spec: ComputePipelineSpec,
): ComputePipelineSpec {
  return { workgroupSize: spec.workgroupSize, name: spec.name, guarded: spec.guarded };
}

/**
 * Declares a compute pipeline over two layout classes, as a module-level `const`.
 * Bind group index is parameter order, so `L0` is group 0 and `L1` is group 1.
 * `guarded` on this form is a diagnostic (PI15).
 */
export function computePipeline2<L0, L1>(
  kernel: (res0: L0, res1: L1, ctx: ComputeInvocation) => void,
  spec: ComputePipelineSpec,
): ComputePipelineSpec {
  return { workgroupSize: spec.workgroupSize, name: spec.name, guarded: spec.guarded };
}

/**
 * Declares a compute pipeline over three layout classes, as a module-level `const`.
 * Bind group index is parameter order, so `L0` is group 0 and `L2` is group 2.
 * `guarded` on this form is a diagnostic (PI15).
 */
export function computePipeline3<L0, L1, L2>(
  kernel: (res0: L0, res1: L1, res2: L2, ctx: ComputeInvocation) => void,
  spec: ComputePipelineSpec,
): ComputePipelineSpec {
  return { workgroupSize: spec.workgroupSize, name: spec.name, guarded: spec.guarded };
}

/**
 * Declares a compute pipeline over four layout classes, as a module-level `const`.
 * Bind group index is parameter order, so `L0` is group 0 and `L3` is group 3.
 * `guarded` on this form is a diagnostic (PI15).
 */
export function computePipeline4<L0, L1, L2, L3>(
  kernel: (res0: L0, res1: L1, res2: L2, res3: L3, ctx: ComputeInvocation) => void,
  spec: ComputePipelineSpec,
): ComputePipelineSpec {
  return { workgroupSize: spec.workgroupSize, name: spec.name, guarded: spec.guarded };
}

function requireHostRunnable(
  method: string,
  spec: ComputePipelineSpec,
  hostRunnable: boolean,
): void {
  if (!hostRunnable) {
    authorTrap("CL2", method, `pipeline=${spec.name}`);
  }
}

function hostInvocation(
  spec: ComputePipelineSpec,
  workgroups: FixedArray<u32, 3>,
  workgroupX: u32,
  workgroupY: u32,
  workgroupZ: u32,
  localX: u32,
  localY: u32,
  localZ: u32,
): ComputeInvocation {
  return new ComputeInvocation(
    new Vec3u(
      workgroupX * spec.workgroupSize[0] + localX,
      workgroupY * spec.workgroupSize[1] + localY,
      workgroupZ * spec.workgroupSize[2] + localZ,
    ),
    new Vec3u(localX, localY, localZ),
    new Vec3u(workgroupX, workgroupY, workgroupZ),
    new Vec3u(workgroups[0], workgroups[1], workgroups[2]),
    (localZ * spec.workgroupSize[1] + localY) * spec.workgroupSize[0] + localX,
  );
}

function simulateComputeLoop<L>(
  kernel: (res: L, ctx: ComputeInvocation) => void,
  res: L,
  spec: ComputePipelineSpec,
  workgroups: FixedArray<u32, 3>,
  bounds: FixedArray<u32, 3>,
  applyBounds: boolean,
): void {
  for (let workgroupZ: u32 = 0; workgroupZ < workgroups[2]; workgroupZ += 1) {
    for (let workgroupY: u32 = 0; workgroupY < workgroups[1]; workgroupY += 1) {
      for (let workgroupX: u32 = 0; workgroupX < workgroups[0]; workgroupX += 1) {
        for (let localZ: u32 = 0; localZ < spec.workgroupSize[2]; localZ += 1) {
          for (let localY: u32 = 0; localY < spec.workgroupSize[1]; localY += 1) {
            for (let localX: u32 = 0; localX < spec.workgroupSize[0]; localX += 1) {
              const invocation: ComputeInvocation = hostInvocation(
                spec,
                workgroups,
                workgroupX,
                workgroupY,
                workgroupZ,
                localX,
                localY,
                localZ,
              );
              if (!applyBounds
                || !spec.guarded
                || (invocation.globalId.x < bounds[0]
                  && invocation.globalId.y < bounds[1]
                  && invocation.globalId.z < bounds[2])) {
                kernel(res, invocation);
              }
            }
          }
        }
      }
    }
  }
}

/**
 * Runs the kernel on the host over every invocation of `workgroups` workgroups, in row-major order.
 * Fill the wrappers in `res` before the call, and read them after it.
 * Pass the generated host-runnable constant as `hostRunnable`, because `false` traps with CL2.
 */
export function simulateCompute<L>(
  kernel: (res: L, ctx: ComputeInvocation) => void,
  res: L,
  spec: ComputePipelineSpec,
  workgroups: FixedArray<u32, 3>,
  hostRunnable: boolean,
): void {
  requireHostRunnable("simulateCompute", spec, hostRunnable);
  simulateComputeLoop<L>(kernel, res, spec, workgroups, [0, 0, 0], false);
}

/**
 * Runs the kernel on the host over `x`, `y`, and `z` threads, rounded up by the workgroup size
 * the way `dispatchThreads` rounds them.
 * For a guarded spec it skips every invocation whose global id is outside those counts.
 * A `hostRunnable` of `false` traps with CL2.
 */
export function simulateComputeThreads<L>(
  kernel: (res: L, ctx: ComputeInvocation) => void,
  res: L,
  spec: ComputePipelineSpec,
  x: u32,
  y: u32,
  z: u32,
  hostRunnable: boolean,
): void {
  requireHostRunnable("simulateComputeThreads", spec, hostRunnable);
  const workgroups: FixedArray<u32, 3> = [
    (x + spec.workgroupSize[0] - 1) / spec.workgroupSize[0],
    (y + spec.workgroupSize[1] - 1) / spec.workgroupSize[1],
    (z + spec.workgroupSize[2] - 1) / spec.workgroupSize[2],
  ];
  simulateComputeLoop<L>(kernel, res, spec, workgroups, [x, y, z], true);
}

/**
 * Runs a two-layout kernel on the host over every invocation of `workgroups` workgroups.
 * A `hostRunnable` of `false` traps with CL2.
 */
export function simulateCompute2<L0, L1>(
  kernel: (res0: L0, res1: L1, ctx: ComputeInvocation) => void,
  res0: L0,
  res1: L1,
  spec: ComputePipelineSpec,
  workgroups: FixedArray<u32, 3>,
  hostRunnable: boolean,
): void {
  requireHostRunnable("simulateCompute2", spec, hostRunnable);
  for (let workgroupZ: u32 = 0; workgroupZ < workgroups[2]; workgroupZ += 1) {
    for (let workgroupY: u32 = 0; workgroupY < workgroups[1]; workgroupY += 1) {
      for (let workgroupX: u32 = 0; workgroupX < workgroups[0]; workgroupX += 1) {
        for (let localZ: u32 = 0; localZ < spec.workgroupSize[2]; localZ += 1) {
          for (let localY: u32 = 0; localY < spec.workgroupSize[1]; localY += 1) {
            for (let localX: u32 = 0; localX < spec.workgroupSize[0]; localX += 1) {
              kernel(
                res0,
                res1,
                hostInvocation(
                  spec,
                  workgroups,
                  workgroupX,
                  workgroupY,
                  workgroupZ,
                  localX,
                  localY,
                  localZ,
                ),
              );
            }
          }
        }
      }
    }
  }
}

/**
 * Runs a three-layout kernel on the host over every invocation of `workgroups` workgroups.
 * A `hostRunnable` of `false` traps with CL2.
 */
export function simulateCompute3<L0, L1, L2>(
  kernel: (res0: L0, res1: L1, res2: L2, ctx: ComputeInvocation) => void,
  res0: L0,
  res1: L1,
  res2: L2,
  spec: ComputePipelineSpec,
  workgroups: FixedArray<u32, 3>,
  hostRunnable: boolean,
): void {
  requireHostRunnable("simulateCompute3", spec, hostRunnable);
  for (let workgroupZ: u32 = 0; workgroupZ < workgroups[2]; workgroupZ += 1) {
    for (let workgroupY: u32 = 0; workgroupY < workgroups[1]; workgroupY += 1) {
      for (let workgroupX: u32 = 0; workgroupX < workgroups[0]; workgroupX += 1) {
        for (let localZ: u32 = 0; localZ < spec.workgroupSize[2]; localZ += 1) {
          for (let localY: u32 = 0; localY < spec.workgroupSize[1]; localY += 1) {
            for (let localX: u32 = 0; localX < spec.workgroupSize[0]; localX += 1) {
              kernel(
                res0,
                res1,
                res2,
                hostInvocation(
                  spec,
                  workgroups,
                  workgroupX,
                  workgroupY,
                  workgroupZ,
                  localX,
                  localY,
                  localZ,
                ),
              );
            }
          }
        }
      }
    }
  }
}

/**
 * Runs a four-layout kernel on the host over every invocation of `workgroups` workgroups.
 * A `hostRunnable` of `false` traps with CL2.
 */
export function simulateCompute4<L0, L1, L2, L3>(
  kernel: (res0: L0, res1: L1, res2: L2, res3: L3, ctx: ComputeInvocation) => void,
  res0: L0,
  res1: L1,
  res2: L2,
  res3: L3,
  spec: ComputePipelineSpec,
  workgroups: FixedArray<u32, 3>,
  hostRunnable: boolean,
): void {
  requireHostRunnable("simulateCompute4", spec, hostRunnable);
  for (let workgroupZ: u32 = 0; workgroupZ < workgroups[2]; workgroupZ += 1) {
    for (let workgroupY: u32 = 0; workgroupY < workgroups[1]; workgroupY += 1) {
      for (let workgroupX: u32 = 0; workgroupX < workgroups[0]; workgroupX += 1) {
        for (let localZ: u32 = 0; localZ < spec.workgroupSize[2]; localZ += 1) {
          for (let localY: u32 = 0; localY < spec.workgroupSize[1]; localY += 1) {
            for (let localX: u32 = 0; localX < spec.workgroupSize[0]; localX += 1) {
              kernel(
                res0,
                res1,
                res2,
                res3,
                hostInvocation(
                  spec,
                  workgroups,
                  workgroupX,
                  workgroupY,
                  workgroupZ,
                  localX,
                  localY,
                  localZ,
                ),
              );
            }
          }
        }
      }
    }
  }
}

/**
 * The descriptor of a render pipeline declaration.
 * The generator reads `format` and `indexFormat`.
 * The runtime passes `topology`, `cullMode`, `frontFace`, and `blend` into the pipeline
 * descriptor.
 */
@Descriptor
export class RenderPipelineSpec {
  format!: GPUTextureFormat;
  topology?: GPUPrimitiveTopology = "triangle-list";
  cullMode?: GPUCullMode = "none";
  frontFace?: GPUFrontFace = "ccw";
  indexFormat?: GPUIndexFormat = "undefined";
  blend?: GPUBlendState | null = null;
}

function hostBlendComponent(
  source: f32,
  destination: f32,
  sourceAlpha: f32,
  component: GPUBlendComponent,
  channel: string,
): f32 {
  if (component.operation !== "add") {
    authorTrap("RN21",
      "hostBlend",
      `${channel} srcFactor=${component.srcFactor} dstFactor=${component.dstFactor} operation=${component.operation}`,
    );
  }
  let result: f32 = 0.0;
  if (component.srcFactor === "src-alpha"
    && component.dstFactor === "one-minus-src-alpha") {
    result = source * sourceAlpha + destination * (1.0 - sourceAlpha);
  } else if (component.srcFactor === "one" && component.dstFactor === "one") {
    result = source + destination;
  } else {
    authorTrap("RN21",
      "hostBlend",
      `${channel} srcFactor=${component.srcFactor} dstFactor=${component.dstFactor} operation=${component.operation}`,
    );
  }
  return Math.min(1.0, Math.max(0.0, result as f64)) as f32;
}

/**
 * Blends one source color over one destination for a host oracle, and clamps each channel to
 * the range 0 to 1.
 * A `blend` of `null` returns `source` unchanged.
 * This function accepts the `add` operation with `src-alpha`/`one-minus-src-alpha` or `one`/`one`
 * only, and traps with RN21 otherwise.
 */
export function hostBlend(
  source: Vec4f,
  destination: Vec4f,
  blend: GPUBlendState | null,
): Vec4f {
  if (blend === null) return source;
  return new Vec4f(
    hostBlendComponent(source.x, destination.x, source.w, blend.color, "color"),
    hostBlendComponent(source.y, destination.y, source.w, blend.color, "color"),
    hostBlendComponent(source.z, destination.z, source.w, blend.color, "color"),
    hostBlendComponent(source.w, destination.w, source.w, blend.alpha, "alpha"),
  );
}

/**
 * One attribute of a vertex buffer layout.
 * `offset` is a byte offset inside the element, which a program reads from the schema's field
 * offset constant.
 * `shaderLocation` is the WGSL `@location` index.
 */
@Descriptor
export class VertexAttributeSpec {
  format!: GPUVertexFormat;
  offset!: u64;
  shaderLocation!: u32;
}

/**
 * The layout of one vertex buffer slot.
 * `arrayStride` is the element stride in bytes, which is the schema's stride constant.
 * `stepMode` is `vertex` for the vertex schema and `instance` for the instance schema.
 */
@Descriptor
export class VertexBufferLayoutSpec {
  arrayStride!: u64;
  stepMode?: GPUVertexStepMode = "vertex";
  attributes!: VertexAttributeSpec[];
}

/**
 * Declares a render pipeline with no bindings, as a module-level `const`.
 * `V` is the vertex schema, and `O` is the varyings class, which needs a `position: Vec4f` field.
 * The generator reads both kernels and the literal spec.
 */
export function renderPipeline<V, O>(
  vertex: (value: V, ctx: VertexInvocation) => O,
  fragment: (input: O, ctx: FragmentInvocation) => Vec4f,
  spec: RenderPipelineSpec,
): RenderPipelineSpec {
  return {
    format: spec.format,
    topology: spec.topology,
    cullMode: spec.cullMode,
    frontFace: spec.frontFace,
    indexFormat: spec.indexFormat,
    blend: spec.blend,
  };
}

/**
 * Declares a render pipeline whose two kernels read one layout class, as a module-level `const`.
 * The bindings form group 0, and the kernels that reach a binding decide its visibility.
 * A binding no kernel reaches is a diagnostic (RN9).
 */
export function renderPipelineL<L, V, O>(
  vertex: (res: L, value: V, ctx: VertexInvocation) => O,
  fragment: (res: L, input: O, ctx: FragmentInvocation) => Vec4f,
  spec: RenderPipelineSpec,
): RenderPipelineSpec {
  return {
    format: spec.format,
    topology: spec.topology,
    cullMode: spec.cullMode,
    frontFace: spec.frontFace,
    indexFormat: spec.indexFormat,
    blend: spec.blend,
  };
}

/**
 * Declares a render pipeline with a second vertex buffer for per-instance data.
 * The instance schema `I` takes slot 1 with the step mode `instance`.
 * Its attribute locations continue after the locations of `V`.
 */
export function renderPipelineInstanced<V, I, O>(
  vertex: (value: V, instance: I, ctx: VertexInvocation) => O,
  fragment: (input: O, ctx: FragmentInvocation) => Vec4f,
  spec: RenderPipelineSpec,
): RenderPipelineSpec {
  return {
    format: spec.format,
    topology: spec.topology,
    cullMode: spec.cullMode,
    frontFace: spec.frontFace,
    indexFormat: spec.indexFormat,
    blend: spec.blend,
  };
}

/**
 * One binding of a generated bind group layout.
 * `kind` is `uniform`, `read-only-storage`, `storage`, `texture`, `storageTexture`, `sampler`,
 * `comparisonSampler`, or `guard`.
 * `visibility` is a shader stage mask, and `minBindingSize` is the byte size the layout engine
 * computes.
 */
@Descriptor
export class BindGroupLayoutEntrySpec {
  binding!: u32;
  visibility!: u64;
  kind!: string;
  minBindingSize!: u64;
  sampleType?: GPUTextureSampleType = "float";
  format?: GPUTextureFormat;
  access?: GPUStorageTextureAccess = "write-only";
  viewDimension?: GPUTextureViewDimension = "2d";
  samplerType?: GPUSamplerBindingType = "filtering";
}

/**
 * The resource of one binding.
 * Exactly one of the three fields is not `null`, and any other count traps with TX4.
 * Build one with `bufferResource`, `textureResource`, or `samplerResource`.
 */
@Descriptor
export class BindingResource {
  buffer?: GPUBuffer | null = null;
  textureView?: GPUTextureView | null = null;
  sampler?: GPUSampler | null = null;
}

/**
 * Wraps a buffer as the resource of a `uniform`, `storage`, or `read-only-storage` binding.
 */
export function bufferResource(buffer: GPUBuffer): BindingResource {
  return { buffer, textureView: null, sampler: null };
}

/**
 * Wraps a texture view as the resource of a `texture` or `storageTexture` binding.
 */
export function textureResource(textureView: GPUTextureView): BindingResource {
  return { buffer: null, textureView, sampler: null };
}

/**
 * Wraps a sampler as the resource of a `sampler` or `comparisonSampler` binding.
 */
export function samplerResource(sampler: GPUSampler): BindingResource {
  return { buffer: null, textureView: null, sampler };
}

/**
 * One generated bind group layout, which the support module exports for each group.
 * The entries follow the layout class's field order, and a hidden `guard` entry comes last.
 */
@Descriptor
export class BindGroupLayoutSpec {
  entries!: BindGroupLayoutEntrySpec[];
}

/**
 * The compute pipeline a program dispatches.
 * It owns the WebGPU pipeline and every guard buffer, and `dispose()` releases them.
 * A guarded pipeline writes its bounds through the queue, so one command encoder carries at most
 * one guarded dispatch (PI15).
 */
export class ComputePipeline {
  private pipeline: GPUComputePipeline;
  private guardQueue: GPUQueue | null;
  private guardQueueOwned: boolean;
  private workgroup: FixedArray<u32, 3>;
  private guarded: boolean;
  private guardGroups: u32[];
  private guardBuffers: GPUBuffer[];
  private guardEncoder: GPUCommandEncoder | null;

  constructor(
    pipeline: GPUComputePipeline,
    workgroup: FixedArray<u32, 3>,
    guardQueue: GPUQueue | null = null,
    guardQueueOwned: boolean = false,
    guarded: boolean = false,
    guardGroups: u32[] = [],
    guardBuffers: GPUBuffer[] = [],
  ) {
    this.pipeline = pipeline;
    this.guardQueue = guardQueue;
    this.guardQueueOwned = guardQueueOwned;
    this.workgroup = workgroup;
    this.guarded = guarded;
    this.guardGroups = guardGroups;
    this.guardBuffers = guardBuffers;
    this.guardEncoder = null;
  }

  bindGroupLayout(group: u32): GPUBindGroupLayout {
    return this.pipeline.getBindGroupLayout(group);
  }

  guardBuffer(group: u32): GPUBuffer | null {
    let index: i32 = 0;
    while (index < this.guardGroups.length) {
      if (this.guardGroups[index] === group) {
        return this.guardBuffers[index];
      }
      index = index + 1;
    }
    return null;
  }

  private writeGuard(
    encoder: GPUCommandEncoder,
    method: string,
    x: u32,
    y: u32,
    z: u32,
  ): void {
    if (!this.guarded) return;
    if (this.guardEncoder !== null) {
      if (this.guardEncoder === encoder) {
        authorTrap("PI15", method, `x=${x} y=${y} z=${z}`);
        return;
      }
    }
    if (this.guardQueue === null) {
      authorTrap("PI15", "ComputePipeline.guard", "queue=missing");
      return;
    }
    this.guardEncoder = encoder;
    const bytes: u8[] = Context.bytesOf<FixedArray<u32, 4>>([x, y, z, 0]);
    let index: i32 = 0;
    while (index < this.guardBuffers.length) {
      this.guardQueue.writeBuffer(this.guardBuffers[index], 0, bytes);
      index = index + 1;
    }
  }

  private recordDispatch(
    encoder: GPUCommandEncoder,
    groups: GPUBindGroup[],
    x: u32,
    y: u32,
    z: u32,
  ): void {
    using pass = encoder.beginComputePassDefault();
    pass.setPipeline(this.pipeline);
    let group: i32 = 0;
    while (group < groups.length) {
      pass.setBindGroup(group as u32, groups[group]);
      group = group + 1;
    }
    pass.dispatchWorkgroups(x, y, z);
    pass.end();
  }

  dispatch(
    encoder: GPUCommandEncoder,
    groups: GPUBindGroup[],
    x: u32,
    y: u32,
    z: u32,
  ): void {
    this.writeGuard(
      encoder,
      "ComputePipeline.dispatch",
      x * this.workgroup[0],
      y * this.workgroup[1],
      z * this.workgroup[2],
    );
    this.recordDispatch(encoder, groups, x, y, z);
  }

  dispatchThreads(
    encoder: GPUCommandEncoder,
    groups: GPUBindGroup[],
    x: u32,
    y: u32,
    z: u32,
  ): void {
    this.writeGuard(encoder, "ComputePipeline.dispatchThreads", x, y, z);
    this.recordDispatch(
      encoder,
      groups,
      (x + this.workgroup[0] - 1) / this.workgroup[0],
      (y + this.workgroup[1] - 1) / this.workgroup[1],
      (z + this.workgroup[2] - 1) / this.workgroup[2],
    );
  }

  dispatchIndirect(
    encoder: GPUCommandEncoder,
    groups: GPUBindGroup[],
    buffer: GPUBuffer,
    offset: u64,
  ): void {
    if (this.guarded) {
      authorTrap("PI16", "ComputePipeline.dispatchIndirect", "guarded=true");
      return;
    }
    using pass = encoder.beginComputePassDefault();
    pass.setPipeline(this.pipeline);
    let group: i32 = 0;
    while (group < groups.length) {
      pass.setBindGroup(group as u32, groups[group]);
      group = group + 1;
    }
    pass.dispatchWorkgroupsIndirect(buffer, offset);
    pass.end();
  }

  dispatchTimed(
    encoder: GPUCommandEncoder,
    groups: GPUBindGroup[],
    x: u32,
    y: u32,
    z: u32,
    pair: TimestampPair,
  ): void {
    this.writeGuard(
      encoder,
      "ComputePipeline.dispatchTimed",
      x * this.workgroup[0],
      y * this.workgroup[1],
      z * this.workgroup[2],
    );
    using pass = encoder.beginComputePass({
      timestampWrites: {
        querySet: pair.querySet(),
        beginningOfPassWriteIndex: 0,
        endOfPassWriteIndex: 1,
      },
    });
    pass.setPipeline(this.pipeline);
    let group: i32 = 0;
    while (group < groups.length) {
      pass.setBindGroup(group as u32, groups[group]);
      group = group + 1;
    }
    pass.dispatchWorkgroups(x, y, z);
    pass.end();
  }

  dispose(): void {
    let index: i32 = 0;
    while (index < this.guardBuffers.length) {
      this.guardBuffers[index].dispose();
      index = index + 1;
    }
    if (this.guardQueue !== null) {
      if (this.guardQueueOwned) {
        this.guardQueue.dispose();
      }
      this.guardQueue = null;
      this.guardQueueOwned = false;
    }
    this.pipeline.dispose();
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

/**
 * A pair of timestamp queries and the 16-byte buffer they resolve into.
 * `resolve` records the resolve, and `copyTo` moves the 16 bytes into a readback buffer.
 * `dispose()` destroys the query set and releases the resolve buffer.
 */
export class TimestampPair {
  private queries: GPUQuerySet;
  private resolved: GPUBuffer;

  constructor(queries: GPUQuerySet, resolved: GPUBuffer) {
    this.queries = queries;
    this.resolved = resolved;
  }

  querySet(): GPUQuerySet {
    return this.queries;
  }

  resolve(encoder: GPUCommandEncoder): void {
    encoder.resolveQuerySet(this.queries, 0, 2, this.resolved, 0);
  }

  copyTo(
    encoder: GPUCommandEncoder,
    readback: Buffer<FixedArray<u64, 2>>,
  ): void {
    encoder.copyBufferToBuffer(this.resolved, 0, readback.handle(), 0, 16);
  }

  dispose(): void {
    this.queries.destroy();
    this.queries.dispose();
    this.resolved.dispose();
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

/**
 * Creates a `TimestampPair` on the device, which the caller disposes.
 * When the device lacks the `timestamp-query` feature, this function returns `null`, so a
 * program can continue without timestamps.
 */
export function createTimestampPair(device: GPUDevice): TimestampPair | null {
  if (!device.hasFeature("timestamp-query")) {
    return null;
  }
  const queries: GPUQuerySet = device.createQuerySet({
    label: "typegpu-timestamps",
    type: "timestamp",
    count: 2,
  });
  const resolved: GPUBuffer = device.createBuffer({
    label: "typegpu-timestamp-resolve",
    size: 16,
    usage: GPUBufferUsage.QUERY_RESOLVE + GPUBufferUsage.COPY_SRC,
  });
  return new TimestampPair(queries, resolved);
}

/**
 * The render pipeline a program binds into a render pass.
 * `bind` sets the pipeline, the bind groups, and each vertex buffer at its full size.
 * When the declaration names no `indexFormat`, `setIndexBuffer` traps with RN18.
 */
export class RenderPipeline {
  private pipeline: GPURenderPipeline;
  private indexFormat: GPUIndexFormat;

  constructor(pipeline: GPURenderPipeline, indexFormat: GPUIndexFormat) {
    this.pipeline = pipeline;
    this.indexFormat = indexFormat;
  }

  bindGroupLayout(group: u32): GPUBindGroupLayout {
    return this.pipeline.getBindGroupLayout(group);
  }

  bind(
    pass: GPURenderPassEncoder,
    groups: GPUBindGroup[],
    vertexBuffers: GPUBuffer[],
  ): void {
    pass.setPipeline(this.pipeline);
    let group: i32 = 0;
    while (group < groups.length) {
      pass.setBindGroup(group as u32, groups[group]);
      group = group + 1;
    }
    let slot: i32 = 0;
    while (slot < vertexBuffers.length) {
      pass.setVertexBuffer(slot as u32, vertexBuffers[slot], 0, vertexBuffers[slot].size);
      slot = slot + 1;
    }
  }

  setIndexBuffer(pass: GPURenderPassEncoder, buffer: GPUBuffer): void {
    if (this.indexFormat === "undefined") {
      authorTrap("RN18", "RenderPipeline.setIndexBuffer", "indexFormat=undefined");
      return;
    }
    pass.setIndexBuffer(buffer, this.indexFormat, 0, buffer.size);
  }

  dispose(): void {
    this.pipeline.dispose();
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

function nativeBindGroupLayoutEntries(
  layouts: BindGroupLayoutSpec[],
): GPUBindGroupLayoutEntry[][] {
  const groups: GPUBindGroupLayoutEntry[][] = [];
  let group: i32 = 0;
  while (group < layouts.length) {
    const entries: GPUBindGroupLayoutEntry[] = [];
    let binding: i32 = 0;
    while (binding < layouts[group].entries.length) {
      const source: BindGroupLayoutEntrySpec = layouts[group].entries[binding];
      if (source.kind === "uniform" || source.kind === "guard") {
        entries.push({
          binding: source.binding,
          visibility: source.visibility,
          buffer: { type: "uniform", minBindingSize: source.minBindingSize },
        });
      } else if (source.kind === "read-only-storage") {
        entries.push({
          binding: source.binding,
          visibility: source.visibility,
          buffer: { type: "read-only-storage", minBindingSize: source.minBindingSize },
        });
      } else if (source.kind === "storage") {
        entries.push({
          binding: source.binding,
          visibility: source.visibility,
          buffer: { type: "storage", minBindingSize: source.minBindingSize },
        });
      } else if (source.kind === "texture") {
        entries.push({
          binding: source.binding,
          visibility: source.visibility,
          texture: {
            sampleType: source.sampleType,
            viewDimension: source.viewDimension,
            multisampled: false,
          },
        });
      } else if (source.kind === "storageTexture") {
        if (source.format !== undefined) {
          entries.push({
            binding: source.binding,
            visibility: source.visibility,
            storageTexture: {
              access: source.access,
              format: source.format,
              viewDimension: source.viewDimension,
            },
          });
        } else {
          authorTrap("TX5", "storageTexture", `binding=${source.binding} has no format`);
        }
      } else if (source.kind === "sampler" || source.kind === "comparisonSampler") {
        entries.push({
          binding: source.binding,
          visibility: source.visibility,
          sampler: { type: source.samplerType },
        });
      } else {
        authorTrap("TX5", "bind group layout", `binding=${source.binding} has unknown kind=${source.kind}`);
      }
      binding = binding + 1;
    }
    groups.push(entries);
    group = group + 1;
  }
  return groups;
}

function createNativeBindGroupLayouts(
  device: GPUDevice,
  entries: GPUBindGroupLayoutEntry[][],
): GPUBindGroupLayout[] {
  const nativeLayouts: GPUBindGroupLayout[] = [];
  let group: i32 = 0;
  while (group < entries.length) {
    nativeLayouts.push(device.createBindGroupLayout({ entries: entries[group] }));
    group = group + 1;
  }
  return nativeLayouts;
}

function createNativeBindGroupLayoutsHost(
  device: GPUHostOwnedDevice,
  entries: GPUBindGroupLayoutEntry[][],
): GPUBindGroupLayout[] {
  const nativeLayouts: GPUBindGroupLayout[] = [];
  let group: i32 = 0;
  while (group < entries.length) {
    nativeLayouts.push(device.createBindGroupLayout({ entries: entries[group] }));
    group = group + 1;
  }
  return nativeLayouts;
}

function computePipelineDescriptor(
  shader: GPUShaderModule,
  layout: GPUPipelineLayout,
  entry: string,
): GPUComputePipelineDescriptor {
  return {
    layout,
    compute: { module: shader, entryPoint: entry },
  };
}

function nativeVertexBufferLayouts(
  vertexLayouts: VertexBufferLayoutSpec[],
): GPUVertexBufferLayout[] {
  const nativeVertexLayouts: GPUVertexBufferLayout[] = [];
  let slot: i32 = 0;
  while (slot < vertexLayouts.length) {
    const attributes: GPUVertexAttribute[] = [];
    let attribute: i32 = 0;
    while (attribute < vertexLayouts[slot].attributes.length) {
      const source: VertexAttributeSpec = vertexLayouts[slot].attributes[attribute];
      attributes.push({
        format: source.format,
        offset: source.offset,
        shaderLocation: source.shaderLocation,
      });
      attribute = attribute + 1;
    }
    nativeVertexLayouts.push({
      arrayStride: vertexLayouts[slot].arrayStride,
      stepMode: vertexLayouts[slot].stepMode,
      attributes,
    });
    slot = slot + 1;
  }
  return nativeVertexLayouts;
}

function renderPipelineDescriptor(
  shader: GPUShaderModule,
  layout: GPUPipelineLayout,
  vertexEntry: string,
  fragmentEntry: string,
  vertexLayouts: GPUVertexBufferLayout[],
  spec: RenderPipelineSpec,
): GPURenderPipelineDescriptor {
  return {
    layout,
    vertex: { module: shader, entryPoint: vertexEntry, buffers: vertexLayouts },
    primitive: {
      topology: spec.topology,
      cullMode: spec.cullMode,
      frontFace: spec.frontFace,
    },
    fragment: {
      module: shader,
      entryPoint: fragmentEntry,
      targets: [{ format: spec.format, blend: spec.blend }],
    },
  };
}

function finishComputePipeline(
  pipeline: GPUComputePipeline,
  workgroup: FixedArray<u32, 3>,
  guardGroups: u32[],
  guardBuffers: GPUBuffer[],
  guardQueue: GPUQueue | null,
  guardQueueOwned: boolean,
): ComputePipeline {
  return new ComputePipeline(
    pipeline,
    workgroup,
    guardQueue,
    guardQueueOwned,
    guardBuffers.length > 0,
    guardGroups,
    guardBuffers,
  );
}

function finishRenderPipeline(
  pipeline: GPURenderPipeline,
  spec: RenderPipelineSpec,
): RenderPipeline {
  return new RenderPipeline(pipeline, spec.indexFormat);
}

/**
 * Creates a `ComputePipeline` from the generated WGSL, entry name, layouts, and workgroup size.
 * The call runs inside the caller's validation error scope and does not await.
 * It creates one guard buffer per `guard` layout entry, and the pipeline disposes them.
 */
export function createComputePipeline(
  device: GPUDevice,
  wgsl: string,
  entry: string,
  layouts: BindGroupLayoutSpec[],
  workgroup: FixedArray<u32, 3>,
): ComputePipeline {
  const guardGroups: u32[] = [];
  const guardBuffers: GPUBuffer[] = [];
  let guardGroup: i32 = 0;
  while (guardGroup < layouts.length) {
    let guardEntry: i32 = 0;
    while (guardEntry < layouts[guardGroup].entries.length) {
      if (layouts[guardGroup].entries[guardEntry].kind === "guard") {
        guardGroups.push(guardGroup as u32);
        guardBuffers.push(device.createBuffer({
          label: "typegpu-dispatch-guard",
          size: 16,
          usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
        }));
      }
      guardEntry = guardEntry + 1;
    }
    guardGroup = guardGroup + 1;
  }
  const layoutEntries: GPUBindGroupLayoutEntry[][] = nativeBindGroupLayoutEntries(layouts);
  const nativeLayouts: GPUBindGroupLayout[] = createNativeBindGroupLayouts(device, layoutEntries);
  using shader = device.createShaderModule({ code: wgsl });
  using layout = device.createPipelineLayout({ bindGroupLayouts: nativeLayouts });
  const pipeline = device.createComputePipeline(computePipelineDescriptor(shader, layout, entry));
  let group: i32 = 0;
  while (group < nativeLayouts.length) {
    nativeLayouts[group].dispose();
    group = group + 1;
  }
  const guardQueue: GPUQueue | null = guardBuffers.length > 0 ? device.queue : null;
  return finishComputePipeline(
    pipeline,
    workgroup,
    guardGroups,
    guardBuffers,
    guardQueue,
    false,
  );
}

/**
 * Creates a `ComputePipeline` through a device the host application owns.
 * The call runs inside the caller's validation error scope and does not await.
 * The pipeline owns the queue wrapper it takes for a guarded dispatch, and disposes that wrapper.
 */
export function createComputePipelineHost(
  device: GPUHostOwnedDevice,
  wgsl: string,
  entry: string,
  layouts: BindGroupLayoutSpec[],
  workgroup: FixedArray<u32, 3>,
): ComputePipeline {
  const guardGroups: u32[] = [];
  const guardBuffers: GPUBuffer[] = [];
  let guardGroup: i32 = 0;
  while (guardGroup < layouts.length) {
    let guardEntry: i32 = 0;
    while (guardEntry < layouts[guardGroup].entries.length) {
      if (layouts[guardGroup].entries[guardEntry].kind === "guard") {
        guardGroups.push(guardGroup as u32);
        guardBuffers.push(device.createBuffer({
          label: "typegpu-dispatch-guard",
          size: 16,
          usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
        }));
      }
      guardEntry = guardEntry + 1;
    }
    guardGroup = guardGroup + 1;
  }
  const layoutEntries: GPUBindGroupLayoutEntry[][] = nativeBindGroupLayoutEntries(layouts);
  const nativeLayouts: GPUBindGroupLayout[] = createNativeBindGroupLayoutsHost(device, layoutEntries);
  using shader = device.createShaderModule({ code: wgsl });
  using layout = device.createPipelineLayout({ bindGroupLayouts: nativeLayouts });
  const pipeline = device.createComputePipeline(computePipelineDescriptor(shader, layout, entry));
  let group: i32 = 0;
  while (group < nativeLayouts.length) {
    nativeLayouts[group].dispose();
    group = group + 1;
  }
  const guardQueue: GPUQueue | null = guardBuffers.length > 0 ? device.queue() : null;
  return finishComputePipeline(
    pipeline,
    workgroup,
    guardGroups,
    guardBuffers,
    guardQueue,
    true,
  );
}

/**
 * Creates a `RenderPipeline` from the generated WGSL, entry names, layouts, and vertex layouts.
 * The call runs inside the caller's validation error scope and does not await.
 * The caller disposes the result.
 */
export function createRenderPipeline(
  device: GPUDevice,
  wgsl: string,
  vertexEntry: string,
  fragmentEntry: string,
  layouts: BindGroupLayoutSpec[],
  vertexLayouts: VertexBufferLayoutSpec[],
  spec: RenderPipelineSpec,
): RenderPipeline {
  const layoutEntries: GPUBindGroupLayoutEntry[][] = nativeBindGroupLayoutEntries(layouts);
  const nativeLayouts: GPUBindGroupLayout[] = createNativeBindGroupLayouts(device, layoutEntries);
  let group: i32 = 0;
  const nativeVertexLayouts: GPUVertexBufferLayout[] = nativeVertexBufferLayouts(vertexLayouts);
  using shader = device.createShaderModule({ code: wgsl });
  using layout = device.createPipelineLayout({ bindGroupLayouts: nativeLayouts });
  const pipeline = device.createRenderPipeline(renderPipelineDescriptor(
    shader,
    layout,
    vertexEntry,
    fragmentEntry,
    nativeVertexLayouts,
    spec,
  ));
  group = 0;
  while (group < nativeLayouts.length) {
    nativeLayouts[group].dispose();
    group = group + 1;
  }
  return finishRenderPipeline(pipeline, spec);
}

/**
 * Creates a `RenderPipeline` through a device the host application owns.
 * The call runs inside the caller's validation error scope and does not await.
 * The caller disposes the result.
 */
export function createRenderPipelineHost(
  device: GPUHostOwnedDevice,
  wgsl: string,
  vertexEntry: string,
  fragmentEntry: string,
  layouts: BindGroupLayoutSpec[],
  vertexLayouts: VertexBufferLayoutSpec[],
  spec: RenderPipelineSpec,
): RenderPipeline {
  const layoutEntries: GPUBindGroupLayoutEntry[][] = nativeBindGroupLayoutEntries(layouts);
  const nativeLayouts: GPUBindGroupLayout[] = createNativeBindGroupLayoutsHost(device, layoutEntries);
  const nativeVertexLayouts: GPUVertexBufferLayout[] = nativeVertexBufferLayouts(vertexLayouts);
  using shader = device.createShaderModule({ code: wgsl });
  using layout = device.createPipelineLayout({ bindGroupLayouts: nativeLayouts });
  const pipeline = device.createRenderPipeline(renderPipelineDescriptor(
    shader,
    layout,
    vertexEntry,
    fragmentEntry,
    nativeVertexLayouts,
    spec,
  ));
  let group: i32 = 0;
  while (group < nativeLayouts.length) {
    nativeLayouts[group].dispose();
    group = group + 1;
  }
  return finishRenderPipeline(pipeline, spec);
}

/**
 * The shader stage masks a generated bind group layout entry carries in `visibility`.
 * A binding that two stages reach carries the sum of the two masks.
 */
export const COMPUTE_VISIBILITY: u64 = GPUShaderStage.COMPUTE;
export const VERTEX_VISIBILITY: u64 = GPUShaderStage.VERTEX;
export const FRAGMENT_VISIBILITY: u64 = GPUShaderStage.FRAGMENT;

function bindGroupEntries(
  spec: BindGroupLayoutSpec,
  resources: BindingResource[],
  guardBuffer: GPUBuffer | null = null,
): GPUBindGroupEntry[] {
  let authorCount: i32 = 0;
  let countIndex: i32 = 0;
  while (countIndex < spec.entries.length) {
    if (spec.entries[countIndex].kind !== "guard") authorCount = authorCount + 1;
    countIndex = countIndex + 1;
  }
  if (authorCount !== resources.length) {
    authorTrap("PI9", "createBindGroup", `expected ${authorCount} resources but received ${resources.length}`);
  }
  const entries: GPUBindGroupEntry[] = [];
  let index: i32 = 0;
  let resourceIndex: i32 = 0;
  while (index < spec.entries.length) {
    const specEntry: BindGroupLayoutEntrySpec = spec.entries[index];
    let resource: BindingResource = { buffer: null, textureView: null, sampler: null };
    if (specEntry.kind === "guard") {
      if (guardBuffer === null) {
        authorTrap("PI15", "createBindGroup", `binding=${specEntry.binding} has no guard buffer`);
      } else {
        resource = bufferResource(guardBuffer);
      }
    } else {
      resource = resources[resourceIndex];
      resourceIndex = resourceIndex + 1;
    }
    let actual: string = "none";
    let fieldCount: u32 = 0;
    if (resource.buffer !== null) {
      actual = "buffer";
      fieldCount = fieldCount + 1;
    }
    if (resource.textureView !== null) {
      actual = "texture";
      fieldCount = fieldCount + 1;
    }
    if (resource.sampler !== null) {
      actual = "sampler";
      fieldCount = fieldCount + 1;
    }
    if (fieldCount !== 1) {
      authorTrap("TX4", "createBindGroup", `binding=${specEntry.binding} resourceFields=${fieldCount}`);
    }
    let expected: string = "unknown";
    if (specEntry.kind === "uniform" || specEntry.kind === "read-only-storage"
      || specEntry.kind === "guard"
      || specEntry.kind === "storage") expected = "buffer";
    if (specEntry.kind === "texture" || specEntry.kind === "storageTexture") expected = "texture";
    if (specEntry.kind === "sampler" || specEntry.kind === "comparisonSampler") expected = "sampler";
    if (expected === "unknown") {
      authorTrap("TX5", "createBindGroup", `binding=${specEntry.binding} has unknown kind=${specEntry.kind}`);
    }
    if (actual !== expected) {
      authorTrap("TX4", "createBindGroup", `binding=${specEntry.binding} expected=${expected} actual=${actual}`);
    }
    entries.push({
      binding: specEntry.binding,
      buffer: resource.buffer,
      textureView: resource.textureView,
      sampler: resource.sampler,
    });
    index = index + 1;
  }
  return entries;
}

/**
 * Creates a bind group from a generated layout spec and one resource per authored binding, in
 * declaration order.
 * A resource count that differs from the authored binding count traps with PI9.
 * A `guard` entry takes `guardBuffer` and consumes no resource.
 */
export function createBindGroup(
  device: GPUDevice,
  layout: GPUBindGroupLayout,
  spec: BindGroupLayoutSpec,
  resources: BindingResource[],
  guardBuffer: GPUBuffer | null = null,
): GPUBindGroup {
  return device.createBindGroup({
    layout,
    entries: bindGroupEntries(spec, resources, guardBuffer),
  });
}

/**
 * Creates a bind group through a device the host application owns.
 * The resource order and the checks are the order and the checks of `createBindGroup`.
 */
export function createBindGroupHost(
  device: GPUHostOwnedDevice,
  layout: GPUBindGroupLayout,
  spec: BindGroupLayoutSpec,
  resources: BindingResource[],
  guardBuffer: GPUBuffer | null = null,
): GPUBindGroup {
  return device.createBindGroup({
    layout,
    entries: bindGroupEntries(spec, resources, guardBuffer),
  });
}
