struct Item {
  value: f32,
}

@group(0) @binding(0) var<storage, read> a: array<Item>;
@group(0) @binding(1) var<storage, read> b: array<Item>;
@group(0) @binding(2) var<storage, read_write> out: array<Item>;

@compute @workgroup_size(64, 1, 1)
fn vecAddKernel(@builtin(global_invocation_id) globalId: vec3<u32>) {
  let i = globalId.x;
  if (i < arrayLength(&out)) {
    var left = a[i];
    var right = b[i];
    var sum = Item(left.value + right.value);
    out[i] = sum;
  }
}
