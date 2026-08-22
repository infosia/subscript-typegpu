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

Round 2 (2026-08-22) closed every item. Planner verification at the
slice close: `tools/gate.sh --require-backend` green, 176 tests in
six executables (facade 3, typegpu-gen unit 5 and integration 10,
harness 10, webgpu-gen 1 and 147), `b01-layout` lays out all
fifteen library classes and equals the golden on both tiers, the
harness's `c_layout` module proves every schema against
`value_class_layouts` and a compiled `offsetof` probe (red recorded
with an unrounded vector size: C 96 against WGSL 112), the class
scanners are gone and the import-statement stub carries its R35
kill date, hygiene clean, the harness executable 6.42 seconds.
Measurement: 45 s / 0.2 s / 13 s / 10 s / 6 (`build-time.md`).

## Slice 2 — the discovery check, `Buffer<T>`, the byte path

R34 and R35 landed at subscript `bb9dadc` (2026-08-22). Delivered:
the discovery check through `check_program_with` (the stub and the
import scan are gone), `Buffer<T>` per BF1–BF5, `b05-buffer` (a
`FixedArray<Particle, 4>` through `bytesOf`, a Noop copy, `fromBytes`,
`roundtrip:match`, 128 bytes printed by value), `x01`–`x03` on the
byte path, three BF8 trap fixtures. Planner verification: gate green
with the backend, 193 tests in six executables, the harness
executable 19.32 seconds, `b05` equal on both tiers. Measurement:
45 s / 0.2 s / 28 s / 25 s / 6.

## Phase review (2026-08-22)

A fresh reviewer found CRITICAL 0, MAJOR 4, MINOR 11. The MAJORs
were records: the plan's P1 exit criteria had not taken the six
criteria (a failed edit), `build-time.md` had lost its rows to a
broken table, the BF8 trap fixtures lacked the backend-pending
guard, and the slice closes quoted no differential wall time. The
first, second, and fourth are corrected in this commit. The third
and the code MINORs (naga offsets over every `b` program, the
fixture name, the empty position in one SC1 diagnostic, `read` as a
free function) go to the coding agent with the P2 findings.

## Exit criteria



| # | Criterion | Evidence |
|---|---|---|
| 1 | Every committed golden vector passes | Slice 1: 4 scalars, 12 vectors, 3 matrices, 3 shapes from TypeGPU 0.12.0, counts asserted |
| 2 | `b01-layout` lays out every library class and matches its golden on both tiers | Slice 1 close, `--require-backend` green |
| 3 | naga's offsets equal the engine's for every emitted struct | Slice 1: spans, member offsets, every array stride, the uniform wrap, `SHADER_FLOAT16` when `enable f16;` |
| 4 | The C numbers match (LY16 Rev 1) | Slice 1: `c_layout` module, `value_class_layouts` and the compiled probe, red recorded |
| 5 | No schema in the corpus holds a padding field | Slices 1 and 2: none |
| 6 | `Buffer<T>` writes a `FixedArray<T, N>` and reads it back through R34 | Slice 2: `b05-buffer` on both tiers |
