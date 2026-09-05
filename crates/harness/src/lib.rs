//! The headless development and ship-tier harness.

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

const PROGRAM_WORKER_COUNT: usize = 4;
static PROGRAM_POOL_LOCK: Mutex<()> = Mutex::new(());

/// Runs `body` on a thread with the stack that the subscript compiler needs.
/// The dev tier compiles in the calling process on Windows, where the main
/// thread holds 1 MB. This function gives every platform the same stack.
pub fn run_on_compiler_stack<T: Send + 'static>(body: impl FnOnce() -> T + Send + 'static) -> T {
    let thread = std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(body)
        .unwrap_or_else(|error| panic!("spawn compiler thread: {error}"));
    match thread.join() {
        Ok(value) => value,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

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
#[doc(hidden)]
pub fn run_program_pool<R, F>(mut programs: Vec<PathBuf>, task: F) -> Vec<(PathBuf, R)>
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
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    outcomes
        .into_iter()
        .map(|(program, outcome)| {
            let value = outcome.unwrap_or_else(|error| {
                panic!("program pool omitted the reported failure: {error}")
            });
            (program, value)
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
    fn message(message: impl Into<String>) -> Self {
        Self {
            diagnostics: None,
            summary: message.into(),
        }
    }

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

fn cargo_command() -> Command {
    let mut command = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    command.current_dir(repository_root()).env(
        "CARGO_BUILD_JOBS",
        std::env::var_os("CARGO_BUILD_JOBS").unwrap_or_else(|| "4".into()),
    );
    command
}

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

fn artifact_path(output: &Output, package: &str) -> Result<PathBuf, String> {
    let wanted = staticlib_file_name(package);
    let crate_name = package.replace('-', "_");
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if !line.contains("\"compiler-artifact\"") || !line.contains(&crate_name) {
            continue;
        }
        for piece in line.split('"') {
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

fn include_directory() -> PathBuf {
    repository_root().join("crates/facade")
}

#[derive(Clone, Copy)]
struct CoverageMemory {
    address: usize,
    len: usize,
}

static COVERAGE_MEMORY: OnceLock<CoverageMemory> = OnceLock::new();

fn coverage_counts() -> &'static [AtomicU64] {
    let memory = COVERAGE_MEMORY.get_or_init(|| {
        let len = native_symbols_generated::facade_export_names().len();
        #[cfg(unix)]
        let address = {
            let byte_len = len
                .checked_mul(std::mem::size_of::<AtomicU64>())
                .expect("facade coverage array size");
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
            assert_ne!(
                address,
                libc::MAP_FAILED,
                "allocate facade coverage counters"
            );
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
        CoverageMemory { address, len }
    });
    // SAFETY: `coverage_counts` creates this process-lifetime storage at exactly
    // `len * size_of::<AtomicU64>()` bytes and never changes its address or length.
    unsafe { std::slice::from_raw_parts(memory.address as *const AtomicU64, memory.len) }
}

pub(crate) fn coverage_hit(index: usize) {
    coverage_counts()[index].fetch_add(1, Ordering::Relaxed);
}

fn coverage_reset() {
    for counter in coverage_counts() {
        counter.store(0, Ordering::Relaxed);
    }
}

fn coverage_reached() -> Vec<String> {
    native_symbols_generated::facade_export_names()
        .iter()
        .zip(coverage_counts())
        .filter(|(_, counter)| counter.load(Ordering::Relaxed) != 0)
        .map(|(name, _)| (*name).to_owned())
        .collect()
}

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

fn load_program_with_library(
    program: &Path,
    library: NativeLibrary,
) -> Result<ReloadSession, ProgramLoadError> {
    let files = prepare_program(program)?;
    if let Err(diagnostics) = subscript_compiler::check_program(&files) {
        return Err(ProgramLoadError::rejected(&files, diagnostics));
    }
    match ReloadSession::new_with_native_libraries(&files, &[library]) {
        Ok(session) => Ok(session),
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
    coverage_reset();
    let session = load_program_with_library(program, facade_counting_library())
        .map_err(|error| error.to_string())?;
    let bytes = run_session(session)?;
    Ok((bytes, coverage_reached()))
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
