// module: m28-tile
// purpose: compute tile area on the host and in a kernel
// exercises: LB2, LB5, K2
// questions: none

import { MutStorage } from "./typegpu";
import { tileSize } from "./m28-tile-size";

export class TileLayout {
  constructor(public output: MutStorage<u32>) {}
}

export function tileArea(): u32 {
  const size = tileSize();
  return size * size;
}

export function tileKernel(res: TileLayout): void {
  res.output[0] = tileArea();
}
