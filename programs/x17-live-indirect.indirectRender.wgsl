struct Vertex {
  @location(0) position: vec2<f32>,
}

struct Varyings {
  @builtin(position) position: vec4<f32>,
}

@vertex
fn vertexStep(value: Vertex) -> Varyings {
  return Varyings(vec4<f32>(value.position.x, value.position.y, 0.0f, 1.0f));
}

@fragment
fn fragmentStep(value: Varyings) -> @location(0) vec4<f32> {
  return vec4<f32>(0.25f, 0.6f, 0.75f, 1.0f);
}
