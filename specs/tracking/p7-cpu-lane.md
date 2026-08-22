# P7 — the CPU lane

Status: **in progress**. Opened 2026-08-23 by the owner's decision.
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

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | `x01`–`x04` and `x09` use `simulateCompute` as their oracle and print `PASS` on Metal | Metal and Dawn at `1da3db8` |
| 2 | One `b` program's host golden is committed and compared on both tiers | `b02-vecadd` `host:out=5,7,9`, and every host-runnable `b` pipeline |
| 3 | `CL2` has a fixture | Slice 1 |
| 4 | Budgets hold | 46 s / 0.2 s / 84 s / 81 s / 6 |
