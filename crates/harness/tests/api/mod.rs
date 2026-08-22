//! Direct tests for the harness API.

use std::path::PathBuf;
use std::process::Command;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("harness crate is under the repository root")
        .to_path_buf()
}

#[test]
fn program_files_have_the_required_order_names_and_modes() {
    let program = repository_root().join("programs/a01-smoke.ts");
    let files = subscript_typegpu_harness::program_files(&program).expect("load program files");
    let names = files
        .iter()
        .map(|file| file.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        vec![
            "subscript-typegpu.generated.d.ts",
            "wire-enum-aliases.generated.d.ts",
            "webgpu.ts",
            "a01-smoke.ts",
        ]
    );
    assert_eq!(
        files.iter().map(|file| file.dts).collect::<Vec<_>>(),
        vec![true, true, false, false]
    );
}

#[test]
fn backend_lib_distinguishes_unset_and_missing_paths() {
    const CASE: &str = "SUBSCRIPT_TYPEGPU_API_BACKEND_CASE";
    match std::env::var(CASE).as_deref() {
        Ok("unset") => {
            assert_eq!(
                subscript_typegpu_harness::backend_lib().expect("read backend setting"),
                None
            );
            return;
        }
        Ok("missing") => {
            let error = subscript_typegpu_harness::backend_lib()
                .expect_err("a missing backend file must fail");
            assert!(error.contains("which is not a file"), "{error}");
            return;
        }
        _ => {}
    }

    let test_binary = std::env::current_exe().expect("test binary path");
    let unset = Command::new(&test_binary)
        .args([
            "--exact",
            "api::backend_lib_distinguishes_unset_and_missing_paths",
            "--nocapture",
        ])
        .env(CASE, "unset")
        .env_remove("SUBSCRIPT_TYPEGPU_BACKEND_LIB")
        .output()
        .expect("run unset backend case");
    assert!(
        unset.status.success(),
        "unset backend case failed:\n{}",
        String::from_utf8_lossy(&unset.stderr)
    );

    let missing = std::env::temp_dir().join(format!(
        "subscript-typegpu-missing-backend-{}",
        std::process::id()
    ));
    assert!(!missing.exists(), "missing backend test path exists");
    let absent = Command::new(test_binary)
        .args([
            "--exact",
            "api::backend_lib_distinguishes_unset_and_missing_paths",
            "--nocapture",
        ])
        .env(CASE, "missing")
        .env("SUBSCRIPT_TYPEGPU_BACKEND_LIB", missing)
        .output()
        .expect("run missing backend case");
    assert!(
        absent.status.success(),
        "missing backend case failed:\n{}",
        String::from_utf8_lossy(&absent.stderr)
    );
}

#[test]
fn facade_library_has_the_generated_symbols_and_header_directory() {
    let library = subscript_typegpu_harness::facade_library();
    let rendered = format!("{library:?}");
    let include_directory = repository_root().join("crates/facade");
    assert!(include_directory.join("subscript-typegpu.h").is_file());
    assert!(
        rendered.contains(&format!("include_directories: [{include_directory:?}]")),
        "facade include directory differs: {rendered}"
    );
    assert!(rendered.contains("c_sources: []"));
    assert_eq!(
        subscript_typegpu_harness::native_symbols_generated::facade_symbols().len(),
        super::FACADE_EXPORT_COUNT
    );
    assert_eq!(
        rendered.matches("(\"subscript_typegpu_").count(),
        super::FACADE_EXPORT_COUNT
    );
}
