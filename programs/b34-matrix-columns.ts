// program: b34-matrix-columns
// purpose: print the weighted sum for each matrix column
// exercises: K10, K36
// questions: none

import { ComputePipelineSpec, computePipeline, simulateCompute } from "./typegpu";
import { hostLayout, writeWords } from "./m34-matrix-columns";
import { words_HOST_RUNNABLE } from "./b34-matrix-columns.typegpu";

const words: ComputePipelineSpec = computePipeline(writeWords, { name: "words", workgroupSize: [1, 1, 1] });

export function main(): void {
  const layout = hostLayout();
  simulateCompute(writeWords, layout, words, [1, 1, 1], words_HOST_RUNNABLE);
  print(`host:word[0]=${layout.words[0]}`);
  for (let i: u32 = 1; i < 9; i++) { print(`word[${i}]=${layout.words[i]}`); }
}
