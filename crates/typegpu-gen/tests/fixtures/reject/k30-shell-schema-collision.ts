// expected-rule: K30
// expected-owner: author
// expected-message: collides

import { ComputeInvocation, ComputePipelineSpec, computePipeline, MutStorage, Storage, WgslShellSpec, wgslShell } from "./typegpu";

@CStruct
class textureSample {
  value: u32;
  constructor(value: u32) { this.value = value; }
}

function textureSample_(value: u32): u32 { return value; }
const shell: WgslShellSpec = wgslShell<(value: u32) => u32>(textureSample_, {
  body: "return value;",
});
class Layout { input!: Storage<textureSample>; output!: MutStorage<u32>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  res.output.set(0, textureSample_(res.input.get(0).value));
}
export const rejected: ComputePipelineSpec = computePipeline<Layout>(kernel, {
  name: "rejected",
  workgroupSize: [1, 1, 1],
});
