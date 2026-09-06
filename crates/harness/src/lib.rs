//! The headless development and ship-tier harness.

// T22: library code holds no panic site. An internal failure returns a `ProgramLoadError` or a
// `String` error. A test module allows the lints again.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable,
    clippy::todo,
    clippy::unimplemented,
    clippy::indexing_slicing
)]

/// The facade export table, plus one wrapper per export that counts its calls (T8).
#[path = "native_symbols.generated.rs"]
#[rustfmt::skip]
pub mod native_symbols_generated;

use std::collections::VecDeque;
use std::ffi::OsStr;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use subscript_codegen::{run_c_aot_with_native_libraries, NativeLibrary, RunError};
use subscript_compiler::SourceFile;

/// Entry argument accepted by a live development session.
pub use subscript_codegen::EntryArg;
/// In-process development session used by the window host.
pub use subscript_codegen::ReloadSession;
/// Native facade types and functions used by non-script hosts.
pub use subscript_typegpu_facade as native;

fn repository_root() -> PathBuf {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();
    root
}

/// The worker bound of every program loop (T18).
///
/// A ship-tier C compile and a JIT session are memory-bound on the reference machine. The bound
/// matches the build-job count, not the core count.
const PROGRAM_WORKER_COUNT: usize = 4;
/// Holds one program loop at a time, so two test modules never run 8 workers together (T18).
static PROGRAM_POOL_LOCK: Mutex<()> = Mutex::new(());

/// Runs `body` on a thread with the stack that the subscript compiler needs.
/// The dev tier compiles in the calling process on Windows, where the main
/// thread holds 1 MB. This function gives every platform the same stack.
/// Thread spawn and join failures return errors.
pub fn run_on_compiler_stack<T: Send + 'static>(
    body: impl FnOnce() -> T + Send + 'static,
) -> Result<T, String> {
    let thread = std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(body)
        .map_err(|error| format!("spawn compiler thread: {error}"))?;
    thread
        .join()
        .map_err(|payload| format!("join compiler thread: {}", panic_message(payload)))
}

/// Returns the text of a panic payload, or a fixed line when the payload carries no string.
fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        return (*message).to_owned();
    }
    "program worker panicked without a string message".to_owned()
}

/// Runs per-program work on the shared four-worker pool.
/// Returns results or worker failures in program-path order.
#[doc(hidden)]
pub fn run_program_pool<R, F>(
    mut programs: Vec<PathBuf>,
    task: F,
) -> Result<Vec<(PathBuf, R)>, String>
where
    R: Send,
    F: Fn(&Path) -> R + Sync,
{
    let _pool = PROGRAM_POOL_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    programs.sort();
    let worker_count = PROGRAM_WORKER_COUNT.min(programs.len());
    let queue = Mutex::new(VecDeque::from(programs));
    let outcomes = Mutex::new(Vec::<(PathBuf, Result<R, String>)>::new());
    std::thread::scope(|scope| {
        for _ in 0..worker_count {
            scope.spawn(|| loop {
                let program = queue
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .pop_front();
                let Some(program) = program else { break };
                // A panic in one program becomes that program's failure. The other workers
                // finish their queue and the caller reports every failure at once.
                let outcome =
                    catch_unwind(AssertUnwindSafe(|| task(&program))).map_err(panic_message);
                outcomes
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .push((program, outcome));
            });
        }
    });
    let mut outcomes = outcomes
        .into_inner()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    // T18: the report follows program order, whatever order the workers finished in.
    outcomes.sort_by(|left, right| left.0.cmp(&right.0));
    let failures = outcomes
        .iter()
        .filter_map(|(program, outcome)| {
            outcome.as_ref().err().map(|error| {
                let name = program
                    .file_name()
                    .and_then(OsStr::to_str)
                    .unwrap_or("<program>");
                format!("{name}: {error}")
            })
        })
        .collect::<Vec<_>>();
    if !failures.is_empty() {
        return Err(failures.join("\n"));
    }
    outcomes
        .into_iter()
        .map(|(program, outcome)| {
            let value = outcome
                .map_err(|error| format!("program pool omitted the reported failure: {error}"))?;
            Ok((program, value))
        })
        .collect()
}

/// A failure to prepare or compile a development program.
#[derive(Debug)]
pub struct ProgramLoadError {
    diagnostics: Option<String>,
    summary: String,
}

impl ProgramLoadError {
    /// Builds an error that carries a summary alone.
    fn message(message: impl Into<String>) -> Self {
        Self {
            diagnostics: None,
            summary: message.into(),
        }
    }

