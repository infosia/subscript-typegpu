struct ReductionValue {
  value: f32,
}

struct ReductionCounter {
  total: atomic<u32>,
}

@group(0u) @binding(0u) var<storage, read> input: array<ReductionValue>;
@group(0u) @binding(1u) var<storage, read_write> output: array<ReductionCounter>;

var<workgroup> partials: array<f32, 256u>;

@compute @workgroup_size(256u, 1u, 1u)
fn reductionKernel(@builtin(global_invocation_id) globalId: vec3<u32>, @builtin(local_invocation_index) localIndex: u32) {
  let global = globalId.x;
  let local = localIndex;
  var _g_conditional_0: f32;
  if (global < 1000u) {
    _g_conditional_0 = input[global].value;
  } else {
    _g_conditional_0 = 0.0f;
  }
  partials[local] = _g_conditional_0;
  workgroupBarrier();
  var stride = 128u;
  while (stride > 0u) {
    if (local < stride) {
      partials[local] = partials[local] + partials[local + stride];
    }
    workgroupBarrier();
    stride = stride / 2u;
  }
  if (local == 0u) {
    atomicAdd(&output[0u].total, u32(partials[0u]));
  }
}
