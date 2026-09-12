// expected-rule: SC11
import { BadName_OFFSET_bad_name } from "./sc11-underscore.typegpu";

@CStruct
class BadName {
  bad_name: u32;

  constructor(bad_name: u32) {
    this.bad_name = bad_name;
  }
}
