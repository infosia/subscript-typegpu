// expected-rule: PI15
// expected-owner: author
// expected-message: one-layout

import { ComputeInvocation, ComputePipelineSpec, computePipeline2, MutStorage } from "./typegpu";

class LeftLayout { left: MutStorage<u32>; constructor(left: MutStorage<u32>) { this.left = left; } }
class RightLayout { right: MutStorage<u32>; constructor(right: MutStorage<u32>) { this.right = right; } }
function guardedKernel(left: LeftLayout, right: RightLayout, ctx: ComputeInvocation): void {
  left.left[ctx.globalId.x] = right.right[ctx.globalId.x];
}
export const rejected: ComputePipelineSpec = computePipeline2<LeftLayout, RightLayout>(guardedKernel, {
  name: "rejected",
  workgroupSize: [1, 1, 1],
  guarded: true,
});
