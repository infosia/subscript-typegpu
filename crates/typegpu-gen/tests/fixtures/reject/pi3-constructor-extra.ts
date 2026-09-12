// expected-rule: PI3
// expected-message: holds a statement after the field assignments
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage } from "./typegpu";
class Layout { values: MutStorage<u32>; constructor(values: MutStorage<u32>) { this.values = values; this.values = values; } }
function kernel(res: Layout, ctx: ComputeInvocation): void {}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
