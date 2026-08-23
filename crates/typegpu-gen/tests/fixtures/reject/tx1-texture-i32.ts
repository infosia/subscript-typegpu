// expected-rule: TX1
// expected-message: sample type must be f32
import { ComputeInvocation, ComputePipelineSpec, Texture2d, computePipeline } from "./typegpu";
class Layout { source!: Texture2d<i32>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
