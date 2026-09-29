// expected-rule: K14
// expected-message: two program declarations share the source name `sdfMin` in one WGSL module
import { Vec2f } from "./typegpu-types";
import { sdBox2d } from "./typegpu-sdf";
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage } from "./typegpu";
class Layout { output: MutStorage<f32>; constructor(output: MutStorage<f32>) { this.output = output; } }
function sdfMin(a: f32, b: f32): f32 { return a < b ? a : b; }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  const box: f32 = sdBox2d(new Vec2f(1.0, 2.0), new Vec2f(0.5, 0.5), new Vec2f(0.25, 0.75));
  res.output[0] = sdfMin(box, 1.0);
}
const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
