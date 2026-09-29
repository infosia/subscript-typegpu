// expected-rule: S100
// expected-owner: checker
// expected-message: type mismatch: the argument expects `(Vertex, DeclaredInstance, VertexInvocation) => Varyings`, got `(Vertex, KernelInstance, VertexInvocation) => Varyings`
import { FragmentInvocation, renderPipelineInstanced, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@ValueType class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@ValueType class DeclaredInstance { offset: Vec2f; constructor(offset: Vec2f) { this.offset = offset; } }
@ValueType class KernelInstance { offset: Vec2f; constructor(offset: Vec2f) { this.offset = offset; } }
@ValueType class Varyings { position: Vec4f; constructor(position: Vec4f) { this.position = position; } }
function vert(value: Vertex, instance: KernelInstance, ctx: VertexInvocation): Varyings { return new Varyings(new Vec4f(value.position.x + instance.offset.x, value.position.y + instance.offset.y, 0.0, 1.0)); }
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
const pipeline: RenderPipelineSpec = renderPipelineInstanced<Vertex, DeclaredInstance, Varyings>(vert, frag, { format: "rgba8unorm" });
