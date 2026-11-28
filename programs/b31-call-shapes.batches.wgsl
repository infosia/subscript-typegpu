struct Batch {
  slots: array<Item, 2u>,
  scale: f32,
}

struct Item {
  value: u32,
}

@group(0u) @binding(0u) var<storage, read_write> data: array<Batch>;

@compute @workgroup_size(1u, 1u, 1u)
fn copyBatch() {
  data[1u] = data[0u];
}
