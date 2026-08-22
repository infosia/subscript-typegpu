// expected-rule: S100
// expected-owner: checker
// expected-message: type mismatch: the argument expects `(Layout, ComputeInvocation) => void`, got `(Layout, WrongContext) => void`
import { computePipeline, ComputePipelineSpec } from "./typegpu";
class Layout {}
class WrongContext {}
function kernel(res: Layout, ctx: WrongContext): void {}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [1, 1, 1] });
