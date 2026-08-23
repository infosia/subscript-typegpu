// expected-rule: K29
// expected-owner: author
// expected-message: top-level statement

import { WgslShellSpec, wgslShell } from "./typegpu";

function passthrough(value: u32): u32 { return value; }
if (true) {
  const nested: WgslShellSpec = wgslShell<(value: u32) => u32>(passthrough, {
    body: "return value;",
  });
}
