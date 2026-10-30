// expected-rule: K9
// expected-message: constructor of `P`
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
@ValueType class P { a: f32; b: f32; constructor(b: f32, a: f32) { this.a = a; this.b = b * 2.0; } }
class Layout { out: MutStorage<P>; constructor(out: MutStorage<P>) { this.out = out; } }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  res.out[ctx.localIndex] = new P(1.0, 5.0);
}
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
