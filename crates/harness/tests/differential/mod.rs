//! Tier differential and determinism gates for every suite program.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Mutex, MutexGuard, Once, OnceLock};

static SUITE_LOCK: Mutex<()> = Mutex::new(());
static FIRST_OUTPUTS: OnceLock<Vec<ProgramOutput>> = OnceLock::new();

fn suite_lock() -> MutexGuard<'static, ()> {
    SUITE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn repository_root() -> PathBuf {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();
    root
}

fn backend_is_available() -> bool {
    match subscript_typegpu_harness::backend_lib() {
        Ok(Some(_)) => true,
        Ok(None) => {
            static PENDING: Once = Once::new();
            PENDING.call_once(|| {
                println!("pending: backend library — set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
            });
            false
        }
        Err(error) => panic!("backend library: {error}"),
    }
}

fn is_program_name(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".ts") else {
        return false;
    };
    let bytes = stem.as_bytes();
    bytes.len() >= 5
        && matches!(bytes[0], b'a' | b'b')
        && bytes[1].is_ascii_digit()
        && bytes[2].is_ascii_digit()
        && bytes[3] == b'-'
        && bytes[4..]
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

fn programs() -> Vec<PathBuf> {
    let directory = repository_root().join("programs");
    let mut programs = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        .map(|entry| entry.expect("read program entry").path())
        .filter(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(is_program_name)
        })
        .collect::<Vec<_>>();
    programs.sort();
    programs
}

fn read(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn first_difference(actual: &[u8], expected: &[u8]) -> usize {
    actual
        .iter()
        .zip(expected)
        .position(|(actual, expected)| actual != expected)
        .unwrap_or_else(|| actual.len().min(expected.len()))
}

fn line_at(bytes: &[u8], offset: usize) -> String {
    let offset = offset.min(bytes.len());
    let start = bytes[..offset]
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |index| index + 1);
    let end = bytes[offset..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(bytes.len(), |index| offset + index);
    String::from_utf8_lossy(&bytes[start..end]).into_owned()
}

fn difference(program: &Path, tier: &str, actual: &[u8], expected: &[u8]) -> Option<String> {
    if actual == expected {
        return None;
    }
    let offset = first_difference(actual, expected);
    Some(format!(
        "{} {tier} differs at byte {offset}\nactual line: {:?}\nexpected line: {:?}",
        program
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("<program>"),
        line_at(actual, offset),
        line_at(expected, offset),
    ))
}

#[derive(Debug)]
struct ProgramOutput {
    program: PathBuf,
    dev: Vec<u8>,
    ship: Vec<u8>,
}

fn child_bytes(program: &Path, tier: &str, output: Output) -> Vec<u8> {
    std::io::stderr()
        .write_all(&output.stderr)
        .expect("forward child stderr");
    assert!(
        output.status.success(),
        "{} {tier} failed with {}",
        program.display(),
        output.status,
    );
    output.stdout
}

fn run_dev(program: &Path) -> Vec<u8> {
    // The dev runner forks, so this tier runs in a child process (T5).
    let output = Command::new(env!("CARGO_BIN_EXE_subscript-typegpu-harness"))
        .arg("dev")
        .arg(program)
        .output()
        .unwrap_or_else(|error| panic!("spawn {} dev: {error}", program.display()));
    child_bytes(program, "dev", output)
}

fn run_ship(program: &Path) -> Vec<u8> {
    // The ship runner sets environment variables, so this tier runs in a child process (T5).
    let output = Command::new(env!("CARGO_BIN_EXE_subscript-typegpu-harness"))
        .arg("ship")
        .arg(program)
        .output()
        .unwrap_or_else(|error| panic!("spawn {} ship: {error}", program.display()));
    child_bytes(program, "ship", output)
}

fn run_suite() -> Vec<ProgramOutput> {
    programs()
        .into_iter()
        .map(|program| ProgramOutput {
            dev: run_dev(&program),
            ship: run_ship(&program),
            program,
        })
        .collect()
}

fn first_outputs() -> &'static [ProgramOutput] {
    FIRST_OUTPUTS.get_or_init(run_suite)
}

fn assert_no_differences(failures: Vec<String>) {
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_program_matches_both_tiers_and_golden() {
    if !backend_is_available() {
        return;
    }
    let _guard = suite_lock();
    let failures = first_outputs()
        .iter()
        .flat_map(|output| {
            let golden = read(&output.program.with_extension("expected"));
            [
                difference(&output.program, "dev", &output.dev, &golden),
                difference(&output.program, "ship", &output.ship, &golden),
            ]
            .into_iter()
            .flatten()
        })
        .collect();
    assert_no_differences(failures);
}

#[test]
fn every_program_is_deterministic_across_repeated_runs() {
    if !backend_is_available() {
        return;
    }
    let _guard = suite_lock();
    let first = first_outputs();
    let second = run_suite();
    assert_eq!(first.len(), second.len(), "suite program count changed");
    let failures = first
        .iter()
        .zip(&second)
        .flat_map(|(expected, actual)| {
            assert_eq!(
                expected.program, actual.program,
                "suite program order changed"
            );
            [
                difference(
                    &actual.program,
                    "dev determinism",
                    &actual.dev,
                    &expected.dev,
                ),
                difference(
                    &actual.program,
                    "ship determinism",
                    &actual.ship,
                    &expected.ship,
                ),
            ]
            .into_iter()
            .flatten()
        })
        .collect();
    assert_no_differences(failures);
}
