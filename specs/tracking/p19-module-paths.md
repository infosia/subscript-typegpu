# P19 — module paths

Plan §8 P19. Contract: `library.md` LB5 Rev 2, `testing.md` T1 Rev 2.

## Base

Subscript `ddc0f0e2` adds §354 "a module is named by its path", in
answer to change request R42 (`language-request.md`). The compiler
exports `module_key(name)` and
`resolve_module_specifier(importer, specifier)`. A relative specifier
resolves against the importer's key. A `..` segment with no segment
before it stays in the key. Two files with one key are rejected.
`CheckOptions::poison_missing_modules` entries are keys relative to
the root of the names.

Design choice: the root of the names is the entry's directory, and a
module outside it keeps a leading `../` in its name. The entry keeps
the name `<stem>.ts`, so the support module key stays
`<stem>.typegpu` for every program. The registered modules sit at the
root, so a file in a subdirectory reaches them through the path to
the root (LB5 Rev 2 rule 3).

## Slice 1

Round 1 (Codex, 2026-11-23). Step 1 measurement at `ddc0f0e2`:
`check_program` accepts `../shared/x.ts` imported from `main.ts`, and
`resolve_module_specifier("../shared/x.ts", "./y")` returns
`../shared/y`. LB5 rule 7 measurement: before the change, `b30` failed
with `error[S100]: K14: two program declarations share the source
name 'width' in one WGSL module (author)`. After the change, the WGSL
golden declares `width_m30_util` and `width_m30_inner_util`, and both
tiers print `base=3`, `inner=5`, `host:widths=3,5`.

## Phase review (2026-11-23)

A no-context Opus reviewer read the working tree: 0 CRITICAL,
4 MAJOR, 10 MINOR. Claude found one MAJOR before the review:
`tools/hygiene.sh` ran `cargo test` for the T1 reach test, which the
gate's workspace run already covers, and it bypassed the gate's
`--offline` wrapper.

- M1: Rev 2 kept the entry's directory as the root, and a module
  outside it had a name with a leading `../`. `module_key` keeps a
  leading `..`, so that module reached no registered module and no
  support module, and `../sub/main` from it loaded the entry a second
  time. LB5 Rev 2 rules 2 and 3 now move the root up as subscript
  §354 rule 8 does, and the library and the support module sit in the
  entry's directory.
- M2: an application module that imports the support module fails
  with SC1 when the entry imports it too. LB5 rule 8 now allows both,
  as one support import.
- M3: rule 7 had a compute test only. LB5 now states the alias form
  and the K14 cases, and asks for a render test and two K14 reds.
- M4: the evidence of the phase is missing. It is recorded at close.
- MINOR: a stale `visit` doc, an untested empty-key guard, missing
  docs on `ModuleNames`, a K14 message that names an alias as a source
  name, LB5 rule 7 and rule 3 wording (fixed in the spec), hygiene
  does not check names below `programs/m<nn>/`, the T1 reach test sits
  in the EG11 module, the two-spelling case (recorded as open in
  LB5), and an untested name without `.ts` in the generator CLI.

Round 2 (Codex, 2026-11-23) fixed every finding. Codex ran only
`cargo fmt --check`, `cargo clippy`, and targeted tests (owner
decision 2026-11-23: one full gate per phase, run by Claude).

- M1: the loader moves the root up as LB5 rule 2 states. The fixture
  entry `sub/main.ts` imports `../shared/x`, which imports
  `../sub/typegpu` and `../sub/main.typegpu`. The names are
  `sub/webgpu.ts`, `sub/typegpu-types.ts`, `sub/typegpu.ts`,
  `shared/x.ts`, `sub/main.ts`, and `sub/main.typegpu.ts`. Both tiers
  print `7`. A loader test proves that `../sub/main` from `shared/x.ts`
  does not load the entry a second time. The generator identifies a
  registered module and the support module by the rule-3 keys in the
  loader, the pipeline, render, schema, and shell recognition.
- M2: `nested.ts` imports `Edge_SIZE` and `sub/tile.ts` imports
  `Tile_SIZE` from one support module. Both tiers print `4,8`.
- M3: a fragment kernel calls two `gain` helpers, and the WGSL holds
  `gain_shared_left` and `gain_sub_right` and passes naga. K14 reds:
  `K14: two program declarations share the source name 'gain' in one
  WGSL module` for an entry helper and a module helper, and `K14:
  helper 'gain' in module 'shared/left' collides with a WGSL
  declaration from module 'sub/render'` for an alias collision.
- Hygiene: no `cargo` call. A path below `programs/m<nn>/` follows the
  name rule. A suite-structure test runs the script in a temporary
  repository and observes `invalid suite module` for
  `programs/m30/Bad/util.ts` and `programs/m30/inner/bad_name.ts`, and
  `generated support module exists` for
  `programs/m30/inner/main.typegpu.ts`. The T1 reach test moved to the
  `suite_structure` module.
- MINOR: the `visit` doc names keys, the empty-key guard stays with a
  test, `ModuleNames` is documented, and the generator CLI reports a
  usage error for an entry name without `.ts`.

Evidence (phase end): `tools/gate.sh --require-backend` green, 494 s,
380 passed, 1 ignored (live). No existing WGSL or `.expected` golden
moved: `git status` shows only the new `b30` goldens under
`programs/`.

Window smoke (owner, Metal): `tools/window.sh examples/triangle/main.ts
--frames 30` and the same for `ui-demo` exit 0 with no `FAIL` line.

P19 is COMPLETE. Open: two spellings of one file (LB5 open item).
