import {
  ComputeInvocation,
  ComputePipeline,
  MutStorage,
  Storage,
  Uniform,
  WorkgroupArray,
  workgroupArray,
  workgroupBarrier,
} from "./typegpu";
import {
  GPUBindGroup,
  GPUCommandEncoder,
} from "./webgpu";

const SORT_WORKGROUP_SIZE: u32 = 256;
const PREFIX_SCAN_LIMIT: u32 = 65536;

function sortTrap(method: string, values: string): void {
  print(`SORT1 ${method} ${values} (author)`);
  unreachable();
}

@CStruct
export class BitonicSortPass {
  k: u32;
  jShift: u32;

  constructor(k: u32, jShift: u32) {
    this.k = k;
    this.jShift = jShift;
  }
}

export class BitonicSortResources {
  values!: MutStorage<u32>;
  pass!: Uniform<BitonicSortPass>;
}

function bitonicSortStride(jShift: u32): u32 {
  let stride: u32 = 1;
  let shift: u32 = 0;
  while (shift < jShift) {
    stride *= 2;
    shift += 1;
  }
  return stride;
}

function requireBitonicSortLength(length: u32): void {
  if (length === 0 || (length & (length - 1)) !== 0) {
    sortTrap("bitonicSortPassCount", `length=${length} is not a power of two`);
  }
}

// The host asks for one pass at a time, which lets a windowed caller submit one step
// per frame while a batch caller can enumerate the same sequence without duplicating it.
export function bitonicSortPassCount(length: u32): u32 {
  requireBitonicSortLength(length);
  let levels: u32 = 0;
  let value: u32 = length;
  while (value > 1) {
    levels += 1;
    value /= 2;
  }
  return levels * (levels + 1) / 2;
}

export function bitonicSortPass(length: u32, passIndex: u32): BitonicSortPass {
  const count: u32 = bitonicSortPassCount(length);
  if (passIndex >= count) {
    sortTrap("bitonicSortPass", `passIndex=${passIndex} passCount=${count}`);
  }
  let current: u32 = 0;
  let k: u32 = 2;
  while (k <= length) {
    let j: u32 = k / 2;
    let jShift: u32 = 0;
    while (j > 1) {
      jShift += 1;
      j /= 2;
    }
    while (true) {
      if (current === passIndex) return new BitonicSortPass(k, jShift);
      current += 1;
      if (jShift === 0) break;
      jShift -= 1;
    }
    k *= 2;
  }
  return new BitonicSortPass(0, 0);
}

// Dispatch length / 2 threads because one invocation owns one comparator pair.
// Committing the ascending comparator here replaces the upstream comparator slot while
// retaining each bitonic merge direction.
export function bitonicSortStep(
  resources: BitonicSortResources,
  invocation: ComputeInvocation,
): void {
  const thread: u32 = invocation.globalId.x;
  const stride: u32 = bitonicSortStride(resources.pass.$.jShift);
  const below: u32 = thread & (stride - 1);
  const above: u32 = thread / stride;
  const lower: u32 = below + above * (stride * 2);
  const upper: u32 = lower + stride;
  const ascending: boolean = (lower & resources.pass.$.k) === 0;
  const lowerValue: u32 = resources.values[lower];
  const upperValue: u32 = resources.values[upper];
  if ((ascending && lowerValue > upperValue) || (!ascending && lowerValue < upperValue)) {
    resources.values[lower] = upperValue;
    resources.values[upper] = lowerValue;
  }
}

export class PrefixScanBlockResources {
  values!: MutStorage<f32>;
  sums!: MutStorage<f32>;
}

export class PrefixScanApplyResources {
  values!: MutStorage<f32>;
  offsets!: Storage<f32>;
}

const prefixScanShared: WorkgroupArray<f32> = workgroupArray<f32>(256);

// Blelloch up-sweep and down-sweep produce an exclusive scan for one 256-value block.
// Lane zero also preserves the block total for the one-level host driver.
export function prefixScanBlockF32(
  resources: PrefixScanBlockResources,
  invocation: ComputeInvocation,
): void {
  const lane: u32 = invocation.localId.x;
  prefixScanShared[lane] = resources.values[invocation.globalId.x];
  workgroupBarrier();

  let offset: u32 = 1;
  let active: u32 = 128;
  while (active > 0) {
    if (lane < active) {
      const right: u32 = (lane + 1) * offset * 2 - 1;
      const left: u32 = right - offset;
      prefixScanShared[right] = prefixScanShared[right] + prefixScanShared[left];
    }
    offset *= 2;
    workgroupBarrier();
    active /= 2;
  }

  if (lane === 0) {
    resources.sums[invocation.workgroupId.x] = prefixScanShared[255];
    prefixScanShared[255] = 0.0;
  }
  workgroupBarrier();

  active = 1;
  while (active < 256) {
    offset /= 2;
    if (lane < active) {
      const right: u32 = (lane + 1) * offset * 2 - 1;
      const left: u32 = right - offset;
      const leftValue: f32 = prefixScanShared[left];
      prefixScanShared[left] = prefixScanShared[right];
      prefixScanShared[right] = prefixScanShared[right] + leftValue;
    }
    active *= 2;
    workgroupBarrier();
  }
  resources.values[invocation.globalId.x] = prefixScanShared[lane];
}

