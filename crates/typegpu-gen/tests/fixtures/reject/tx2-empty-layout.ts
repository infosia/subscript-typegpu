// expected-rule: TX2
// expected-message: layout class `EmptyLayout` is empty
import { ComputeInvocation, ComputePipelineSpec, computePipeline } from "./typegpu";
class EmptyLayout {}
function kernel(empty: EmptyLayout, ctx: ComputeInvocation): void {}
export const pipeline: ComputePipelineSpec = computePipeline<EmptyLayout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
