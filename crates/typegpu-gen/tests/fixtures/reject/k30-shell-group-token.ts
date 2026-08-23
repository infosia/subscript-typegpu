// expected-rule: K30
// expected-owner: author
// expected-message: @group

import { WgslShellSpec, wgslShell } from "./typegpu";

function rejected(value: u32): u32 { return value; }

const shell: WgslShellSpec = wgslShell<(value: u32) => u32>(rejected, {
  body: "@group(0) var<uniform> hidden: u32; return value;",
});
