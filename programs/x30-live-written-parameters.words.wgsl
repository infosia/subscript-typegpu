struct Item {
  value: u32,
}

@group(0u) @binding(0u) var<storage, read_write> words: array<u32>;

fn bumpItem(_g_parameter_item: Item) -> u32 {
  var item = _g_parameter_item;
  item.value += 3u;
  return item.value;
}

fn bumpScalar(_g_parameter_x: u32) -> u32 {
  var x = _g_parameter_x;
  x += 1u;
  return x;
}

fn bumpVector(_g_parameter_v: vec2<f32>) -> u32 {
  var v = _g_parameter_v;
  v.x += 2.0f;
  return u32(v.x);
}

@compute @workgroup_size(1u, 1u, 1u)
fn writeWords() {
  let scalar = 4u;
  var vector = vec2<f32>(5.0f, 0.0f);
  var item = Item(6u);
  words[0u] = bumpScalar(scalar);
  words[1u] = scalar;
  words[2u] = bumpVector(vector);
  words[3u] = u32(vector.x);
  words[4u] = bumpItem(item);
  words[5u] = item.value;
}
