struct FormsItem {
  value: u32,
}

@group(0u) @binding(0u) var<storage, read_write> output: array<FormsItem>;

fn FormsItem_weight(self_: FormsItem) -> u32 {
  return self_.value * 2u;
}

fn identity_f32(value: f32) -> f32 {
  return value;
}

fn identity_u32(value: u32) -> u32 {
  return value;
}

fn localScore_plusOne(value: u32) -> u32 {
  return value + 1u;
}

fn localScore(value: u32) -> u32 {
  return localScore_plusOne(value);
}

fn modeValue(mode: u32) -> u32 {
  switch (mode) {
    case 3u: {
      return 3u;
    }
    case 5u, default: {
      return 5u;
    }
  }
}

fn weighted_FormsItem(value: FormsItem) -> u32 {
  return FormsItem_weight(value);
}

@compute @workgroup_size(2u, 1u, 1u)
fn formsKernel(@builtin(global_invocation_id) globalId: vec3<u32>) {
  var iteration = 0u;
  var total = 0u;
  loop {
    iteration += 1u;
    if (iteration == 2u) {
      continue;
    }
    total += iteration;
    continuing {
      break if !(iteration < 4u);
    }
  }
  var selected = globalId.x == 0u;
  if (selected) {
    selected = total == 8u;
  }
  if (!(selected)) {
    selected = false;
  }
  let base = f32(total);
  let maximum = max(max(base, base + 2.0f), base + 1.0f);
  let mode = 5u;
  var item = FormsItem(identity_u32(3u));
  var result = total + u32(identity_f32(maximum)) + modeValue(mode) + item.value + FormsItem_weight(item) + weighted_FormsItem(item) + localScore(2u);
  if (selected) {
    result += 1u;
  }
  output[globalId.x] = FormsItem(result);
}
