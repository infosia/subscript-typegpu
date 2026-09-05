use std::path::{Path, PathBuf};
use std::process::ExitCode;

use subscript_compiler::SourceFile;

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))
}

fn run() -> Result<(), String> {
    const USAGE: &str = "usage: subscript-typegpu-gen gen <program.ts> --lib <dir> -o <dir>";
    let mut arguments = std::env::args_os().skip(1);
    let command = arguments.next();
    if command.as_deref() == Some(std::ffi::OsStr::new("ui-atlas")) {
        let root = arguments
            .next()
            .map(PathBuf::from)
            .ok_or("usage: subscript-typegpu-gen ui-atlas <repo-root>")?;
        if arguments.next().is_some() {
            return Err("usage: subscript-typegpu-gen ui-atlas <repo-root>".to_owned());
        }
        let module = subscript_typegpu_gen::generate_ui_atlas(&root)?;
        let destination = root.join("lib/typegpu-ui-atlas.generated.ts");
        return std::fs::write(&destination, module)
            .map_err(|error| format!("write {}: {error}", destination.display()));
    }
    if command.as_deref() != Some(std::ffi::OsStr::new("gen")) {
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
    let program_file = SourceFile::new(program_name, read(&program)?);
    let mut files = subscript_typegpu_gen::load_library_files(&library, &program_file)
        .map_err(|error| error.to_string())?;
    files.push(program_file);
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
