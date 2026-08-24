struct WorkCounter {
  total: atomic<u32>,
}

@group(0u) @binding(0u) var<storage, read_write> counters: array<WorkCounter>;

var<private> privateOffset: u32 = 3u;
var<workgroup> sharedValues: array<u32, 4u>;
var<workgroup> sharedCounter: atomic<u32>;

@compute @workgroup_size(4u, 1u, 1u)
fn workgroupKernel(@builtin(workgroup_id) workgroupId: vec3<u32>, @builtin(local_invocation_index) localIndex: u32) {
  privateOffset = privateOffset + 1u;
  sharedValues[localIndex] = localIndex + privateOffset;
  if (localIndex == 0u) {
    atomicStore(&sharedCounter, 0u);
  }
  workgroupBarrier();
  atomicAdd(&sharedCounter, sharedValues[localIndex]);
  workgroupBarrier();
  if (localIndex == 0u) {
    atomicAdd(&counters[workgroupId.x].total, atomicLoad(&sharedCounter));
  }
}
