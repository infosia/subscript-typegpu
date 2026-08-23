@group(0) @binding(0) var<storage, read_write> output: array<u32>;

@compute @workgroup_size(1, 1, 1)
fn computeStep(@builtin(global_invocation_id) globalId: vec3<u32>) {
  if (globalId.x == 0u) {
    output[0u] = 1u;
  }
}
