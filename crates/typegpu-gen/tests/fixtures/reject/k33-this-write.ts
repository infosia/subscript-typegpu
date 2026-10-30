// expected-rule: K33
// expected-message: cannot write a field of this
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { out: MutStorage<u32>; constructor(out: MutStorage<u32>) { this.out = out; } }
@ValueType class P { constructor(public value: u32) {} mutate(): u32 { this.value += 1; return this.value; } }
function kernel(res: Layout, ctx: ComputeInvocation): void { let p: P = new P(1); res.out[0] = p.mutate(); }
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
