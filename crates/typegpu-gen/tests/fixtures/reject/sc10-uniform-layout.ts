// expected-rule: SC10
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Uniform } from "./typegpu";
@CStruct class Item { values: FixedArray<f32, 2>; constructor(values: FixedArray<f32, 2>) { this.values = values; } }
class Layout { params!: Uniform<Item>; }
function kernel(res: Layout, ctx: ComputeInvocation): void { const item: Item = res.params.get(); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
