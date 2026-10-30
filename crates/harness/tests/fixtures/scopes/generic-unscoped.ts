import { createComputePipeline } from "./typegpu";
import { gpu } from "./webgpu";

export async function main(): Promise<void> {
  const adapter = await gpu.requestAdapter();
  if (adapter == null) return;
  const device = await adapter.requestDevice();
  if (device == null) return;
  using pipeline = createComputePipeline(
    device,
    "@compute @workgroup_size(1) fn main() {}",
    "main",
    [],
    [1, 1, 1],
  );
}
