// expected-rule: K14
// expected-message: two program declarations share the source name `P_read`
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
@ValueType class P { constructor(public value: u32) {} read(): u32 { return this.value; } }
class Layout { constructor(public out: MutStorage<u32>) {} }
function P_read(): u32 { return 2; }
function kernel(res: Layout, ctx: ComputeInvocation): void { res.out[0] = new P(1).read() + P_read(); }
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
