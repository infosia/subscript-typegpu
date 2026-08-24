struct State {
  counter: u32,
  incrementBy: u32,
}

@group(0u) @binding(0u) var<storage, read_write> state: array<State>;

@compute @workgroup_size(1u, 1u, 1u)
fn incrementCounter() {
  var state_ = state[0u];
  state_.counter += state_.incrementBy;
  state[0u] = state_;
}
