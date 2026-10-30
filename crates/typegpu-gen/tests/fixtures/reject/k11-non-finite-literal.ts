// expected-rule: K11
// expected-message: non-finite float literal Infinity
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { constructor(public out: MutStorage<f32>) {} }

function kernel(res: Layout, ctx: ComputeInvocation): void { res.out[0] = Infinity as f32; }
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
