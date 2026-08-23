# P8 — library breadth, slice 1

Contracts: `specs/blocks/buffer.md` Rev 1 (BF9–BF11),
`specs/blocks/kernel.md` Rev 5 (K10 Rev 1, K25–K28). Plan §8 P8.

## Status

2026-08-23: contracts written. No code yet.

2026-08-23: BF9 is blocked on subscript. Measured on the dev tier at
pin `bb9dadc` with one-file probes: an `async` method on a generic
class is `S100: async methods on generic class templates are not in
the decided surface`; a generic `async` free function is `S100: is
not a directly declared async function`; `u8[] | null` is `S011:
unions are limited to Ref | null`; an `async` method on a
non-generic class runs. Escalated as subscript request R36 (generic
`async`). Proposed BF9 Rev 1 for the owner's decision: the return
type is `Promise<u8[]>` and a failed map traps with `BF9`, so no
nullable array is needed. Until the decision, step 1 holds, steps 2
and 3 proceed, and `x14` reads through the BF3 explicit path. The
plan's exit item for `x14` through `read` moves to the step 1 close.

## Decisions

- Swizzles are methods (`v.xy()`), because subscript has no
  user-defined accessors. The set is the in-order subsets, so the
  method count stays at 3 per `Vec3*` and 10 per `Vec4*`.
- A scalar `select` is the K9 conditional. No function is added.
- Derivatives (`dpdx`, `dpdy`, `fwidth`) and the pack/unpack
  builtins are not in this slice. Their host bodies need a
  rasterizer-side definition that RN14's host rasterizer does not
  have.
- `Vec*b` classes are value classes, never schemas (SC5).
