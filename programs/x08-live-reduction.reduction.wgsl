struct ReductionValue {
  value: f32,
}

struct ReductionCounter {
  total: atomic<u32>,
}

var<workgroup> partials: array<f32, 256>;

@group(0) @binding(0) var<storage, read> input: array<ReductionValue>;
@group(0) @binding(1) var<storage, read_write> output: array<ReductionCounter>;

@compute @workgroup_size(256, 1, 1)
fn reductionKernel(@builtin(global_invocation_id) globalId: vec3<u32>, @builtin(local_invocation_index) localIndex: u32) {
  let global = globalId.x;
  let local = localIndex;
  var _g_conditional_0: f32;
  if (global < 1024u) {
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
