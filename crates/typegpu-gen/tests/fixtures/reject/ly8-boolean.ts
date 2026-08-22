// expected-rule: LY8
import { BadBoolean_OFFSET_flag } from "./ly8-boolean.typegpu";

@CStruct
class BadBoolean {
  flag: boolean;
}
