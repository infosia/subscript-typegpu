const SHELL_BIAS: u32 = 7u;

fn addBias(value: u32) -> u32 {
  return value + SHELL_BIAS;
}

@group(0u) @binding(0u) var<storage, read_write> output: array<u32>;

@compute @workgroup_size(1u, 1u, 1u)
fn shellKernel(@builtin(global_invocation_id) globalId: vec3<u32>) {
  if (globalId.x == 0u) {
    output[0u] = addBias(5u);
  }
}
