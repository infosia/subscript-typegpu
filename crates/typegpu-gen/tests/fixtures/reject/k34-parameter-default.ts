// expected-rule: K34
// expected-message: default for parameter `bias` of `pair`
import { ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { constructor(public output: MutStorage<u32>) {} }
function pair(value: u32, bias: u32 = value): u32 { return value + bias; }
function kernel(res: Layout): void { res.output[0] = pair(2); }
const pipeline: ComputePipelineSpec = computePipeline(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
