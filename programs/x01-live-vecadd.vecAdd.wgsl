struct Item {
  value: f32,
}

@group(0u) @binding(0u) var<storage, read> a: array<Item>;
@group(0u) @binding(1u) var<storage, read> b: array<Item>;
@group(0u) @binding(2u) var<storage, read_write> out: array<Item>;

@compute @workgroup_size(64u, 1u, 1u)
fn vecAddKernel(@builtin(global_invocation_id) globalId: vec3<u32>) {
  let i = globalId.x;
  if (i < arrayLength(&out)) {
    var left = a[i];
    var right = b[i];
    out[i] = Item(left.value + right.value);
  }
}
