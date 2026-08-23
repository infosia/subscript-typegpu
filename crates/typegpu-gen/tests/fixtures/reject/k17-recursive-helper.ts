// expected-rule: K2
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Item { value: f32; constructor(value: f32) { this.value = value; } }
class Layout { input!: Storage<Item>; }
function recurse(value: f32): f32 { if (value > 0.0) { return recurse(value - 1.0); } return value; }
function kernel(res: Layout, ctx: ComputeInvocation): void { const bad: f32 = recurse(1.0); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
