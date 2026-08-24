@group(0) @binding(0) var source: texture_storage_2d<r32float, read>;
@group(0) @binding(1) var target_: texture_storage_2d<r32float, read_write>;

@compute @workgroup_size(2, 2, 1)
fn readStorageKernel(@builtin(global_invocation_id) globalId: vec3<u32>) {
  var size = textureDimensions(source);
  if (globalId.x >= size.x || globalId.y >= size.y) {
    return;
  }
  var coords = vec2<i32>(i32(globalId.x), i32(globalId.y));
  var source_ = textureLoad(source, coords);
  var target__ = textureLoad(target_, coords);
  textureStore(target_, coords, source_ + target__);
}
