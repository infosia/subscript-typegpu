// expected-rule: K29
// expected-owner: author
// expected-message: kernel

import { ComputeInvocation, ComputePipelineSpec, computePipeline, MutStorage, WgslShellSpec, wgslShell } from "./typegpu";

class EmptyLayout { output!: MutStorage<u32>; }

function both(res: EmptyLayout, ctx: ComputeInvocation): void {}

const shell: WgslShellSpec = wgslShell<(res: EmptyLayout, ctx: ComputeInvocation) => void>(both, {
  body: "return;",
});

export const rejected: ComputePipelineSpec = computePipeline<EmptyLayout>(both, {
  name: "rejected",
  workgroupSize: [1, 1, 1],
});
