use std::path::{Path, PathBuf};

use subscript_compiler::SourceFile;

pub(crate) fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("generator crate is under the repository root")
        .to_path_buf()
}

pub(crate) fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

pub(crate) fn b01_files() -> Vec<SourceFile> {
    program_files(&root().join("programs/b01-layout.ts"))
}

pub(crate) fn program_files(program: &Path) -> Vec<SourceFile> {
    let root = root();
    let name = program
        .file_name()
        .expect("program file name")
        .to_string_lossy();
    vec![
        SourceFile::ambient(
            "subscript-typegpu.generated.d.ts",
            read(&root.join("lib/subscript-typegpu.generated.d.ts")),
        ),
        SourceFile::ambient(
            "wire-enum-aliases.generated.d.ts",
            read(&root.join("lib/wire-enum-aliases.generated.d.ts")),
        ),
        SourceFile::new("webgpu.ts", read(&root.join("lib/webgpu.ts"))),
        SourceFile::new("typegpu-types.ts", read(&root.join("lib/typegpu-types.ts"))),
        SourceFile::new("typegpu.ts", read(&root.join("lib/typegpu.ts"))),
        SourceFile::new(name, read(program)),
    ]
}

pub(crate) fn b_programs() -> Vec<PathBuf> {
    let directory = root().join("programs");
    let mut programs = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        .map(|entry| entry.expect("program entry").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with('b') && name.ends_with(".ts"))
        })
        .collect::<Vec<_>>();
    programs.sort();
    programs
}
