use std::path::PathBuf;
use std::process::Command;

use crate::support;

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let directory =
            std::env::temp_dir().join(format!("subscript-typegpu-cli-{}", std::process::id()));
        if directory.exists() {
            std::fs::remove_dir_all(&directory).expect("remove stale CLI directory");
        }
        std::fs::create_dir_all(&directory).expect("create CLI directory");
        Self(directory)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn cli_writes_the_same_support_module_as_the_library() {
    let root = support::root();
    let output = TempDir::new();
    let result = Command::new(env!("CARGO_BIN_EXE_subscript-typegpu-gen"))
        .arg("gen")
        .arg(root.join("programs/b01-layout.ts"))
        .arg("--lib")
        .arg(root.join("lib"))
        .arg("-o")
        .arg(&output.0)
        .output()
        .expect("run generator CLI");
    assert!(
        result.status.success(),
        "generator CLI failed:\n{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let written = support::read(&output.0.join("b01-layout.typegpu.ts"));
    let expected = subscript_typegpu_gen::generate(&support::b01_files())
        .expect("generate b01 in memory")
        .support_module;
    assert_eq!(written, expected);
}
