// expected-rule: PI3
import { ComputeInvocation, computePipeline, ComputePipelineSpec } from "./typegpu";
@ValueType class Layout { value: f32; constructor(value: f32) { this.value = value; } }
function kernel(res: Layout, ctx: ComputeInvocation): void {}
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
