// expected-rule: CL2

import {
  ComputeInvocation,
  ComputePipelineSpec,
  computePipeline,
  simulateCompute,
} from "./typegpu";

class BlockedLayout {}

function blockedKernel(res: BlockedLayout, ctx: ComputeInvocation): void {}

const blocked: ComputePipelineSpec = computePipeline<BlockedLayout>(blockedKernel, {
  name: "blocked",
  workgroupSize: [1, 1, 1],
});

export function main(): void {
  simulateCompute<BlockedLayout>(
    blockedKernel,
    new BlockedLayout(),
    blocked,
    [1, 1, 1],
    false,
  );
}
