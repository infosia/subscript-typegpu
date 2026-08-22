# P0 — seed and generator import

Status: **in progress**. Opened 2026-08-22. Plan §8 P0. Contracts:
`specs/blocks/generator-import.md`, `facade.md`, `testing.md`.

## Pins

| Input | Pin |
|---|---|
| subscript | `4313dcfe74df3445dd98b5002b751ac6af28e77b` (slice 1). Slice 2 re-pins to `ba6aa2e`, which carries R33 (`@CStruct({ align })`) |
| webgpu-headers | `a11ef4462405c4506ad7284e5b1edeff2750bb54` |
| gpuweb | `d390da5f80f18e82d9535a40c6f2f1f65e6884ae` |
| Rust | `1.95.0` |

## Slice 1 — the import

Round 1 (2026-08-22): the coding agent delivered the workspace, the
imported generator (143 tests in one executable, count equal before
and after the merge), the facade with the function-table loader, the
harness skeleton, and the six regenerated outputs. Measured by the
planner: fmt, clippy, and tests green. Facade dependency closure
`libloading` → `cfg-if`. `syn` present only through `serde_derive`.
Regeneration stable across two runs. Exports 163, symbol-table rows
163. I10 measured: the CEnum alias list derives from the policy's
`[api].enums`, so the facade generates once and
`generate_with_cenum_aliases` left, with all six outputs
byte-identical before and after.

Two handoff defects found by round 1: the `syn` criterion named
`Cargo.lock` instead of direct dependencies, and two spec lines
carried the old prefix token the hygiene scan bans. Both corrected in
the specs at `bcb1672`.

Slice review (a fresh reviewer, 2026-08-22): CRITICAL 2, MAJOR 5,
MINOR 9. C1: two exports without a handle parameter reach a shim with
no table and abort. C2: the shim abort prints nothing. M1: no T8
test. M2: the gate cannot show a pending line. M3: the regen test
re-implements the driver. M4: libclang absence disables five tests
that do not need it. M5: the proof of concept's rule ids are cited
throughout and collide with this repository's ids. The id inventory
found 62 distinct ids over 388 citations. Resolution: this
repository's colliding ids moved (facade.md F → L, the plan's G
aliases dropped, pipelines D → PL, H15 → J13), the proof of concept's
rules are restated in `facade-generator.md` and `api-layer.md`, and
`rule-ids.txt` gates every citation. E2 was a drafting slip in the
proof of concept and does not exist here.

Round 2 handoff issued 2026-08-22 with every finding. Result:
pending.

## Slice 2 — the first program

Not started.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | `a01` byte-identical on both tiers and the golden | — |
| 2 | Regeneration gate red then green | — |
| 3 | Five build-time measurements recorded | `build-time.md` |
| 4 | Facade deps: `libloading` alone. `syn` not a direct dependency | — |
| 5 | No `[features]`, no `build.rs` | — |
| 6 | I4–I9 green by their gates. I10 measured | — |
| 7 | Test count before and after I8 equal | — |
