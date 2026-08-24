struct Vertex {
  @location(0u) position: vec2<f32>,
  @location(1u) color: vec3<f32>,
}

struct Varyings {
  @builtin(position) position: vec4<f32>,
  @location(0u) color: vec3<f32>,
}

const FRAGMENT_ALPHA: f32 = 1.0f;

@vertex
fn vert(value: Vertex) -> Varyings {
  return Varyings(vec4<f32>(value.position.x, value.position.y, 0.0f, 1.0f), value.color);
}

@fragment
fn frag(input: Varyings) -> @location(0u) vec4<f32> {
  return vec4<f32>(input.color.x, input.color.y, input.color.z, FRAGMENT_ALPHA);
}
