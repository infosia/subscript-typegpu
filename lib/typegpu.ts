/** Runtime bindings shared by generated TypeGPU support modules and programs. */

import { Vec2f, Vec2i, Vec2u, Vec3u, Vec4f } from "./typegpu-types";
import {
  GPUBindGroup,
  GPUBindGroupEntry,
  GPUBindGroupLayout,
  GPUBindGroupLayoutEntry,
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
  GPUVertexAttribute,
  GPUVertexBufferLayout,
} from "./webgpu";

function authorTrap(rule: string, method: string, values: string): void {
  print(`${rule} ${method} ${values} (author)`);
  unreachable();
}

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

export class Sampler {
  private filterMode: string;

  constructor(filterMode: string) {
    this.filterMode = filterMode;
  }

  isNearest(): boolean {
    return this.filterMode === "nearest";
  }
}

export function samplerFromDescriptor(descriptor: GPUSamplerDescriptor): Sampler {
  if (descriptor.minFilter === "nearest" && descriptor.magFilter === "nearest") {
    return new Sampler("nearest");
  }
  return new Sampler("non-nearest");
}

export class Rgba8unorm {}
export class Rgba16float {}
export class R32float {}
export class Rgba32float {}

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
    device.queue().submit([command]);
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

export function readOne<T>(readback: Buffer<T>, elementIndex: u32): u8[] {
  if (elementIndex >= readback.count) {
    authorTrap("BF8", "readOne", `elementIndex=${elementIndex} elementCount=1 count=${readback.count}`);
  }
  return readback.buffer.readMappedRange(
    (elementIndex as u64) * (readback.elementSize as u64),
    readback.elementSize as u64,
  );
}

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

export class VertexInvocation {
  vertexIndex!: u32;
  instanceIndex!: u32;
}

export class FragmentInvocation {
  frontFacing!: boolean;
}

export class Uniform<T> {
  private values: T[];

  constructor(value: T) {
    this.values = [value];
  }

  get(): T {
    return this.values[0];
  }
}

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

export class PrivateVar<T> {
  private value: T;

  constructor(value: T) {
    this.value = value;
  }

  get(): T {
    return this.value;
  }

  set(value: T): void {
    this.value = value;
  }
}

export class WorkgroupVar<T> {
  private values: T[];

  constructor() {
    this.values = [];
  }

  get(): T {
    return this.values[0];
  }

  set(value: T): void {
    if (this.values.length === 0) {
      this.values.push(value);
    } else {
      this.values[0] = value;
    }
  }
}

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

export function privateVar<T>(init: T): PrivateVar<T> {
  return new PrivateVar<T>(init);
}

export function workgroupVar<T>(): WorkgroupVar<T> {
  return new WorkgroupVar<T>();
}

export function workgroupArray<T>(n: u32): WorkgroupArray<T> {
  return new WorkgroupArray<T>(n);
}

export function workgroupBarrier(): void {}

export function storageBarrier(): void {}

@Descriptor
export class WgslShellSpec {
  body!: string;
}

export function wgslShell<F>(fn: F, spec: WgslShellSpec): WgslShellSpec {
  return spec;
}

export function wgslDeclarations(text: string): void {}

@Descriptor
export class ComputePipelineSpec {
  workgroupSize!: FixedArray<u32, 3>;
  name?: string = "";
  guarded?: boolean = false;
}

export function computePipeline<L>(
  kernel: (res: L, ctx: ComputeInvocation) => void,
  spec: ComputePipelineSpec,
): ComputePipelineSpec {
  return { workgroupSize: spec.workgroupSize, name: spec.name, guarded: spec.guarded };
}

export function computePipeline2<L0, L1>(
  kernel: (res0: L0, res1: L1, ctx: ComputeInvocation) => void,
  spec: ComputePipelineSpec,
): ComputePipelineSpec {
  return { workgroupSize: spec.workgroupSize, name: spec.name, guarded: spec.guarded };
}

export function computePipeline3<L0, L1, L2>(
  kernel: (res0: L0, res1: L1, res2: L2, ctx: ComputeInvocation) => void,
  spec: ComputePipelineSpec,
): ComputePipelineSpec {
  return { workgroupSize: spec.workgroupSize, name: spec.name, guarded: spec.guarded };
}

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

@Descriptor
export class RenderPipelineSpec {
  format!: GPUTextureFormat;
  topology?: GPUPrimitiveTopology = "triangle-list";
  cullMode?: GPUCullMode = "none";
  frontFace?: GPUFrontFace = "ccw";
  indexFormat?: GPUIndexFormat = "undefined";
}

