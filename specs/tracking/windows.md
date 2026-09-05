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

## W5 follow-up: T19 gates the rule (2026-08-25)

RN14 Rev 1 forbade the tie one day before `x20-live-strip` used
`0.5`. A prose rule did not hold. T19 gates it now.

`crates/harness/tests/pixel_colors/mod.rs` reads the checked HIR of
every `x` program. It selects the pixel oracles, collects the color
literals, and applies the RN14 and RN21 predicates.

Demonstrated red, one scratch fixture per rule:

```
x20-live-strip.ts:61: literal 0.5 has product 127.5 with 255 and violates RN14
x22-live-blend.ts:180: literal 0.9 has product 229.5 with 255 and violates RN14
x22-live-blend.ts:180: literal 0.9 has product 229.5 with 255 and violates RN21
```

Both fixtures were restored. Evidence: `tools/gate.sh
--require-backend` green, 254 passed and 1 ignored, 106 s. The tree
gained one test function and no test executable.

## Re-check at 5b0a7f0 (2026-09-02)

The tree gained P12 to P16 after the last re-check. This run measures
the port again on windows-msvc.

Machine: Windows 11, `x86_64-pc-windows-msvc`, rustc 1.95.0, Git Bash.
Backend: a yawgpu Windows release build with the default Noop backend.

The first run was red. `cargo test --offline --workspace` gave 10
failures in two executables. Two port defects caused all 10. The
backend is not implicated. No test result on macOS changes.

After the fixes below, `tools/gate.sh --require-backend` exits 0 and
prints `gate: green` with zero pending. The run has 265 passed and 1
ignored in seven executables, and takes 196 s. This number is not a
reference-machine measurement.

The tree holds 266 `#[test]` attributes and one `#[ignore]`, the live
lane. The gate runs 265 and ignores the live lane, so windows-msvc
skips no test.

W1 to W5 hold.

### W6 The host runner link missed the native search paths

Two tests in `crates/typegpu-gen/tests/library/mod.rs` write a Rust
source file, compile it with a direct `rustc` call, and run the
result. The call passed `-L dependency=<deps>` alone.

Evidence: both tests printed
`LINK : fatal error LNK1181: cannot open input file 'windows.0.52.0.lib'`.

`windows.0.52.0.lib` and `windows.0.53.0.lib` come from the
`windows_x86_64_msvc` crates. Their build scripts emit
`cargo:rustc-link-search=native=<crate>/lib`. Cargo passes that path to
`rustc`. A direct `rustc` call receives nothing. Unix has no equivalent
library, so the tests passed there.

Fix: a helper reads `<target>/<profile>/build/*/output`, takes every
line with the prefix `cargo:rustc-link-search=native=`, and passes each
remainder to `rustc` as `-L native=<path>`. The helper ignores a
missing directory and an unreadable file, so no other platform changes.
The same helper now holds the runner source, the compile, and the run
for both tests.

### W7 The Windows main thread stack is 1 MB

The dev tier compiles a program in the calling process on Windows. W1
records the cause. The Unix main thread holds 8 MB and the Windows main
thread holds 1 MB, so only Windows overflows.

Evidence: eight `crates/harness` tests failed. Every one spawns
`subscript-typegpu-harness` and reads its output. The child printed one
line and exited with 127:

```
target/debug/subscript-typegpu-harness dev programs/a01-smoke.ts --coverage
thread 'main' has overflowed its stack
```

The `differential` message `dev lacks the coverage separator` is a
symptom of that crash. The child dies before it prints the separator.

The window binary carries the same defect:

```
target/debug/subscript-typegpu-window examples/window-triangle/main.ts --frames 1
thread 'main' has overflowed its stack
```

Measurement: a probe called `run_dev_with_coverage` on a thread of a
chosen size. 2 MiB compiles and runs `programs/a01-smoke.ts`,
`programs/b22-texture-array.ts`, `examples/stable-fluid/main.ts`, and
`examples/radiance-cascades-drawing/main.ts`. 1 MB overflows on every
one. The cause of the increase since `8a25831` is not measured.

Fix: `subscript_typegpu_harness::run_on_compiler_stack` runs a closure
on a thread with an 8 MiB stack, and resumes a panic in the caller. The
value is the Unix main-thread size, so both platforms compile with the
same stack. `crates/harness/src/main.rs` wraps `run`. Both runner
sources in `crates/typegpu-gen/tests/library/mod.rs` wrap their body.

`crates/window/src/main.rs` wraps `run` on Windows alone, and builds
the event loop with `EventLoopBuilderExtWindows::with_any_thread(true)`
there. macOS requires the event loop on the main thread, so no other
platform changes. The window binary then prints `window:surface lost`
in a shell with no GPU, and no stack overflow.

Review finding, fixed before the gate ran: the first fix moved a
`ReloadSession` from the compiler thread to the main thread under an
`unsafe impl Send`. `ReloadSession` holds `Box<Context>`,
`Vec<JITModule>`, and `Vec<*const u8>`, and its own comment states that
field order is load-bearing for the drop order of workers and JIT
modules. No measurement covered a move or a drop on another thread, and
the gate does not cover this binary. The accepted fix keeps the
compile, the event loop, and the drop on one thread.

## Reference-machine check of cdfe09a (2026-09-02)

