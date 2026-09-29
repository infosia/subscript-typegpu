// expected-rule: LY8
import { BadBoolean_OFFSET_flag } from "./ly8-boolean.typegpu";

@ValueType
class BadBoolean {
  flag: boolean;

  constructor(flag: boolean) {
    this.flag = flag;
  }
}
