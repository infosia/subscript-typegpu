// expected-rule: PI3
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@ValueType class Item { value: f32; constructor(value: f32) { this.value = value; } }
class Layout { input: Storage<Item>; size(): u32 { return 1; } constructor(input: Storage<Item>) { this.input = input; } }
function kernel(res: Layout, ctx: ComputeInvocation): void {}
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