Machine: Apple M2, macOS 26.6.2, rustc 1.95.0. Backend: the yawgpu
macOS release build with the default Noop backend.

`tools/gate.sh --require-backend` at `cdfe09a` exits 0 and prints
`gate: green` with zero pending. The run has 265 passed and 1 ignored in
seven executables, and takes 223 s. The run includes the rebuild after
the pull of `cdfe09a`.

`run_on_compiler_stack` now wraps the harness `run` on every platform.
The harness executable passes 36 and ignores 1, the same result as at
`5b0a7f0`. This check did not run `tools/live.sh`.

## Re-check at ada9e24 (2026-09-05)

The workspace pin moved three times after the last re-check: `e1c2be1`
to `d45c0c1`, then to `db3449d`, then to `587d6da`. This run measures
the port again on windows-msvc.

Machine: Windows 11, `x86_64-pc-windows-msvc`, rustc 1.95.0, Git Bash.
Backend: a yawgpu Windows release build with the default Noop backend.

`tools/gate.sh --require-backend` exits 0 and prints `gate: green` with
zero pending. The run has 265 passed and 1 ignored in seven executables,
and takes 239 s. This number is not a reference-machine measurement.

`tools/gate.sh` with no backend library prints `gate: green, pending 1`
and takes 210 s. The pending line names the backend library. The test
counts match the backend run.

The tree holds 266 `#[test]` attributes and one `#[ignore]`, the live
lane. The gate runs 265 and ignores the live lane, so windows-msvc
skips no test.

The port needed no source change. W1 to W7 hold. The working tree is
clean after both runs, so every golden is byte-identical across the
three pins.

The `d45c0c1` re-pin moved two fixture codes, and
`specs/tracking/language-request.md` records it. The `db3449d` and
`587d6da` re-pins carry subscript changes that this project did not
ask for. subscript's own records name them §83 and §84
(https://github.com/infosia/subscript).

## Re-check at f63fc4d (2026-09-05)

The tree gained the U0 to U4 phases and the ui module after the last
re-check. The workspace pin moved from `587d6da` to `3677d1f`. This
run measures the port again on windows-msvc.

Machine: Windows 11, `x86_64-pc-windows-msvc`, rustc 1.95.0, Git Bash.
Backend: a yawgpu Windows release build with the default Noop backend.

The run was red. `tools/gate.sh --require-backend` exits 1 after 233 s.
The harness executable has 37 passed, 4 failed, and 1 ignored. One port
defect causes all four failures. The backend is not implicated. No test
result on macOS changes. W1 to W7 hold.

Failed:

- `differential::every_program_matches_both_tiers_and_golden`
- `differential::every_program_is_deterministic_across_repeated_runs`
- `simulate::every_host_runnable_b_pipeline_prints_a_host_golden`
- `coverage::dev_corpus_matches_committed_facade_coverage`

### W8 MSVC rejects the atlas alpha literal

`lib/typegpu-ui-atlas.generated.ts` declared `UI_ATLAS_ALPHA_HEX` as
one string literal of 32,768 characters. The ship tier emits that
literal into one line of `program.c`, and `cl` rejects the line:

```
program.c(68721): error C2026
```

C2026 names a string that exceeds the MSVC length limit for one
literal. The compiler truncates the trailing characters. MSVC states
the limit as 16,380 single-byte characters *(docs)*.

Two programs import the atlas and run outside the live lane:
`programs/b23-ui-core.ts` and `programs/b24-ui-render.ts`. Both fail
the ship tier, and the four tests above each run one of them. Every
other program passes both tiers. `programs/x24-live-ui.ts` imports the
atlas and stays ignored, so it hides the same defect.

clang accepts the literal, so the branch closed green on the reference
machine.

The cause is in subscript's C emitter, which writes a long string
constant as one literal. Any subscript program with a literal of this
length fails the ship tier on windows-msvc. The owner decided a
downstream workaround now, and escalates the emitter limit to
subscript separately. UI2 Rev 1 holds the workaround: the generator
emits the alpha as an array of hex chunks.

### The W8 fix

`crates/typegpu-gen/src/ui_atlas.rs` emits `UI_ATLAS_ALPHA_CHUNK` and
`UI_ATLAS_ALPHA_HEX: string[]`, 8 chunks of 4,096 hex digits. The chunk
length is even, so no hex pair splits across two chunks.
`uiAtlasAlpha()` walks the chunks in order and decodes each pair.
`crates/typegpu-gen/tests/ui_atlas/mod.rs` parses the array and asserts
8 chunks, 4,096 digits for each chunk, and 32,768 digits in total.
`programs/b23-ui-core.ts` checks the chunk count, the chunk length, and
the total. `lib/typegpu-ui.ts` needed no change.

Evidence at the fix tree: `tools/gate.sh --require-backend` green with
zero pending, 276 passed and 1 ignored in seven executables, 210 s.
This number is not a reference-machine measurement. Every `.wgsl` and
`.expected` golden is byte-identical. The longest string literal in
`lib/typegpu-ui-atlas.generated.ts` is 4,096 characters.

`tools/window.sh examples/ui-demo/main.ts` runs on yawgpu Vulkan. The
30-frame smoke prints `window:frames=30`. An interactive run prints
`window:frames=3509` and exits 0.
