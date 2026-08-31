// program: bitonic-sort-trap
// purpose: prove that the host sort driver rejects a non-power-of-two length
// exercises: SORT1
// questions: none
// expected-rule: SORT1

import { bitonicSortPassCount } from "./typegpu-sort";

export function main(): void {
  bitonicSortPassCount(3);
}
