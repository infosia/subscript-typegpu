// expected-rule: RN8
import { FragmentInvocation, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@CStruct class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@CStruct class Varyings { position: Vec4f; constructor(position: Vec4f) { this.position = position; } }
function renderPipeline<V, O>(vertex: (value: V, ctx: VertexInvocation) => O, fragment: (input: O, ctx: FragmentInvocation) => f32, spec: RenderPipelineSpec): RenderPipelineSpec { return spec; }
function vert(value: Vertex, ctx: VertexInvocation): Varyings { return new Varyings(new Vec4f(0.0, 0.0, 0.0, 1.0)); }
function frag(input: Varyings, ctx: FragmentInvocation): f32 { return 1.0; }
export const pipeline: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