@Descriptor
export class VertexAttributeSpec {
  format!: GPUVertexFormat;
  offset!: u64;
  shaderLocation!: u32;
}

@Descriptor
export class VertexBufferLayoutSpec {
  arrayStride!: u64;
  stepMode?: GPUVertexStepMode = "vertex";
  attributes!: VertexAttributeSpec[];
}

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
  };
}

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
  };
}

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
  };
}

@Descriptor
export class BindGroupLayoutEntrySpec {
  binding!: u32;
  visibility!: u64;
  kind!: string;
  minBindingSize!: u64;
  sampleType?: GPUTextureSampleType = "float";
  format?: GPUTextureFormat;
  samplerType?: GPUSamplerBindingType = "filtering";
}

@Descriptor
export class BindingResource {
  buffer?: GPUBuffer | null = null;
  textureView?: GPUTextureView | null = null;
  sampler?: GPUSampler | null = null;
}

export function bufferResource(buffer: GPUBuffer): BindingResource {
  return { buffer, textureView: null, sampler: null };
}

export function textureResource(textureView: GPUTextureView): BindingResource {
  return { buffer: null, textureView, sampler: null };
}

export function samplerResource(sampler: GPUSampler): BindingResource {
  return { buffer: null, textureView: null, sampler };
}

@Descriptor
export class BindGroupLayoutSpec {
  entries!: BindGroupLayoutEntrySpec[];
}

export class ComputePipeline {
  private pipeline: GPUComputePipeline;
  private guardQueue: GPUQueue | null;
  private workgroup: FixedArray<u32, 3>;
  private guarded: boolean;
  private guardGroups: u32[];
  private guardBuffers: GPUBuffer[];
  private guardEncoder: GPUCommandEncoder | null;

  constructor(
    pipeline: GPUComputePipeline,
    workgroup: FixedArray<u32, 3>,
    guardQueue: GPUQueue | null = null,
    guarded: boolean = false,
    guardGroups: u32[] = [],
    guardBuffers: GPUBuffer[] = [],
  ) {
    this.pipeline = pipeline;
    this.guardQueue = guardQueue;
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
      this.guardQueue.dispose();
      this.guardQueue = null;
    }
    this.pipeline.dispose();
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

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
      pass.setVertexBuffer(slot as u32, vertexBuffers[slot], 0, vertexBuffers[slot].size());
      slot = slot + 1;
    }
  }

  setIndexBuffer(pass: GPURenderPassEncoder, buffer: GPUBuffer): void {
    if (this.indexFormat === "undefined") {
      authorTrap("RN18", "RenderPipeline.setIndexBuffer", "indexFormat=undefined");
      return;
    }
    pass.setIndexBuffer(buffer, this.indexFormat, 0, buffer.size());
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
          texture: { sampleType: source.sampleType, viewDimension: "2d", multisampled: false },
        });
      } else if (source.kind === "storageTexture") {
        if (source.format !== undefined) {
          entries.push({
            binding: source.binding,
            visibility: source.visibility,
            storageTexture: { access: "write-only", format: source.format, viewDimension: "2d" },
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
      targets: [{ format: spec.format }],
    },
  };
}

function finishComputePipeline(
  pipeline: GPUComputePipeline,
  workgroup: FixedArray<u32, 3>,
  guardGroups: u32[],
  guardBuffers: GPUBuffer[],
  guardQueue: GPUQueue | null,
): ComputePipeline {
  return new ComputePipeline(
    pipeline,
    workgroup,
    guardQueue,
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

/** Creates inside the caller's validation error scope; this helper does not await. */
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
  const guardQueue: GPUQueue | null = guardBuffers.length > 0 ? device.queue() : null;
  return finishComputePipeline(
    pipeline,
    workgroup,
    guardGroups,
    guardBuffers,
    guardQueue,
  );
}

/** Creates inside the caller's validation error scope; this helper does not await. */
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
  );
}

/** Creates inside the caller's validation error scope; this helper does not await. */
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

/** Creates inside the caller's validation error scope; this helper does not await. */
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

export const COMPUTE_VISIBILITY: u64 = GPUShaderStage.COMPUTE;
export const VERTEX_VISIBILITY: u64 = GPUShaderStage.VERTEX;
export const FRAGMENT_VISIBILITY: u64 = GPUShaderStage.FRAGMENT;

export function createBindGroup(
  device: GPUDevice,
  layout: GPUBindGroupLayout,
  spec: BindGroupLayoutSpec,
  resources: BindingResource[],
  guardBuffer: GPUBuffer | null = null,
): GPUBindGroup {
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
  return device.createBindGroup({ layout, entries });
}
