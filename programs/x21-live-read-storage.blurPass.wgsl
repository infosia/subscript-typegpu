@group(0u) @binding(0u) var source: texture_storage_2d<r32float, read>;
@group(0u) @binding(1u) var target_: texture_storage_2d<r32float, write>;

@compute @workgroup_size(4u, 4u, 1u)
fn blurKernel(@builtin(global_invocation_id) globalId: vec3<u32>) {
  var size = textureDimensions(source);
  if (globalId.x >= size.x || globalId.y >= size.y) {
    return;
  }
  let x = i32(globalId.x);
  let y = i32(globalId.y);
  let width = i32(size.x);
  let height = i32(size.y);
  var _g_conditional_0: i32;
  if (x > 0i) {
    _g_conditional_0 = x - 1i;
  } else {
    _g_conditional_0 = x;
  }
  let left = _g_conditional_0;
  var _g_conditional_1: i32;
  if (x + 1i < width) {
    _g_conditional_1 = x + 1i;
  } else {
    _g_conditional_1 = x;
  }
  let right = _g_conditional_1;
  var _g_conditional_2: i32;
  if (y > 0i) {
    _g_conditional_2 = y - 1i;
  } else {
    _g_conditional_2 = y;
  }
  let down = _g_conditional_2;
  var _g_conditional_3: i32;
  if (y + 1i < height) {
    _g_conditional_3 = y + 1i;
  } else {
    _g_conditional_3 = y;
  }
  let up = _g_conditional_3;
  let value = (textureLoad(source, vec2<i32>(x, y)).x + textureLoad(source, vec2<i32>(left, y)).x + textureLoad(source, vec2<i32>(right, y)).x + textureLoad(source, vec2<i32>(x, down)).x + textureLoad(source, vec2<i32>(x, up)).x) * 0.2f;
  textureStore(target_, vec2<i32>(x, y), vec4<f32>(value, 0.0f, 0.0f, 1.0f));
}
