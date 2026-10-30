// expected-rule: K10
// expected-message: binding method `isNearest` is outside K10
import { ComputeInvocation, ComputePipelineSpec, MutStorage, Sampler, computePipeline } from "./typegpu";
class Layout { constructor(public sampler: Sampler, public out: MutStorage<u32>) {} }
function kernel(res: Layout, ctx: ComputeInvocation): void { if (res.sampler.isNearest()) { res.out[0] = 1; } }
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
