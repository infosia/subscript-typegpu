struct Offset {
  value: vec4<f32>,
}

struct Tint {
  value: vec4<f32>,
}

struct Vertex {
  @location(0u) position: vec2<f32>,
}

struct Varyings {
  @builtin(position) position: vec4<f32>,
}

@group(0u) @binding(0u) var<uniform> params: Offset;
@group(0u) @binding(1u) var<storage, read> tint: array<Tint>;

@vertex
fn vert(value: Vertex) -> Varyings {
  var offset = params;
  return Varyings(vec4<f32>(value.position.x + offset.value.x, value.position.y + offset.value.y, 0.0f, 1.0f));
}

@fragment
fn frag(input: Varyings) -> @location(0u) vec4<f32> {
  var color = tint[0u];
  return color.value;
}
