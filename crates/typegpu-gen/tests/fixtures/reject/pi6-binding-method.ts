// expected-rule: S100
// expected-owner: checker
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Item { value: f32; constructor(value: f32) { this.value = value; } }
class Layout { input!: Storage<Item>; }
function kernel(res: Layout, ctx: ComputeInvocation): void { res.input.set(0, new Item(1.0)); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
