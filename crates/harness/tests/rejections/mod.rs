//! Every generator rejection fixture is red with its named rule and owner.

use std::path::{Path, PathBuf};

use subscript_compiler::SourceFile;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("harness crate is below repository root")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn files(fixture: &Path) -> Vec<SourceFile> {
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
        SourceFile::new(
            fixture.file_name().expect("fixture name").to_string_lossy(),
            read(fixture),
        ),
    ]
}

#[test]
fn every_fixture_is_red_with_rule_and_owner() {
    let directory = root().join("crates/typegpu-gen/tests/fixtures/reject");
    let mut fixtures = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        .map(|entry| entry.expect("read fixture entry").path())
        .collect::<Vec<_>>();
    fixtures.sort();
    assert_eq!(fixtures.len(), 30, "rejection fixture count");
    for fixture in fixtures {
        let inputs = files(&fixture);
        let expected = inputs
            .last()
            .expect("fixture source")
            .source
            .lines()
            .next()
            .and_then(|line| line.strip_prefix("// expected-rule: "))
            .expect("expected-rule header");
        let diagnostics = subscript_typegpu_gen::generate(&inputs)
            .expect_err("rejection fixture unexpectedly generated");
        let matching = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message.starts_with(&format!("{expected}:")));
        assert!(
            matching.is_some(),
            "{} expected {expected}:\n{}",
            fixture.display(),
            diagnostics
                .iter()
                .map(|item| item.message.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        );
        assert!(
            matching
                .expect("matching diagnostic")
                .message
                .contains("(author)"),
            "{} matching diagnostic lacks its owner",
            fixture.display()
        );
    }
}
