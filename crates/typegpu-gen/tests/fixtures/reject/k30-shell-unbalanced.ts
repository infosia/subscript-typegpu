// expected-rule: K30
// expected-owner: author
// expected-message: braces

import { WgslShellSpec, wgslShell } from "./typegpu";

function rejected(value: u32): u32 { return value; }

const shell: WgslShellSpec = wgslShell<(value: u32) => u32>(rejected, {
  body: "if (value > 0u) { return value;",
});
