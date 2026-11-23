// program: b28-app-modules
// purpose: use the same tile functions on the host and in a compute pipeline
// exercises: LB2, LB5, K2, CL2
// questions: none

import { computePipeline, ComputePipelineSpec, MutStorage, simulateCompute } from "./typegpu";
import { tileArea, tileKernel, TileLayout } from "./m28-tile";
import { tileSize } from "./m28-tile-size";
import { tile_HOST_RUNNABLE } from "./b28-app-modules.typegpu";

const tile: ComputePipelineSpec = computePipeline(tileKernel, {
  name: "tile", workgroupSize: [1, 1, 1],
});

export function main(): void {
  print(`tile-size=${tileSize()}`);
  print(`tile-area=${tileArea()}`);
  const layout = new TileLayout(new MutStorage<u32>([0]));
  simulateCompute(tileKernel, layout, tile, [1, 1, 1], tile_HOST_RUNNABLE);
  print(`host:tile-area=${layout.output[0]}`);
}
