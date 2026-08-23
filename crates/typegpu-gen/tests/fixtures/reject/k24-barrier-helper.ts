// expected-rule: K22
// expected-message: is not legal in a helper
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage, workgroupBarrier } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
function helper(): void { workgroupBarrier(); }
function kernel(res: Layout, ctx: ComputeInvocation): void { helper(); res.output[0] = new Item(1); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
