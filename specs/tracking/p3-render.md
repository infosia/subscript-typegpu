# P3 — render

Status: **in progress**. Opened 2026-08-22. Plan §8 P3. Contract:
`specs/blocks/render.md` (RN).

## Slice 1 — the render emitter, the runtime, `b06-render`, `x05-live-triangle`

Delivered 2026-08-23. Planner verification: `tools/gate.sh
--require-backend` green, 197 tests in six executables (facade 4,
typegpu-gen 7 and 19, harness 17 in 23.14 seconds plus the ignored
live test, webgpu-gen 3 and 147). `b06-render` equal on both tiers
with its `.tri.wgsl` golden (19 lines, printed as 20 by the program's split: the vertex input struct with
locations, the varyings with `@builtin(position)`, `@vertex` and
`@fragment` entries). `x05-live-triangle` checks all 4096 pixel
centers against the host rasterizer and found none on an edge. Six
RN16 fixtures. Measurement: 45 s / 0.2 s / 33 s / 32 s / 6. Live
run by the planner outside the sandbox at `3184883` (Metal):
`live::every_x_program_passes_on_a_real_adapter ... ok`, 10.70
seconds, `x01`–`x05` `PASS`, so every pixel of the rendered
triangle equals the host rasterizer's.

## Slice 2 — draw variants

Delivered 2026-08-23, committed at `b32380d`. Planner verification:
gate green, 197 tests (the harness executable 28.13 seconds on the
coding agent's run), `b07-draw-variants` equal on both tiers with
the `quad` (instanced, 23 lines) and `tri` (18 lines) goldens. Live
run by the planner outside the sandbox at `b32380d` (Metal): ok,
12.87 seconds, `x01`–`x06` `PASS` — three instanced quads through
`drawIndexed(6, 3)` equal the host rasterizer pixel by pixel (the
coding agent's host probe counted 225, 210, and 210 covered pixels
per instance, 58 of them in more than one instance). Measurement: 45 s / 0.2 s / 38 s
/ 35 s / 6.

## Phase review (2026-08-23)

A fresh reviewer ran the gate (green, 34.92 s, 197 tests) and found
CRITICAL 1, MAJOR 3, MINOR 15. C1: RN3 gave `FragmentInvocation` a
`position` that emitted a second `@builtin(position)` beside the
varyings' one — a contract contradiction cemented by a passing
emitter test. M1: RN7 named no type set for varyings. M2: render
bindings (`renderPipelineL`, visibility by reach) had no program
and no emitter test. M3: the generator recognized the declaration
functions by name alone, and three fixtures reached their rule
through program-declared look-alikes. Resolutions: RN3 Rev 1 drops
`position` from `FragmentInvocation`, RN7 Rev 1 names the type set,
RN8 defers the multi-target form to a typed overload, RN9 makes an
unreached binding a diagnostic, RN1 recognizes by declaring file,
RN16 Rev 1 sends the checker-first cases to the checker, RN17 adds
`b08-render-bindings` and `x07-live-render-uniform`. The P4 and P5
program ids shift by one. T12 Rev: a row names the tree it
measured. The code findings go to the coding agent.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | `b06-render` and `b07-draw-variants` gate-green with WGSL goldens | Slices 1 and 2, `--require-backend` green |
| 2 | `x05-live-triangle` and `x06-live-draw-variants` `PASS` against the host rasterizer | Metal at `3184883` (10.70 s) and `b32380d` (12.87 s) |
| 3 | Every RN16 rejection has a red fixture | Six in slice 1, two in slice 2 |
| 4 | Budgets hold | 45 s / 0.2 s / 38 s / 35 s / 6 |
