// expected-rule: K8
// expected-message: a kernel cannot write an invocation builtin `ctx.globalId`
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { constructor(public output: MutStorage<u32>) {} }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  ctx.globalId.x += 2;
}
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
