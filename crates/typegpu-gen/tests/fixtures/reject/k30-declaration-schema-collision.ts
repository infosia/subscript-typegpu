// expected-rule: K30
// expected-owner: author
// expected-message: collides

import { ComputeInvocation, ComputePipelineSpec, computePipeline, Storage, wgslDeclarations } from "./typegpu";

@CStruct
class Collision {
  value: u32;
  constructor(value: u32) { this.value = value; }
}

wgslDeclarations("struct Collision { value: u32, }");

class CollisionLayout { input!: Storage<Collision>; }
function collisionKernel(res: CollisionLayout, ctx: ComputeInvocation): void {
  const value: Collision = res.input.get(ctx.globalId.x);
}
export const rejected: ComputePipelineSpec = computePipeline<CollisionLayout>(collisionKernel, {
  name: "rejected",
  workgroupSize: [1, 1, 1],
});
