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

Delivered 2026-08-23, committed at `e07b434`. `a02-textures`,
`a03-encoders`, `a04-errors` reach every export the API layer can
call; six exports left through policy rows (157 exports, 157
reached, the coverage test fails on any unreached export).
`tools/live.sh` accepts `default`. `TimestampPair` and
`a05-timestamps`. `README.md`, `docs/tutorial.md`, and the docs
quote test. Planner verification: gate green, 221 tests.
Measurement: 45 s / 0.2 s / 97 s / 94 s / 6 — inside budget, and
the program-change gate approaches its 120 s line because the
coverage test runs every program's dev tier a second time.

Live runs by the planner outside the sandbox at `e07b434`: Metal
(yawgpu) ok, 26.28 seconds, `x01`–`x12` `PASS`. **Dawn** (EG5,
`libwebgpu_dawn.dylib`, `SUBSCRIPT_TYPEGPU_BACKEND=default`): ok,
25.23 seconds, `x01`–`x12` `PASS`. Dawn accepted every module and
every bind group, the uniform bindings with `minBindingSize` 8
included. The P2 review's M6 claim (a uniform struct needs a
16-byte minimum binding size) is refuted by measurement: LY11
stands as written, and the layout engine keeps one layout.

## Phase review (2026-08-23)

A fresh reviewer ran the gate (green, 96.58 s, 221 tests) and found
CRITICAL 1, MAJOR 6, MINOR 13. CRITICAL: `README.md` listed `noop`
among the accepted backend values; the facade rejects it. MAJOR: the
coverage close deleted six exports through an undocumented policy
row (now F22, 163 → 157 recorded); PI8 still denied generated types;
the sweep accepted `covers-rule` claims for PI2, RN2, and RN8 sites
the checker makes unreachable; the coverage pass re-ran every
program and discarded its output; `TimestampPair.resolve` ignored
its buffer; `a04-errors` printed two backend feature values. The
reviewer measured the gate's cost structure — 60 dev and 32 ship
child processes per program-change gate — and proposed counting
coverage during the differential's own dev runs, which removes 16
runs and makes the counted run the gated run. Resolutions in the
specs: F22, PI8 Rev 2, PI11, BF3 wording, EG4 Rev 1, EG7 wording.
The code findings go to the coding agent.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | EG1 typed resources used by one `b` program | `b02-vecadd` |
| 2 | EG4's list is closed | 157 of 157 at `e07b434` |
| 3 | EG5's Dawn run is recorded with its decision | Dawn at `e07b434`: ok, 25.23 s, LY11 stands |
| 4 | EG7's sweep finds no diagnostic without a rule id, an owner, and a fixture | Slice 1, green |
| 5 | `README.md` and the tutorial exist with their quote gate | Slice 2 |
| 6 | Budgets hold | 45 s / 0.2 s / 97 s / 94 s / 6 |
