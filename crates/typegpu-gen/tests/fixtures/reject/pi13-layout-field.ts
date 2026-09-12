// expected-rule: PI3
import { ComputeInvocation, computePipeline, ComputePipelineSpec } from "./typegpu";
class Layout { bad: f32; constructor(bad: f32) { this.bad = bad; } }
function kernel(res: Layout, ctx: ComputeInvocation): void {}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
