// expected-rule: TX8
// expected-message: fragment kernel
import { ComputeInvocation, ComputePipelineSpec, Sampler, Texture2d, computePipeline } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
class Layout { source!: Texture2d<f32>; nearest!: Sampler; }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  const color: Vec4f = res.source.sample(res.nearest, new Vec2f(0.5, 0.5));
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [1, 1, 1] });
