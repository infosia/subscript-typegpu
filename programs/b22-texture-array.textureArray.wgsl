@group(0u) @binding(0u) var sampled: texture_2d_array<f32>;
@group(0u) @binding(1u) var source: texture_storage_2d_array<rgba16float, read>;
@group(0u) @binding(2u) var target_: texture_storage_2d_array<rgba16float, write>;

const WIDTH: u32 = 2u;
const HEIGHT: u32 = 1u;
const LAYERS: u32 = 2u;

@compute @workgroup_size(2u, 1u, 1u)
fn textureArrayKernel(@builtin(global_invocation_id) globalId: vec3<u32>) {
  if (globalId.x >= WIDTH || globalId.y >= HEIGHT || globalId.z >= LAYERS) {
    return;
  }
  var coords = vec2<i32>(i32(globalId.x), i32(globalId.y));
  let layer = i32(globalId.z);
  var sampled_ = textureLoad(sampled, coords, layer, 0u);
  var stored = textureLoad(source, coords, layer);
  textureStore(target_, coords, layer, sampled_ + stored);
}
