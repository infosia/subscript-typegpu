// expected-rule: K4
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Item { value: f16; constructor(value: f16) { this.value = value; } }
class Layout { input!: Storage<Item>; }
function kernel(res: Layout, ctx: ComputeInvocation): void { const item: Item = res.input[0]; const bad: f16 = item.value; }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
