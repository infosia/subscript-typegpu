// expected-rule: TX8
// expected-message: gap at group 0
import { ComputeInvocation, ComputePipelineSpec, Storage, computePipeline2 } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class EmptyLayout {}
class DataLayout { values!: Storage<Item>; }
function kernel(empty: EmptyLayout, data: DataLayout, ctx: ComputeInvocation): void {}
export const pipeline: ComputePipelineSpec = computePipeline2<EmptyLayout, DataLayout>(kernel, { workgroupSize: [1, 1, 1] });
