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
| 2026-08-23 | PI14 error scopes in every program, `x13-live-rejection` | 46 s | 0 s | 88 s | 86 s | 6 |
| 2026-08-23 | windows-msvc port merged (`2d592b7`), reference machine re-check | — | — | — | 93 s | 6 |
| 2026-08-23 | EG9 Rev 1, `docs/from-typegpu.md` and the docs gate over `docs/*.md` | 47 s | 0 s | 88 s | 85 s | 6 |
| 2026-08-23 | EG9 Rev 2, README rewrite under the quote gate | 46 s | 0 s | 89 s | 86 s | 6 |
| 2026-08-23 | P8 slice 1 steps 2–3 (K25–K27, b13, x14), measured under load: `syspolicyd` 107 %, `XprotectService` 60 %, a concurrent subscript `cargo test`; re-measure at the P8 close | 46 s | 0 s | 178 s | 137 s | 6 |
| 2026-08-23 | Re-pin subscript `ac9436f` (R36), load average 3.6 | — | — | — | 117 s | 6 |
| 2026-08-23 | P8 slice 1 step 1 (`Buffer<T>.read`, b12), load average 6.2 during the run | 45 s | 0 s | 131 s | 132 s | 6 |
| 2026-08-23 | P8 review fixes, load average 2.9 | 45 s | 0 s | 117 s | 114 s | 6 |
| 2026-08-23 | P8 slice 2 (4 `b`, 4 `x` programs, 11 fixtures), load average 2.6 | 45 s | 0 s | 148 s | 142 s | 6 |
| 2026-08-23 | P8 slice 2 review fixes | 45 s | 0 s | 148 s | 146 s | 6 |
| 2026-08-23 | P9 window host crate added (cold build 45 s → 48 s; the seventh executable is the window binary, no tests) | 48 s | 0 s | 149 s | 144 s | 7 |
| 2026-08-23 | P9 review fixes, one program loader (gate 146 s → 168 s: the window example compiles in the gate and the loader generates the support module for every dev-lane program in one place) | 49 s | 0 s | 168 s | 164 s | 7 |
| 2026-08-24 | Backend request merged (`7eb7d79`), reference machine re-check | — | — | — | 177 s | 7 |
| 2026-08-24 | P10 slice 1 round 1 (seven example ports) | 49 s | 0 s | 172 s | 169 s | 7 |
| 2026-08-24 | P10 slice 1 round 2 (three simulation ports) and the EX2 comments | 49 s | 0 s | 175 s | 175 s | 7 |
| 2026-08-24 | P10 slice 1 round 3 (EG10 migration, cleanup) | 49 s | 0 s | 173 s | 172 s | 7 |
| 2026-08-24 | P10 slice 1 round 4 (PI15 Rev 2, xor tone-map) | 48 s | 0 s | 173 s | 172 s | 7 |
| 2026-08-24 | P10 slice 1 round 5 (simulation cleanup), slice close | 48 s | 0 s | 173 s | 173 s | 7 |
| 2026-08-24 | P10 slice 2 (sdf and noise modules, five ports) | 48 s | 1 s | 178 s | 177 s | 7 |
| 2026-08-24 | P11 slice 1 (TX9/TX10 upload, RN20 strip; 2 `b`, 2 `x` programs) | 51 s | 3.9 s first no-op after a build, then sub-second | 198 s | 198 s | 7 |
| 2026-08-24 | P10 slice 3 (clouds, strip de-reductions) | 49 s | 0 s | 197 s | 197 s | 7 |
| 2026-08-24 | P11 slice 2 (pointer input) | 48 s | 0 s | 196 s | 196 s | 7 |
| 2026-08-24 | P10 slice 4 (fluid-with-atomics) | 49 s | 0 s | 197 s | 197 s | 7 |
| 2026-08-24 | P11 slice 3 (read-access storage textures) | 47 s | 1 s | 209 s | 205 s | 7 |
| 2026-08-24 | K14 Rev 6 (suffixed literals, 40 goldens regenerated) | 49 s | 0 s | 205 s | 203 s | 7 |
| 2026-08-24 | P10 slice 5 (slime-mold, game-of-life) | 49 s | 0 s | 206 s | 203 s | 7 |
| 2026-08-24 | P11 slice 4 (blending) and the slice 5 fix round | 49 s | 0 s | 215 s | 216 s | 7 |
| 2026-08-24 | slime-mold symmetry fix | 48 s | 1 s | 222 s | 218 s | 7 |
| 2026-08-24 | Emitter parenthesizes mixed logical chains | 49 s | 0 s | 216 s | 213 s | 7 |
| 2026-08-24 | T18 bounded worker pool: differential 154.5 s → 43.2 s, coverage 112.3 s → 33.9 s, simulate 95.1 s → 26.6 s, c_layout 31.1 s → 10.5 s; full gate 214 s → 106 s | 49 s | 0 s | — | 106 s | 7 |
| 2026-08-25 | Windows W5, the pixel-oracle fix (`24ca42e`), the tree at `bea7da5` | 49 s | 0 s | 112 s | 120 s | 7 |
| 2026-08-25 | P13 close: subscript re-pinned to `a2228d9`, the tree at `a753747` plus the accessor sweep | 51 s | 0 s | 186 s | 183 s | 7 |

