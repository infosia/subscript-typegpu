// expected-rule: K22
// expected-message: binding `input`
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage, workgroupBarrier } from "./typegpu";
@ValueType class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { input: Storage<Item>; constructor(input: Storage<Item>) { this.input = input; } }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  while (res.input[0].value > 0) { workgroupBarrier(); break; }
}
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [4, 1, 1] });
