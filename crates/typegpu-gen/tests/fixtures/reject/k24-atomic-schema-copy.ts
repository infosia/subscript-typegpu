// expected-rule: K21
// expected-message: contains an atomic cannot be copied to a local
import { AtomicU32 } from "./typegpu-types";
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage } from "./typegpu";
@CStruct class Counter { value: AtomicU32; constructor(value: AtomicU32) { this.value = value; } }
class Layout { counters!: MutStorage<Counter>; }
function kernel(res: Layout, ctx: ComputeInvocation): void { const copy: Counter = res.counters[0]; copy.value.load(); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
