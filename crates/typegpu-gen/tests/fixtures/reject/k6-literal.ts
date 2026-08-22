// expected-rule: K6
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Uniform } from "./typegpu";
class Layout { value!: Uniform<u32>; }
function kernel(res: Layout, ctx: ComputeInvocation): void { const value: i64 = 1; }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [1, 1, 1] });
