// expected-rule: RN9
import { FragmentInvocation, renderPipelineL, RenderPipelineSpec, Storage, Uniform, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@ValueType class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@ValueType class Offset { value: Vec4f; constructor(value: Vec4f) { this.value = value; } }
@ValueType class Tint { value: Vec4f; constructor(value: Vec4f) { this.value = value; } }
@ValueType class Varyings { position: Vec4f; constructor(position: Vec4f) { this.position = position; } }
class Layout { params: Uniform<Offset>; unused: Storage<Tint>; constructor(params: Uniform<Offset>, unused: Storage<Tint>) { this.params = params; this.unused = unused; } }
function vert(res: Layout, value: Vertex, ctx: VertexInvocation): Varyings { const offset: Offset = res.params.$; return new Varyings(new Vec4f(value.position.x + offset.value.x, value.position.y + offset.value.y, 0.0, 1.0)); }
function frag(res: Layout, input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
const pipeline: RenderPipelineSpec = renderPipelineL<Layout, Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
