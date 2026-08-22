// expected-rule: K10
import { FragmentInvocation, renderPipeline, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@CStruct class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@CStruct class Varyings { position: Vec4f; constructor(position: Vec4f) { this.position = position; } }
function vert(value: Vertex, ctx: VertexInvocation): Varyings { return new Varyings(new Vec4f(value.position.x, value.position.y, 0.0, 1.0)); }
@CStruct class Sample { value: Vec4f; constructor(value: Vec4f) { this.value = value; } textureSample(): Vec4f { return this.value; } }
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f { const sample = new Sample(input.position); return sample.textureSample(); }
export const pipeline: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
