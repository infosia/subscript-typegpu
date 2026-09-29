// expected-rule: S100
// expected-owner: checker
// expected-message: type mismatch: the argument expects `(Vertex, VertexInvocation) => Varyings`, got `(Vertex, FragmentInvocation) => Varyings`
import { FragmentInvocation, renderPipeline, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@ValueType class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@ValueType class Varyings { position: Vec4f; constructor(position: Vec4f) { this.position = position; } }
function vert(value: Vertex, ctx: FragmentInvocation): Varyings { return new Varyings(new Vec4f(value.position.x, value.position.y, 0.0, 1.0)); }
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
const pipeline: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
