// expected-rule: S013
// expected-owner: checker
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Item { value: f32; constructor(value: f32) { this.value = value; } }
class Layout { input!: Storage<Item>; }
async function waitForEvent(): Promise<void> { await Context.suspend(); }
function kernel(res: Layout, ctx: ComputeInvocation): void { waitForEvent(); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
