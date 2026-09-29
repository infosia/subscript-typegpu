// expected-rule: RN7
import { FragmentInvocation, renderPipeline, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@ValueType class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@ValueType class Payload { value: f32; constructor(value: f32) { this.value = value; } }
@ValueType class Varyings { position: Vec4f; bad: Payload; constructor(position: Vec4f, bad: Payload) { this.position = position; this.bad = bad; } }
function vert(value: Vertex, ctx: VertexInvocation): Varyings { return new Varyings(new Vec4f(value.position.x, value.position.y, 0.0, 1.0), new Payload(1.0)); }
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
const pipeline: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
