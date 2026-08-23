// expected-rule: K21
// expected-message: cannot be written as a whole
import { AtomicU32 } from "./typegpu-types";
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage } from "./typegpu";
@CStruct class Counter { value: AtomicU32; constructor(value: AtomicU32) { this.value = value; } }
class Layout { counters!: MutStorage<Counter>; }
function kernel(res: Layout, ctx: ComputeInvocation): void { res.counters[0] = new Counter(new AtomicU32(1)); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
