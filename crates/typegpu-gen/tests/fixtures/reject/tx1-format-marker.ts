// expected-rule: TX1
// expected-message: float-channel library marker
import { ComputeInvocation, ComputePipelineSpec, StorageTexture2d, computePipeline } from "./typegpu";
class FakeFormat {}
class Layout { target: StorageTexture2d<FakeFormat>; constructor(target: StorageTexture2d<FakeFormat>) { this.target = target; } }
function kernel(res: Layout, ctx: ComputeInvocation): void {}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
