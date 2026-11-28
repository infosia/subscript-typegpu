@group(0u) @binding(0u) var<storage, read_write> words: array<f32>;

@compute @workgroup_size(1u, 1u, 1u)
fn writeWords() {
  var m2 = mat2x2<f32>(vec2<f32>(1.0f, 2.0f), vec2<f32>(3.0f, 4.0f));
  m2[0u] = vec2<f32>(5.0f, 6.0f);
  var weights2 = vec2<f32>(1.0f, 2.0f);
  words[0u] = dot(m2[0u], weights2);
  words[1u] = dot(m2[1u], weights2);
  var m3 = mat3x3<f32>(vec3<f32>(11.0f, 12.0f, 13.0f), vec3<f32>(14.0f, 15.0f, 16.0f), vec3<f32>(17.0f, 18.0f, 19.0f));
  m3[1u].y = 20.0f;
  var weights3 = vec3<f32>(1.0f, 2.0f, 4.0f);
  words[2u] = dot(m3[0u], weights3);
  words[3u] = dot(m3[1u], weights3);
  words[4u] = dot(m3[2u], weights3);
  var m4 = mat4x4<f32>(vec4<f32>(21.0f, 22.0f, 23.0f, 24.0f), vec4<f32>(25.0f, 26.0f, 27.0f, 28.0f), vec4<f32>(29.0f, 30.0f, 31.0f, 32.0f), vec4<f32>(33.0f, 34.0f, 35.0f, 36.0f));
  m4[3u].w += 4.0f;
  var weights4 = vec4<f32>(1.0f, 2.0f, 4.0f, 8.0f);
  words[5u] = dot(m4[0u], weights4);
  words[6u] = dot(m4[1u], weights4);
  words[7u] = dot(m4[2u], weights4);
  words[8u] = dot(m4[3u], weights4);
}
