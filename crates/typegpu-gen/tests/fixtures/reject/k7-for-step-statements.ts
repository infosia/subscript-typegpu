// expected-rule: K7
// expected-message: a `for` step must be one expression statement
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { out: MutStorage<u32>; constructor(out: MutStorage<u32>) { this.out = out; } }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  for (let i: u32 = 0; i < 2; res.out[ctx.localIndex + 0]++) { res.out[ctx.localIndex] = i; }
}
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
