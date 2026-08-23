// expected-rule: PI5
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
class Layout { bad!: Storage<string>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
