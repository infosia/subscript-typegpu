// expected-rule: K11
// expected-message: non-finite float literal Infinity
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { out: MutStorage<f32>; constructor(out: MutStorage<f32>) { this.out = out; } }
function kernel(res: Layout, ctx: ComputeInvocation): void { res.out[0] = Math.min() as f32; }
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
