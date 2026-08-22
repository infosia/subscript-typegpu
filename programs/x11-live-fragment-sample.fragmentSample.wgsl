struct FragmentVertex {
  @location(0) position: vec2<f32>,
  @location(1) uv: vec2<f32>,
}

struct FragmentVarying {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var nearest: sampler;

@vertex
fn fragmentVertex(value: FragmentVertex) -> FragmentVarying {
  return FragmentVarying(vec4<f32>(value.position.x, value.position.y, 0.0f, 1.0f), value.uv);
}

@fragment
fn fragmentColor(input: FragmentVarying) -> @location(0) vec4<f32> {
  return textureSample(source, nearest, input.uv);
}
