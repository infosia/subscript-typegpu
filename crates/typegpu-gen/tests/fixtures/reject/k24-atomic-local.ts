// expected-rule: K21
// expected-message: atomic method receiver is not a storage or workgroup place
import { AtomicU32 } from "./typegpu-types";
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
function kernel(res: Layout, ctx: ComputeInvocation): void { const local: AtomicU32 = new AtomicU32(0); res.output[0] = new Item(local.add(1)); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [1, 1, 1] });
