// expected-rule: K14
// expected-message: two program classes share the source name `BitonicSortResources`
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage } from "./typegpu";
import { BitonicSortResources as SortResources, bitonicSortStep } from "./typegpu-sort";
class BitonicSortResources { output: MutStorage<u32>; constructor(output: MutStorage<u32>) { this.output = output; } }
function kernel(res: BitonicSortResources, ctx: ComputeInvocation): void {
  res.output[ctx.globalId.x] = 1;
}
const sort: ComputePipelineSpec = computePipeline<SortResources>(bitonicSortStep, { name: "sort", workgroupSize: [256, 1, 1] });
const pipeline: ComputePipelineSpec = computePipeline<BitonicSortResources>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