    /// Builds an error that carries the rendered compiler diagnostics of a rejected program.
    ///
    /// `files` must hold every file the diagnostics point into, because the renderer resolves a
    /// position through that list.
    fn rejected(files: &[SourceFile], diagnostics: Vec<subscript_compiler::Diagnostic>) -> Self {
        Self {
            summary: format!("compile: rejected with {} diagnostic(s)", diagnostics.len()),
            diagnostics: Some(subscript_compiler::render_diagnostics(files, &diagnostics)),
        }
    }

    /// Returns compiler diagnostics for a rejected program.
    #[must_use]
    pub fn diagnostics(&self) -> Option<&str> {
        self.diagnostics.as_deref()
    }

    /// Returns the one-line failure summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        &self.summary
    }
}

impl std::fmt::Display for ProgramLoadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(diagnostics) = &self.diagnostics {
            writeln!(formatter, "{diagnostics}")?;
        }
        formatter.write_str(&self.summary)
    }
}

impl std::error::Error for ProgramLoadError {}

/// Builds a cargo command that runs in the repository root with a bounded job count.
///
/// The nested build inherits `CARGO` from the outer run, so it runs the same binary. It also
/// inherits `CARGO_BUILD_JOBS`, so the two builds never oversubscribe the machine.
fn cargo_command() -> Command {
    let mut command = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    command.current_dir(repository_root()).env(
        "CARGO_BUILD_JOBS",
        std::env::var_os("CARGO_BUILD_JOBS").unwrap_or_else(|| "4".into()),
    );
    command
}

