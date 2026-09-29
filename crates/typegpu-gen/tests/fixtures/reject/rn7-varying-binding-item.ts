// expected-rule: RN7
import { FragmentInvocation, renderPipelineL, RenderPipelineSpec, Storage, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@ValueType class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@ValueType class Varyings { position: Vec4f; constructor(position: Vec4f) { this.position = position; } }
class Layout { values: Storage<Varyings>; constructor(values: Storage<Varyings>) { this.values = values; } }
function vert(res: Layout, value: Vertex, ctx: VertexInvocation): Varyings { return res.values[0]; }
function frag(res: Layout, input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
const pipeline: RenderPipelineSpec = renderPipelineL<Layout, Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
