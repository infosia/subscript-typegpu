// expected-rule: RN18
// expected-owner: author
// expected-message: indexFormat

import { FragmentInvocation, RenderPipelineSpec, renderPipeline, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";

@CStruct class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@CStruct class Varyings { position: Vec4f; constructor(position: Vec4f) { this.position = position; } }
function vertex(value: Vertex, ctx: VertexInvocation): Varyings { return new Varyings(new Vec4f(value.position.x, value.position.y, 0.0, 1.0)); }
function fragment(value: Varyings, ctx: FragmentInvocation): Vec4f { return new Vec4f(1.0, 1.0, 1.0, 1.0); }
const format: GPUIndexFormat = "uint16";

export const rejected: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vertex, fragment, {
  format: "rgba8unorm",
  indexFormat: format,
});
