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
| 2026-08-22 | P1 slice 1, the tree committed as `a633f12` (six executables from here) | 45 s | 0.2 s | 13 s | 10 s | 6 |
| 2026-08-22 | P2 slice 1, the tree committed as `9c32b4b` | 44 s | 0.2 s | 40 s | 25 s | 6 |
| 2026-08-22 | P2 slice 2, the tree committed as `fae7d46` | 44 s | 0.2 s | 25 s | 22 s | 6 |
| 2026-08-22 | P1 slice 2, the tree committed as `b7e2533` | 45 s | 0.2 s | 28 s | 25 s | 6 |
| 2026-08-22 | P1 and P2 close, the tree committed as `0e0156c` | 42 s | 0.2 s | 29 s | 27 s | 6 |
| 2026-08-23 | P3 slice 1, the tree committed as `3184883` | 45 s | 0.2 s | 33 s | 32 s | 6 |
| 2026-08-23 | P3 slice 2, the tree committed as `b32380d` | 45 s | 0.2 s | 38 s | 35 s | 6 |
| 2026-08-23 | P3 close, the tree committed as b32f14a | 44 s | 1 s | 41 s | 38 s | 6 |
| 2026-08-23 | P4 slice 1, the tree committed as `010e846` | 45 s | 0.2 s | 48 s | 47 s | 6 |
| 2026-08-23 | P4 slice 1 round 2, the tree committed as `9670180` | 44 s | 0.2 s | 48 s | 47 s | 6 |
| 2026-08-23 | P4 close, the tree committed as `ea25b01` | 44 s | 0.2 s | 49 s | 46 s | 6 |
| 2026-08-23 | P5 slice 1, the tree committed as `4b34f4a` | 44 s | 0.2 s | 54 s | 51 s | 6 |
| 2026-08-23 | P5 close, the tree committed as `3d9d988` | 45 s | 0.2 s | 55 s | 53 s | 6 |
| 2026-08-23 | P6 slice 1, the tree committed as `bd8c0b1` | 44 s | 1 s | 78 s | 75 s | 6 |
| 2026-08-23 | P6 slice 2, the tree committed as `e07b434` | 45 s | 0.2 s | 97 s | 94 s | 6 |
| 2026-08-23 | P6 close, the tree committed as `4c2eff8` (coverage in the differential runs) | 45 s | 0.2 s | 84 s | 78 s | 6 |
| 2026-08-23 | P7 slice 1, the tree committed as `1da3db8` | 46 s | 0.2 s | 84 s | 81 s | 6 |
| 2026-08-23 | P7 close, the tree committed as `167d34e` | 45 s | 0.2 s | 86 s | 84 s | 6 |
| 2026-08-23 | Retrospective close, the tree committed as `c0ebfed` | 45 s | 0.2 s | 86 s | 84 s | 6 |

Row 1: the planner, before T12 Rev 1 fixed the order (the cold build
excluded the ship-tier release build, and the codegen-change gate
paid for the first `target/ship-build`). Row 2 onward: the T12 Rev 1
order. Rows 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22: the coding agent. Row 3: the planner. The
cold build is genuine: `cargo clean` removes `target/`, no user-wide
build cache exists, and the rebuilt `target/` is 1.5 GB. Every
number is inside the plan §7 budget (480 s / 5 s / 240 s / 120 s /
2 per crate).
