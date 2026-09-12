// expected-rule: PI6
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Item { value: f32; constructor(value: f32) { this.value = value; } }
class Layout { input: Storage<Item>; constructor(input: Storage<Item>) { this.input = input; } }
function kernel(res: Layout, ctx: ComputeInvocation): void { const bad: Layout = new Layout(res.input); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