/// Runs one nested cargo command and returns its output.
///
/// `action` names the step in both error forms. A non-zero exit returns an error that carries
/// cargo's stderr, because the caller reads the JSON messages on stdout.
fn run_nested_cargo(command: &mut Command, action: &str) -> Result<Output, String> {
    let output = command
        .output()
        .map_err(|error| format!("run nested cargo for {action}: {error}"))?;
    if output.status.success() {
        return Ok(output);
    }
    Err(format!(
        "nested cargo failed for {action}:\n{}",
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn staticlib_file_name(package: &str) -> String {
    let crate_name = package.replace('-', "_");
    if cfg!(target_env = "msvc") {
        format!("{crate_name}.lib")
    } else {
        format!("lib{crate_name}.a")
    }
}

/// Finds the static library that a nested cargo build produced for `package`.
///
/// The harness carries no JSON dependency, so the search splits the `compiler-artifact` lines on
/// the quote character. It returns an error when no line names the expected file.
fn artifact_path(output: &Output, package: &str) -> Result<PathBuf, String> {
    let wanted = staticlib_file_name(package);
    let crate_name = package.replace('-', "_");
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if !line.contains("\"compiler-artifact\"") || !line.contains(&crate_name) {
            continue;
        }
        for piece in line.split('"') {
            // A JSON string doubles the Windows path separator. The compare needs the raw form.
            let path = PathBuf::from(piece.replace("\\\\", "\\"));
            if path.file_name() == Some(OsStr::new(&wanted)) {
                return Ok(path);
            }
        }
    }
    Err(format!(
        "nested cargo produced no {wanted} artifact for {package}"
    ))
}

/// Builds one package as a release static library and returns the archive path.
///
/// The build writes into `target/ship-build`, which the cold-build measurement counts as its own
/// step (T12).
fn build_staticlib(package: &str) -> Result<PathBuf, String> {
    let target = repository_root().join("target/ship-build");
    let mut command = cargo_command();
    command.args([
        "build",
        "--offline",
        "--release",
        "-p",
        package,
        "--message-format=json",
        "--target-dir",
    ]);
    command.arg(target);
    let output = run_nested_cargo(&mut command, &format!("build {package}"))?;
    artifact_path(&output, package)
}

/// Builds the facade archive and returns it with the platform libraries that it needs.
///
/// The facade links `libloading` and the platform dynamic loader (L5), which the C link of a
/// ship-tier program must repeat. `cargo rustc --print native-static-libs` names them.
fn facade_link_inputs() -> Result<Vec<String>, String> {
    let target = repository_root().join("target/ship-build");
    let mut command = cargo_command();
    command.args([
        "rustc",
        "--offline",
        "--release",
        "-p",
        "subscript-typegpu-facade",
        "--message-format=json",
        "--target-dir",
    ]);
    command
        .arg(target)
        .args(["--", "--print", "native-static-libs"]);
    let output = run_nested_cargo(
        &mut command,
        "build facade and query native static libraries",
    )?;
    let facade = artifact_path(&output, "subscript-typegpu-facade")?;
    for line in String::from_utf8_lossy(&output.stderr)
        .lines()
        .chain(String::from_utf8_lossy(&output.stdout).lines())
    {
        if let Some((_, libraries)) = line.split_once("native-static-libs:") {
            // A JSON message holds the whole note on one line. The list ends at the first
            // escaped newline.
            let libraries = libraries.split("\\n").next().unwrap_or(libraries);
            let mut inputs = vec![facade.to_string_lossy().into_owned()];
            inputs.extend(libraries.split_whitespace().map(str::to_owned));
            return Ok(inputs);
        }
    }
    Err("cargo rustc reported no native static libraries for the facade".to_owned())
}

/// Builds the ship-tier runtime archive in the dedicated target directory.
#[doc(hidden)]
pub fn ensure_runtime_staticlib() -> Result<PathBuf, String> {
    static RUNTIME: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    RUNTIME
        .get_or_init(|| build_staticlib("subscript-runtime"))
        .clone()
}

/// Returns the directory that holds the generated `subscript-typegpu.h`.
///
/// Both tiers compile against that header, so the dev-tier session and the ship-tier C build
/// receive the same include path.
fn include_directory() -> PathBuf {
    repository_root().join("crates/facade")
}

/// The address and element count of the facade coverage counter array.
#[derive(Clone, Copy)]
struct CoverageMemory {
    address: usize,
    len: usize,
}

/// The one coverage allocation of the process. A failure stays recorded and repeats its message.
static COVERAGE_MEMORY: OnceLock<Result<CoverageMemory, String>> = OnceLock::new();

/// Returns the per-export counter array, and allocates it on the first call.
///
/// On Unix the array lives in one anonymous shared mapping, because the dev-tier runner forks
/// and the child must count into the parent's array. Other platforms leak one boxed slice.
fn coverage_counts() -> Result<&'static [AtomicU64], String> {
    let memory = COVERAGE_MEMORY
        .get_or_init(|| {
            let len = native_symbols_generated::facade_export_names().len();
            #[cfg(unix)]
            let address = {
                let byte_len = len
                    .checked_mul(std::mem::size_of::<AtomicU64>())
                    .ok_or_else(|| "facade coverage array size".to_owned())?;
                // SAFETY: The anonymous shared mapping remains live for the process lifetime.
                // Its zeroed storage is valid for `AtomicU64`.
                let address = unsafe {
                    libc::mmap(
                        std::ptr::null_mut(),
                        byte_len,
                        libc::PROT_READ | libc::PROT_WRITE,
                        libc::MAP_SHARED | libc::MAP_ANON,
                        -1,
                        0,
                    )
                };
                if address == libc::MAP_FAILED {
                    return Err("allocate facade coverage counters".to_owned());
                }
                address as usize
            };
            #[cfg(not(unix))]
            let address = {
                let counters = (0..len)
                    .map(|_| AtomicU64::new(0))
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                // SAFETY: The leaked allocation remains live for the process lifetime.
                // Each counter starts at zero with valid `AtomicU64` storage.
                Box::leak(counters).as_ptr() as usize
            };
            Ok(CoverageMemory { address, len })
        })
        .as_ref()
        .map_err(Clone::clone)?;
    // SAFETY: `coverage_counts` creates this process-lifetime storage at exactly
    // `len * size_of::<AtomicU64>()` bytes and never changes its address or length.
    Ok(unsafe { std::slice::from_raw_parts(memory.address as *const AtomicU64, memory.len) })
}

/// Counts one call of the facade export at `index` in `facade_export_names()`.
///
/// The generated native symbol wrappers call this. The counters live in one shared anonymous
/// mapping, so a forked dev-tier child records into the parent's array.
///
/// Ignores an index outside the export table or unavailable counters.
pub(crate) fn coverage_hit(index: usize) {
    if let Ok(counters) = coverage_counts() {
        if let Some(counter) = counters.get(index) {
            counter.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn coverage_reset() -> Result<(), String> {
    for counter in coverage_counts()? {
        counter.store(0, Ordering::Relaxed);
    }
    Ok(())
}

/// Returns the names of the facade exports whose counter is not zero.
fn coverage_reached() -> Result<Vec<String>, String> {
    Ok(native_symbols_generated::facade_export_names()
        .iter()
        .zip(coverage_counts()?)
        .filter(|(_, counter)| counter.load(Ordering::Relaxed) != 0)
        .map(|(name, _)| (*name).to_owned())
        .collect())
}

/// Returns the facade library for the ship tier, with the archive and its platform libraries.
///
/// The link inputs are empty on the dev tier, because the JIT calls the exports that this
/// process already links.
fn ship_facade_library() -> Result<NativeLibrary, String> {
    let inputs = ship_link_inputs()?.into_iter().map(PathBuf::from).collect();
    // SAFETY: the generated table contains static facade exports with
    // the signatures declared by the committed ambient mirror.
    Ok(unsafe {
        NativeLibrary::new(
            vec![include_directory()],
            inputs,
            native_symbols_generated::facade_symbols(),
        )
    })
}

/// Loads the `lib/` modules that `program` reaches through its imports.
///
/// A read failure becomes a summary error. A parse failure becomes a rejection that carries the
/// library file's own diagnostics.
fn library_files(program: &SourceFile) -> Result<Vec<SourceFile>, ProgramLoadError> {
    subscript_typegpu_gen::load_library_files(&repository_root().join("lib"), program).map_err(
        |error| match error {
            subscript_typegpu_gen::LibraryLoadError::Read(message) => {
                ProgramLoadError::message(message)
            }
            subscript_typegpu_gen::LibraryLoadError::Parse { file, diagnostics } => {
                ProgramLoadError::rejected(&[file], diagnostics)
            }
        },
    )
}

/// Builds the source set of one program: the library modules, the program, and its support module.
///
/// The support module `./<stem>.typegpu` never exists on disk, so the first check marks that one
/// import poisoned instead of missing. The generator then produces the module in memory.
///
/// Returns a rejection that carries the rendered diagnostics when a check or the generation
/// fails.
fn prepare_program(program: &Path) -> Result<Vec<SourceFile>, ProgramLoadError> {
    let stem = program.file_stem().and_then(OsStr::to_str).ok_or_else(|| {
        ProgramLoadError::message(format!("program has no UTF-8 stem: {}", program.display()))
    })?;
    let program_source = std::fs::read_to_string(program).map_err(|error| {
        ProgramLoadError::message(format!("read {}: {error}", program.display()))
    })?;
    let program_file = SourceFile::new(format!("{stem}.ts"), program_source);
    let mut files = library_files(&program_file)?;
    files.push(program_file);
    let support_module = format!("./{stem}.typegpu");
    let mut options = subscript_compiler::CheckOptions::default();
    options.poison_missing_modules = vec![support_module.clone()];
    let discovery = subscript_compiler::check_program_with(&files, &options)
        .map_err(|diagnostics| ProgramLoadError::rejected(&files, diagnostics))?;
    // A program with no kernel imports no support module, so the generator stays unused.
    if discovery
        .poisoned_imports
        .iter()
        .any(|import| import.module == support_module)
    {
        let generated = subscript_typegpu_gen::generate(&files)
            .map_err(|diagnostics| ProgramLoadError::rejected(&files, diagnostics))?;
        files.push(SourceFile::new(
            format!("{stem}.typegpu.ts"),
            generated.support_module,
        ));
    }
    Ok(files)
}

/// Loads the core sources, import-reachable library modules, the program, and its generated support module.
pub fn program_files(program: &Path) -> Result<Vec<SourceFile>, String> {
    prepare_program(program).map_err(|error| error.to_string())
}

/// Compiles one program into a development session against `library`.
fn load_program_with_library(
    program: &Path,
    library: NativeLibrary,
) -> Result<ReloadSession, ProgramLoadError> {
    load_program_with_exports_and_library(program, library).map(|(session, _)| session)
}

/// Compiles one program and returns the session with the exported function names of the program
/// file.
///
/// The names come from the checked module, so no text scan guesses them. A library export never
/// appears, because the filter keeps the entry file alone.
fn load_program_with_exports_and_library(
    program: &Path,
    library: NativeLibrary,
) -> Result<(ReloadSession, Vec<String>), ProgramLoadError> {
    let files = prepare_program(program)?;
    let module = subscript_compiler::check_program(&files)
        .map_err(|diagnostics| ProgramLoadError::rejected(&files, diagnostics))?;
    let entry_file = program.file_name().unwrap_or_default().to_string_lossy();
    let entry_count = files.iter().filter(|file| file.name == entry_file).count();
    // The export filter below needs exactly one file under the entry name.
    if entry_count != 1 {
        return Err(ProgramLoadError::message(format!(
            "exactly one loaded file must match program {entry_file}, found {entry_count}"
        )));
    }
    let exports = module
        .functions
        .iter()
        .filter(|function| function.exported && function.pos.file == entry_file)
        .map(|function| function.name.clone())
        .collect();
    match ReloadSession::new_with_native_libraries(&files, &[library]) {
        Ok(session) => Ok((session, exports)),
        Err(RunError::Rejected(diagnostics)) => {
            Err(ProgramLoadError::rejected(&files, diagnostics))
        }
        Err(error) => Err(ProgramLoadError::message(format!("compile: {error}"))),
    }
}

/// Generates, checks, and compiles one program into a development session.
pub fn load_program(program: &Path) -> Result<ReloadSession, ProgramLoadError> {
    load_program_with_library(program, facade_library())
}

/// Loads a development session and returns its checked entry function names.
pub fn load_program_with_exports(
    program: &Path,
) -> Result<(ReloadSession, Vec<String>), ProgramLoadError> {
    load_program_with_exports_and_library(program, facade_library())
}

/// Calls `main`, steps the session until no async work is pending, and returns the raw output.
///
/// The API layer pumps the facade inside the program's own wait, so this loop only advances the
/// session.
fn run_session(mut session: ReloadSession) -> Result<Vec<u8>, String> {
    session
        .call_export("main")
        .map_err(|error| error.to_string())?;
    while session.async_pending() != 0 {
        session.async_step().map_err(|error| error.to_string())?;
    }
    Ok(session.take_output())
}

/// Returns the facade symbols and include directory for the dev tier.
pub fn facade_library() -> NativeLibrary {
    // SAFETY: the generated table contains static facade exports with
    // the signatures declared by the committed ambient mirror.
    unsafe {
        NativeLibrary::new(
            vec![include_directory()],
            Vec::new(),
            native_symbols_generated::facade_symbols(),
        )
    }
}

/// Returns the facade library whose wrappers count each export call for the coverage run.
fn facade_counting_library() -> NativeLibrary {
    // SAFETY: the generated table contains ABI-preserving wrappers for the same
    // static facade exports declared by the committed ambient mirror.
    unsafe {
        NativeLibrary::new(
            vec![include_directory()],
            Vec::new(),
            native_symbols_generated::facade_counting_symbols(),
        )
    }
}

/// Builds and returns the facade archive and its platform libraries.
pub fn ship_link_inputs() -> Result<Vec<String>, String> {
    static INPUTS: OnceLock<Result<Vec<String>, String>> = OnceLock::new();
    INPUTS.get_or_init(facade_link_inputs).clone()
}

/// Runs one program through the development JIT.
pub fn run_dev(program: &Path) -> Result<Vec<u8>, String> {
    run_session(load_program(program).map_err(|error| error.to_string())?)
}

/// Runs one program through the development JIT and returns the facade exports reached.
pub fn run_dev_with_coverage(program: &Path) -> Result<(Vec<u8>, Vec<String>), String> {
    coverage_reset()?;
    let session = load_program_with_library(program, facade_counting_library())
        .map_err(|error| error.to_string())?;
    let bytes = run_session(session)?;
    Ok((bytes, coverage_reached()?))
}

/// Separates a dev program's stderr from its facade coverage report.
pub const COVERAGE_SEPARATOR: &str = "--- subscript-typegpu coverage ---";

/// Runs one program through the emitted-C ship tier.
pub fn run_ship(program: &Path) -> Result<Vec<u8>, String> {
    run_c_aot_with_native_libraries(&program_files(program)?, &[ship_facade_library()?])
        .map_err(|error| error.to_string())
}

/// Returns the existing backend shared library named by the process environment.
pub fn backend_lib() -> Result<Option<PathBuf>, String> {
    let Some(value) =
        std::env::var_os("SUBSCRIPT_TYPEGPU_BACKEND_LIB").filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    let path = PathBuf::from(value);
    if path.is_file() {
        Ok(Some(path))
    } else {
        Err(format!(
            "SUBSCRIPT_TYPEGPU_BACKEND_LIB points at {}, which is not a file",
            path.display()
        ))
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented,
        clippy::indexing_slicing
    )]

    use super::*;

    #[test]
    fn coverage_ignores_out_of_range_indices_and_counts_valid_calls() {
        coverage_reset().expect("reset coverage");
        let counters = coverage_counts().expect("coverage counters");
        assert!(!counters.is_empty());
        coverage_hit(counters.len());
        coverage_hit(usize::MAX);
        assert!(coverage_reached().expect("coverage report").is_empty());
        coverage_hit(0);
        coverage_hit(0);
        coverage_hit(counters.len() - 1);
        assert_eq!(counters[0].load(Ordering::Relaxed), 2);
        assert_eq!(counters[counters.len() - 1].load(Ordering::Relaxed), 1);
        let names = native_symbols_generated::facade_export_names();
        assert_eq!(
            coverage_reached().expect("coverage report"),
            vec![names[0], names[names.len() - 1]]
        );
        coverage_reset().expect("reset coverage");
        assert!(coverage_reached().expect("coverage report").is_empty());
    }
}
