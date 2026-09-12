// expected-rule: PI3
// expected-message: is not `this.left = left`
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Item { value: f32; constructor(value: f32) { this.value = value; } }
class Layout { left: Storage<Item>; right: Storage<Item>; constructor(left: Storage<Item>, right: Storage<Item>) { this.left = right; this.right = left; } }
function kernel(res: Layout, ctx: ComputeInvocation): void {}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
