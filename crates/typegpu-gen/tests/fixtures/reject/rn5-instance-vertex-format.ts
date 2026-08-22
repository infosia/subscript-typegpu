// expected-rule: RN5
import { FragmentInvocation, renderPipelineInstanced, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec2f, Vec3h, Vec4f } from "./typegpu-types";
@CStruct class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@CStruct class Instance { value: Vec3h; constructor(value: Vec3h) { this.value = value; } }
@CStruct class Varyings { position: Vec4f; constructor(position: Vec4f) { this.position = position; } }
function vert(value: Vertex, instance: Instance, ctx: VertexInvocation): Varyings { return new Varyings(new Vec4f(value.position.x, value.position.y, 0.0, 1.0)); }
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
export const pipeline: RenderPipelineSpec = renderPipelineInstanced<Vertex, Instance, Varyings>(vert, frag, { format: "rgba8unorm" });
