// expected-rule: TX3
// expected-message: array sampled texture
import { ComputeInvocation, ComputePipelineSpec, Sampler, Texture2dArray, computePipeline } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
class Layout { source!: Texture2dArray<f32>; nearest!: Sampler; }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  const color: Vec4f = res.source.sampleLevel(res.nearest, new Vec2f(0.5, 0.5), 0.0);
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
