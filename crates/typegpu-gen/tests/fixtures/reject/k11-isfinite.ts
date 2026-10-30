// expected-rule: K11
// expected-message: NaN and infinity tests are not portable in WGSL
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { out: MutStorage<f32>; constructor(out: MutStorage<f32>) { this.out = out; } }
function kernel(res: Layout, ctx: ComputeInvocation): void { if (isFinite(0.0)) { res.out[0] = 0.0; } }
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
