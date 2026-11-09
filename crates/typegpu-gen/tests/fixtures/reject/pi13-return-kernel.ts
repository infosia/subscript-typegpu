// expected-rule: K1
// expected-owner: author
// expected-message: kernel `kernel` returns `f32`; a kernel returns void
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@ValueType class Item { value: f32; constructor(value: f32) { this.value = value; } }
class Layout { input: Storage<Item>; constructor(input: Storage<Item>) { this.input = input; } }
function kernel(res: Layout, ctx: ComputeInvocation): f32 { return 0.0; }
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
