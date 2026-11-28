struct Batch {
  slots: array<Item, 2u>,
  scale: f32,
}

struct Item {
  value: u32,
}

struct Vertex {
  @location(0u) xy: vec2<f32>,
}

struct Varyings {
  @builtin(position) position: vec4<f32>,
}

@group(0u) @binding(0u) var<storage, read> data: array<Batch>;

@vertex
fn vert(vertex: Vertex) -> Varyings {
  return Varyings(vec4<f32>(vertex.xy.x, vertex.xy.y, 0.0f, 1.0f));
}

@fragment
fn frag(input: Varyings) -> @location(0u) vec4<f32> {
  let scale = data[0u].scale;
  return vec4<f32>(scale, 0.0f, 0.0f, 1.0f);
}
