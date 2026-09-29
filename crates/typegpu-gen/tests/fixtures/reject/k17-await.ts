// expected-rule: S013
// expected-owner: checker
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@ValueType class Item { value: f32; constructor(value: f32) { this.value = value; } }
class Layout { input: Storage<Item>; constructor(input: Storage<Item>) { this.input = input; } }
async function waitForEvent(): Promise<void> { await Context.suspend(); }
function kernel(res: Layout, ctx: ComputeInvocation): void { waitForEvent(); }
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
