// expected-rule: K30
// expected-owner: author
// expected-message: second

import { wgslDeclarations } from "./typegpu";

wgslDeclarations("const FIRST: u32 = 1u;");
wgslDeclarations("const SECOND: u32 = 2u;");
