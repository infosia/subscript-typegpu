// expected-rule: K1
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Item { value: f32; constructor(value: f32) { this.value = value; } }
class Layout { input!: Storage<Item>; }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>((res: Layout, ctx: ComputeInvocation): void => {}, { name: "pipeline", workgroupSize: [1, 1, 1] });
