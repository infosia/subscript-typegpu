# P3 — render

Status: **in progress**. Opened 2026-08-22. Plan §8 P3. Contract:
`specs/blocks/render.md` (RN).

## Slice 1 — the render emitter, the runtime, `b06-render`, `x05-live-triangle`

Delivered 2026-08-23. Planner verification: `tools/gate.sh
--require-backend` green, 197 tests in six executables (facade 4,
typegpu-gen 7 and 19, harness 17 in 23.14 seconds plus the ignored
live test, webgpu-gen 3 and 147). `b06-render` equal on both tiers
with its `.tri.wgsl` golden (20 lines: the vertex input struct with
locations, the varyings with `@builtin(position)`, `@vertex` and
`@fragment` entries). `x05-live-triangle` checks all 4096 pixel
centers against the host rasterizer and found none on an edge. Six
RN16 fixtures. Measurement: 45 s / 0.2 s / 33 s / 32 s / 6.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | `b06-render` and `b07-draw-variants` gate-green with WGSL goldens | — |
| 2 | `x05-live-triangle` and `x06-live-draw-variants` `PASS` against the host rasterizer | — |
| 3 | Every RN16 rejection has a red fixture | — |
| 4 | Budgets hold | — |
