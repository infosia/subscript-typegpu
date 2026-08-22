# P6 — ergonomics and diagnostics

Status: **in progress**. Opened 2026-08-23. Plan §8 P6. Contract:
`specs/blocks/ergonomics.md` (EG1–EG9).

## Slice 1 — typed resources, buffer patches, the coverage list, the sweep

Delivered 2026-08-23, committed at `bd8c0b1`. Typed `LResources`
records and `create<Name>BindGroup<g>` factories per layout class
(`b02-vecadd` uses them), `Buffer<T>.patch` and `readOne<T>`
(`b05-buffer`), counting thunks on all 163 facade exports with a
`--coverage` mode and `specs/tracking/coverage.md` (64 reached, 99
unreached — labels, render bundles, debug markers, query sets,
indirect draws, error scopes, limits and features, texture getters,
async pipeline creation, and the descriptor-less
`adapter_request_device` the API layer never calls), the diagnostic
and trap sweep (every site cites a rule id, an owner, and has a
fixture). Planner verification: gate green. Measurement: 44 s / 1 s
/ 78 s / 75 s / 6 — the coverage test runs every program's dev tier
a second time, which the gate budgets absorb (240 s / 120 s).

## Slice 2 — the coverage close, the Dawn run, timestamps, README and tutorial

Not started.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | EG1 typed resources used by one `b` program | — |
| 2 | EG4's list is closed | — |
| 3 | EG5's Dawn run is recorded with its decision | — |
| 4 | EG7's sweep finds no diagnostic without a rule id, an owner, and a fixture | — |
| 5 | `README.md` and the tutorial exist with their quote gate | — |
| 6 | Budgets hold | — |
