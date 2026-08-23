// expected-rule: S100
// expected-owner: checker
// expected-message: type mismatch: the argument expects `(Layout, ComputeInvocation) => void`, got `(Layout, ComputeInvocation) => f32`
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Item { value: f32; constructor(value: f32) { this.value = value; } }
class Layout { input!: Storage<Item>; }
function kernel(res: Layout, ctx: ComputeInvocation): f32 { return 0.0; }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
