// expected-rule: TX3
// expected-message: sampled texture
import { ComputeInvocation, ComputePipelineSpec, Texture2d, computePipeline } from "./typegpu";
import { Vec2i, Vec4f } from "./typegpu-types";
class Layout { source!: Texture2d<f32>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  res.source.store(new Vec2i(0, 0), new Vec4f(1.0, 0.0, 0.0, 1.0));
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [1, 1, 1] });
