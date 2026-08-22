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
fn cli_writes_the_same_support_and_wgsl_as_the_library_for_every_program() {
    let root = support::root();
    let output = TempDir::new();
    let directory = root.join("programs");
    let mut programs = std::fs::read_dir(&directory)
        .expect("read programs")
        .map(|entry| entry.expect("program entry").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    matches!(name.as_bytes().first(), Some(b'b' | b'x')) && name.ends_with(".ts")
                })
        })
        .collect::<Vec<_>>();
    programs.sort();
    assert!(!programs.is_empty(), "no b/x programs");
    for program in programs {
        let result = Command::new(env!("CARGO_BIN_EXE_subscript-typegpu-gen"))
            .arg("gen")
            .arg(&program)
            .arg("--lib")
            .arg(root.join("lib"))
            .arg("-o")
            .arg(&output.0)
            .output()
            .expect("run generator CLI");
        assert!(
            result.status.success(),
            "generator CLI failed for {}:\n{}",
            program.display(),
            String::from_utf8_lossy(&result.stderr)
        );
        let stem = program
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("program stem");
        let generated = subscript_typegpu_gen::generate(&support::program_files(&program))
            .unwrap_or_else(|diagnostics| panic!("generate {stem} in memory: {diagnostics:?}"));
        let expected_host_runnable = !matches!(stem, "b10-workgroup" | "x08-live-reduction");
        for (pipeline, _) in &generated.pipelines {
            if !generated
                .support_module
                .contains(&format!("{pipeline}_HOST_RUNNABLE"))
            {
                continue;
            }
            assert!(
                generated.support_module.contains(&format!(
                    "export const {pipeline}_HOST_RUNNABLE: boolean = {expected_host_runnable};"
                )),
                "{stem}.{pipeline} has the wrong host-runnable constant:\n{}",
                generated.support_module,
            );
        }
        assert_eq!(
            support::read(&output.0.join(format!("{stem}.typegpu.ts"))),
            generated.support_module,
            "{stem} support module"
        );
        for (pipeline, wgsl) in generated.pipelines {
            assert_eq!(
                support::read(&output.0.join(format!("{stem}.{pipeline}.wgsl"))),
                wgsl,
                "{stem}.{pipeline}.wgsl"
            );
        }
    }
}
