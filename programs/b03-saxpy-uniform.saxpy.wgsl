struct SaxpyParams {
  a: f32,
  count: u32,
}

struct Item {
  value: f32,
}

@group(0) @binding(0) var<uniform> params: SaxpyParams;
@group(0) @binding(1) var<storage, read> x: array<Item>;
@group(0) @binding(2) var<storage, read_write> y: array<Item>;

@compute @workgroup_size(64, 1, 1)
fn saxpyKernel(@builtin(global_invocation_id) globalId: vec3<u32>) {
  var settings = params;
  let i = globalId.x;
  if (i < settings.count) {
    var xItem = x[i];
    var yItem = y[i];
    y[i] = Item(settings.a * xItem.value + yItem.value);
  }
}
