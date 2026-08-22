// expected-rule: S100
// expected-owner: checker
import { BadNullable_OFFSET_value } from "./sc3-nullable.typegpu";

class Item {}

@CStruct
class BadNullable {
  value: Item | null;
}
