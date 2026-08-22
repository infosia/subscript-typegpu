// expected-rule: S100
// expected-owner: checker
// expected-message: type mismatch: the argument expects `(Varyings, FragmentInvocation) => Vec4f`, got `(Varyings, FragmentInvocation) => f32`
import { FragmentInvocation, renderPipeline, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@CStruct class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@CStruct class Varyings { position: Vec4f; constructor(position: Vec4f) { this.position = position; } }
function vert(value: Vertex, ctx: VertexInvocation): Varyings { return new Varyings(new Vec4f(value.position.x, value.position.y, 0.0, 1.0)); }
function frag(input: Varyings, ctx: FragmentInvocation): f32 { return 1.0; }
export const pipeline: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
