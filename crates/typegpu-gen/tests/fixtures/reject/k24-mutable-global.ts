// expected-rule: K19
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
let mutableValue: u32 = 1;
function kernel(res: Layout, ctx: ComputeInvocation): void { res.output[0] = new Item(mutableValue); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [1, 1, 1] });
