use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use subscript_codegen::ReloadSession;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("harness crate is under the repository root")
        .to_path_buf()
}

fn is_live_program(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".ts") else {
        return false;
    };
    let bytes = stem.as_bytes();
    bytes.len() >= 5
        && bytes[0] == b'x'
        && bytes[1].is_ascii_digit()
        && bytes[2].is_ascii_digit()
        && bytes[3] == b'-'
        && bytes[4..]
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

fn live_programs() -> Vec<PathBuf> {
    let directory = repository_root().join("programs");
    let mut programs = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        .map(|entry| entry.expect("read program entry").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(is_live_program)
        })
        .collect::<Vec<_>>();
    programs.sort();
    programs
}

fn run_dev(program: &Path) -> String {
    let files = subscript_typegpu_harness::program_files(program)
        .unwrap_or_else(|error| panic!("load {}: {error}", program.display()));
    let libraries = [subscript_typegpu_harness::facade_library()];
    let mut session = ReloadSession::new_with_native_libraries(&files, &libraries)
        .unwrap_or_else(|error| panic!("compile {} dev: {error}", program.display()));
    session
        .call_main()
        .unwrap_or_else(|error| panic!("run {} dev main: {error}", program.display()));
    while session.async_pending() != 0 {
        session
            .async_step()
            .unwrap_or_else(|error| panic!("step {} dev: {error}", program.display()));
    }
    String::from_utf8(session.take_output())
        .unwrap_or_else(|error| panic!("{} dev output is not UTF-8: {error}", program.display()))
}

fn ship_output(program: &Path, output: Output) -> String {
    assert!(
        output.status.success(),
        "{} ship failed with {}\nstdout:\n{}\nstderr:\n{}",
        program.display(),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8(output.stdout)
        .unwrap_or_else(|error| panic!("{} ship output is not UTF-8: {error}", program.display()))
}

fn run_ship(program: &Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_subscript-typegpu-harness"))
        .arg("ship")
        .arg(program)
        .output()
        .unwrap_or_else(|error| panic!("spawn {} ship: {error}", program.display()));
    ship_output(program, output)
}

fn assert_pass(program: &Path, tier: &str, output: &str) {
    assert_eq!(
        output.lines().last(),
        Some("PASS"),
        "{} {tier} did not pass:\n{output}",
        program.display(),
    );
}

#[test]
fn live_tool_removes_the_default_backend_request() {
    let path = repository_root().join("tools/live.sh");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    assert!(source.contains("default|metal|vulkan"));
    assert!(source
        .contains("if [ \"$backend\" = \"default\" ]; then\n  unset SUBSCRIPT_TYPEGPU_BACKEND"));
    assert!(source.contains("[ \"$backend\" = \"metal\" ]"));
}

#[test]
#[ignore = "requires a real adapter through tools/live.sh"]
fn every_x_program_passes_on_a_real_adapter() {
    let programs = live_programs();
    assert!(!programs.is_empty(), "no live programs");
    for program in programs {
        let dev = run_dev(&program);
        assert_pass(&program, "dev", &dev);
        let ship = run_ship(&program);
        assert_pass(&program, "ship", &ship);
    }
}
