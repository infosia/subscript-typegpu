// expected-rule: RN7
import { FragmentInvocation, renderPipeline, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@CStruct class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@CStruct class Varyings { position: Vec4f; bad: boolean; constructor(position: Vec4f, bad: boolean) { this.position = position; this.bad = bad; } }
function vert(value: Vertex, ctx: VertexInvocation): Varyings { return new Varyings(new Vec4f(value.position.x, value.position.y, 0.0, 1.0), true); }
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
export const pipeline: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
