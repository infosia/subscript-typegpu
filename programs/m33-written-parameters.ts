// module: m33-written-parameters
// purpose: write helper results and the original arguments
// exercises: K8, K13, LB5
// questions: none

import { MutStorage } from "./typegpu";
import { Vec2f } from "./typegpu-types";

@ValueType
class Item {
  constructor(public value: u32) {}
}

export class WordLayout {
  constructor(public words: MutStorage<u32>) {}
}

function bumpScalar(x: u32): u32 { x++; return x; }
function bumpVector(v: Vec2f): u32 { v.x += 2.0; return v.x as u32; }
function bumpItem(item: Item): u32 { item.value += 3; return item.value; }

export function writeWords(res: WordLayout): void {
  const scalar: u32 = 4;
  const vector = new Vec2f(5.0, 0.0);
  const item = new Item(6);
  res.words[0] = bumpScalar(scalar);
  res.words[1] = scalar;
  res.words[2] = bumpVector(vector);
  res.words[3] = vector.x as u32;
  res.words[4] = bumpItem(item);
  res.words[5] = item.value;
}

export function hostLayout(): WordLayout {
  return new WordLayout(new MutStorage<u32>([0, 0, 0, 0, 0, 0]));
}
