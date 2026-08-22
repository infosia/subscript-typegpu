use subscript_compiler::SourceFile;

use crate::support;

#[test]
fn every_rejection_fixture_reports_its_rule() {
    let directory = support::root().join("crates/typegpu-gen/tests/fixtures/reject");
    let mut fixtures = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        .map(|entry| entry.expect("read fixture entry").path())
        .collect::<Vec<_>>();
    fixtures.sort();
    assert_eq!(fixtures.len(), 10, "schema rejection fixture count");
    for fixture in fixtures {
        let stem = fixture
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("fixture stem");
        let expected = stem
            .split_once('-')
            .map_or(stem, |(rule, _)| rule)
            .to_ascii_uppercase();
        let source = support::read(&fixture);
        let diagnostics = subscript_typegpu_gen::generate(&[SourceFile::new(
            fixture.file_name().expect("fixture name").to_string_lossy(),
            source,
        )])
        .expect_err("rejection fixture generated");
        let rendered = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            rendered.contains(&expected),
            "{} expected {expected}:\n{rendered}",
            fixture.display()
        );
        assert!(
            rendered.contains("(author)"),
            "{} lacks the diagnostic owner:\n{rendered}",
            fixture.display()
        );
    }
}
