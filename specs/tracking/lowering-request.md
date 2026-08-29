# lowering-request — the re-pin from `a2228d9` to `2a65724`

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

## What stayed open, and what it hid

L1 and L2 stopped lowering before most sources reached a run, so the
list of rejections behind them was unknown. Two findings came out
when subscript closed them.

## The pin `24e772e`, measured 2026-08-28

L1 and L2 are closed. subscript selects `LoadAddress` for a by-value
receiver, and the reaching-version pass now collects the known
predecessor versions instead of waiting for every one of them.

R1 is closed here. The fixture renames its second `storage` to
`storageTexture`.

Two findings remain.

### L3 — a nullable nested boundary struct arrives zeroed

Every program that creates a render pipeline fails: `a03`, `a05`,
`b06`, `b07`, `b08`, `b16`, `b17`, `b19`, and `b21`. yawgpu panics
with `WGPUShaderModule must not be null`. The panic crosses a C ABI,
so the process aborts with `SIGABRT` and the run prints nothing.

A temporary probe in the facade printed the fields the facade reads
from one descriptor. Same probe, same program
(`programs/b06-render.ts`), both pins:

| Field | `a2228d9` | `24e772e` |
|---|---|---|
| `vertex.module` | non-null | non-null |
| `vertex.entryPoint` length | 4 | 4 |
| `fragment.module` | non-null | 0 |
| `fragment.entryPoint` length | 4 | 0 |
| `fragment.targets` count | 1 | 0 |

`vertex` is a boundary value class that the descriptor holds by
value, and it reads correctly. `fragment` is the same kind of class,
nullable, and the descriptor holds it as `*const`. Its first 32 bytes
read as zero, and two pointers sit at offset 32 and offset 48.

The dev tier and the ship tier fail the same way, so the defect sits
below the tier split.

### R2 — the emitted C identifiers changed

`c_layout` compiles a C probe that names subscript's emitted structs
and fields. At `a2228d9` a class was `Sub_{id}_{name}` and a field
kept its source name. At `24e772e` a class is `SubC{id}` and a field
is `d{id}`. The probe must follow the new spelling. This is a fix
here, not a subscript defect.

## Closed at `2a65724`, 2026-08-28

`root_storage.rs` typed an address as zero root slots, so no address
kept its base alive. §68.2 rule 8 made the storage scope the live
range, and the base then died under a live address. subscript computes
borrowed bases as one fixed point, and both tiers read the one plan. The escape
rule the round first proposed was withdrawn on measurement, so no new
rejection reaches this repository.

R2 is closed here. subscript answered the question in §66.1: the
emitted C spelling is not an interface, because a class name carries a
class-table index. The probe now names only the types and members of
`crates/facade/subscript-typegpu.h`, and takes the other side from
`subscript_codegen::value_class_layouts`, which is keyed by source
names. The check compares sizes, alignments, and member offsets of the
52 mirror structs. It cannot see L3, which wrote zeros at correct
offsets. The differential suite sees L3: every render program aborted
at `24e772e` and prints `PASS` at `2a65724`.

The rewrite drops one check. The old probe compiled each program's
emitted C and read the offsets of every schema struct in it. No test
here reads emitted C now. Two gates hold the property: the
differential suite runs every program under both tiers against one
golden, so a ship-tier struct that disagrees with the engine moves the
bytes a program writes and reads. subscript's `a153` corpus entry
round-trips a nested `@CStruct` array through both tiers.

## Status

The pin is `2a65724`. `tools/gate.sh --require-backend` green, 257
passed, 1 ignored, 161.35 s. The gate cost of the pin is recorded in
`specs/tracking/build-time.md`.

## The pin `f43e3b2`, 2026-08-29

Two findings, both closed here.

**R3 — nine unreachable match arms.** subscript removed
`#[non_exhaustive]` from `hir::Stmt`, `hir::Callee`, and
`hir::ExprKind`. A wildcard arm after a complete enumeration is
unreachable, and `clippy -D warnings` reports it. The nine arms in
`crates/typegpu-gen/src/kernel.rs` and `pipeline.rs` are deleted. No
variant was added, so no behaviour moved.

**R4 — the K19 cycle case is unreachable.** subscript §67.1 rule 4c
rejects a module initializer that reads a binding declared after it,
and every constant cycle contains one such read. `tsc` rejects the
same shape as TS2448. The fixture `k24-constant-cycle.ts` expected
K19 and got `S100` from subscript first. K19 Rev 5 removes the cycle
from its list, the fixture is deleted, and the generator's recursion
guard reports through the internal generator form instead of K19.

§33.5 replaced the address representation of `T | null` with a
managed box, and S015 is deleted. `lib/webgpu.ts` needs no change.

`tools/gate.sh --require-backend` green, 257 passed, 1 ignored,
177.64 s on a quiet machine. One run measured 244.04 s while a second
gate ran on the same machine. Discard it.

