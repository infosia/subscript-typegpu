// expected-rule: K9
// expected-message: constructor of `P`
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
@ValueType class P { a: f32; w: f32; constructor(v: f32) { this.a = v; this.w = v; } }
class Layout { out: MutStorage<P>; constructor(out: MutStorage<P>) { this.out = out; } }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  res.out[ctx.localIndex] = new P(1.0);
}
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
