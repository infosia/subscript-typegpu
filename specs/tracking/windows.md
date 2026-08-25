# windows-msvc — the port

Target: `x86_64-pc-windows-msvc`, rustc 1.95.0. `tools/*.sh` run under
Git Bash. The backend library was a Windows build of
[yawgpu](https://github.com/infosia/yawgpu).

## Result

`tools/gate.sh --require-backend` exits 0 and prints `gate: green` with
zero pending. The tree is `93c1606` plus the changes below. The run has 234
tests in six executables and takes 106 s. This number is not a
reference-machine measurement. `specs/tracking/build-time.md` keeps the
reference-machine rows.

The dev tier, the ship tier, the WGSL goldens, the C layouts, the traps,
and the facade coverage all pass. The `x` live lane stays ignored.

## Defects the port found

### W1 The harness did not compile

`coverage_counts` in `crates/harness/src/lib.rs` called `libc::mmap` with
`MAP_SHARED | MAP_ANON`. The `libc` crate declares no such item on
windows-msvc. `rustc` reported six `E0425` errors.

The shared mapping exists because `subscript-codegen` runs the dev JIT in
a forked child on Unix. The counters must outlive that child. On non-Unix
the program runs in-process, so a shared mapping is not required. The
pinned revision states this in the `run_jit_with_native_libraries` doc
comment.

Fix: the `mmap` allocation stays under `cfg(unix)`. The non-Unix branch
leaks a zeroed `AtomicU64` slice. `libc` moved to
`[target.'cfg(unix)'.dependencies]`.

### W2 The facade did not load a backend DLL with a sibling dependency

The emitted loader called `libloading::Library::new`. That call reaches
`LoadLibraryExW` with no flags. This search order excludes the directory
of the named DLL, so a dependency beside the backend DLL stays unfound.

Evidence: the backend DLL sat beside its own shim DLL. The facade printed
`load ...: LoadLibraryExW failed` and every gate program printed
`FAIL adapter`. The same run with the backend directory on `PATH` printed
`PASS`.

Fix: `crates/webgpu-gen/src/emit_rust.rs` emits
`libloading::os::windows::Library::load_with_flags` with
`LOAD_WITH_ALTERED_SEARCH_PATH` on Windows. Other platforms keep
`libloading::Library::new`.

The flag has defined behaviour only for an absolute path. Therefore
`initialize_table` in `crates/facade/src/runtime.rs` makes the path
absolute with `std::path::absolute` before it loads. A relative
`SUBSCRIPT_TYPEGPU_BACKEND_LIB` value now loads and the program prints
`PASS`.

### W3 The C layout probe left an object file in the repository

`compile_probe` in `crates/harness/tests/c_layout/mod.rs` gave the
compiler no object directory. `cl` writes the object into the working
directory, so `cargo test` left `crates/harness/probe.obj` and
`tools/hygiene.sh` went red.

Fix: the probe calls `add_object_directory`, and the include argument
comes from `include_directory_arg`. Both come from the pinned
`subscript-codegen`.

### W4 A Windows clone can corrupt every golden

The repository had no `.gitattributes`. The default Git for Windows
install sets `core.autocrlf=true`. That setting rewrites every
`.expected`, `.wgsl`, and `.ts` file to CRLF on checkout, and the
byte-identical differential fails. The machine under test carried
`core.autocrlf=false` in its system config, so the defect stayed hidden
there.

Fix: `.gitattributes` holds one rule, `* text=auto eol=lf`.

## Notes

Every `cfg(unix)` branch keeps the previous code. One change reaches all
platforms: `initialize_table` makes the backend path absolute. On Unix this
changes only the path text inside a load-failure message.

`crates/webgpu-gen/tests/fixtures/mini.generated.rs.expected` follows the
emitter change, because the fixture snapshots the emitted loader.

## Reference-machine check

The port at `2d592b7` on the reference machine (Apple M2, macOS):
`tools/gate.sh --require-backend` green, 93 s, 234 passed, 1 ignored.
`tools/live.sh` on Metal (yawgpu): x01–x13 PASS, 28.51 s. A relative
`SUBSCRIPT_TYPEGPU_BACKEND_LIB` value with `SUBSCRIPT_TYPEGPU_BACKEND=metal`
runs `x01-live-vecadd` to `PASS` on the ship tier. The `cfg(unix)`
branches are unchanged in behaviour.

## Re-check at c4ff8a7 (2026-08-23)

The tree gained P8 slice 2 and P9 after the port. This run measures the
port again on windows-msvc.

Machine: Windows 11, `x86_64-pc-windows-msvc`, rustc 1.95.0, Git Bash.
Backend: a yawgpu Windows release build with the default Noop backend.

`cargo build --offline --workspace --tests` compiles every crate.
`crates/window` compiles with it. `cargo fmt --all -- --check` and
`cargo clippy --offline --workspace -- -D warnings` exit 0.

`tools/gate.sh --require-backend` exits 0 and prints `gate: green` with
zero pending. The run has 243 passed and 1 ignored in six executables,
and takes 188 s. This number is not a reference-machine measurement.

The tree holds 244 test functions. Every one runs here, so windows-msvc
skips no test. `specs/tracking/p9-window.md` records 246 passed at this
tree. That count needs a reference-machine re-check.

The port needed no new source change. W1 to W4 hold.

## Re-check at 8a25831 (2026-08-25)

The tree gained P10 and P11 after the last re-check. This run measures
the port again on windows-msvc. It adds a Vulkan live run on a real
adapter.

Machine: Windows 11, `x86_64-pc-windows-msvc`, rustc 1.95.0, Git Bash.
Backend: a yawgpu Windows release build. Adapter: NVIDIA RTX 5060 Ti.

`tools/gate.sh --require-backend` exits 0 and prints `gate: green` with
zero pending. The run has 253 passed and 1 ignored in six executables,
and takes 129 s. This number is not a reference-machine measurement.

The tree holds 254 test functions. The gate runs 253 and ignores the
live lane, so windows-msvc skips no test. The `examples/` programs
compile inside the gate, through
`window_example_compiles_through_the_host_loader_without_a_device`.

The port needed no new source change. W1 to W4 hold. The Vulkan live
run found two program defects, recorded as W5.

### W5 Two pixel oracles carried an unorm tie

`tools/live.sh` with `SUBSCRIPT_TYPEGPU_BACKEND=vulkan` printed two
failures. Both are oracle defects. The backend is not implicated.

`x20-live-strip` printed `FAIL x=3 y=3`. Its fragment color was `0.5`.
`0.5 * 255` is `127.5`. NVIDIA returns 127 and Apple returns 128. RN14
Rev 1 forbids this constant already. The program is newer than the rule.

`x22-live-blend` printed
`FAIL x=39 y=14 expected=15,46,138,153 got=15,46,137,153`. Its blue
source was `0.9`. `0.9 * 255` is `229.5`. NVIDIA converts the source to
229 before it blends, and returns 137. Apple and Dawn blend the float
value, and return 138.

`specs/tracking/p11-feature-gaps.md` records x01–x22 PASS at slice 3 and
slice 4. Those runs used Metal and Dawn on macOS. The claim did not hold
on NVIDIA.

Fix: `RN14` Rev 2 covers every color a blend consumes. `RN21` Rev 1
requires an exact source color and a converted destination between
draws. `x20-live-strip` uses `0.6`, expects 153, and prints the full
RN14 message. `x22-live-blend` uses source channels that are exact
multiples of 1/255, and converts the expected color after each draw.

Evidence: `tools/regen.sh` changes only
`programs/x20-live-strip.stripLive.wgsl`, from `0.5f` to `0.6f`.
`tools/live.sh` on Vulkan prints x01–x22 PASS: yawgpu 70.56 s, Dawn
64.49 s.

## Correction to the c4ff8a7 re-check (2026-08-25)

The c4ff8a7 re-check states "The tree holds 244 test functions" and
concludes that windows-msvc skips no test. The count is wrong, so the
conclusion is wrong.

Measurement: `c4ff8a7` holds 247 `#[test]` attributes under `crates/`
and one `#[ignore]`, the live lane. `specs/tracking/p9-window.md`
records 246 passed at that tree. 246 passed plus 1 ignored is 247, so
the reference-machine count is correct and needs no re-check.

The windows-msvc run had 243 passed plus 1 ignored, which is 244.
windows-msvc ran three fewer test functions than the tree holds.

The cause is not recorded. No test carries a `cfg(unix)`,
`cfg(windows)`, or `cfg(target_family)` attribute at `c4ff8a7` or
today. The only `cfg(unix)` in `crates/` guards the shared counter
mapping in `crates/harness/src/lib.rs`, which is library code. The run
log does not exist, so the three tests stay unnamed.

The gap is closed at the current tree. `8a25831` and `bea7da5` hold
254 `#[test]` attributes and one `#[ignore]`. windows-msvc ran 253
passed and 1 ignored at `8a25831`. The reference machine ran 253
passed and 1 ignored at `bea7da5`. Both platforms run every test
function.
