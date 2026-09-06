//! The generator driver.
//!
//! The binary takes one argument, the repository root. It reads the pinned `webgpu.yml`, the
//! policy file, and the pinned GPUWeb `.bs` files, then writes every generated output in place.
//! The second pass calls `subscript_bindgen`, which loads libclang at run time.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Reads one input file. The error text names the path that failed.
fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))
}

/// Writes one output file and creates its parent directory first.
///
/// The error text names the path that failed.
fn write(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    std::fs::write(path, contents).map_err(|error| format!("write {}: {error}", path.display()))
}

/// Drops the `@subscript-cenum` directive lines from the header text.
///
/// A directive makes `subscript bind` spell that enum with its public IDL alias in the mirror.
/// The API join must read the boundary spelling, so the base mirror comes from a header without
/// the directives.
fn without_cenum_directives(header: &str) -> String {
    header
        .split_inclusive('\n')
        .filter(|line| !line.contains("@subscript-cenum"))
        .collect()
}

/// Runs the first pass and writes the four outputs that need no libclang.
///
/// The pass returns the generated artifacts, because the second pass needs the header text and
/// the CEnum alias list.
fn write_libclang_free_outputs(
    root: &Path,
) -> Result<subscript_typegpu_webgpu_gen::Generated, String> {
    let yml = read(&root.join("third_party/webgpu-headers/webgpu.yml"))?;
    let policy = read(&root.join("crates/webgpu-gen/policy.toml"))?;
    let generated =
        subscript_typegpu_webgpu_gen::generate(&yml, &policy).map_err(|error| error.to_string())?;
    write(
        &root.join("crates/facade/subscript-typegpu.h"),
        &generated.header,
    )?;
    write(
        &root.join("crates/facade/src/generated.rs"),
        &generated.rust,
    )?;
    write(
        &root.join("crates/facade/src/surface.rs"),
        &generated.surface,
    )?;
    write(
        &root.join("crates/harness/src/native_symbols.generated.rs"),
        &generated.native_symbols,
    )?;
    Ok(generated)
}

/// Runs the second pass and writes the three `lib/` outputs.
///
/// The pass builds the base mirror from the directive-free header, joins it with the pinned
/// GPUWeb IDL and the API policy, then builds the shipped mirror from the full header. It fails
/// when the policy-derived alias list and the API-joined alias list disagree.
fn write_libclang_outputs(
    root: &Path,
    generated: &subscript_typegpu_webgpu_gen::Generated,
) -> Result<(), String> {
    let policy = read(&root.join("crates/webgpu-gen/policy.toml"))?;
    let gpuweb = subscript_typegpu_webgpu_gen::GPUWEB_IDL_INPUTS
        .iter()
        .map(|relative| read(&root.join(relative)))
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    let base_header = without_cenum_directives(&generated.header);
    let base_mirror = subscript_bindgen::generate_for_header(&base_header, "subscript-typegpu.h")
        .map_err(|error| error.to_string())?;
    let api = subscript_typegpu_webgpu_gen::generate_api(&gpuweb, &base_mirror, &policy)
        .map_err(|error| error.to_string())?;
    if api.cenum_aliases != generated.cenum_aliases {
        return Err("policy-derived and API-joined CEnum alias lists differ".to_owned());
    }
    let mirror = subscript_bindgen::generate_for_header(&generated.header, "subscript-typegpu.h")
        .map_err(|error| error.to_string())?;
    write(&root.join("lib/subscript-typegpu.generated.d.ts"), &mirror)?;
    write(
        &root.join("lib/wire-enum-aliases.generated.d.ts"),
        &api.wire_enum_aliases,
    )?;
    write(&root.join("lib/webgpu.ts"), &api.source)?;
    Ok(())
}

/// Parses the one argument, the repository root, and runs the two passes in order.
fn run() -> Result<(), String> {
    let mut arguments = std::env::args_os().skip(1);
    let root = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "usage: subscript-typegpu-webgpu-gen <repository-root>".to_owned())?;
    if arguments.next().is_some() {
        return Err("usage: subscript-typegpu-webgpu-gen <repository-root>".to_owned());
    }
    let generated = write_libclang_free_outputs(&root)
        .map_err(|error| format!("libclang-free output pass: {error}"))?;
    write_libclang_outputs(&root, &generated)
        .map_err(|error| format!("libclang output pass: {error}"))?;
    Ok(())
}

/// Prints a failure on stderr and reports it through the exit code.
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
