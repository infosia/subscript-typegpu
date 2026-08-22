// expected-rule: K21
// expected-message: atomic schema in uniform or read-only storage
import { AtomicU32 } from "./typegpu-types";
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Counter { value: AtomicU32; constructor(value: AtomicU32) { this.value = value; } }
class Layout { counters!: Storage<Counter>; }
function kernel(res: Layout, ctx: ComputeInvocation): void { res.counters[0].value.load(); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [1, 1, 1] });
