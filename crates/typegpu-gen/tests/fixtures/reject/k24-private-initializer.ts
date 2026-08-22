// expected-rule: K20
// expected-message: private variable `state` initializer is not evaluable
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage, PrivateVar, privateVar } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
function initial(): u32 { return 1; }
const state: PrivateVar<u32> = privateVar<u32>(initial());
function kernel(res: Layout, ctx: ComputeInvocation): void { res.output[0] = new Item(state.get()); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [1, 1, 1] });
