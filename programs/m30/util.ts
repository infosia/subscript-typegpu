// module: m30/util
// purpose: provide the base width and the storage layout
// exercises: K2, LB2, LB5
// questions: none

import { MutStorage } from "../typegpu";

export class Widths {
  constructor(public output: MutStorage<u32>) {}
}

export function width(): u32 { return 3; }
