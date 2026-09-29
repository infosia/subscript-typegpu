// expected-rule: SC5
import { Vec2b } from "./typegpu-types";
import { BoolPack_OFFSET_mask } from "./k28-bool-schema.typegpu";

@ValueType
class BoolPack {
  mask: Vec2b;

  constructor(mask: Vec2b) {
    this.mask = mask;
  }
}
