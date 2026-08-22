// expected-rule: SC3
import { BadWide_OFFSET_value } from "./sc3-64-bit.typegpu";

@CStruct
class BadWide {
  value: u64;
}