// The scanned block sums are indexed by workgroup, so each lane adds one shared offset.
export function prefixScanApplyF32(
  resources: PrefixScanApplyResources,
  invocation: ComputeInvocation,
): void {
  resources.values[invocation.globalId.x] = resources.values[invocation.globalId.x]
    + resources.offsets[invocation.workgroupId.x];
}

function requirePrefixScanLength(length: u32): void {
  if (length === 0 || length > PREFIX_SCAN_LIMIT) {
    sortTrap("prefixScanPlanF32", `length=${length} maximum=${PREFIX_SCAN_LIMIT}`);
  }
}

@CStruct
export class PrefixScanPlanF32 {
  paddedLength: u32;
  blockCount: u32;

  constructor(paddedLength: u32, blockCount: u32) {
    this.paddedLength = paddedLength;
    this.blockCount = blockCount;
  }
}

export function prefixScanPlanF32(length: u32): PrefixScanPlanF32 {
  requirePrefixScanLength(length);
  const blockCount: u32 = (length + SORT_WORKGROUP_SIZE - 1) / SORT_WORKGROUP_SIZE;
  return new PrefixScanPlanF32(blockCount * SORT_WORKGROUP_SIZE, blockCount);
}

// Buffers are padded with the 0.0 identity by the caller. The second scan always covers
// one full sums block, then apply adds its exclusive offsets to every padded value.
export function runPrefixScanF32(
  encoder: GPUCommandEncoder,
  blockPipeline: ComputePipeline,
  valuesGroup: GPUBindGroup,
  sumsGroup: GPUBindGroup,
  applyPipeline: ComputePipeline,
  applyGroup: GPUBindGroup,
  length: u32,
): void {
  const plan: PrefixScanPlanF32 = prefixScanPlanF32(length);
  blockPipeline.dispatch(encoder, [valuesGroup], plan.blockCount, 1, 1);
  blockPipeline.dispatch(encoder, [sumsGroup], 1, 1, 1);
  applyPipeline.dispatch(encoder, [applyGroup], plan.blockCount, 1, 1);
}

function prefixScanBlockHostF32(values: f32[], start: u32): f32 {
  let offset: u32 = 1;
  let active: u32 = SORT_WORKGROUP_SIZE / 2;
  while (active > 0) {
    let lane: u32 = 0;
    while (lane < active) {
      const right: u32 = start + (lane + 1) * offset * 2 - 1;
      const left: u32 = right - offset;
      values[right as i32] = values[right as i32] + values[left as i32];
      lane += 1;
    }
    offset *= 2;
    active /= 2;
  }
  const total: f32 = values[(start + SORT_WORKGROUP_SIZE - 1) as i32];
  values[(start + SORT_WORKGROUP_SIZE - 1) as i32] = 0.0;
  active = 1;
  while (active < SORT_WORKGROUP_SIZE) {
    offset /= 2;
    let lane: u32 = 0;
    while (lane < active) {
      const right: u32 = start + (lane + 1) * offset * 2 - 1;
      const left: u32 = right - offset;
      const leftValue: f32 = values[left as i32];
      values[left as i32] = values[right as i32];
      values[right as i32] = values[right as i32] + leftValue;
      lane += 1;
    }
    active *= 2;
  }
  return total;
}

// This mirrors both GPU scan levels and the apply pass, including their f32 addition
// order, so a readback can be compared byte for byte rather than with a tolerance.
export function prefixScanHostF32(input: f32[]): f32[] {
  const plan: PrefixScanPlanF32 = prefixScanPlanF32(input.length as u32);
  const values: f32[] = [];
  let index: u32 = 0;
  while (index < plan.paddedLength) {
    values.push(index < (input.length as u32) ? input[index as i32] : 0.0);
    index += 1;
  }
  const sums: f32[] = [];
  let block: u32 = 0;
  while (block < SORT_WORKGROUP_SIZE) {
    sums.push(block < plan.blockCount
      ? prefixScanBlockHostF32(values, block * SORT_WORKGROUP_SIZE)
      : 0.0);
    block += 1;
  }
  prefixScanBlockHostF32(sums, 0);
  block = 0;
  while (block < plan.blockCount) {
    index = 0;
    while (index < SORT_WORKGROUP_SIZE) {
      const valueIndex: u32 = block * SORT_WORKGROUP_SIZE + index;
      values[valueIndex as i32] = values[valueIndex as i32] + sums[block as i32];
      index += 1;
    }
    block += 1;
  }
  const output: f32[] = [];
  index = 0;
  while (index < (input.length as u32)) {
    output.push(values[index as i32]);
    index += 1;
  }
  return output;
}
