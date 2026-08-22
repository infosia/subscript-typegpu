/** Runtime bindings shared by generated TypeGPU support modules and programs. */

import { Vec3u, Vec4f } from "./typegpu-types";
import {
  GPUBindGroup,
  GPUBindGroupEntry,
  GPUBindGroupLayout,
  GPUBindGroupLayoutEntry,
  GPUBuffer,
  GPUCommandEncoder,
  GPUComputePipeline,
  GPUDevice,
  GPUQueue,
  GPURenderPassEncoder,
  GPURenderPipeline,
  GPUShaderStage,
  GPUVertexAttribute,
  GPUVertexBufferLayout,
} from "./webgpu";

export class Buffer<T> {
  buffer: GPUBuffer;
  elementSize: u32;
  count: u32;

  constructor(buffer: GPUBuffer, elementSize: u32, count: u32) {
    this.buffer = buffer;
    this.elementSize = elementSize;
    this.count = count;
  }

  write(queue: GPUQueue, elementIndex: u32, bytes: u8[]): void {
    const byteLength: u32 = bytes.length as u32;
    const remainder: u32 = byteLength % this.elementSize;
    if (remainder !== 0) {
      print(
        `BF8 Buffer.write byteLength=${byteLength} elementSize=${this.elementSize} remainder=${remainder}`,
      );
      unreachable();
    }
    const elementCount: u32 = byteLength / this.elementSize;
    if (elementIndex > this.count || elementCount > this.count - elementIndex) {
      print(
        `BF8 Buffer.write elementIndex=${elementIndex} elementCount=${elementCount} count=${this.count}`,
      );
      unreachable();
    }
    queue.writeBuffer(
      this.buffer,
      (elementIndex as u64) * (this.elementSize as u64),
      bytes,
    );
  }

  writeOne(queue: GPUQueue, elementIndex: u32, bytes: u8[]): void {
    const byteLength: u32 = bytes.length as u32;
    if (byteLength !== this.elementSize) {
      print(
        `BF8 Buffer.writeOne elementIndex=${elementIndex} byteLength=${byteLength} elementSize=${this.elementSize}`,
      );
      unreachable();
    }
    if (elementIndex >= this.count) {
      print(`BF8 Buffer.writeOne elementIndex=${elementIndex} elementCount=1 count=${this.count}`);
      unreachable();
    }
    queue.writeBuffer(
      this.buffer,
      (elementIndex as u64) * (this.elementSize as u64),
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
      print(
        `BF8 Buffer.copyTo elementIndex=${elementIndex} elementCount=${elementCount} count=${this.count}`,
      );
      unreachable();
    }
    if (elementCount > target.count) {
      print(
        `BF8 Buffer.copyTo targetCount=${target.count} elementCount=${elementCount} elementSize=${this.elementSize}`,
      );
      unreachable();
    }
    encoder.copyBufferToBuffer(
      this.buffer,
      (elementIndex as u64) * (this.elementSize as u64),
      target.buffer,
      0,
      (elementCount as u64) * (this.elementSize as u64),
    );
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
    print(
      `BF8 readBuffer elementIndex=${elementIndex} elementCount=${elementCount} count=${readback.count}`,
    );
    unreachable();
  }
  return readback.buffer.readMappedRange(
    (elementIndex as u64) * (readback.elementSize as u64),
    (elementCount as u64) * (readback.elementSize as u64),
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
  );
}

export class ComputeInvocation {
  globalId!: Vec3u;
  localId!: Vec3u;
  workgroupId!: Vec3u;
  numWorkgroups!: Vec3u;
  localIndex!: u32;
}

export class VertexInvocation {
  vertexIndex!: u32;
  instanceIndex!: u32;
}

export class FragmentInvocation {
  position!: Vec4f;
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

@Descriptor
export class ComputePipelineSpec {
  workgroupSize!: FixedArray<u32, 3>;
}

export function computePipeline<L>(
  kernel: (res: L, ctx: ComputeInvocation) => void,
  spec: ComputePipelineSpec,
): ComputePipelineSpec {
  return { workgroupSize: spec.workgroupSize };
}

export function computePipeline2<L0, L1>(
  kernel: (res0: L0, res1: L1, ctx: ComputeInvocation) => void,
  spec: ComputePipelineSpec,
): ComputePipelineSpec {
  return { workgroupSize: spec.workgroupSize };
}

export function computePipeline3<L0, L1, L2>(
  kernel: (res0: L0, res1: L1, res2: L2, ctx: ComputeInvocation) => void,
  spec: ComputePipelineSpec,
): ComputePipelineSpec {
  return { workgroupSize: spec.workgroupSize };
}

export function computePipeline4<L0, L1, L2, L3>(
  kernel: (res0: L0, res1: L1, res2: L2, res3: L3, ctx: ComputeInvocation) => void,
  spec: ComputePipelineSpec,
): ComputePipelineSpec {
  return { workgroupSize: spec.workgroupSize };
}

@Descriptor
export class RenderPipelineSpec {
  format!: GPUTextureFormat;
  topology?: GPUPrimitiveTopology = "triangle-list";
  cullMode?: GPUCullMode = "none";
  frontFace?: GPUFrontFace = "ccw";
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
  };
}

