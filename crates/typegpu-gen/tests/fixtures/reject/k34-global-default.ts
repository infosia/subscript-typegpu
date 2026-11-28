// expected-rule: K34
// expected-message: default for parameter `a` of `f` uses an unsupported global
import { ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { constructor(public output: MutStorage<u32>) {} }
let seed: u32 = 7;
function f(a: u32 = seed): u32 { return a; }
function kernel(res: Layout): void { res.output[0] = f(2); }
const pipeline: ComputePipelineSpec = computePipeline(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
