// program: b33-written-parameters
// purpose: print helper results and caller values after each call
// exercises: K8, K13
// questions: none

import { ComputePipelineSpec, computePipeline, simulateCompute } from "./typegpu";
import { hostLayout, writeWords } from "./m33-written-parameters";
import { words_HOST_RUNNABLE } from "./b33-written-parameters.typegpu";

const words: ComputePipelineSpec = computePipeline(writeWords, { name: "words", workgroupSize: [1, 1, 1] });

export function main(): void {
  const layout = hostLayout();
  simulateCompute(writeWords, layout, words, [1, 1, 1], words_HOST_RUNNABLE);
  print(`host:scalar=${layout.words[0]},${layout.words[1]}`);
  print(`vector=${layout.words[2]},${layout.words[3]}`);
  print(`item=${layout.words[4]},${layout.words[5]}`);
}
