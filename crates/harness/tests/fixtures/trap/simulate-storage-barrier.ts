// expected-rule: CL2

import {
  ComputeInvocation,
  ComputePipelineSpec,
  MutStorage,
  computePipeline,
  simulateCompute,
  storageBarrier,
} from "./typegpu";
import { blocked_HOST_RUNNABLE } from "./simulate-storage-barrier.typegpu";

class BlockedLayout {
  output!: MutStorage<u32>;
}

function blockedKernel(res: BlockedLayout, ctx: ComputeInvocation): void {
  storageBarrier();
}

const blocked: ComputePipelineSpec = computePipeline<BlockedLayout>(blockedKernel, {
  name: "blocked",
  workgroupSize: [1, 1, 1],
});

export function main(): void {
  const layout = new BlockedLayout();
  layout.output = new MutStorage<u32>([0]);
  simulateCompute<BlockedLayout>(
    blockedKernel,
    layout,
    blocked,
    [1, 1, 1],
    blocked_HOST_RUNNABLE,
  );
}
