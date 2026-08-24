struct SampleParams {
  width: u32,
  height: u32,
}

@group(0u) @binding(0u) var source: texture_2d<f32>;
@group(0u) @binding(1u) var nearest: sampler;
@group(0u) @binding(2u) var target_: texture_storage_2d<rgba8unorm, write>;
@group(1u) @binding(0u) var<uniform> params: SampleParams;

@compute @workgroup_size(4u, 4u, 1u)
fn textureKernel(@builtin(global_invocation_id) globalId: vec3<u32>) {
  var params_ = params;
  if (globalId.x >= params_.width || globalId.y >= params_.height) {
    return;
  }
  var coords = vec2<i32>(i32(globalId.x), i32(globalId.y));
  var loaded = textureLoad(source, coords, 0u);
  var uv = vec2<f32>((f32(globalId.x) + 0.5f) / f32(params_.width), (f32(globalId.y) + 0.5f) / f32(params_.height));
  var sampled = textureSampleLevel(source, nearest, uv, 0.0f);
  textureStore(target_, coords, (loaded + sampled) * 0.5f);
}
