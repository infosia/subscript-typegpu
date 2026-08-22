// expected-rule: RN4
import { FragmentInvocation, RenderPipelineSpec, VertexInvocation, renderPipeline } from "./typegpu";
import { Vec4f } from "./typegpu-types";
class Vertex {}
@CStruct class Varyings {
  position: Vec4f;
  constructor(position: Vec4f) { this.position = position; }
}
function vert(value: Vertex, ctx: VertexInvocation): Varyings { return new Varyings(new Vec4f(0.0, 0.0, 0.0, 1.0)); }
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
export const bad: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
