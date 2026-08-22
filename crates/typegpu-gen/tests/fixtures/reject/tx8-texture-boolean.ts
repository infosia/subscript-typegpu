// expected-rule: TX8
// expected-message: f32, i32, or u32
import { ComputeInvocation, ComputePipelineSpec, Texture2d, computePipeline } from "./typegpu";
class Layout { source!: Texture2d<boolean>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [1, 1, 1] });
