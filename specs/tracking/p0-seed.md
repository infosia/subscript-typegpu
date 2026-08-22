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

Handoff issued 2026-08-22. Result: pending.

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
