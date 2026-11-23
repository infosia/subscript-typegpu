//! Every generator rejection fixture is red with its named rule and owner.
//!
//! The checker owns unreachable library-call shapes such as workgroup initializers.

use std::path::{Path, PathBuf};

use subscript_compiler::SourceFile;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("harness crate is below repository root")
        .to_path_buf()
}

fn files(fixture: &Path) -> Vec<SourceFile> {
    subscript_typegpu_gen::load_program_files(&root().join("lib"), fixture)
        .unwrap_or_else(|error| panic!("load {}: {error}", fixture.display()))
}

fn assert_missing_import(entry: &str, specifier: &str) {
    let program = root()
        .join("crates/harness/tests/fixtures/app-modules")
        .join(entry);
    let error = subscript_typegpu_harness::load_program(&program)
        .expect_err("the compiler must reject the unresolved import");
    let diagnostics = error.diagnostics().expect("compiler diagnostic");
    assert!(
        diagnostics.contains(&format!("error[S100]: imported module `{specifier}`")),
        "{diagnostics}"
    );
    assert!(
        diagnostics.contains(&format!(" --> {entry}:2:")),
        "{diagnostics}"
    );
}

#[test]
fn missing_application_module_reports_the_import_site() {
    assert_missing_import("missing.ts", "./absent");
}

#[test]
fn subdirectory_import_reports_the_import_site() {
    assert_missing_import("nested.ts", "./sub/tile");
}

#[test]
fn every_fixture_is_red_with_rule_and_owner() {
    let directory = root().join("crates/typegpu-gen/tests/fixtures/reject");
    let mut fixtures = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        .map(|entry| entry.expect("read fixture entry").path())
        .collect::<Vec<_>>();
    fixtures.sort();
    assert!(!fixtures.is_empty(), "rejection fixture corpus is empty");
    for fixture in fixtures {
        let inputs = files(&fixture);
        let source = &inputs.last().expect("fixture source").source;
        let expected = source
            .lines()
            .find_map(|line| line.strip_prefix("// expected-rule: "))
            .expect("expected-rule header");
        let owner = source
            .lines()
            .find_map(|line| line.strip_prefix("// expected-owner: "))
            .unwrap_or("author");
        let expected_message = source
            .lines()
            .find_map(|line| line.strip_prefix("// expected-message: "));
        let diagnostics = match subscript_typegpu_gen::generate(&inputs) {
            Ok(generated) => panic!(
                "{} unexpectedly generated: {generated:?}",
                fixture.display()
            ),
            Err(diagnostics) => diagnostics,
        };
        assert_eq!(
            diagnostics.len(),
            1,
            "{} must reach one check only:\n{}",
            fixture.display(),
            diagnostics
                .iter()
                .map(|item| item.message.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        );
        let matching = &diagnostics[0];
        let rule_matches = if owner == "checker" {
            matching.code.as_str() == expected
        } else {
            matching.message.starts_with(&format!("{expected}:"))
        };
        assert!(
            rule_matches,
            "{} expected {expected}:\n{}",
            fixture.display(),
            diagnostics
                .iter()
                .map(|item| item.message.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        );
        match owner {
            "author" => assert!(
                matching.message.ends_with("(author)"),
                "{} matching diagnostic lacks author owner",
                fixture.display()
            ),
            "checker" => assert!(
                !matching.message.ends_with("(author)")
                    && !matching.message.ends_with("(generator)"),
                "{} checker diagnostic was relabeled",
                fixture.display()
            ),
            value => panic!("{} unknown expected owner {value}", fixture.display()),
        }
        if let Some(expected_message) = expected_message {
            assert!(
                matching.message.contains(expected_message),
                "{} diagnostic lacks `{expected_message}`: {}",
                fixture.display(),
                matching.message
            );
        }
    }
}
