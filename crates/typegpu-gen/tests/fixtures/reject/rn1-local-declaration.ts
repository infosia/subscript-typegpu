// expected-rule: RN1
import { FragmentInvocation, RenderPipelineSpec, VertexInvocation, renderPipeline } from "./typegpu";
import { Vec4f } from "./typegpu-types";
@CStruct class Vertex {
  position: Vec4f;
  constructor(position: Vec4f) { this.position = position; }
}
@CStruct class Varyings {
  position: Vec4f;
  constructor(position: Vec4f) { this.position = position; }
}
function vert(value: Vertex, ctx: VertexInvocation): Varyings { return new Varyings(value.position); }
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
function local(): void {
  const bad: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
}
