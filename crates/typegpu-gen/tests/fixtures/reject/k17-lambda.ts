// expected-rule: K9
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Item { value: f32; constructor(value: f32) { this.value = value; } }
class Layout { input!: Storage<Item>; }
function kernel(res: Layout, ctx: ComputeInvocation): void { const bad: (value: f32) => f32 = (value: f32): f32 => value; }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
