// expected-rule: K14
// expected-message: two program declarations share the source name `id_f32`
import { FragmentInvocation, renderPipeline, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@ValueType class id_f32 { constructor(public position: Vec2f) {} }
@ValueType class Varyings { constructor(public position: Vec4f) {} }
function id<T>(value: T): T { return value; }
function vert(value: id_f32, ctx: VertexInvocation): Varyings { return new Varyings(new Vec4f(value.position.x, value.position.y, 0.0, 1.0)); }
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f { return new Vec4f(id<f32>(1.0), 0.0, 0.0, 1.0); }
const pipeline: RenderPipelineSpec = renderPipeline<id_f32, Varyings>(vert, frag, { format: "rgba8unorm" });
