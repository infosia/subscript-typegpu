// expected-rule: SC3
import { BadArray_OFFSET_values } from "./sc3-array.typegpu";

@CStruct
class BadArray {
  values: u32[];
}
