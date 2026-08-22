/** Runtime bindings shared by generated TypeGPU support modules and programs. */

import { Vec3u } from "./typegpu-types";
import {
  GPUBindGroup,
  GPUBindGroupEntry,
  GPUBindGroupLayout,
  GPUBindGroupLayoutEntry,
  GPUBuffer,
  GPUCommandEncoder,
  GPUComputePipeline,
  GPUDevice,
  GPUShaderStage,
} from "./webgpu";

export class ComputeInvocation {
  globalId!: Vec3u;
  localId!: Vec3u;
  workgroupId!: Vec3u;
  numWorkgroups!: Vec3u;
  localIndex!: u32;
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
export class ComputePipelineOptions {
  workgroupSize!: FixedArray<u32, 3>;
}

@Descriptor
export class ComputePipelineSpec {
  workgroupSize!: FixedArray<u32, 3>;
}

export function computePipeline<L>(
  kernel: (res: L, ctx: ComputeInvocation) => void,
  spec: ComputePipelineOptions,
): ComputePipelineSpec {
  return { workgroupSize: spec.workgroupSize };
}

export function computePipeline2<L0, L1>(
  kernel: (res0: L0, res1: L1, ctx: ComputeInvocation) => void,
  spec: ComputePipelineOptions,
): ComputePipelineSpec {
  return { workgroupSize: spec.workgroupSize };
}

export function computePipeline3<L0, L1, L2>(
  kernel: (res0: L0, res1: L1, res2: L2, ctx: ComputeInvocation) => void,
  spec: ComputePipelineOptions,
): ComputePipelineSpec {
  return { workgroupSize: spec.workgroupSize };
}

export function computePipeline4<L0, L1, L2, L3>(
  kernel: (res0: L0, res1: L1, res2: L2, res3: L3, ctx: ComputeInvocation) => void,
  spec: ComputePipelineOptions,
): ComputePipelineSpec {
  return { workgroupSize: spec.workgroupSize };
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

export const COMPUTE_VISIBILITY: u64 = GPUShaderStage.COMPUTE;

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
