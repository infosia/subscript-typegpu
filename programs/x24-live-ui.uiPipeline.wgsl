struct UiViewport {
  width: f32,
  height: f32,
}

struct UiVertex {
  @location(0u) position: vec2<f32>,
  @location(1u) uv: vec2<f32>,
  @location(2u) color: u32,
}

struct UiVarying {
  @builtin(position) position: vec4<f32>,
  @location(0u) uv: vec2<f32>,
  @location(1u) color: vec4<f32>,
}

@group(0u) @binding(0u) var<uniform> viewport: UiViewport;
@group(0u) @binding(1u) var atlas: texture_2d<f32>;
@group(0u) @binding(2u) var nearest: sampler;

@vertex
fn uiVertex(vertex: UiVertex) -> UiVarying {
  return UiVarying(vec4<f32>(vertex.position.x * 2.0f / viewport.width - 1.0f, 1.0f - vertex.position.y * 2.0f / viewport.height, 0.0f, 1.0f), vertex.uv, vec4<f32>(f32(vertex.color % 256u) / 255.0f, f32(vertex.color / 256u % 256u) / 255.0f, f32(vertex.color / 65536u % 256u) / 255.0f, f32(vertex.color / 16777216u) / 255.0f));
}

@fragment
fn uiFragment(input: UiVarying) -> @location(0u) vec4<f32> {
  let alpha = textureSample(atlas, nearest, input.uv).x;
  return vec4<f32>(input.color.x, input.color.y, input.color.z, input.color.w * alpha);
}
