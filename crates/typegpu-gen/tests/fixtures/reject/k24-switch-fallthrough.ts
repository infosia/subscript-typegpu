// expected-rule: K18
// expected-message: switch case falls through with statements
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  switch (ctx.globalId.x) {
    case 0: res.output[0] = new Item(1);
    case 1: return;
    default: return;
  }
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
