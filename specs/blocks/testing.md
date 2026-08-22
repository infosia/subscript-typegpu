# Block: testing (T-rules)

P0 contract. Rev 0, 2026-08-22. CLAUDE.md "Build time" and "Core
principles" govern this block.

## Programs

- **T1 — Naming.** API-layer gate programs are
  `programs/a<nn>-<slug>.ts`. TypeGPU gate programs are
  `programs/b<nn>-<slug>.ts`. Each has a golden
  `programs/<stem>.expected`. Live programs are
  `programs/x<nn>-<slug>.ts` with no golden. Ids are stable and never
  renumbered.
- **T2 — Determinism.** Programs never print backend-reported
  values. A printed value is a generated constant read by name, a
  host-computed result, or a plain marker.
- **T3 — Header.** Each program opens with four comment lines:
  `// program:`, `// purpose:`, `// exercises:`, `// questions:`.
- **T4 — Live output.** A live program prints one progress line per
  stage that changes observable state, then one final `PASS` or
  `FAIL <fact>` line. The host computes the expected result.

## Gates

- **T5 — Differential.** Every `a` and `b` program runs on both
  tiers, dev JIT and ship C AOT, on the backend that
  `SUBSCRIPT_TYPEGPU_BACKEND_LIB` names, in its Noop mode. Both raw
  byte outputs must equal the golden exactly.
- **T6 — Regeneration.** Every committed generated file has a byte-
  identity test: regenerate into a scratch directory, compare to the
  committed file. The failure message names `tools/regen.sh`.
- **T7 — Demonstrated red.** A guard or policy rule counts only
  after a recorded red run against a fixture that violates it.
- **T8 — The symbol table matches the exports.** A harness test
  builds the list of `#[no_mangle]` exports the facade links and
  compares it with `native_symbols.generated.rs`. A name in one and
  not the other fails with the name.
- **T9 — One executable per crate.** `crates/*/tests/main.rs` is the
  only integration-test file of a crate. A new test is a `mod`. A
  hygiene check counts `tests/*.rs` files and fails above one.
- **T10 — Loud pending.** If a prerequisite is absent (the backend
  library, libclang, a TypeGPU checkout), the test prints one
  `pending: <what> — <fix>` line and passes. The gate prints the
  pending lines at the end.
- **T11 — No features, no build scripts.** A hygiene check fails on
  any `[features]` table or `build.rs` in the workspace.

## Build time

- **T12 — The five measurements.** `tools/gate.sh --measure` records,
  on the reference machine with `CARGO_BUILD_JOBS=4`: the cold build
  (`cargo build --workspace --tests` after `cargo clean`), the warm
  no-op (`cargo test --workspace --no-run`), the warm full gate after
  a one-line change in `crates/typegpu-gen/src/lib.rs` (P0: in
  `crates/webgpu-gen/src/lib.rs`), the warm full gate after a
  program-only change, and the test-executable count. The numbers go
  to `specs/tracking/build-time.md` with the date, the commit, and
  the command.
- **T13 — A budget is a gate.** Plan §7 holds the budgets. A phase
  close quotes the five numbers. A number above budget is a red
  finding of the phase review.

## Lanes

- **T14 — The gate.** `tools/gate.sh` runs, in order: `cargo fmt
  --all -- --check`, `cargo clippy --workspace -- -D warnings`,
  `cargo test --workspace`, `tools/hygiene.sh`. It sets
  `CARGO_BUILD_JOBS=4` unless the environment sets it. It fails on the
  first failure and prints the pending lines at the end.
- **T15 — Live is not the gate.** `tools/live.sh` runs `x` programs
  on a real adapter through `ReloadSession` and ship AOT, never the
  forking JIT runner. Results go to `specs/tracking/` with the date
  and the commit. Phase exits need them. Test runs do not.
- **T16 — No sandboxed device runs.** Device programs never run from
  a sandboxed agent shell.

## Toolchain

- **T17 — Pin.** `rust-toolchain.toml` pins channel `1.95.0` with
  `rustfmt` and `clippy`. The tree is rustfmt-canonical under that
  pin.
