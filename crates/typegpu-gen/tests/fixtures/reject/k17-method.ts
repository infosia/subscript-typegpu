// expected-rule: K10
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Item { value: f32; constructor(value: f32) { this.value = value; } double(): f32 { return this.value * 2.0; } }
class Layout { input!: Storage<Item>; }
function kernel(res: Layout, ctx: ComputeInvocation): void { const item: Item = res.input[0]; const bad: f32 = item.double(); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [1, 1, 1] });
