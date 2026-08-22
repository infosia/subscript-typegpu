struct SwitchValue {
  value: u32,
}

@group(0) @binding(0) var<storage, read_write> output: array<SwitchValue>;

@compute @workgroup_size(16, 1, 1)
fn liveSwitchKernel(@builtin(global_invocation_id) globalId: vec3<u32>) {
  let mode = globalId.x % 4u;
  var iteration = 0u;
  var value = 0u;
  while (iteration < 4u) {
    switch (mode) {
      case 0u: {
        value += 1u;
        break;
      }
      case 1u: {
        {
          value += 2u;
          iteration += 1u;
          continue;
        }
      }
      case 2u, 3u: {
        value += 3u;
        break;
      }
      default: {
        return;
      }
    }
    iteration += 1u;
  }
  output[globalId.x] = SwitchValue(value);
}
