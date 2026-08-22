struct SampleParams {
  width: u32,
  height: u32,
}

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var linear: sampler;
@group(0) @binding(2) var target_: texture_storage_2d<rgba8unorm, write>;
@group(1) @binding(0) var<uniform> params: SampleParams;

@compute @workgroup_size(4, 4, 1)
fn textureKernel(@builtin(global_invocation_id) globalId: vec3<u32>) {
  var params = params;
  if (globalId.x >= params.width || globalId.y >= params.height) {
    return;
  }
  var coords = vec2<i32>(i32(globalId.x), i32(globalId.y));
  var loaded = textureLoad(source, coords, 0u);
  var uv = vec2<f32>((f32(globalId.x) + 0.5f) / f32(params.width), (f32(globalId.y) + 0.5f) / f32(params.height));
  var sampled = textureSampleLevel(source, linear, uv, 0.0f);
  textureStore(target_, coords, (loaded + sampled) * 0.5f);
}
