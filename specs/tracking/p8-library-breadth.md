# P8 — library breadth, slice 1

Contracts: `specs/blocks/buffer.md` Rev 1 (BF9–BF11),
`specs/blocks/kernel.md` Rev 5 (K10 Rev 1, K25–K28). Plan §8 P8.

## Status

2026-08-23: contracts written. No code yet.

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
