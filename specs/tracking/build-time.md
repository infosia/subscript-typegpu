# Build time — measurements

Reference machine: Apple M2, 16 GB. `CARGO_BUILD_JOBS=4`. Procedure:
`specs/blocks/testing.md` T12. Budgets: plan §7.

| Date | Commit | Cold build | Warm no-op | Gate after codegen change | Gate after program change | Test executables |
|---|---|---|---|---|---|---|
| 2026-08-22 | slice 2 tree on `11dc4de` | 34 s | 0.2 s | 39 s | 10 s | 4 |

| 2026-08-22 | slice 2 round 2 tree on `481782f` | 37 s | 0.2 s | 9 s | 6 s | 4 |

Row 1 was run by the planner with the round 1 script, before T12
Rev 1 fixed the measurement order (the cold build excluded the
ship-tier release build, and the codegen-change gate paid for the
first `target/ship-build`). Row 2 was run by the coding agent with
the T12 Rev 1 order: the cold build includes the ship-tier release
builds, and an untimed gate precedes each timed gate. Row 1 was
run by the planner with `tools/gate.sh --measure --yes`, backend set.
The coding agent's run on the same tree gave 27 s / 0.2 s / 36 s /
10 s / 4. The cold build is genuine: `cargo clean` removed `target/`,
no user-wide build cache exists, and the rebuilt `target/` is 1.5 GB.
Every number is inside the plan §7 budget (480 s / 5 s / 240 s /
120 s / 4).
