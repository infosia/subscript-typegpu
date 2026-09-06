# comments-pass — documentation comments on the hand-written sources

2026-09-06. Owner instruction: every public function carries a
documentation comment that is simple, non-redundant, and easy to read
for a developer. Opus subagents write them, one per file group, and
Codex writes none. Generated files are out of scope: `lib/webgpu.ts`,
the two mirrors, `lib/typegpu-ui-atlas.generated.ts`,
`crates/facade/src`, and `crates/harness/src/native_symbols.generated.rs`
take their comments from the generators.

## Coverage

| File group | Before | After |
|---|---|---|
| `lib/typegpu.ts` | 4 of 73 exports | 73 of 73 (68 blocks, two family groups) |
| `lib/typegpu-ui.ts` | 1 of 59 | 59 of 59, plus 60 public methods and 6 constant groups |
| `lib/typegpu-types.ts` | 0 of 62 | 36 documented, 26 family members self-describing after the first |
| `lib/typegpu-noise.ts`, `-sort.ts`, `-radiance-cascades.ts` | 12 of 29 | 29 of 29 |
| `lib/typegpu-color.ts`, `-sdf.ts` | complete | 5 rewritten (one wrong sign, four `-ing` forms) |
| `crates/typegpu-gen/src` | 40 of 99 items | 99 of 99, plus a crate header on the binary |
| `crates/harness/src`, `crates/window/src` | 17 of 23 | 23 of 23, plus two crate headers |
| `crates/webgpu-gen/src` | 165 of 265 | 265 of 265 across 22 files, plus a crate header on the binary |

## Spec drifts the writers found, fixed the same day

`9fe31bb`: UI20 lacked `pushId`, `popId`, `[Symbol.dispose]`, and UI6
the no-root case. `485b86b`: TX9 said the float formats pass bits
through (the `rgba16float` channel converts to `f16`), BF2 said
`patch` applies the three checks (it applies the range checks and
traps with `EG2`), PI9 and CL1 carried stale signatures, BF9 and PI15
said `device.queue()` where `GPUDevice` has an accessor. `72b4d8d`:
K4 scoped the `f16` rejection to a phase and the `h` factories are
host code, SC5 read as the whole class list, SC6 named an `identity`
factory that does not exist.

`generator-import.md` I7 said the driver writes six outputs. It
writes seven: `crates/facade/src/surface.rs` is the seventh (F23).
Fixed with this record.

Reported and kept as written: `render::reject_vertex_storage_writes`
emits `RN9`, and RN16 lists the rejection without an id. F22 states
that an excluded export is never emitted. The code emits and then
filters the header and the Rust source, and `emit_rust::render`
applies the exclusion to five chunk kinds. The observable result
matches F22, and the mechanism does not. The comment on
`emit_rust::render` states the actual scope. A pre-existing residue
comment in `plan.rs` (two schedule labels) is rewritten.

## Panic paths in library code, open

Core principle 6 forbids panics in library code. The writers listed
these sites for a later coding round: `crates/typegpu-gen/src/emit.rs`
(`expect` on an array stride), `kernel.rs` (`expect` on an atomic
receiver class name), `shell.rs` (`expect` on a character boundary),
`crates/harness/src/lib.rs` (`panic!` in `run_on_compiler_stack` and
in the pool, `expect` on the coverage array size, `assert_ne!` on
`MAP_FAILED`, unchecked indexing in `coverage_hit`),
`crates/window/src/main.rs` (two `expect` on the stored window), and
unchecked `module.classes[id.0]` indexing in `pipeline.rs` and
`render.rs`. `crates/webgpu-gen/src` holds 55 more (`plan.rs` 15,
`emit_rust.rs` 16, `patterns/descriptor.rs` 7, `emit_header.rs` 4,
`native_symbols.rs` 4, `patterns/handle_array.rs` 3,
`patterns/future_poll.rs` 2, `api.rs`, `model.rs`, `patterns/sync.rs`
1 each). Two of those reach a panic from the generator's own output
rather than from a proven invariant: `emit_rust::function_signatures`
on a declaration it cannot split, and `native_symbols::rust_signature`
on an export the emitted Rust lacks. Each site is reachable only
through an internal invariant, and none has a fixture. A round decides each: return an error, or
state the invariant in the comment.

## Evidence

`cargo fmt --all -- --check` clean. `tools/gate.sh --require-backend`
green, 276 passed, 1 ignored, 222.1 s wall. The first run was red at
`rule_ids::every_cited_rule_id_resolves`: a comment in
`crates/typegpu-gen/src/library.rs` cites `LB1`, and
`specs/blocks/rule-ids.txt` had no `LB` line although `library.md`
defines LB1 to LB4. The seven `LB` ids joined the file. Every golden
is byte-identical. The diff: 44 files, 1,519 comment lines added, 15
comment lines rewritten, no code line changed.

## The examples (2026-09-06)

Owner instruction: the examples are the tutorial, so comments inside
functions explain each step (EX2 Rev 3). Five Opus writers covered the
32 examples by group, with the TypeGPU v0.12.0 originals beside them
for the divergence sentences. Comment lines went from 384 to 2,550
over 15,068 file lines (the count after the pass). Every change is a
comment line: the code, the order, and the formatting are
byte-identical, and the W13 compile of every example is green.

