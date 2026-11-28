// program: b32-evaluation-order
// purpose: evaluate operand order and short-circuit conditions on the host
// exercises: K35
// questions: none

import { ComputePipelineSpec, computePipeline, simulateCompute } from "./typegpu";
import { hostLayout, orderKernel } from "./m32-evaluation-order";
import { evaluation_HOST_RUNNABLE } from "./b32-evaluation-order.typegpu";

const evaluation: ComputePipelineSpec = computePipeline(orderKernel, { name: "evaluation", workgroupSize: [1, 1, 1] });

export function main(): void {
  const layout = hostLayout();
  simulateCompute(orderKernel, layout, evaluation, [1, 1, 1], evaluation_HOST_RUNNABLE);
  print(`host:word[0]=${layout.words[0]}`);
  for (let i: u32 = 1; i < 18; i++) { print(`word[${i}]=${layout.words[i]}`); }
}
