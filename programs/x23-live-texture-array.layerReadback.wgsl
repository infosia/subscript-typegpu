@group(0u) @binding(0u) var source: texture_2d<f32>;
@group(0u) @binding(1u) var<storage, read_write> output: array<vec4<f32>>;

const WIDTH: u32 = 2u;
const HEIGHT: u32 = 2u;

@compute @workgroup_size(2u, 2u, 1u)
fn layerReadbackKernel(@builtin(global_invocation_id) globalId: vec3<u32>) {
  if (globalId.x >= WIDTH || globalId.y >= HEIGHT) {
    return;
  }
  var coords = vec2<i32>(i32(globalId.x), i32(globalId.y));
  let index = globalId.y * WIDTH + globalId.x;
  output[index] = textureLoad(source, coords, 0u);
}
