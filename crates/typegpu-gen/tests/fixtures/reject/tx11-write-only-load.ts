// expected-rule: TX11
// expected-message: write-only storage texture
import { ComputeInvocation, ComputePipelineSpec, R32float, StorageTexture2d, computePipeline } from "./typegpu";
import { Vec2i } from "./typegpu-types";
class Layout { target!: StorageTexture2d<R32float>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  res.target.load(new Vec2i(0, 0));
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
