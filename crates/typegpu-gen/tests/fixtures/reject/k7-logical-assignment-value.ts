// expected-rule: K7
// expected-message: an assignment is used as a value
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { out: MutStorage<u32>; constructor(out: MutStorage<u32>) { this.out = out; } }

function kernel(res: Layout, ctx: ComputeInvocation): void { let p: boolean = true; const b: boolean = (p &&= false); if (b) { res.out[0] = 1; } }
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
