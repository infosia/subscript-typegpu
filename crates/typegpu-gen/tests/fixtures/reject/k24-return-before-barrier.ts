// expected-rule: K22
// expected-message: `return` statement precedes a barrier
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage, workgroupBarrier } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output: MutStorage<Item>; constructor(output: MutStorage<Item>) { this.output = output; } }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  const lane: u32 = ctx.localIndex;
  if (lane > 0) { return; }
  workgroupBarrier();
  res.output[0] = new Item(1);
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [4, 1, 1] });
