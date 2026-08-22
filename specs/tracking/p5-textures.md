# P5 — textures and samplers

Status: **in progress**. Opened 2026-08-23. Plan §8 P5. Contract:
`specs/blocks/texture.md` (TX1–TX8).

## Slice 1 — the wrappers, the texture calls, `b11-texture`, `x10`, `x11`

Delivered 2026-08-23, committed at `4b34f4a`. Planner verification:
`tools/gate.sh --require-backend` green, 214 tests in six
executables (facade 4, typegpu-gen 7 and 35, harness 18 in 41.21
seconds plus the ignored live test, webgpu-gen 3 and 147).
`b11-texture` equal on both tiers with its golden: two groups, a
sampled texture, a sampler, a write-only storage texture, a uniform
in group 1, `textureLoad`, `textureSampleLevel`, `textureStore`.
Six TX8 fixtures and the resource-kind trap. Live run outside the
sandbox at `4b34f4a` (Metal): ok, 23.60 seconds, `x01`–`x11`
`PASS` — the compute sampling of the checkerboard and the fragment
`textureSample` both equal the host sampling body. Measurement:
44 s / 0.2 s / 54 s / 51 s / 6.

Noted for the phase review: the emitter copies a uniform binding
into a local `var` of the same name (`var params = params;`) when
the author's local shares the binding's name. Valid WGSL, but a
shadowing shape.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | `b11-texture` gate-green with its WGSL golden | Slice 1, `--require-backend` green |
| 2 | `x10-live-texture` and `x11-live-fragment-sample` `PASS` against the host sampling body | Metal at `4b34f4a`, 23.60 s |
| 3 | Every TX8 rejection has a red fixture | Six fixtures and one trap, one diagnostic each |
| 4 | Budgets hold | 44 s / 0.2 s / 54 s / 51 s / 6 |
