struct Vertex {
  @location(0u) position: vec4<f32>,
  @location(1u) color: vec4<f32>,
}

struct Varyings {
  @builtin(position) position: vec4<f32>,
  @location(0u) color: vec4<f32>,
}

@vertex
fn vertexStep(value: Vertex) -> Varyings {
  return Varyings(value.position, value.color);
}

@fragment
fn fragmentStep(value: Varyings) -> @location(0u) vec4<f32> {
  return value.color;
}
