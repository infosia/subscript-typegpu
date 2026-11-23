@group(0u) @binding(0u) var<storage, read_write> output: array<u32>;

fn tileSize() -> u32 {
  return 4u;
}

fn tileArea() -> u32 {
  let size = tileSize();
  return size * size;
}

@compute @workgroup_size(1u, 1u, 1u)
fn tileKernel() {
  output[0u] = tileArea();
}
