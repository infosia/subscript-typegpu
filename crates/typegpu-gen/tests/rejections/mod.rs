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
    assert_eq!(fixtures.len(), 30, "rejection fixture count");
    for fixture in fixtures {
        let stem = fixture
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("fixture stem");
        let source = support::read(&fixture);
        let expected = source
            .lines()
            .next()
            .and_then(|line| line.strip_prefix("// expected-rule: "))
            .map(str::to_owned)
            .unwrap_or_else(|| {
                stem.split_once('-')
                    .map_or(stem, |(rule, _)| rule)
                    .to_ascii_uppercase()
            });
        let mut files = support::b01_files();
        files.pop();
        files.push(SourceFile::new(
            fixture.file_name().expect("fixture name").to_string_lossy(),
            source,
        ));
        let diagnostics = match subscript_typegpu_gen::generate(&files) {
            Err(diagnostics) => diagnostics,
            Ok(_) => panic!("rejection fixture generated: {}", fixture.display()),
        };
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
