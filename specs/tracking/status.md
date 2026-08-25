# Project status

Updated 2026-08-25.

| Phase | Status | Close |
|---|---|---|
| P0 seed and generator import | COMPLETE | `5f6840b`, 2026-08-22 |
| P1 schemas and layout | COMPLETE | `0e0156c`, 2026-08-22 |
| P2 compute kernels | COMPLETE | `0e0156c`, 2026-08-22 |
| P3 render | COMPLETE | `b32f14a`, 2026-08-23 |
| P4 kernel depth | COMPLETE | `ea25b01`, 2026-08-23 |
| P5 textures and samplers | COMPLETE | `3d9d988`, 2026-08-23 |
| P6 ergonomics and diagnostics | COMPLETE | `4c2eff8`, 2026-08-23 |
| P7 the CPU lane | COMPLETE | `167d34e`, 2026-08-23 |

Numbers at `42ecb7f`: 233 tests in six executables, 28 programs
(`a01`–`a05`, `b01`–`b11`, `x01`–`x12`), 157 facade exports all
reached, the live lane green on yawgpu Metal and on Dawn with the
kernel bodies as the oracle for `x01`–`x04` and `x09`, the gate 86 s
after a generator change and 84 s after a program change (budgets
240 s and 120 s), the cold build 45 s (budget 480 s).

Subscript requests this project made and that landed: R33
(`@CStruct({ align })`), R34 (`Context.bytesOf`, `bytesInto`,
`fromBytes`), R35 (`check_program_with`, the discovery check). The
pin is `bb9dadc`.

Defects the live lane found that naga did not report: the barrier
after a non-uniform early return (P4, K22 Rev 1 and Rev 3) — both
backends rejected it at shader-module creation, and PI14 now makes
such a rejection visible.
Defects the phase reviews found that the gate could not see: the
operator precedence after a method lowering, the `?:` lowering in a
loop condition (P2), the duplicated `@builtin(position)` (P3), the
kernel local that shadowed a binding (P5), the README's backend
values (P6).

Open items carried in the blocks: RN8's multiple targets, TX1's
integer sample types and formats, a phased `simulateCompute` for
barrier kernels (CL2), render pipelines beyond one group (TX2).
Every planned phase is complete. The retrospective review ran at
`82175df` (`specs/tracking/retrospective.md`): CRITICAL 0, MAJOR 2,
MINOR 12, the contract side resolved at `09dd4be`, the code side at
`c0ebfed`. Live at `c0ebfed`: Metal ok 27.25 s, Dawn ok 24.42 s,
`x01`–`x12` `PASS` on both.

## PI14 round (2026-08-23)

Every program now pushes a `validation` error scope before its first
creation call and pops it after the last (PI14). `x13-live-rejection`
submits a non-uniform-barrier module as raw WGSL and passes when the
popped error names uniform control flow. Evidence: `tools/gate.sh
--require-backend` green, 89 s, 234 passed, 1 ignored (live). Live
lane `tools/live.sh`, x01–x13 PASS: Metal (yawgpu) 28.59 s, Dawn
26.11 s. Goldens unchanged.

## P8 — library breadth, slice 1 (2026-08-23)

`Buffer<T>.read` and `readOne` own the staging buffer (BF9 Rev 1,
on subscript R36 at `ac9436f`), BF10 usage traps, componentwise
vector builtins (K25 Rev 1), `Vec*b` with comparisons and `select`
(K26), in-order swizzle methods and `From`/`Splat` factories (K27).
Programs `b12-readback`, `b13-vector-builtins`,
`x14-live-vector-builtins`. Phase review: 1 CRITICAL (`smoothstep`
argument order, shared by host body and emission) / 3 MAJOR / 14
MINOR, all closed. Evidence: gate green 235 passed 114 s; live
x01–x14 PASS on Metal 35.55 s and Dawn 32.88 s. Record:
`specs/tracking/p8-library-breadth.md`.

## P8 — library breadth, slice 2 (2026-08-23)

WGSL shells with host bodies and `naga` attribution (K29–K31), guarded
dispatch as a runtime-owned `guard` layout entry (PI15), indirect
dispatch and the argument schemas (PI16, PI17), `indexFormat` on the
render spec (RN18), cull proven live (RN19), one module order (K14
Rev 5), four-byte alignment traps (BF2 Rev 1). Programs `b14`–`b17`,
`x15`–`x18`. Phase review: 1 CRITICAL (a barrier inside the injected
guard) / 7 MAJOR / 15 MINOR, all closed. Evidence: gate green 240
passed 146 s; live x01–x18 PASS on Metal 47.17 s and Dawn 43.50 s.
Record: `specs/tracking/p8-library-breadth.md`.

## P9 — the window host (2026-08-23)

`crates/window`: a winit host that owns the window, the surface, the
instance, the device, and the loop, and calls `init`, `frame`, and
`shutdown` on a script (W1–W13). The surface family is a generated
Rust-only module resolved on first use (L14, F23). `examples/window-triangle`
and `tools/window.sh`. Phase review: 1 CRITICAL (`await_future` on a
failure status) / 4 MAJOR / 14 MINOR, all closed, plus one measured
regression (the host compiled without the support module) closed by
one shared program loader. Evidence: gate 246 passed 168 s; `--frames
120` on Metal and Dawn; the owner's interactive Dawn run. Record:
`specs/tracking/p9-window.md`.

## P10 — the example ports; P11 — the feature gaps (2026-08-25)

P10: twenty TypeGPU example ports under `examples/` (EX1–EX7), the
sdf and noise library modules, every windowed port visually passed
by the owner, and the survey of all 77 upstream examples. P11: the
four planned gap slices — texture upload and strip, pointer input,
read-access storage textures, blending — each proven by a `b` and
an `x` program. The emitter gained three Tint-compatibility rules
the examples surfaced (literal suffixes, the `i32` minimum
spelling, logic parentheses), and T18's bounded worker pool cut the
full gate from 214 s to 106 s. Records:
`specs/tracking/p10-examples.md`, `specs/tracking/p11-feature-gaps.md`,
`specs/tracking/build-time.md`.

## P12 — the TypeGPU spelling parity (2026-08-25)

EG11 fixes the authored spelling. The index form replaced `get(i)`
and `set(i, v)` on `Storage`, `MutStorage`, and `WorkgroupArray`
everywhere, and the vector factory family took the `vec` root that
TypeGPU uses. The measurement behind the scope is the table in
`specs/tracking/p12-spelling.md`: it names the subscript rule that
forces every divergence the change did not close. Two rules now hold
the spelling: T20 gates the authored form in `programs/` and
`examples/`, and T21 pins the two forms to one WGSL. Phase review: 1
MAJOR (the sweep erased the only proof of that equality) and 12
MINOR, all closed. Evidence: gate green with the backend, 256
passed, 1 ignored, 116 s, every `.wgsl` and `.expected` golden
byte-identical, live green on Metal 67.49 s. Record:
`specs/tracking/p12-spelling.md`.
