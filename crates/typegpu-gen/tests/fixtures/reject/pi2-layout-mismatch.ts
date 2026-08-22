// expected-rule: S100
// expected-owner: checker
// expected-message: type mismatch: the argument expects `(Declared, ComputeInvocation) => void`, got `(KernelLayout, ComputeInvocation) => void`
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Item { value: f32; constructor(value: f32) { this.value = value; } }
class Declared { input!: Storage<Item>; }
class KernelLayout { input!: Storage<Item>; }
function kernel(res: KernelLayout, ctx: ComputeInvocation): void { const value: Item = res.input[0]; }
export const pipeline: ComputePipelineSpec = computePipeline<Declared>(kernel, { workgroupSize: [1, 1, 1] });
