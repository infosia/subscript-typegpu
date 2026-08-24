struct FragmentVertex {
  @location(0u) position: vec2<f32>,
  @location(1u) uv: vec2<f32>,
}

struct FragmentVarying {
  @builtin(position) position: vec4<f32>,
  @location(0u) uv: vec2<f32>,
}

@group(0u) @binding(0u) var source: texture_2d<f32>;
@group(0u) @binding(1u) var nearest: sampler;

@vertex
fn fragmentVertex(value: FragmentVertex) -> FragmentVarying {
  return FragmentVarying(vec4<f32>(value.position.x, value.position.y, 0.0f, 1.0f), value.uv);
}

@fragment
fn fragmentColor(input: FragmentVarying) -> @location(0u) vec4<f32> {
  return textureSample(source, nearest, input.uv);
}
