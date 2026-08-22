//! The headless development and ship-tier harness.

#[path = "native_symbols.generated.rs"]
#[rustfmt::skip]
pub mod native_symbols_generated;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use subscript_codegen::{
    run_c_aot_with_native_libraries, run_jit_with_native_libraries, NativeLibrary,
};
use subscript_compiler::SourceFile;

fn repository_root() -> PathBuf {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();
    root
}

fn read(relative: &str) -> Result<String, String> {
    let path = repository_root().join(relative);
    std::fs::read_to_string(&path).map_err(|error| format!("read {relative}: {error}"))
}

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

/// Loads the mirrors, libraries, program, and optional schema support.
pub fn program_files(program: &Path) -> Result<Vec<SourceFile>, String> {
    let stem = program
        .file_stem()
        .and_then(OsStr::to_str)
        .ok_or_else(|| format!("program has no UTF-8 stem: {}", program.display()))?;
    let program_source = std::fs::read_to_string(program)
        .map_err(|error| format!("read {}: {error}", program.display()))?;
    let mut files = vec![
        SourceFile::ambient(
            "subscript-typegpu.generated.d.ts",
            read("lib/subscript-typegpu.generated.d.ts")?,
        ),
        SourceFile::ambient(
            "wire-enum-aliases.generated.d.ts",
            read("lib/wire-enum-aliases.generated.d.ts")?,
        ),
        SourceFile::new("webgpu.ts", read("lib/webgpu.ts")?),
        SourceFile::new("typegpu-types.ts", read("lib/typegpu-types.ts")?),
        SourceFile::new("typegpu.ts", read("lib/typegpu.ts")?),
        SourceFile::new(format!("{stem}.ts"), program_source),
    ];
    if stem.starts_with('b') || stem.starts_with('x') {
        let generated = subscript_typegpu_gen::generate(&files)
            .map_err(|diagnostics| subscript_compiler::render_diagnostics(&files, &diagnostics))?;
        files.push(SourceFile::new(
            format!("{stem}.typegpu.ts"),
            generated.support_module,
        ));
    }
    Ok(files)
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

/// Builds and returns the facade archive and its platform libraries.
pub fn ship_link_inputs() -> Result<Vec<String>, String> {
    static INPUTS: OnceLock<Result<Vec<String>, String>> = OnceLock::new();
    INPUTS.get_or_init(facade_link_inputs).clone()
}

/// Runs one program through the development JIT.
pub fn run_dev(program: &Path) -> Result<Vec<u8>, String> {
    run_jit_with_native_libraries(&program_files(program)?, &[facade_library()])
        .map_err(|error| error.to_string())
}

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
