// expected-rule: K7
// expected-message: statement is outside the current kernel subset
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { out: MutStorage<u32>; constructor(out: MutStorage<u32>) { this.out = out; } }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  do { res.out[ctx.localIndex] = 1; } while (false);
}
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
