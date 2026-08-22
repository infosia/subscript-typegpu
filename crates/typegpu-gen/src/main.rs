use std::path::{Path, PathBuf};
use std::process::ExitCode;

use subscript_compiler::SourceFile;

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))
}

fn run() -> Result<(), String> {
    const USAGE: &str = "usage: subscript-typegpu-gen gen <program.ts> --lib <dir> -o <dir>";
    let mut arguments = std::env::args_os().skip(1);
    if arguments.next().as_deref() != Some(std::ffi::OsStr::new("gen")) {
        return Err(USAGE.to_owned());
    }
    let program = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| USAGE.to_owned())?;
    if arguments.next().as_deref() != Some(std::ffi::OsStr::new("--lib")) {
        return Err(USAGE.to_owned());
    }
    let library = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| USAGE.to_owned())?;
    if arguments.next().as_deref() != Some(std::ffi::OsStr::new("-o")) {
        return Err(USAGE.to_owned());
    }
    let output = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| USAGE.to_owned())?;
    if arguments.next().is_some() {
        return Err(USAGE.to_owned());
    }
    let program_name = program
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("program has no UTF-8 file name: {}", program.display()))?;
    let files = [
        SourceFile::ambient(
            "subscript-typegpu.generated.d.ts",
            read(&library.join("subscript-typegpu.generated.d.ts"))?,
        ),
        SourceFile::ambient(
            "wire-enum-aliases.generated.d.ts",
            read(&library.join("wire-enum-aliases.generated.d.ts"))?,
        ),
        SourceFile::new("webgpu.ts", read(&library.join("webgpu.ts"))?),
        SourceFile::new("typegpu-types.ts", read(&library.join("typegpu-types.ts"))?),
        SourceFile::new("typegpu.ts", read(&library.join("typegpu.ts"))?),
        SourceFile::new(program_name, read(&program)?),
    ];
    let generated = subscript_typegpu_gen::generate(&files)
        .map_err(|diagnostics| subscript_compiler::render_diagnostics(&files, &diagnostics))?;
    std::fs::create_dir_all(&output)
        .map_err(|error| format!("create {}: {error}", output.display()))?;
    let stem = program
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| format!("program has no UTF-8 stem: {}", program.display()))?;
    let destination = output.join(format!("{stem}.typegpu.ts"));
    std::fs::write(&destination, generated.support_module)
        .map_err(|error| format!("write {}: {error}", destination.display()))?;
    for (pipeline, source) in generated.pipelines {
        let destination = output.join(format!("{stem}.{pipeline}.wgsl"));
        std::fs::write(&destination, source)
            .map_err(|error| format!("write {}: {error}", destination.display()))?;
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
