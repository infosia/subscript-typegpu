@group(0u) @binding(0u) var source: texture_storage_2d_array<rgba16float, read>;
@group(0u) @binding(1u) var target_: texture_storage_2d_array<rgba16float, write>;

const WIDTH: u32 = 2u;
const HEIGHT: u32 = 2u;
const LAYERS: u32 = 2u;

@compute @workgroup_size(2u, 2u, 1u)
fn arrayPingPongKernel(@builtin(global_invocation_id) globalId: vec3<u32>) {
  if (globalId.x >= WIDTH || globalId.y >= HEIGHT || globalId.z >= LAYERS) {
    return;
  }
  var coords = vec2<i32>(i32(globalId.x), i32(globalId.y));
  let layer = i32(globalId.z);
  var _g_conditional_0: i32;
  if (layer == 0i) {
    _g_conditional_0 = 1i;
  } else {
    _g_conditional_0 = 0i;
  }
  let pairedLayer = _g_conditional_0;
  var _g_conditional_1: f32;
  if (layer == 0i) {
    _g_conditional_1 = 0.5f;
  } else {
    _g_conditional_1 = 0.0f;
  }
  let pairScale = _g_conditional_1;
  var _g_conditional_2: vec4<f32>;
  if (layer == 1i) {
    _g_conditional_2 = vec4<f32>(1.0f, 1.0f, 0.0f, 0.0f);
  } else {
    _g_conditional_2 = vec4<f32>(0.0f, 0.0f, 0.0f, 0.0f);
  }
  var coordinateStep = _g_conditional_2;
  var current = textureLoad(source, coords, layer);
  var paired = textureLoad(source, coords, pairedLayer);
  textureStore(target_, coords, layer, current + paired * pairScale + coordinateStep);
}
