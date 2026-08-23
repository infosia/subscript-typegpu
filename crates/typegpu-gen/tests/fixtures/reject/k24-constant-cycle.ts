// expected-rule: K19
// expected-message: module constant cycle
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
const FIRST: u32 = SECOND + 1;
const SECOND: u32 = FIRST + 1;
function kernel(res: Layout, ctx: ComputeInvocation): void { res.output[0] = new Item(FIRST); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
