// expected-rule: K30
// expected-owner: author
// expected-message: top-level statement

import { wgslDeclarations } from "./typegpu";

if (true) {
  wgslDeclarations("const NESTED: u32 = 1u;");
}
