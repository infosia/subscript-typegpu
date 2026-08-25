// expected-rule: PI15
// expected-owner: author
// expected-message: barrier

import { ComputeInvocation, ComputePipelineSpec, computePipeline, MutStorage, workgroupBarrier } from "./typegpu";

class Layout { output!: MutStorage<u32>; }
function guardedKernel(res: Layout, ctx: ComputeInvocation): void {
  workgroupBarrier();
  res.output[ctx.globalId.x] = 1;
}
export const rejected: ComputePipelineSpec = computePipeline<Layout>(guardedKernel, {
  name: "rejected",
  workgroupSize: [1, 1, 1],
  guarded: true,
});
