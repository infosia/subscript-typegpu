// expected-rule: TX8
// expected-message: TX1 library marker
import { ComputeInvocation, ComputePipelineSpec, StorageTexture2d, computePipeline } from "./typegpu";
class FakeFormat {}
class Layout { target!: StorageTexture2d<FakeFormat>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [1, 1, 1] });
