// expected-rule: TX13
// expected-message: read-only array storage texture
import { ComputeInvocation, ComputePipelineSpec, ReadStorageTexture2dArray, Rgba16float, computePipeline } from "./typegpu";
import { Vec2i, Vec4f } from "./typegpu-types";
class Layout { source!: ReadStorageTexture2dArray<Rgba16float>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  res.source.store(new Vec2i(0, 0), 0, new Vec4f(0.0, 0.0, 0.0, 0.0));
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
