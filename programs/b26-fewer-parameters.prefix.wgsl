@group(0u) @binding(0u) var<storage, read_write> output: array<u32>;
@group(1u) @binding(0u) var<storage, read> spare: array<u32>;

@compute @workgroup_size(1u, 1u, 1u)
fn prefixKernel() {
  output[0u] = output[0u] + 3u;
}
