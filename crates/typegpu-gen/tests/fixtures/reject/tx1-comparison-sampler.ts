// expected-rule: TX1
// expected-message: comparison samplers are not supported
import { ComputeInvocation, ComputePipelineSpec, computePipeline } from "./typegpu";
class ComparisonSampler {}
class Layout { comparison!: ComparisonSampler; }
function kernel(res: Layout, ctx: ComputeInvocation): void {}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
