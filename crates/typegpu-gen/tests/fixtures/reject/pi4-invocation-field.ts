// expected-rule: S100
// expected-owner: checker
import { ComputeInvocation, computePipeline, ComputePipelineSpec } from "./typegpu";
class Layout {}
function kernel(res: Layout, ctx: ComputeInvocation): void { const value: u32 = ctx.unknown; }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
