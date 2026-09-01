// program: radiance-cascade-host
// purpose: exercise the radiance-cascade sizing and ping-pong helpers through the host lane
// questions: none

import { cascadeDimensions, cascadeWriteSide } from "./typegpu-radiance-cascades";

export function main(): void {
  const full = cascadeDimensions(512);
  print(`cascadeDimensions 512 probes=${full.cascadeProbes} dim=${full.cascadeDim} count=${full.cascadeCount}`);

  const drawing = cascadeDimensions(128);
  print(`cascadeDimensions 128 probes=${drawing.cascadeProbes} dim=${drawing.cascadeDim} count=${drawing.cascadeCount}`);

  let count: u32 = 1;
  while (count <= 8) {
    let layer: u32 = 0;
    while (layer < count) {
      const expected: u32 = (count - 1 - layer) % 2 === 0 ? 0 : 1;
      const actual: u32 = cascadeWriteSide(count, layer);
      if (actual !== expected) {
        print(`FAIL cascadeWriteSide count=${count} layer=${layer} expected=${expected} actual=${actual}`);
      }
      layer += 1;
    }
    count += 1;
  }
  print("cascadeWriteSide side A when count - 1 - layer is even");
}
