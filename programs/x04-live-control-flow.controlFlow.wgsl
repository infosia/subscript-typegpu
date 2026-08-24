struct Item {
  value: f32,
}

@group(0u) @binding(0u) var<storage, read> input: array<Item>;
@group(0u) @binding(1u) var<storage, read_write> output: array<Item>;

@compute @workgroup_size(1u, 1u, 1u)
fn controlFlowKernel() {
  var index = 0u;
  var total = 0.0f;
  loop {
    var _g_conditional_0: bool;
    if (index < u32(4i)) {
      _g_conditional_0 = true;
    } else {
      _g_conditional_0 = false;
    }
    if (!(_g_conditional_0)) {
      break;
    }
    let source = input[index].value;
    var _g_conditional_2: f32;
    if (source > 0.0f) {
      var _g_conditional_1: f32;
      if (source > 2.0f) {
        _g_conditional_1 = source;
      } else {
        _g_conditional_1 = 2.0f;
      }
      _g_conditional_2 = _g_conditional_1;
    } else {
      _g_conditional_2 = 1.0f;
    }
    let chosen = _g_conditional_2;
    total += chosen;
    index += 1u;
  }
  output[0u] = Item(total);
}
