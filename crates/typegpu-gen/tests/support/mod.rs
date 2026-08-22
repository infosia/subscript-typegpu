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
    let root = root();
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
        SourceFile::new("b01-layout.ts", read(&root.join("programs/b01-layout.ts"))),
    ]
}
