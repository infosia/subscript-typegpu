// expected-rule: K20
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage, WorkgroupArray, workgroupArray } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
const INITIAL_LENGTH: u32 = 4;
const shared: WorkgroupArray<u32> = workgroupArray<u32>(INITIAL_LENGTH);
function kernel(res: Layout, ctx: ComputeInvocation): void { shared[0] = 1; res.output[0] = new Item(shared[0]); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [1, 1, 1] });
