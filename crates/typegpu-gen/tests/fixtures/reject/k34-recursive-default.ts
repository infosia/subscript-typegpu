// expected-rule: K34
// expected-message: default for parameter `a` of `f` recursively requires itself
import { ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { constructor(public output: MutStorage<u32>) {} }
function f(a: u32 = f()): u32 { return a; }
function kernel(res: Layout): void { res.output[0] = f(); }
const pipeline: ComputePipelineSpec = computePipeline(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
