// expected-rule: K8
// expected-message: a kernel cannot write an invocation builtin `ctx.frontFacing`
import { FragmentInvocation, RenderPipelineSpec, renderPipeline } from "./typegpu";
import { Vec4f } from "./typegpu-types";
@ValueType class Vertex { constructor(public xyzw: Vec4f) {} }
@ValueType class Varyings { constructor(public position: Vec4f) {} }
function vert(vertex: Vertex): Varyings { return new Varyings(vertex.xyzw); }
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f {
  ctx.frontFacing = true;
  return input.position;
}
const pipeline: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
