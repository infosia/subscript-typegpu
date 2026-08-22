# Block: generator import (I-rules)

P0 contract. Rev 0, 2026-08-22. Plan §3 D11 and §5 name the decision.
This block states what enters, what does not, and the acceptance
gate of each reshape item. Other documents cite these rules by id.

## Source

The proof of concept is
<https://github.com/infosia/subscript-gpu>, directory `gpu/codegen`
(crate `subscript-gpu-codegen`) and `gpu/facade/src/runtime.rs`.
The import carries no git history. The handoff names the local
checkout the coding agent reads. No committed file names that
checkout.

## Inventory

- **I1 — What enters.** `src/model.rs`, `src/policy.rs`,
  `src/naming.rs`, `src/plan.rs`, `src/emit_header.rs`,
  `src/emit_rust.rs`, `src/patterns/` (17 files), `src/idl.rs`,
  `src/api_model.rs`, `src/api.rs`, `src/native_symbols.rs`,
  `src/lib.rs`, `policy.toml`, `tests/fixtures/`, `tests/support/`,
  and these test files: `api_e5_red`, `api_e6_red`, `api_e7_red`,
  `api_e8_red`, `api_host_ownership_red`, `api_join_red`,
  `api_policy_red`, `api_r19_red`, `area2_patterns` through
  `area7_patterns`, `fixtures`, `instance_descriptor_red`,
  `native_symbols`, `policy_red`, `regen`, `typed_pair_red`. The
  facade's `runtime.rs` enters into `crates/facade/src/`.
- **I2 — What does not enter.** `src/engine_header.rs`,
  `src/engine_facade_abi.rs`, `src/measure_backend.rs`, the
  `bind_engine_mirror` function and its `set_current_dir`, the
  `surface` exclusion row of the policy, `gpu/facade/src/surface_host.rs`,
  and the tests `engine_facade_abi_red`, `engine_header_red`,
  `measure_backend_red`, `surface_host_only`. Every `pub use` of a
  dropped item leaves `lib.rs`.
- **I3 — The crate is `subscript-typegpu-webgpu-gen`** at
  `crates/webgpu-gen/`, binary `subscript-typegpu-webgpu-gen`.
  Dependencies: `serde`, `serde_yaml`, `toml`, `weedle2`,
  `subscript-bindgen`. No others. No dev-dependencies.

## Reshape items

Each item is one commit. Each commit's message names the item and
quotes its gate result.

- **I4 — Names.** `naming.rs` derives `subscript_typegpu_` +
  snake_case for exports (`subscript_typegpu_device_create_buffer`),
  `SubscriptTypegpu` + Pascal for types
  (`SubscriptTypegpuBufferDescriptor`), and
  `SUBSCRIPT_TYPEGPU_` + upper snake for constants
  (`SUBSCRIPT_TYPEGPU_BUFFER_USAGE_STORAGE`). The header is
  `subscript-typegpu.h`. The mirror is
  `subscript-typegpu.generated.d.ts`. The bindgen pragma comments keep
  their spelling, because `subscript bind` reads them. Gate: a grep
  for the old four-letter prefix, in both cases, over every
  regenerated output and every file under `crates/webgpu-gen/` finds
  nothing. `tools/hygiene.sh` carries the pattern.
- **I5 — The function table.** `emit_rust.rs` emits, in place of
  the `extern "C"` block, one `struct WebgpuTable` with one
  `unsafe extern "C" fn` pointer field per webgpu.h function the plan
  uses, one `fn load(path: &std::path::Path) -> Result<WebgpuTable,
  String>` that resolves every field by name with `libloading`, and
  one module-level shim `unsafe fn wgpuX(...) -> ... { (table().wgpuX)(...) }`
  per function. Call sites in the pattern modules do not change. The
  three hard-coded call names (`wgpu{Pascal}Release` in `handles.rs`
  and `future_poll.rs`, `wgpuAdapterInfoFreeMembers` in
  `adapter_limits.rs`) are verified present in the table by a test.
  `runtime.rs` owns `table()`: a `OnceLock<WebgpuTable>` filled by
  `subscript_typegpu_create_instance` from
  `SUBSCRIPT_TYPEGPU_BACKEND_LIB` (facade.md L1 through L4). Gate:
  `cargo tree -p subscript-typegpu-facade` lists `libloading` and
  nothing else. The facade crate has no `[features]` and no
  `build.rs`.
