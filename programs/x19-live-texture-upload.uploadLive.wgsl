@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var nearest: sampler;
@group(0) @binding(2) var<storage, read_write> output: array<vec4<f32>>;

const WIDTH: u32 = 64u;
const HEIGHT: u32 = 2u;

@compute @workgroup_size(8, 1, 1)
fn uploadLiveKernel(@builtin(global_invocation_id) globalId: vec3<u32>) {
  if (globalId.x >= WIDTH || globalId.y >= HEIGHT) {
    return;
  }
  let index = globalId.y * WIDTH + globalId.x;
  var uv = vec2<f32>((f32(globalId.x) + 0.5f) / f32(WIDTH), (f32(globalId.y) + 0.5f) / f32(HEIGHT));
  output[index] = textureSampleLevel(source, nearest, uv, 0.0f);
}
