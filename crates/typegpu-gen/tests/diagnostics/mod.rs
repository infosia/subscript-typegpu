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
    fn collect(directory: &Path, files: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(directory)
            .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        {
            let path = entry.expect("read Rust source entry").path();
            if path.is_dir() {
                collect(&path, files);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }

    let mut files = Vec::new();
    collect(directory, &mut files);
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
            for rule in source
                .lines()
                .filter_map(|line| line.strip_prefix("// expected-rule: "))
            {
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
    const K15_GENERATOR_SITE_EXEMPTION: &str = "K15";

    let root = root();
    let allowed = allowed_rules(&root);
    let fixtures = fixture_rules(&root);
    let meta = BTreeSet::from(["K17", "K24", "PI13", "RN16", "SC14", "TX8"]);

    let mut generator_sites = 0usize;
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
                source.contains("format!(\"K15: {} (generator)\"")
                    || source.contains("format!(\n        \"K15: {} (generator)\""),
                "{} generator diagnostic helper lacks the K15 owner",
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
                generator_sites += 1;
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

    assert!(
        generator_sites != 0,
        "K15 exemption matched no generator sites"
    );
    assert!(
        allowed.contains(K15_GENERATOR_SITE_EXEMPTION)
            && !meta.contains(K15_GENERATOR_SITE_EXEMPTION),
        "K15 generator-site exemption must name a non-meta rule",
    );
    let naga_gate = read(&root.join("crates/harness/tests/wgsl_goldens/mod.rs"));
    for evidence in [
        "naga::front::wgsl::parse_str",
        "Validator::new",
        "K15 (generator): naga",
    ] {
        assert!(
            naga_gate.contains(evidence),
            "K15 generator-site exemption lacks harness naga evidence `{evidence}`",
        );
    }

    let library_spec = read(&root.join("specs/blocks/library.md"));
    for module in [
        "typegpu.ts",
        "typegpu-types.ts",
        "typegpu-color.ts",
        "typegpu-noise.ts",
        "typegpu-radiance-cascades.ts",
        "typegpu-sdf.ts",
        "typegpu-sort.ts",
        "typegpu-ui-atlas.generated.ts",
        "typegpu-ui.ts",
    ] {
        let path = root.join("lib").join(module);
        let source = read(&path);
        let trap_count = source.matches("unreachable();").count();
        match module {
            "typegpu.ts" => {
                assert_eq!(
                    trap_count,
                    1,
                    "{} traps must go through authorTrap",
                    path.display()
                );
                assert!(
                    source.contains("function authorTrap(rule: string, method: string, values: string): void {\n  print(`${rule} ${method} ${values} (author)`);\n  unreachable();\n}"),
                    "{} trap helper must lead with its documented rule id",
                    path.display(),
                );
            }
            "typegpu-ui.ts" => {
                assert_eq!(trap_count, 1, "UI traps must use uiTrap");
                assert!(source.contains("function uiTrap(rule: string, method: string, values: string): void {\n  print(`${rule} ${method} ${values} (author)`);\n  unreachable();\n}"));
                for rule in ["UIT1", "UIT2", "UIT3"] {
                    assert!(library_spec.contains(&format!("`{rule}`")));
                }
            }
            "typegpu-sort.ts" => {
                assert_eq!(
                    trap_count,
                    1,
                    "{} traps must go through sortTrap",
                    path.display()
                );
                assert!(
                    source.contains("function sortTrap(method: string, values: string): void {\n  print(`SORT1 ${method} ${values} (author)`);\n  unreachable();\n}"),
                    "{} trap helper must lead with SORT1",
                    path.display(),
                );
                assert!(
                    library_spec.contains("`SORT1` —"),
                    "SORT1 trap lacks a library rule-table entry",
                );
            }
            _ => assert_eq!(
                trap_count,
                0,
                "{} has a trap outside a helper",
                path.display()
            ),
        }
    }

    let runtime = root.join("lib/typegpu.ts");
    let source = read(&runtime);
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
