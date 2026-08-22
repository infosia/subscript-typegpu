// expected-rule: S100
// expected-owner: checker
import { BadString_OFFSET_value } from "./sc3-string.typegpu";

@CStruct
class BadString {
  value: string;
}
