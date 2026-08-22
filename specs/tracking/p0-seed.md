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

Round 1 (2026-08-22): the harness crate (source-file set, facade
`NativeLibrary`, ship link inputs from a nested release build into
`target/ship-build`, dev and ship runners, the `dev`/`ship` CLI),
the differential module, `a01-smoke`, and `tools/gate.sh --measure`.
Planner verification: gate green with the backend (153 tests), the
CLI's dev output equals the golden, and the five measurements
(`build-time.md`).

Slice review (a fresh reviewer, 2026-08-22): CRITICAL 0, MAJOR 6,
MINOR 11. M1: the golden held backend-returned bytes (T2). M2:
`set_var` on a test thread. M3, M4: the cold measurement excluded
the ship-tier release build, and the program-change gate paid for a
restore. M5 doubted the 27-second cold build. The planner's own
clean-and-build measured 34 seconds with no user-wide build cache and
a 1.5 GB rebuilt `target/`, so M5 is refuted and only its emptiness
assertion was adopted. M6: no direct tests for the harness API.
Round 2 closed every item. T5, T12, and T14 changed at `481782f` to
carry the child-process rule, the measurement order, and `--offline`.

Planner verification at the slice close: gate green, 156 tests in
four executables (facade 3, harness 6, generator unit 1, generator
integration 146), the golden holds markers and host comparisons
only, `set_var` exists only in the harness binary's single-threaded
start, hygiene clean.

## Phase review (2026-08-22)

A fresh reviewer ran the gate (green, 156 tests, four executables)
and found CRITICAL 0, MAJOR 7, MINOR 13. The MAJORs: I12 neither
implemented nor deferred. Handoff residue in one module doc. The
I4 hygiene regex could not match an identifier form of the banned
prefix. 24 `unsafe` blocks in the generated facade without a
`// SAFETY:` comment (L10). The mirror's banner cites subscript's
specs. The slice-close citation could not distinguish a run with the
backend from one without. Invariant 3 and L7 stated an absolute the
pinned header's uncaptured-error callback cannot satisfy.

Resolutions in the specs: I12 Rev 1 defers coverage to a measured
list from P2 (the harness counts facade exports each dev run calls)
and P6 closes it. F21 records the mirror banner as a deviation: the
file is bindgen's output byte for byte, and a banner change is a
subscript change request. Invariant 3 and L7 name the
uncaptured-error exception. T14 gains `--require-backend` and the
`gate: green, pending <n>` line. T9 and build-time rule 3 count one
integration executable plus at most one unit executable per crate.
The plan's output count is six, its probe path is
`crates/typegpu-gen/src/lib.rs`, and I1 names the renamed tests.

Code resolutions: handed off as the phase-close round.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | `a01` byte-identical on both tiers and the golden | Slice 2 close: dev, ship, golden equal. A one-byte golden change fails both tiers with the offset and both lines |
| 2 | Regeneration gate red then green | Slice 1: a one-byte change to `lib/webgpu.ts` fails T6 naming `tools/regen.sh`, green after regen |
| 3 | Five build-time measurements recorded | `build-time.md`: two rows, both inside budget |
| 4 | Facade deps: `libloading` alone. `syn` not a direct dependency | Measured at the slice 1 close: `libloading` → `cfg-if` only, `cargo tree -e normal --depth 1` lists no `syn` |
| 5 | No `[features]`, no `build.rs` | `tools/hygiene.sh` checks both, clean at the slice 1 close |
| 6 | I4–I9 green by their gates. I10 measured | I10: the 28 aliases derive from `[api].enums`. One pass. Six outputs byte-identical before and after |
| 7 | Test count before and after I8 equal | 143 → 143 at the merge. 146 after the round 2 and 3 additions |
