struct Params {
  value: u32,
}

struct ShadowResult {
  local: u32,
  reread: u32,
}

@group(0u) @binding(0u) var<uniform> params: Params;
@group(0u) @binding(1u) var<storage, read_write> output: array<ShadowResult>;

@compute @workgroup_size(1u, 1u, 1u)
fn shadowKernel() {
  var params_ = params;
  params_.value = params_.value + 7u;
  var reread = params;
  output[0u] = ShadowResult(params_.value, reread.value);
}
