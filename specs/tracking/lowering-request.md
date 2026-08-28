# lowering-request — two LIR defects block the pin `c45d164`

The workspace pin is `a2228d9`. The candidate pin is `c45d164`,
committed 2026-08-28. Between the two, subscript §68 made LIR the one
ordered IR for both tiers. Two lowering defects at the candidate pin
stop every program in this repository. One defect of this repository
appears behind them.

## Evidence, 2026-08-28, at the candidate pin

`tools/gate.sh --require-backend` red. Three test executables green:
3, 8, and 55 tests. The harness executable: 22 passed, 14 failed, 1
ignored, 36.18 s. Every failure carries one of the two messages
below.

The two smallest programs below ran through a subscript CLI built
from the candidate revision. The gate produced the same two messages
from the workspace build.

## L1 — a value class method that uses `this` by value emits invalid LIR

    internal lowering error: LIR construction failed: a1.ts:5:3:
    produced invalid LIR:
    function 1 (`self`): block 0 instruction 0 implicit coercion
    signature is invalid: kind=Coerce, operands=[Address(AddressType
    { pointee: Class(ClassId(0)), array_base: None })],
    result=Some(Data(Class(ClassId(0))))

Trigger: a `@CStruct` class method returns `this`, or passes `this`
as a by-value argument. The lowering emits `Coerce` from an address
to data, and the LIR verifier rejects that signature.

    @CStruct({ align: 8 })
    class V {
      x: f32;
      y: f32;
      constructor(x: f32, y: f32) { this.x = x; this.y = y; }
      self(): V { return this; }
    }
    export function main(): void {
      const value: V = new V(3.0, 4.0);
      print(`${value.self().x}`);
    }

The same program without `@CStruct` prints `3`. The argument form
fails the same way:

    dot(other: V): f32 { return this.x * other.x + this.y * other.y; }
    length(): f32 { return this.dot(this); }

Where it hits here: `lib/typegpu-types.ts` holds both forms.
`length()` reads `this.dot(this)`, and `faceForward()` returns
`this`, on `Vec2f`, `Vec3f`, and `Vec4f`. That library compiles into
every program, so both tiers fail for every entry in `programs/` and
`examples/`.

## L2 — an await result has no reaching definition at a later suspend

    internal lowering error: LIR construction failed: b9.ts:5:23:
    value 1 is live at suspend in block 4 but has no reaching
    definition

Trigger: an `await` defines a local, a loop follows, and a second
`await` follows the loop.

    async function get(seed: i32): Promise<i32> {
      await Context.suspend();
      return seed;
    }
    export async function main(): Promise<void> {
      const first: i32 = await get(1);
      let index: i32 = 0;
      while (index < 4) {
        index = index + 1;
      }
      const second: i32 = await get(3);
      print(`${first},${second}`);
    }

The same program prints `1,3` when a constant defines `first`. The
loop and the second suspension are both necessary.

Where it hits here: `programs/b12-readback.ts`, which awaits an
adapter, then loops over four decoded values, then awaits a second
read.

## R1 — this repository declares `storage` twice in one function

The runtime fixture in `crates/harness/tests/runtime/mod.rs` declares
`const storage` twice inside `main`: once as `Storage<u32>` and once
as `StorageTexture2d<Rgba8unorm>`. The candidate pin reports
`S100: duplicate declaration of storage in one scope`, then
`S100: Storage<u32> has no method store`.

`tsc --noEmit --strict` reports
`TS2451: Cannot redeclare block-scoped variable 'storage'` for the
same shape. subscript §67 is right, and the fixture is wrong. The fix
belongs here, and it holds at both pins.

## What stays open

L1 and L2 stop lowering before most sources reach a run, so more
rejections of the R1 class can hide behind them. The full list needs
a build with L1 and L2 closed.

## Status

The pin stays at `a2228d9`. `Cargo.toml` and `Cargo.lock` carry no
change. The owner decides the order: subscript closes L1 and L2, then
this repository re-pins and re-measures the gate.
