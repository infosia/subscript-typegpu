# P1 — schemas and layout

Status: **in progress**. Opened 2026-08-22. Plan §8 P1. Contracts:
`specs/blocks/schema.md` (SC), `specs/blocks/layout.md` (LY).

## Slice 1 — the layout engine, the type library, the first program

Handoff issued 2026-08-22. Scope: crate `subscript-typegpu-gen`
(layout engine, schema walker, WGSL struct and constant emission,
the support module), `lib/typegpu-types.ts`, the golden-vector tool
and file, `b01-layout`, the naga test.

Round 1 (2026-08-22): delivered. Planner verification: gate green
with the backend, six executables, `b01-layout` equal on both tiers
with `Particle` 32/16, `Mixed` p at 16, `Grid` 144 with `extent` at
128, `Mat3x3f` 48/16 — the R33 numbers end to end. Golden vectors
from TypeGPU 0.12.0.

Slice review (a fresh reviewer, 2026-08-22): CRITICAL 1, MAJOR 7,
MINOR 12. C1: LY16 was not proven — both tiers print the generated
constant, and nothing measured C. M1: the generator carried a text
scanner over the program to build a placeholder module, because the
program imports the support module before it exists (D2). M2:
library types matched by name alone. M3: `enable f16;` per struct.
M4: the CLI read the library from the program's directory. M5: no
methods on integer vectors and matrices. M6: three of fifteen
library classes laid out by a program. M7: an illegal schema skipped
in silence. Resolutions in the specs: LY16 Rev 1 moves the C proof
to the harness (`value_class_layouts` plus a compiled `offsetof`
probe). SC1a records the discovery stub as a deviation with
subscript request R35 as its kill date. SC5 names the declaring
file. SC6 tabulates the method set per kind. LY15 places `enable
f16;` once. Plan §8 P1 exit criteria grew to six. Round 2 handoff
issued with every finding.

## Slice 2 — `Buffer<T>` write and read

Blocked on subscript request R34 (the bytes of a value class). See
plan §9 RC-14. R35 (the discovery check) is RC-15.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | Every committed golden vector passes | — |
| 2 | `b01-layout` prints layout constants that match its golden on both tiers | — |
| 3 | naga's offsets equal the engine's for every emitted struct | — |
| 4 | The ship tier's `offsetof` numbers match (LY16) | — |
| 5 | No schema in the corpus holds a padding field | — |
