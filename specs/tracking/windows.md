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
