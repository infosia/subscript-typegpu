# Project status

Updated 2026-08-23 at `4c2eff8`.

| Phase | Status | Close |
|---|---|---|
| P0 seed and generator import | COMPLETE | `5f6840b`, 2026-08-22 |
| P1 schemas and layout | COMPLETE | `0e0156c`, 2026-08-22 |
| P2 compute kernels | COMPLETE | `0e0156c`, 2026-08-22 |
| P3 render | COMPLETE | `b32f14a`, 2026-08-23 |
| P4 kernel depth | COMPLETE | `ea25b01`, 2026-08-23 |
| P5 textures and samplers | COMPLETE | `3d9d988`, 2026-08-23 |
| P6 ergonomics and diagnostics | COMPLETE | `4c2eff8`, 2026-08-23 |
| P7 the CPU lane | in progress | opened 2026-08-23 by the owner's decision |

Numbers at `4c2eff8`: 223 tests in six executables, 28 programs
(`a01`–`a05`, `b01`–`b11`, `x01`–`x12`), 157 facade exports all
reached, the live lane green on yawgpu Metal and on Dawn, the gate
84 s after a generator change and 78 s after a program change
(budgets 240 s and 120 s), the cold build 45 s (budget 480 s).

Subscript requests this project made and that landed: R33
(`@CStruct({ align })`), R34 (`Context.bytesOf`, `bytesInto`,
`fromBytes`), R35 (`check_program_with`, the discovery check). The
pin is `bb9dadc`.

Defects the live lane found that no validator reported: the
barrier after a non-uniform early return (P4, K22 Rev 1 and Rev 3).
Defects the phase reviews found that the gate could not see: the
operator precedence after a method lowering, the `?:` lowering in a
loop condition (P2), the duplicated `@builtin(position)` (P3), the
kernel local that shadowed a binding (P5), the README's backend
values (P6).

Open items carried in the blocks: RN8's multiple targets, TX1's
integer sample types and formats, a phased `simulateCompute` for
barrier kernels (CL2), a typed per-field record for render
pipelines beyond one group (TX2).
