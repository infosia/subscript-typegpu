// expected-rule: K7
// expected-message: `throw` statement in kernel
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage } from "./typegpu";
@ValueType class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output: MutStorage<Item>; constructor(output: MutStorage<Item>) { this.output = output; } }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  if (ctx.globalId.x > 4) { throw new Error("out of range"); }
  res.output[ctx.globalId.x] = new Item(1);
}
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
