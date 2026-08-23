struct DepthItem {
  value: u32,
}

@group(0) @binding(0) var<storage, read_write> output: array<DepthItem>;

const ITERATIONS: u32 = 4u;
const INCREMENTS: vec2<u32> = vec2<u32>(2u, 3u);

@compute @workgroup_size(8, 1, 1)
fn depthKernel(@builtin(global_invocation_id) globalId: vec3<u32>, @builtin(local_invocation_index) localIndex: u32) {
  var iteration = 0u;
  var value = localIndex;
  while (iteration < ITERATIONS) {
    switch (globalId.x % 4u) {
      case 0u, 1u: {
        value += INCREMENTS.x;
        break;
      }
      case 2u: {
        {
          iteration += 1u;
          continue;
        }
      }
      default: {
        value += INCREMENTS.y;
        break;
      }
    }
    {
      value += 1u;
    }
    iteration += 1u;
  }
  output[globalId.x] = DepthItem(value);
}
