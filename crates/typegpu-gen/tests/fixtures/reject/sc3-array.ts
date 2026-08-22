// expected-rule: S100
// expected-owner: checker
import { BadArray_OFFSET_values } from "./sc3-array.typegpu";

@CStruct
class BadArray {
  values: u32[];
}
