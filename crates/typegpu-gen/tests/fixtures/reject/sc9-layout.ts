// expected-rule: SC9
import { Padded_SIZE } from "./sc9-layout.typegpu";

@CStruct({ align: 16 })
class Padded {
  x: f32;
}
