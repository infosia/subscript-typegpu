struct SaxpyParams {
  a: f32,
  count: u32,
}

struct Item {
  value: f32,
}

@group(0u) @binding(0u) var<uniform> params: SaxpyParams;
@group(0u) @binding(1u) var<storage, read> x: array<Item>;
@group(0u) @binding(2u) var<storage, read_write> y: array<Item>;

@compute @workgroup_size(64u, 1u, 1u)
fn saxpyKernel(@builtin(global_invocation_id) globalId: vec3<u32>) {
  var settings = params;
  let i = globalId.x;
  if (i < settings.count) {
    var xItem = x[i];
    var yItem = y[i];
    y[i] = Item(settings.a * xItem.value + yItem.value);
  }
}
