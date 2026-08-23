// expected-rule: PI3
import { ComputeInvocation, computePipeline, ComputePipelineSpec } from "./typegpu";
function kernel(res: f32, ctx: ComputeInvocation): void {}
export const pipeline: ComputePipelineSpec = computePipeline<f32>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
