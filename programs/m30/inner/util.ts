// module: m30/inner/util
// purpose: compute a width from the parent module
// exercises: K2, LB2, LB5
// questions: none

import { width as baseWidth } from "../util";

export function width(): u32 { return baseWidth() + 2; }
