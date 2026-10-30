// expected-rule: K14
// expected-message: two program declarations share the source name `id_u32`
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { out: MutStorage<u32>; constructor(out: MutStorage<u32>) { this.out = out; } }
function id<T>(x: T): T { return x; }
function id_u32(x: u32): u32 { return x + 1; }
function kernel(res: Layout, ctx: ComputeInvocation): void { res.out[0] = id<u32>(1) + id_u32(2); }
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
