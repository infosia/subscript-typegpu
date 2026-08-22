// expected-rule: K2
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Item { value: f32; constructor(value: f32) { this.value = value; } }
class Layout { input!: Storage<Item>; }
function helper(res: Layout): f32 { return res.input[0].value; }
function kernel(res: Layout, ctx: ComputeInvocation): void { const bad: f32 = helper(res); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [1, 1, 1] });
