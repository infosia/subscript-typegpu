# Build time — measurements

Reference machine: Apple M2, 16 GB. `CARGO_BUILD_JOBS=4`. Procedure:
`specs/blocks/testing.md` T12. Budgets: plan §7. Command for every
row: `tools/gate.sh --measure --yes` with
`SUBSCRIPT_TYPEGPU_BACKEND_LIB` set.

| Date | Tree | Cold build | Warm no-op | Gate after codegen change | Gate after program change | Test executables |
|---|---|---|---|---|---|---|
| 2026-08-22 | P0 slice 2 round 1 (committed as `46070bf`), T12 Rev 0 order | 34 s | 0.2 s | 39 s | 10 s | 4 |
| 2026-08-22 | P0 slice 2 round 2 (committed as `46070bf`), T12 Rev 1 order | 37 s | 0.2 s | 9 s | 6 s | 4 |
| 2026-08-22 | P0 close, `5f6840b` | 42 s | 0.2 s | 9 s | 6 s | 4 |
| 2026-08-22 | P1 slice 1, `318729c` (six executables from here) | 45 s | 0.2 s | 13 s | 10 s | 6 |
| 2026-08-22 | P2 slice 1, `4da1875` | 44 s | 0.2 s | 40 s | 25 s | 6 |
| 2026-08-22 | P2 slice 2, `7e81ef9` | 44 s | 0.2 s | 25 s | 22 s | 6 |
| 2026-08-22 | P1 slice 2, `4502030` | 45 s | 0.2 s | 28 s | 25 s | 6 |
| 2026-08-22 | P1 and P2 close, `3011014` | 42 s | 0.2 s | 29 s | 27 s | 6 |
| 2026-08-23 | P3 slice 1, `b54c9e8` | 45 s | 0.2 s | 33 s | 32 s | 6 |
| 2026-08-23 | P3 slice 2, `d3a4f88` | 45 s | 0.2 s | 38 s | 35 s | 6 |

Row 1: the planner, before T12 Rev 1 fixed the order (the cold build
excluded the ship-tier release build, and the codegen-change gate
paid for the first `target/ship-build`). Row 2 onward: the T12 Rev 1
order. Rows 2, 4, 5, 6, 7, 8, 9, 10: the coding agent. Row 3: the planner. The
cold build is genuine: `cargo clean` removes `target/`, no user-wide
build cache exists, and the rebuilt `target/` is 1.5 GB. Every
number is inside the plan §7 budget (480 s / 5 s / 240 s / 120 s /
2 per crate).
