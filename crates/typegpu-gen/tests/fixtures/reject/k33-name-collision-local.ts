// expected-rule: K14
// expected-message: two program declarations share the source name `outer_local`
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { constructor(public out: MutStorage<u32>) {} }
function outer(): u32 { function local(): u32 { return 1; } return local(); }
function outer_local(): u32 { return 2; }
function kernel(res: Layout, ctx: ComputeInvocation): void { res.out[0] = outer() + outer_local(); }
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
