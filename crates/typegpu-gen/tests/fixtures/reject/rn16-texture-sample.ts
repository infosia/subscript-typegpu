// expected-rule: RN16
import { FragmentInvocation, renderPipeline, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@CStruct class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@CStruct class Varyings { position: Vec4f; constructor(position: Vec4f) { this.position = position; } }
function textureSample(value: Vec4f): Vec4f { return value; }
function vert(value: Vertex, ctx: VertexInvocation): Varyings { return new Varyings(textureSample(new Vec4f(0.0, 0.0, 0.0, 1.0))); }
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
export const pipeline: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
