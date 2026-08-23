// expected-rule: K7
// expected-message: statement is outside the current kernel subset
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  visit([1 as u32, 2 as u32]);
}
function visit(values: u32[]): void { for (const value of values) {} }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
