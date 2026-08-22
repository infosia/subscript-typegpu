//! EG7/EG8 diagnostic and runtime-trap accountability sweep.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("typegpu-gen is under the repository root")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn rust_sources(directory: &Path) -> Vec<PathBuf> {
    let mut files = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        .map(|entry| entry.expect("read Rust source entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn quoted_argument(text: &str) -> Option<&str> {
    let start = text.find('"')? + 1;
    let end = text[start..].find('"')? + start;
    Some(&text[start..end])
}

fn fixture_rules(root: &Path) -> BTreeSet<String> {
    let mut rules = BTreeSet::new();
    for relative in [
        "crates/typegpu-gen/tests/fixtures/reject",
        "crates/harness/tests/fixtures/trap",
    ] {
        let directory = root.join(relative);
        for entry in std::fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        {
            let path = entry.expect("read fixture entry").path();
            if path.extension().is_none_or(|extension| extension != "ts") {
                continue;
            }
            let source = read(&path);
            let mut named = false;
            for rule in source.lines().filter_map(|line| {
                line.strip_prefix("// expected-rule: ")
                    .or_else(|| line.strip_prefix("// covers-rule: "))
            }) {
                rules.insert(rule.to_owned());
                named = true;
            }
            assert!(named, "{} lacks expected-rule", path.display());
        }
    }
    rules
}

fn allowed_rules(root: &Path) -> BTreeSet<String> {
    read(&root.join("specs/blocks/rule-ids.txt"))
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.contains(' '))
        .map(str::to_owned)
        .collect()
}

fn assert_site(
    path: &Path,
    line_number: usize,
    rule: &str,
    allowed: &BTreeSet<String>,
    fixtures: &BTreeSet<String>,
) {
    assert!(
        allowed.contains(rule),
        "{}:{line_number} cites unknown rule `{rule}`",
        path.display(),
    );
    assert!(
        fixtures.contains(rule),
        "{}:{line_number} cites `{rule}` without a fixture",
        path.display(),
    );
}

#[test]
fn every_diagnostic_and_trap_has_a_rule_owner_and_fixture() {
    let root = root();
    let allowed = allowed_rules(&root);
    let fixtures = fixture_rules(&root);
    let meta = BTreeSet::from(["K17", "K24", "PI13", "RN16", "SC14", "TX8"]);

    for path in rust_sources(&root.join("crates/typegpu-gen/src")) {
        let source = read(&path);
        if source.contains("fn diagnostic(") {
            assert!(
                source.contains("(author)"),
                "{} diagnostic helper lacks the author owner",
                path.display(),
            );
        }
        if source.contains("fn generator_diagnostic(") {
            assert!(
                source.contains("(generator)"),
                "{} generator diagnostic helper lacks the generator owner",
                path.display(),
            );
        }
        let lines = source.lines().collect::<Vec<_>>();
        for (line_index, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("fn diagnostic(")
                || trimmed.starts_with("fn generator_diagnostic(")
                || line.contains("identity_diagnostic(")
            {
                continue;
            }
            if line.contains("Diagnostic::new(") {
                let preceding = lines[..line_index].iter().rev().take(3);
                assert!(
                    preceding.into_iter().any(|line| {
                        line.contains("fn diagnostic(") || line.contains("fn generator_diagnostic(")
                    }),
                    "{}:{} constructs a diagnostic outside an owner helper",
                    path.display(),
                    line_index + 1,
                );
            }
            let Some(offset) = line.find("diagnostic(") else {
                continue;
            };
            if line[..offset].ends_with("generator_") {
                assert!(allowed.contains("K15") && !meta.contains("K15"));
                continue;
            }
            let tail = lines[line_index..lines.len().min(line_index + 4)].join("\n");
            let rule = quoted_argument(&tail).unwrap_or_else(|| {
                panic!(
                    "{}:{} diagnostic rule is not a string literal",
                    path.display(),
                    line_index + 1,
                )
            });
            assert!(
                !meta.contains(rule),
                "{}:{} cites meta-rule `{rule}`",
                path.display(),
                line_index + 1,
            );
            assert_site(&path, line_index + 1, rule, &allowed, &fixtures);
        }
    }

    let runtime = root.join("lib/typegpu.ts");
    let source = read(&runtime);
    assert!(
        source.contains("function authorTrap(") && source.contains("(author)"),
        "runtime trap helper lacks the author owner",
    );
    assert_eq!(
        source.matches("unreachable();").count(),
        1,
        "runtime trap sites must go through authorTrap",
    );
    for (line_index, line) in source.lines().enumerate() {
        if !line.contains("authorTrap(") || line.contains("function authorTrap(") {
            continue;
        }
        let rule = quoted_argument(line).unwrap_or_else(|| {
            panic!(
                "{}:{} trap rule is not a string literal",
                runtime.display(),
                line_index + 1,
            )
        });
        assert_site(&runtime, line_index + 1, rule, &allowed, &fixtures);
    }
}
