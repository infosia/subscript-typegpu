import { FragmentInvocation, renderPipeline, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec4f } from "./typegpu-types";
import { gain as left } from "../shared/left";
import { gain as right } from "./right";
import { tri_VERTEX_ENTRY } from "./render.typegpu";

@ValueType class Vertex { x: f32 = 0.0; }
@ValueType class Varyings {
  position: Vec4f;
  constructor(position: Vec4f) { this.position = position; }
}

function vert(value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(new Vec4f(value.x, 0.0, 0.0, 1.0));
}

function frag(input: Varyings, ctx: FragmentInvocation): Vec4f {
  return new Vec4f(left(), right(), 0.0, 1.0).scale(1.0);
}

const tri: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
