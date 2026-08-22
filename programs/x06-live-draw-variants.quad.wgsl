struct Vertex {
  @location(0) position: vec2<f32>,
}

struct Instance {
  @location(1) offset: vec2<f32>,
  @location(2) color: vec3<f32>,
}

struct Varyings {
  @builtin(position) position: vec4<f32>,
  @location(0) color: vec3<f32>,
}

@vertex
fn quadVert(value: Vertex, instance: Instance) -> Varyings {
  return Varyings(vec4<f32>(value.position.x + instance.offset.x, value.position.y + instance.offset.y, 0.0f, 1.0f), instance.color);
}

@fragment
fn frag(input: Varyings) -> @location(0) vec4<f32> {
  return vec4<f32>(input.color.x, input.color.y, input.color.z, 1.0f);
}
