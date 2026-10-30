// expected-rule: K14
// expected-message: two program declarations share the source name `id_f32`
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
@ValueType class id_f32 { constructor(public value: f32) {} }
class Layout { constructor(public out: MutStorage<id_f32>) {} }
function id<T>(x: T): T { return x; }
function kernel(res: Layout, ctx: ComputeInvocation): void { res.out[0] = new id_f32(id<f32>(1.0)); }
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
