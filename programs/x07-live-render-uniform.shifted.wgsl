struct Offset {
  value: vec4<f32>,
}

struct Tint {
  value: vec4<f32>,
}

struct Vertex {
  @location(0) position: vec2<f32>,
}

struct Varyings {
  @builtin(position) position: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: Offset;
@group(0) @binding(1) var<storage, read> tint: array<Tint>;

@vertex
fn vert(value: Vertex) -> Varyings {
  var offset = params;
  return Varyings(vec4<f32>(value.position.x + offset.value.x, value.position.y + offset.value.y, 0.0f, 1.0f));
}

@fragment
fn frag(input: Varyings) -> @location(0) vec4<f32> {
  var color = tint[0u];
  return color.value;
}
