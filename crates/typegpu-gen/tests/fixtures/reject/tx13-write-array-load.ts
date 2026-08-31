// expected-rule: TX11
// expected-message: write-only array storage texture
import { ComputeInvocation, ComputePipelineSpec, Rgba16float, WriteStorageTexture2dArray, computePipeline } from "./typegpu";
import { Vec2i } from "./typegpu-types";
class Layout { target!: WriteStorageTexture2dArray<Rgba16float>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  res.target.load(new Vec2i(0, 0), 0);
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
