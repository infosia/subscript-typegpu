// expected-rule: K22
// expected-message: builtin `ctx.localIndex`
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage, workgroupBarrier } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  if (ctx.localIndex === 0) { workgroupBarrier(); }
  res.output[0] = new Item(1);
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [4, 1, 1] });
