struct Vertex {
  @location(0u) position: vec2<f32>,
  @location(1u) color: vec4<f32>,
}

struct Varyings {
  @builtin(position) position: vec4<f32>,
  @location(0u) color: vec4<f32>,
}

@vertex
fn vertexStep(value: Vertex) -> Varyings {
  return Varyings(vec4<f32>(value.position.x, value.position.y, 0.0f, 1.0f), value.color);
}

@fragment
fn fragmentStep(value: Varyings) -> @location(0u) vec4<f32> {
  return value.color;
}
