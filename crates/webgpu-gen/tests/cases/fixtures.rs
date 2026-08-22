//! Fixture snapshot tests (T6): small hand-authored yml+policy inputs
//! generate exactly the committed expected text. Snapshots are
//! expectations, never build inputs.

use std::path::PathBuf;

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn repo_file(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("generator crate is under the repository root")
        .join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn preceding_comment_has_safety(lines: &[&str], index: usize) -> bool {
    let mut previous = index;
    while previous > 0 {
        previous -= 1;
        let line = lines[previous].trim();
        if line.is_empty() {
            continue;
        }
        if !line.starts_with("//") {
            return false;
        }
        if line.starts_with("// SAFETY:") {
            return true;
        }
    }
    false
}

#[test]
fn emitted_unsafe_lines_have_safety_comments() {
    let generated = subscript_typegpu_webgpu_gen::generate(
        &repo_file("third_party/webgpu-headers/webgpu.yml"),
        &repo_file("crates/webgpu-gen/policy.toml"),
    )
    .expect("facade inputs generate");
    let lines = generated.rust.lines().collect::<Vec<_>>();
    let uncovered = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.contains("unsafe {") || line.contains("unsafe extern"))
        .filter(|(index, _)| !preceding_comment_has_safety(&lines, *index))
        .map(|(index, line)| format!("{}: {}", index + 1, line.trim()))
        .collect::<Vec<_>>();
    assert!(
        uncovered.is_empty(),
        "emitted unsafe line lacks a SAFETY comment:\n{}",
        uncovered.join("\n")
    );
}

#[test]
fn mini_fixture_header_snapshot() {
    let generated =
        subscript_typegpu_webgpu_gen::generate(&fixture("mini.yml"), &fixture("mini.policy.toml"))
            .expect("mini fixture generates");
    assert_eq!(
        generated.header,
        fixture("mini.subscript-typegpu.h.expected"),
        "mini fixture header drifted from the committed snapshot"
    );
}

#[test]
fn mini_fixture_rust_snapshot() {
    let generated =
        subscript_typegpu_webgpu_gen::generate(&fixture("mini.yml"), &fixture("mini.policy.toml"))
            .expect("mini fixture generates");
    assert_eq!(
        generated.rust,
        fixture("mini.generated.rs.expected"),
        "mini fixture Rust drifted from the committed snapshot"
    );
}

/// Byte-stability: two runs over the same inputs emit identical bytes.
#[test]
fn generation_is_deterministic() {
    let yml = fixture("mini.yml");
    let policy = fixture("mini.policy.toml");
    let a = subscript_typegpu_webgpu_gen::generate(&yml, &policy).expect("generates");
    let b = subscript_typegpu_webgpu_gen::generate(&yml, &policy).expect("generates");
    assert_eq!(a.header, b.header);
    assert_eq!(a.rust, b.rust);
}
