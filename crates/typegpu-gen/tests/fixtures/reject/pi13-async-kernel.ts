// expected-rule: S100
// expected-owner: checker
// expected-message: async functions are not first-class values; call them directly in await position
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@ValueType class Item { value: f32; constructor(value: f32) { this.value = value; } }
class Layout { input: Storage<Item>; constructor(input: Storage<Item>) { this.input = input; } }
async function kernel(res: Layout, ctx: ComputeInvocation): Promise<void> { await Context.suspend(); }
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