Row 1: the planner, before T12 Rev 1 fixed the order (the cold build
excluded the ship-tier release build, and the codegen-change gate
paid for the first `target/ship-build`). Row 2 onward: the T12 Rev 1
order. Rows 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22: the coding agent. Rows 3 and 53: the planner. The
cold build is genuine: `cargo clean` removes `target/`, no user-wide
build cache exists, and the rebuilt `target/` is 1.5 GB. Every
number is inside the plan §7 budget (480 s / 5 s / 240 s / 120 s /
2 per crate).

Row 53 measures 120 s for the program-change gate. That is the
budget exactly. Row 52 measured 106 s. The tree gained one program
between the two rows, `b22-first-program`. The differential suite
runs every program, so the program count sets the gate time.

## Where the time goes (2026-08-24)

The harness executable holds 195 s of the 214 s gate: the
differential suite runs every program serially inside one test
function (dev JIT, ship C compile, run, compare). The growth from
86 s at the P8 open tracks the program count. Round overhead on top:
the full measurement (`--measure --yes`, about ten minutes) ran in
every coding round and the planning side re-ran the same gate after
green rounds. Practice from today: the measurement runs at slice
closes only, and a green round's gate is not re-run on the same
tree — only the lanes the coding agent cannot run (live, window).
Proposed and pending the owner's go: a bounded parallel program
loop in the differential suite.

## The P13 row's spread (2026-08-25)

The two gate columns rose from 112 s and 120 s to 186 s and 183 s.
The budget is 240 s, so the row is not red. The rise is not in the
diff.

Evidence: the harness executable ran the same 35 tests three times
on the same day. It took 94.91 s in the `--require-backend` gate on
the P13 tree, 122.70 s in the `--require-backend` gate on the
re-pinned tree with no source change, and 159.32 s inside the
measurement. The measurement runs four gates after a `cargo clean`,
so the machine is hot by the last one. The same tree therefore
produced 94.91 s and 159.32 s within one hour.

The new test of P12, T20, is not the cause. It checks every program
and every example and takes 4.79 s.

Read the row as the hot-machine bound and the `--require-backend`
runs as the warm bound.

## The `2a65724` re-pin (2026-08-28)

The warm `--require-backend` gate moved from 121.13 s at `a2228d9` to
161.35 s at `2a65724`. The harness executable holds the change: 95.05 s
before, 138.83 s after.

The cause is not in this repository's diff. At the new pin, and before
the `c_layout` probe changed, the same executable already took
145.09 s while `c_layout` failed early. subscript's one-ordered-IR
pipeline landed between the two pins.

The probe rewrite pays part of it back. Two tests each ran the program
pool, so every `b` program was checked twice. One test now does both
comparisons from one check, which took the executable from 156.18 s to
138.83 s.

One run measured 288.63 s. A second full gate ran on the machine at
the same time. Discard it.
