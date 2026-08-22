// expected-rule: S100
// expected-owner: checker
// expected-message: type mismatch: the argument expects `(Layout, ComputeInvocation) => void`, got `(Layout) => void`
import { ComputeInvocation, computePipeline, ComputePipelineSpec } from "./typegpu";
class Layout {}
function kernel(res: Layout): void {}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [1, 1, 1] });
