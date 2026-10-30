// expected-rule: K18
// expected-message: repeated switch selector value 1
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { constructor(public out: MutStorage<f32>) {} }

function kernel(res: Layout, ctx: ComputeInvocation): void { switch (ctx.localIndex) { case 1: break; case 1: break; default: break; } }
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
