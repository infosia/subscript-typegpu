// program: sampled-texture-store
// purpose: prove the host body of a generator-rejected sampled-texture store traps
// exercises: TX3
// questions: none
// expected-rule: TX3

import { Texture2d } from "./typegpu";
import { Vec2i, Vec4f } from "./typegpu-types";

export function main(): void {
  const texture = new Texture2d<f32>([new Vec4f(0.0, 0.0, 0.0, 1.0)], 1, 1);
  texture.store(new Vec2i(0, 0), new Vec4f(1.0, 0.0, 0.0, 1.0));
}
