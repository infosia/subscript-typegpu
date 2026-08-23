// expected-rule: PI5
import { Vec3b } from "./typegpu-types";
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";

class Layout {
  values!: Storage<Vec3b>;
}

function kernel(res: Layout, ctx: ComputeInvocation): void {}

export const pipeline: ComputePipelineSpec = computePipeline<Layout>(
  kernel,
  { name: "pipeline", workgroupSize: [1, 1, 1] },
);
