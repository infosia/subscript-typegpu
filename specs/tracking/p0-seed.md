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

Round 2 (2026-08-22) closed every finding. Round 3 closed two
defects the planner found in round 2: the driver read the committed
mirror as an input of the pass that wrote `webgpu.ts` (regeneration
was not idempotent after a header change), and the driver edited
bindgen's output to hide a `Q13` token. The split is now three
libclang-free outputs (header, Rust, symbol table) and three
libclang outputs (mirror, wire aliases, `webgpu.ts`). The rule-id
scanner allows every `Q<n>` and `R<n>`.

Planner verification at the slice close: `tools/gate.sh` green, 151
tests in four executables (facade 3, harness 1, generator unit 1,
generator integration 146), hygiene clean, facade closure
`libloading` → `cfg-if`, three deleted outputs restored
byte-identically by one `tools/regen.sh` run, two runs stable.
Reds recorded by the coding agent: a removed symbol-table row fails
T8 with the name, a one-byte change to `lib/webgpu.ts` fails T6 with
`run tools/regen.sh`.

Deviation: generator-import.md says each reshape item is one commit.
The three rounds landed as one working tree, so slice 1 is one
commit. The per-item evidence is in the handoff reports, restated
above.

## Slice 2 — the first program

Not started.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | `a01` byte-identical on both tiers and the golden | — |
| 2 | Regeneration gate red then green | — |
| 3 | Five build-time measurements recorded | `build-time.md` |
| 4 | Facade deps: `libloading` alone. `syn` not a direct dependency | Measured at the slice 1 close: `libloading` → `cfg-if` only, `cargo tree -e normal --depth 1` lists no `syn` |
| 5 | No `[features]`, no `build.rs` | `tools/hygiene.sh` checks both, clean at the slice 1 close |
| 6 | I4–I9 green by their gates. I10 measured | I10: the 28 aliases derive from `[api].enums`. One pass. Six outputs byte-identical before and after |
| 7 | Test count before and after I8 equal | 143 → 143 at the merge. 146 after the round 2 and 3 additions |
