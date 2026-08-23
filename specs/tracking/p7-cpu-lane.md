# P7 — the CPU lane

Status: **COMPLETE 2026-08-23.** Opened 2026-08-23 by the owner's
decision. Closed at `167d34e`.
Plan §8 P7. Contract: `specs/blocks/cpu-lane.md` (CL1–CL4).

## Slice 1 — `simulateCompute`, the host-runnable constant, the oracles

Delivered 2026-08-23, committed at `1da3db8`. `simulateCompute` and
its layout forms loop the invocations in row-major order and call
the kernel with a built `ComputeInvocation`; the generator emits
`<name>_HOST_RUNNABLE` from the call graph (`x08` and `b10` false,
the rest true); `x01`–`x04` and `x09` take the kernel body as their
oracle through the wrappers' host storage; every host-runnable `b`
pipeline prints a `host:` line into its golden (`b02`: `host:out=5,7,9`)
and a harness `simulate` module requires it; the CL2 trap fixture.
Planner verification: `tools/gate.sh --require-backend` green, 223
tests in six executables (facade 3, typegpu-gen 6 and 38, harness 24
in 70.76 seconds plus the ignored live test, webgpu-gen 3 and 149).
Live runs outside the sandbox at `1da3db8`: Metal ok 26.57 seconds,
Dawn ok 24.27 seconds, `x01`–`x12` `PASS` on both — the GPU results
equal the kernel bodies run on the host. Measurement: 46 s / 0.2 s /
84 s / 81 s / 6.

The coding agent found `rule-ids.txt` without the CL ids (the
planner's omission, corrected at `f590e81`'s successor) and made the
scanners read `cpu-lane.md` headings instead; the close round
returns the scanners to the table.

## Phase review (2026-08-23)

A fresh reviewer ran the gate (green) and found CRITICAL 0, MAJOR 6,
MINOR 14. MAJOR: a written private variable was host-runnable while
the host holds one shared instance; the barrier and atomic branches
of the analysis had no program that could fail them; the CL2 fixture
passed a literal `false` instead of the generator's constant; the
`hostRunnable` argument was unenforceable; `b11`'s host run bound a
nearest sampler beside a linear GPU sampler; the scanners carried a
duplicate rule list. Resolutions in the specs: CL2 Rev 1 (written
private variables, the harness pairing check over the HIR), CL4 Rev
1 (host-runnability from `Generated`), CL5, CL6, `simulateComputeThreads`
in CL1, PI4, PI8, and PI11 cross-references. The code findings landed at
`167d34e`: written private variables are not host-runnable, five
CL6 emitter tests, the CL2 fixture traps through the generated
constant, the harness pairs every `simulateCompute*` call with its
pipeline's constant over the HIR (a scratch program passing `true`
fails with the call's position), `simulateComputeThreads` takes the
dispatch's counts, samplers carry their filter mode on the host and
`b11` binds nearest on both sides, the rule-id table is the one
authority. Planner verification at the close: `tools/gate.sh
--require-backend` green, 231 tests in six executables (facade 3,
typegpu-gen 6 and 44, harness 26 in 75.48 seconds plus the ignored
live test, webgpu-gen 3 and 149). Live runs outside the sandbox at
`167d34e`: Metal ok 26.64 seconds, Dawn ok 24.35 seconds,
`x01`–`x12` `PASS` on both with the kernel bodies as the oracle.
Measurement: 45 s / 0.2 s / 86 s / 84 s / 6.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | `x01`–`x04` and `x09` use `simulateCompute` as their oracle and print `PASS` on Metal | Metal and Dawn at `1da3db8` |
| 2 | One `b` program's host golden is committed and compared on both tiers | `b02-vecadd` `host:out=5,7,9`, and every host-runnable `b` pipeline |
| 3 | `CL2` has a fixture that reaches the trap through the generator's constant | `simulate-storage-barrier.ts` at the close |
| 4 | CL6's five emitter tests exist. The lane is a gate module | The close; `crates/harness/tests/simulate/mod.rs` |
| 5 | Budgets hold | 45 s / 0.2 s / 86 s / 84 s / 6 at the close |
