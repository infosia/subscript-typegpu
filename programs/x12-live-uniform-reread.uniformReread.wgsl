struct Params {
  value: u32,
}

struct ShadowResult {
  local: u32,
  reread: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read_write> output: array<ShadowResult>;

@compute @workgroup_size(1, 1, 1)
fn shadowKernel() {
  var params_ = params;
  params_.value = params_.value + 7u;
  var reread = params;
  output[0u] = ShadowResult(params_.value, reread.value);
}
