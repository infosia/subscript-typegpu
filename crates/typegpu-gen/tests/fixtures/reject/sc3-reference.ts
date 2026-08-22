import { BadReference_OFFSET_value } from "./sc3-reference.typegpu";

class Item {}

@CStruct
class BadReference {
  value: Item;
}
