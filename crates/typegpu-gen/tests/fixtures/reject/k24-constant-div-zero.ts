// expected-rule: K19
// expected-message: divides by zero
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
const BAD: u32 = 4 / 0;
function kernel(res: Layout, ctx: ComputeInvocation): void { res.output[0] = new Item(BAD); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
