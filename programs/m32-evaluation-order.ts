// module: m32-evaluation-order
// purpose: record operand order in a shared kernel
// exercises: K35, LB5
// questions: none

import { ComputeInvocation, MutStorage, WgslShellSpec, wgslShell } from "./typegpu";
import { Vec2f } from "./typegpu-types";

// The last storage word holds the device counter.
function tick(): u32 {
  const words = hostLayout().words;
  words[18] = words[18] + 1;
  return words[18];
}
const tickShell: WgslShellSpec = wgslShell(tick, { body: "words[18u] += 1u; return words[18u];" });

export class OrderLayout {
  constructor(public words: MutStorage<u32>) {}
}

@ValueType
class Digits {
  tens: u32;
  ones: u32;
  constructor(ones: u32, tens: u32) { this.ones = ones; this.tens = tens; }
  plus(value: u32): u32 { return this.tens * 10 + value; }
}

function nextDigits(): Digits { return new Digits(0, tick()); }
function pair(a: u32, b: u32): u32 { return a * 10 + b; }
function nextVec(): Vec2f {
  const v = tick() as f32;
  return new Vec2f(v, v);
}
function edgeVec(): Vec2f { return nextVec(); }

export function orderKernel(res: OrderLayout, ctx: ComputeInvocation): void {
  for (let k: u32 = 0; k < 2; k++) {
    const base = k * 9;
    res.words[18] = 0;
    res.words[base] = pair(tick(), k == 0 ? tick() : 0);
    res.words[18] = 0;
    res.words[base + 1] = nextVec().step(edgeVec()).x as u32;
    res.words[18] = 0;
    res.words[base + 2] = nextVec().smoothstep(edgeVec(), edgeVec()).x as u32;
    res.words[18] = 0;
    const both = k != 0 && (k == 0 ? tick() > 0 : false);
    res.words[base + 3] = pair(res.words[18], both ? 1 : 0);
    res.words[18] = 0;
    const either = k != 0 || (k != 0 ? tick() > 0 : false);
    res.words[base + 4] = pair(res.words[18], either ? 1 : 0);
    res.words[18] = 0;
    const digits = new Digits(tick(), tick());
    res.words[base + 5] = pair(digits.tens, digits.ones);
    res.words[18] = 0;
    res.words[base + 6] = nextDigits().plus(k == 0 ? tick() : 0);
    res.words[18] = 0;
    res.words[base + 7] = pair(7, k == 0 ? tick() : 0) + pair(1, 2);
    res.words[18] = 0;
    res.words[base + 8] = pair(tick(), k == 0 ? pair(tick(), k == 0 ? tick() : 0) : 0);
  }
}

function makeLayout(): OrderLayout {
  const words: u32[] = [];
  for (let i: u32 = 0; i < 19; i++) { words.push(0); }
  return new OrderLayout(new MutStorage<u32>(words));
}

const host = makeLayout();
export function hostLayout(): OrderLayout { return host; }
