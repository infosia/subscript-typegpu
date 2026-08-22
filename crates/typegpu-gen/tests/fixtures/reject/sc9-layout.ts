// expected-rule: SC9
import { Mixed_OFFSET_p } from "./sc9-layout.typegpu";

@CStruct({ align: 16 })
class Padded {
  x: f32;
}

@CStruct
class Mixed {
  a: f32;
  p: Padded;
}
