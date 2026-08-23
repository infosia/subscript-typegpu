@group(0) @binding(0) var<storage, read_write> output: array<u32>;
@group(0) @binding(1) var<uniform> guardedPipeline_guard: vec3<u32>;

@compute @workgroup_size(4, 1, 1)
fn guardedKernel(@builtin(global_invocation_id) globalId: vec3<u32>) {
  if (globalId.x < guardedPipeline_guard.x && globalId.y < guardedPipeline_guard.y && globalId.z < guardedPipeline_guard.z) {
    output[globalId.x] = globalId.x + 100u;
  }
}
