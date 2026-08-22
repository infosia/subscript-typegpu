# P5 — textures and samplers

Status: **in progress**. Opened 2026-08-23. Plan §8 P5. Contract:
`specs/blocks/texture.md` (TX1–TX8).

## Slice 1 — the wrappers, the texture calls, `b11-texture`, `x10`, `x11`

Handoff issued 2026-08-23. Result: pending.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | `b11-texture` gate-green with its WGSL golden | — |
| 2 | `x10-live-texture` and `x11-live-fragment-sample` `PASS` against the host sampling body | — |
| 3 | Every TX8 rejection has a red fixture | — |
| 4 | Budgets hold | — |
