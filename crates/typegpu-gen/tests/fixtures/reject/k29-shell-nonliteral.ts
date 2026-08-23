// expected-rule: K29
// expected-owner: author
// expected-message: literal

import { WgslShellSpec, wgslShell } from "./typegpu";

const bodyText: string = "return value;";

function passthrough(value: u32): u32 {
  return value;
}

const shell: WgslShellSpec = wgslShell<(value: u32) => u32>(passthrough, {
  body: bodyText,
});
