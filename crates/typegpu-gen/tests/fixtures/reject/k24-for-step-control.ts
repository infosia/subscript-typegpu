// expected-rule: K22
// expected-message: builtin `ctx.localIndex`
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage, workgroupBarrier } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  let index: u32 = 0;
  for (index = 0; index < ctx.localIndex; index += 1) {}
  if (index < 4) { workgroupBarrier(); }
  res.output[0] = new Item(index);
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [4, 1, 1] });
