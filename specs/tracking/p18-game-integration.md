# P18 — application modules and typed depth state

Plan §8 P18. Contracts: `library.md` LB5, `render.md` RN22.

## The report (2026-11-23)

A downstream game author reported two blocks at `3761363`. Claude
reproduced both at `3761363` on both tiers with
`cargo run --offline -p subscript-typegpu-harness -- <tier> <file>`.

1. An entry imports `./tile` from the same directory. Both tiers
   reject it: `error[S100]: imported module './tile' is not among
   the program's files`. The subscript CLI runs the same two files
   *(report, not run here)*. Cause: `prepare_program` in
   `crates/harness/src/lib.rs` builds the file set from the entry
   and the registered `lib/` modules only (LB1 Rev 2).
2. A `RenderPipelineSpec` literal with `depthStencil` fails both
   tiers: `error[S004]: descriptor class 'RenderPipelineSpec' has no
   declared property 'depthStencil'`. The API layer already carries
   `GPURenderPipelineDescriptor.depthStencil` and the pass depth
   attachment. `lib/typegpu.ts` does not.

## Slice 1 — application modules

Round 1 (Codex, 2026-11-23) stopped at the resolution check, as the
handoff required. Measured with `subscript_compiler::check_program`
at `31e14f1c`: an entry `sub/a.ts` that imports `./x` passes with
`sub/x.ts`, `x.ts`, or `unrelated/x.ts`, and fails with S100 with
only `sub/y.ts`. An entry `main.ts` that imports `./sub/a` fails with
S100 at `main.ts:1:24` while `sub/a.ts` is present. Claude read the
cause in subscript: the checker matches the specifier after `./`
against each file's stem (`compiler/src/check/signatures.rs`), and
the CLI loader (`cli/src/program_loader.rs`) follows only `./<name>`
with no `/`, in the importing file's directory. The modules form one
flat namespace. Gate during the round: `tools/gate.sh` green, 318 s,
358 passed, 1 ignored, 2 pending (backend).

Decision: LB5 Rev 1 follows the language. Application modules are
`./<name>` files in the entry's directory. The report's reproduction
uses that form, so the flat rule unblocks it. Test programs name
their modules `m<nn>-<slug>.ts` (`testing.md` T1 Rev 1).

Owner decision 2026-11-23: subscript gains subdirectory imports.
Change request R42 is written to the subscript repository as
`REPORT-import.md`: relative `./` and `../` specifiers against the
importer, the module key as the entry-relative path, and one public
resolver that the checker, the CLI, and this repository's loader
share. Claude ran the probes with the subscript CLI at `75f49b2a`:
`./sub/x`, `./sub/a`, and `../sub/x` are S100, and `./x` beside the
entry prints `v=7`. When R42 lands, LB5 Rev 2 adopts the resolver.

Round 2 (Codex, 2026-11-23) implemented LB5 Rev 1 with no deviation.
`load_program_files` in `crates/typegpu-gen/src/library.rs` is the one
loader. The harness `prepare_program` and the generator CLI call it.
`b28-app-modules` imports `m28-tile` and `m28-tile-size`. Its WGSL
golden `b28-app-modules.tile.wgsl` holds `tileSize` and `tileArea`. The
reds assert S100 at line 2 of `missing.ts` and `nested.ts`. A loader
test puts unreadable `typegpu-ui.ts` and `main.typegpu.ts` files beside
the entry and proves that rule 3 never reads them.

Claude review: the diff matches LB5 Rev 1. One MINOR stays open for the
phase review: the new private loader items carry no doc comments.

Evidence: `tools/gate.sh --require-backend` green, 590 s, 361 passed,
1 ignored (live). The report's reproduction prints `block-size=1` on
`dev` and `ship`. The previous row was 497 s at `31e14f1c`. The
differential module took 489 s against 367 s. The increase is under
the red line (2x) and its cause is not measured.

Window smoke (owner, Metal, yawgpu built with `--features metal`):
`tools/window.sh examples/triangle/main.ts --frames 30` and the same
for `ui-demo` exit 0 with no `FAIL` line.

## Slice 2 — depth state

Round 1 (Codex, 2026-11-23): `RenderPipelineSpec` gains
`depthStencil`, the three declaration functions copy it, and
`renderPipelineDescriptor` passes it. `b29-depth` prints the depth
format, write flag, and compare function on both tiers.

Claude review of round 1, two findings. MAJOR: `x27-live-depth`
passed a hand-written WGSL string and vertex layout to
`createRenderPipeline`, so the live lane did not run the declaration
path that RN22 changes. MINOR: `b29-depth.ts` used `===`, and scripts
write `==` and `!=` only. Round 2 fixed both: `x27` declares the
pipeline through `renderPipeline` with subscript kernels and uses the
generated support module, and its WGSL golden is committed.

`x27-live-depth`: triangle A at z 0.25 in red, triangle B at z 0.75 in
blue, 144 overlap pixel centers. The host expects red in the overlap
for both draw orders.

Evidence: `tools/gate.sh --require-backend` green, 657 s, 361 passed,
1 ignored (live). The report's depth reproduction prints `bgra8unorm`
on `dev` and `ship`. `tools/live.sh` on Metal (yawgpu with
`--features metal`): `every_x_program_passes_on_a_real_adapter` ok,
344 s. A direct `x27-live-depth` run prints `PASS` on both tiers.

The gate wall time grew from 590 s to 657 s. The differential module
took 537 s against 489 s, with one new program on each tier. The
increase is under the red line.
