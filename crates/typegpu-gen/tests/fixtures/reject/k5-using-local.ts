// expected-rule: K5
// expected-message: `using` declaration in kernel
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Item { value: f32; constructor(value: f32) { this.value = value; } }
class Layout { input!: Storage<Item>; }
class Disposable { [Symbol.dispose](): void {} }
function kernel(res: Layout, ctx: ComputeInvocation): void { using local = new Disposable(); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