- **I6 — The symbol table from the plan.** `native_symbols.rs`
  takes the plan's export list and emits
  `crates/harness/src/native_symbols.generated.rs`. It does not parse
  Rust source. `syn` leaves `Cargo.toml`. `facade_native_symbols()`
  and the `include_str!` of a sibling directory leave. Gate: `syn` is
  not a direct dependency of any workspace crate (`cargo tree -e
  normal --depth 1`). A transitive `syn` through `serde_derive` is
  accepted. A harness test compares the table's names with the
  facade's exports (testing.md T8).
- **I7 — The driver.** `main.rs` takes one argument, the
  repository root, and reads `third_party/webgpu-headers/webgpu.yml`,
  `crates/webgpu-gen/policy.toml`, and the two gpuweb `.bs` files. It
  writes exactly: `crates/facade/subscript-typegpu.h`,
  `crates/facade/src/generated.rs`,
  `lib/subscript-typegpu.generated.d.ts`,
  `lib/wire-enum-aliases.generated.d.ts`, `lib/webgpu.ts`,
  `crates/harness/src/native_symbols.generated.rs`. `tools/regen.sh`
  runs it. Gate: the regen test regenerates into a scratch directory
  and compares bytes with every committed output (testing.md T6).
- **I8 — One test executable.** `tests/main.rs` declares one
  `mod` per former file. The pin canaries (the IDL block count, the
  namespace-constant table, the weedle2 definition count, the API
  accounting tuple, the absence-enum list) move into
  `tests/pins/mod.rs` with a comment that names the re-pin
  procedure. Former test files live under `tests/cases/`. Gate: `cargo test
  -p subscript-typegpu-webgpu-gen` builds one executable. The test
  count before and after is recorded in the tracking entry and is
  equal.
- **I9 — Prose.** Every comment and doc string that names the
  proof of concept, its crates, its directories, or its phases is
  rewritten to this repository's names, or cites the upstream URL.
  A rule id cited anywhere in the crate, the policy file, or a
  generated output resolves in this repository's `specs/blocks/`:
  F-, S-, A-, B-, C-, PL-, E-, G-, H-rules in
  `facade-generator.md`, J-rules in `api-layer.md`, T-rules in
  `testing.md`, Q- and R-ids as subscript's. A phase or plan
  reference of the proof of concept becomes prose. The module docs
  follow CLAUDE.md's writing rules. Gate: `tools/hygiene.sh` is
  clean, and a test lists every cited id and checks it against the
  id table in `specs/blocks/rule-ids.txt`.
- **I10 — The two-pass question.** The coding agent measures
  whether `api.cenum_aliases` can be computed from `policy.toml` and
  `webgpu.yml` without the base mirror. If it can, the driver
  generates the facade once, `generate_with_cenum_aliases` leaves,
  and `Error::CEnum` leaves with it. If it cannot, the two-pass stays
  and the tracking entry states which input the base mirror supplies.
  Gate: every output is byte-identical before and after the change.

## Policy at P0

- **I11 — The subset.** `policy.toml` at P0 names the proof of
  concept's subset minus the surface family. Every later area adds
  rows in its own phase. The two-way validation keeps its five error
  classes (unknown, dead, duplicate, unpoliced, invalid).
- **I12 — No silent widening.** A policy row exists because a program
  in `programs/` exercises it, or because the proof of concept's
  suite exercised it and the row is marked `carried = true` until a
  program here exercises it. P6 closes every `carried` row.