Header corrections the writers reported and this pass made: the
`window-triangle` header took the `// example:` form, and thirteen
headers gained the reductions their bodies carry (`dispatch`,
`trippy-raymarching`, `disco`, `box-raytracing`, `game-of-life`,
`boids`, `clouds`, `stable-fluid`, `radiance-cascades`,
`radiance-cascades-drawing`, `caustics`, `matrix-next`, `confetti`,
`fluid-double-buffering`, `jump-flood-voronoi`). EX5 Rev 1 records the
`noop` state and several `check:` lines.

Defects the writers found in the code, recorded for a coding round:
`smoky-triangle`, `caustics`, `vaporrave`, and `ray-marching` set
their bind group to `null` in `shutdown()` without `dispose()`, where
the other windowed examples dispose it. The `init` failure path is
not affected: the group is created after the validation check, so
that path has no group to free (read, not the writer's claim). `fluid-double-buffering` integrates a velocity field that nothing
reads, so its density moves by diffusion only, where upstream advects
along the velocity. The header now states the port's behavior.

Evidence: `tools/gate.sh --require-backend` green, 283 passed, 1
ignored, 221.7 s wall. `tools/hygiene.sh` exit 0.

### The two defects, fixed (2026-09-06)

The commit above this record: the four examples dispose the bind
group in `shutdown()` before the buffers it references, and
`fluid-double-buffering` moves density along a per-cell velocity as
the TypeGPU original does. The velocity is the unit step toward the
open neighbor with the least cost (density plus 0.5 per row upward),
and each cell keeps what it does not send and receives the out-flow of
each neighbor whose velocity points at it. The diffusion is gone, and
the evaporation and the obstacle passes stay. The header and two step
comments moved with the code. Evidence: `tools/gate.sh
--require-backend` green, 283 passed, 1 ignored, 216.4 s wall.
`tools/window.sh --frames 30` on yawgpu Metal: `fluid-double-buffering`
and `smoky-triangle` print `window:frames=30` with no `FAIL`. The
visual result of the flow waits for the owner's run.

### `fluid-double-buffering`, the owner's run (2026-09-06)

The owner saw the fluid pool at the bottom and no flow. Cause: the
upstream gravity rule moves density toward lower `y`, and the port's
source sat in the bottom three rows, so the fluid had nowhere to fall.
The grid was also 32 cells against upstream's 256, so one cell filled
a large block of the window. The commit above moves the source to the
top three rows with a downward unit velocity, sets `GRID_SIZE` to 256,
and expresses the initial dense block relative to the grid. Five
comment lines moved with the code. Evidence: `tools/gate.sh
--require-backend` green, 283 passed, 1 ignored, 220.1 s wall.
`tools/window.sh --frames 30` prints `window:frames=30` with no `FAIL`.
The visual result waits for the owner's run.

The owner ran `fluid-double-buffering` on yawgpu Metal (963 frames)
and accepted the visual result: the fluid falls from the top source
and flows around the obstacle. The open item is closed.

## The Rust private items and T23 (2026-09-07)

Owner instruction: every public API carries a comment, and the Rust
sources carry more comments. `cargo clippy -W missing_docs` at the
start: 2 public items without a comment in `typegpu-gen` (two enum
variant fields), 3 in the harness's generated symbol table, 0 in
`webgpu-gen` and `facade`. Three Opus writers documented the private
and `pub(crate)` items and added `//` step comments in the long
functions, comment lines only:

| Crate | Documented items before | After |
|---|---|---|
| `typegpu-gen` | 99 of 247 functions | 293 of 304 items (11 self-describing) |
| `harness` (lib and main), `window`, `facade` hand-written | 19 of 89 functions | 77 of 89 |
| `webgpu-gen` | 116 of 343 items | 332 of 343 |

The commit `484eb20` holds 2,189 comment lines. T23 (`testing.md`)
makes `missing_docs` a gate: the four library crate roots deny it,
and the harness symbol-table emitter and the facade surface emitter
write a `///` line for every public item they emit, so the two
generated files carry comments through their generators. Landed in
the commit above this record. Evidence: `tools/gate.sh
--require-backend` green, 283 passed, 1 ignored, 217.5 s wall.

Spec drifts the writers found, fixed the same day: W9 named
`create_surface` as the only platform-conditional code (`1e1b98e`), W5
the failure message, PI1 the name literal and the zero axis, RN16 the
report rule (`aeb2a00`), H2 the u32 limit constant (`fefe929`).

Reported and open for a coding round: `IdlModel::from_definitions`
merges two full definitions of a dictionary that declares no parent
instead of an error, because the duplicate test reads the
inheritance flag. Recorded and kept as written: F22's exclusion is a
filter over five chunk kinds (`emit_rust::render` states the scope);
K19's cycle guard in `fold_global_constant` is unreachable by the
rule and stays as defence; `mapping::ident` mangles the `_g_` prefix
the emitter owns.
