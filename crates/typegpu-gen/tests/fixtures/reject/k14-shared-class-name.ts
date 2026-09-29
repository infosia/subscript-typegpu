// expected-rule: K14
// expected-message: two program classes share the source name `RandomF32`
import { randF32 } from "./typegpu-noise";
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage } from "./typegpu";
@ValueType
class RandomF32 { seed: u32; scale: f32; weight: f32; constructor(seed: u32, scale: f32, weight: f32) { this.seed = seed; this.scale = scale; this.weight = weight; } }
class Layout { output: MutStorage<RandomF32>; constructor(output: MutStorage<RandomF32>) { this.output = output; } }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  const sample = randF32(7);
  res.output[0] = new RandomF32(sample.state, sample.value, 1.0);
}
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
