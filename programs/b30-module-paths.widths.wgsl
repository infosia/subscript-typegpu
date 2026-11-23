@group(0u) @binding(0u) var<storage, read_write> output: array<u32>;

fn width_m30_util() -> u32 {
  return 3u;
}

fn width_m30_inner_util() -> u32 {
  return width_m30_util() + 2u;
}

@compute @workgroup_size(1u, 1u, 1u)
fn writeWidths() {
  output[0u] = width_m30_util();
  output[1u] = width_m30_inner_util();
}
