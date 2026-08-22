// expected-rule: RN7
import { FragmentInvocation, renderPipelineL, RenderPipelineSpec, Storage, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@CStruct class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@CStruct class Varyings { position: Vec4f; constructor(position: Vec4f) { this.position = position; } }
class Layout { values!: Storage<Varyings>; }
function vert(res: Layout, value: Vertex, ctx: VertexInvocation): Varyings { return res.values[0]; }
function frag(res: Layout, input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
export const pipeline: RenderPipelineSpec = renderPipelineL<Layout, Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
