// expected-rule: RN9
import { FragmentInvocation, renderPipelineL, RenderPipelineSpec, Storage, Uniform, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@CStruct class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@CStruct class Offset { value: Vec4f; constructor(value: Vec4f) { this.value = value; } }
@CStruct class Tint { value: Vec4f; constructor(value: Vec4f) { this.value = value; } }
@CStruct class Varyings { position: Vec4f; constructor(position: Vec4f) { this.position = position; } }
class Layout { params!: Uniform<Offset>; unused!: Storage<Tint>; }
function vert(res: Layout, value: Vertex, ctx: VertexInvocation): Varyings { const offset: Offset = res.params.$; return new Varyings(new Vec4f(value.position.x + offset.value.x, value.position.y + offset.value.y, 0.0, 1.0)); }
function frag(res: Layout, input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
export const pipeline: RenderPipelineSpec = renderPipelineL<Layout, Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
