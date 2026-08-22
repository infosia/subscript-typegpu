# P1 — schemas and layout

Status: **in progress**. Opened 2026-08-22. Plan §8 P1. Contracts:
`specs/blocks/schema.md` (SC), `specs/blocks/layout.md` (LY).

## Slice 1 — the layout engine, the type library, the first program

Handoff issued 2026-08-22. Scope: crate `subscript-typegpu-gen`
(layout engine, schema walker, WGSL struct and constant emission,
the support module), `lib/typegpu-types.ts`, the golden-vector tool
and file, `b01-layout`, the naga test. Result: pending.

## Slice 2 — `Buffer<T>` write and read

Blocked on subscript request R34 (the bytes of a value class). See
plan §9 RC-14.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | Every committed golden vector passes | — |
| 2 | `b01-layout` prints layout constants that match its golden on both tiers | — |
| 3 | naga's offsets equal the engine's for every emitted struct | — |
| 4 | The ship tier's `offsetof` numbers match (LY16) | — |
| 5 | No schema in the corpus holds a padding field | — |
