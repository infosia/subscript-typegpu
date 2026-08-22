// expected-rule: TX1
// expected-message: float-channel library marker
import { ComputeInvocation, ComputePipelineSpec, StorageTexture2d, computePipeline } from "./typegpu";
class Rgba8uint {}
class Layout { target!: StorageTexture2d<Rgba8uint>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [1, 1, 1] });
