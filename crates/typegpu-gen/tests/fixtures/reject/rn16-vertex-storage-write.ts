// expected-rule: RN9
import { FragmentInvocation, MutStorage, renderPipelineL, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@ValueType class Item { value: f32; constructor(value: f32) { this.value = value; } }
@ValueType class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@ValueType class Varyings { position: Vec4f; constructor(position: Vec4f) { this.position = position; } }
class Layout { values: MutStorage<Item>; constructor(values: MutStorage<Item>) { this.values = values; } }
function vert(res: Layout, value: Vertex, ctx: VertexInvocation): Varyings { res.values[0] = new Item(1.0); return new Varyings(new Vec4f(0.0, 0.0, 0.0, 1.0)); }
function frag(res: Layout, input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
const pipeline: RenderPipelineSpec = renderPipelineL<Layout, Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
