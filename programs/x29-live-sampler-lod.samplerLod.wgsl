@group(0u) @binding(0u) var source: texture_2d<f32>;
@group(0u) @binding(1u) var nearest: sampler;
@group(0u) @binding(2u) var clamped: sampler;
@group(0u) @binding(3u) var<storage, read_write> output: array<vec4<f32>>;

@compute @workgroup_size(1u, 1u, 1u)
fn sampleKernel() {
  var uv = vec2<f32>(0.75f, 0.5f);
  output[0u] = textureSampleLevel(source, nearest, uv, 0.0f);
  output[1u] = textureSampleLevel(source, clamped, uv, 0.0f);
}
