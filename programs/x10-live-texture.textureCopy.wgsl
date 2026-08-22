@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var nearest: sampler;
@group(0) @binding(2) var target_: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(4, 4, 1)
fn textureCopyKernel(@builtin(global_invocation_id) globalId: vec3<u32>) {
  if (globalId.x >= 4u || globalId.y >= 4u) {
    return;
  }
  var uv = vec2<f32>((f32(globalId.x) + 0.25f) / 4.0f, (f32(globalId.y) + 0.25f) / 4.0f);
  var color = textureSampleLevel(source, nearest, uv, 0.0f);
  textureStore(target_, vec2<i32>(i32(globalId.x), i32(globalId.y)), color);
}
