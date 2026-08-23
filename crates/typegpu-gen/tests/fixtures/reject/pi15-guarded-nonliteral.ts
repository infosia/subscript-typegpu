// expected-rule: PI15
// expected-owner: author
// expected-message: guarded

import { ComputeInvocation, ComputePipelineSpec, computePipeline, MutStorage } from "./typegpu";

class EmptyLayout { output!: MutStorage<u32>; }
function guardedKernel(res: EmptyLayout, ctx: ComputeInvocation): void {}
const guardedValue: boolean = true;

export const rejected: ComputePipelineSpec = computePipeline<EmptyLayout>(guardedKernel, {
  name: "rejected",
  workgroupSize: [1, 1, 1],
  guarded: guardedValue,
});
