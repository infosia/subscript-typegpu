// expected-rule: K5
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Item { value: f32; constructor(value: f32) { this.value = value; } }
class Layout { input!: Storage<Item>; }
function sum(values: f32[]): f32 { return values[0]; }
function zValues(): f32[] { return JSON.parse<f32[]>("[]").value; }
function kernel(res: Layout, ctx: ComputeInvocation): void { const bad: f32 = sum(zValues()); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
