// program: unsupported-host-blend
// purpose: prove the host rasterizer rejects a blend factor outside its RN21 set
// exercises: RN21
// questions: none
// expected-rule: RN21

import { hostBlend } from "./typegpu";
import { Vec4f } from "./typegpu-types";
import { GPUBlendState } from "./webgpu";

export function main(): void {
  const unsupported: GPUBlendState = {
    color: {
      operation: "add",
      srcFactor: "zero",
      dstFactor: "one-minus-src-alpha",
    },
    alpha: {
      operation: "add",
      srcFactor: "one",
      dstFactor: "one",
    },
  };
  hostBlend(
    new Vec4f(0.5, 0.5, 0.5, 0.5),
    new Vec4f(0.0, 0.0, 0.0, 0.0),
    unsupported,
  );
}
