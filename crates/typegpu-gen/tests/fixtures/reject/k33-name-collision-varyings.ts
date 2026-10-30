// expected-rule: K14
// expected-message: two program declarations share the source name `id_f32`
import { FragmentInvocation, renderPipeline, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@ValueType class Vertex { constructor(public position: Vec2f) {} }
@ValueType class id_f32 { constructor(public position: Vec4f) {} }
function id<T>(value: T): T { return value; }
function vert(value: Vertex, ctx: VertexInvocation): id_f32 { return new id_f32(new Vec4f(value.position.x, value.position.y, 0.0, 1.0)); }
function frag(input: id_f32, ctx: FragmentInvocation): Vec4f { return new Vec4f(id<f32>(1.0), 0.0, 0.0, 1.0); }
const pipeline: RenderPipelineSpec = renderPipeline<Vertex, id_f32>(vert, frag, { format: "rgba8unorm" });
