// expected-rule: K18
// expected-message: repeated switch selector value 1
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { constructor(public out: MutStorage<f32>) {} }
enum E { A = 1, B = 1, C = 2 }
function kernel(res: Layout, ctx: ComputeInvocation): void { const e: E = E.A; switch (e) { case E.A: break; case E.B: break; case E.C: break; } }
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
