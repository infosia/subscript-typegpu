// expected-rule: RN2
import { FragmentInvocation, RenderPipelineSpec } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@CStruct class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@CStruct class Varyings { position: Vec4f; constructor(position: Vec4f) { this.position = position; } }
function renderPipeline<V, O>(vertex: (value: V) => O, fragment: (input: O, ctx: FragmentInvocation) => Vec4f, spec: RenderPipelineSpec): RenderPipelineSpec { return spec; }
function vert(value: Vertex): Varyings { return new Varyings(new Vec4f(0.0, 0.0, 0.0, 1.0)); }
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
export const pipeline: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
