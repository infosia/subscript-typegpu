// expected-rule: SC9
import { Padded_SIZE } from "./sc9-layout.typegpu";

@ValueType({ align: 16 })
class Padded {
  x: f32;

  constructor(x: f32) {
    this.x = x;
  }
}