@Descriptor
export class BindGroupLayoutEntrySpec {
  binding!: u32;
  visibility!: u64;
  kind!: string;
  minBindingSize!: u64;
}

@Descriptor
export class BindGroupLayoutSpec {
  entries!: BindGroupLayoutEntrySpec[];
}

export class ComputePipeline {
  private pipeline: GPUComputePipeline;
  private workgroup: FixedArray<u32, 3>;

  constructor(pipeline: GPUComputePipeline, workgroup: FixedArray<u32, 3>) {
    this.pipeline = pipeline;
    this.workgroup = workgroup;
  }

  bindGroupLayout(group: u32): GPUBindGroupLayout {
    return this.pipeline.getBindGroupLayout(group);
  }

  dispatch(
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

  dispatchThreads(
    encoder: GPUCommandEncoder,
    groups: GPUBindGroup[],
    x: u32,
    y: u32,
    z: u32,
  ): void {
    this.dispatch(
      encoder,
      groups,
      (x + this.workgroup[0] - 1) / this.workgroup[0],
      (y + this.workgroup[1] - 1) / this.workgroup[1],
      (z + this.workgroup[2] - 1) / this.workgroup[2],
    );
  }

  dispose(): void {
    this.pipeline.dispose();
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class RenderPipeline {
  private pipeline: GPURenderPipeline;

  constructor(pipeline: GPURenderPipeline) {
    this.pipeline = pipeline;
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

  dispose(): void {
    this.pipeline.dispose();
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export function createComputePipeline(
  device: GPUDevice,
  wgsl: string,
  entry: string,
  layouts: BindGroupLayoutSpec[],
  workgroup: FixedArray<u32, 3>,
): ComputePipeline {
  const nativeLayouts: GPUBindGroupLayout[] = [];
  let group: i32 = 0;
  while (group < layouts.length) {
    const entries: GPUBindGroupLayoutEntry[] = [];
    let binding: i32 = 0;
    while (binding < layouts[group].entries.length) {
      const source: BindGroupLayoutEntrySpec = layouts[group].entries[binding];
      if (source.kind === "uniform") {
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
      } else {
        entries.push({
          binding: source.binding,
          visibility: source.visibility,
          buffer: { type: "storage", minBindingSize: source.minBindingSize },
        });
      }
      binding = binding + 1;
    }
    nativeLayouts.push(device.createBindGroupLayout({ entries }));
    group = group + 1;
  }
  using shader = device.createShaderModule({ code: wgsl });
  using layout = device.createPipelineLayout({ bindGroupLayouts: nativeLayouts });
  const pipeline = device.createComputePipeline({
    layout,
    compute: { module: shader, entryPoint: entry },
  });
  group = 0;
  while (group < nativeLayouts.length) {
    nativeLayouts[group].dispose();
    group = group + 1;
  }
  return new ComputePipeline(pipeline, workgroup);
}

export function createRenderPipeline(
  device: GPUDevice,
  wgsl: string,
  vertexEntry: string,
  fragmentEntry: string,
  layouts: BindGroupLayoutSpec[],
  vertexLayouts: VertexBufferLayoutSpec[],
  spec: RenderPipelineSpec,
): RenderPipeline {
  const nativeLayouts: GPUBindGroupLayout[] = [];
  let group: i32 = 0;
  while (group < layouts.length) {
    const entries: GPUBindGroupLayoutEntry[] = [];
    let binding: i32 = 0;
    while (binding < layouts[group].entries.length) {
      const source: BindGroupLayoutEntrySpec = layouts[group].entries[binding];
      if (source.kind === "uniform") {
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
      } else {
        entries.push({
          binding: source.binding,
          visibility: source.visibility,
          buffer: { type: "storage", minBindingSize: source.minBindingSize },
        });
      }
      binding = binding + 1;
    }
    nativeLayouts.push(device.createBindGroupLayout({ entries }));
    group = group + 1;
  }
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
  using shader = device.createShaderModule({ code: wgsl });
  using layout = device.createPipelineLayout({ bindGroupLayouts: nativeLayouts });
  const pipeline = device.createRenderPipeline({
    layout,
    vertex: { module: shader, entryPoint: vertexEntry, buffers: nativeVertexLayouts },
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
  });
  group = 0;
  while (group < nativeLayouts.length) {
    nativeLayouts[group].dispose();
    group = group + 1;
  }
  return new RenderPipeline(pipeline);
}

export const COMPUTE_VISIBILITY: u64 = GPUShaderStage.COMPUTE;
export const VERTEX_VISIBILITY: u64 = GPUShaderStage.VERTEX;
export const FRAGMENT_VISIBILITY: u64 = GPUShaderStage.FRAGMENT;

export function createBindGroup(
  device: GPUDevice,
  layout: GPUBindGroupLayout,
  spec: BindGroupLayoutSpec,
  buffers: GPUBuffer[],
): GPUBindGroup {
  if (spec.entries.length !== buffers.length) {
    print(`createBindGroup expected ${spec.entries.length} buffers but received ${buffers.length}`);
    unreachable();
  }
  const entries: GPUBindGroupEntry[] = [];
  let index: i32 = 0;
  while (index < buffers.length) {
    entries.push({ binding: spec.entries[index].binding, buffer: buffers[index] });
    index = index + 1;
  }
  return device.createBindGroup({ layout, entries });
}
