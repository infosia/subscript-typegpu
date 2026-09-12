// expected-rule: PI3
// expected-message: declares no parameter for field `params`
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage, Uniform } from "./typegpu";
class Layout { values: MutStorage<u32>; params: Uniform<u32> = new Uniform<u32>(0); constructor(values: MutStorage<u32>) { this.values = values; } }
function kernel(res: Layout, ctx: ComputeInvocation): void {}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
