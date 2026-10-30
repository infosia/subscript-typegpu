// expected-rule: K14
// expected-message: two program declarations share the source name `id_u32`
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { constructor(public id_u32: MutStorage<u32>) {} }
function id<T>(x: T): T { return x; }
function kernel(res: Layout, ctx: ComputeInvocation): void { res.id_u32[0] = id<u32>(1); }
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
