// program: b30-module-paths
// purpose: call both width helpers on the host and in one compute kernel
// exercises: K2, LB2, LB5, CL2
// questions: none

import { computePipeline, ComputePipelineSpec, MutStorage, simulateCompute } from "./typegpu";
import { width as baseWidth, Widths } from "./m30/util";
import { width as innerWidth } from "./m30/inner/util";
import { widths_HOST_RUNNABLE } from "./b30-module-paths.typegpu";

function writeWidths(res: Widths): void {
  res.output[0] = baseWidth();
  res.output[1] = innerWidth();
}

const widths: ComputePipelineSpec = computePipeline(writeWidths, {
  name: "widths", workgroupSize: [1, 1, 1],
});

export function main(): void {
  print(`base=${baseWidth()}`);
  print(`inner=${innerWidth()}`);
  const res = new Widths(new MutStorage<u32>([0, 0]));
  simulateCompute(writeWidths, res, widths, [1, 1, 1], widths_HOST_RUNNABLE);
  print(`host:widths=${res.output[0]},${res.output[1]}`);
}
