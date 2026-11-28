@group(0u) @binding(0u) var<storage, read> pixels_0: array<u32>;
@group(1u) @binding(0u) var<storage, read_write> pixels_1: array<u32>;

fn pair(value: u32, bias: u32) -> u32 {
  return value + bias;
}

@compute @workgroup_size(1u, 1u, 1u)
fn copyPixels() {
  pixels_1[0u] = pair(pixels_0[0u], 7u);
  pixels_1[1u] = pair(pixels_0[1u], 0u);
}
