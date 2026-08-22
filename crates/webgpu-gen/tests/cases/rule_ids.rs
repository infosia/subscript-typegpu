//! Every rule citation in generator-owned text resolves in this repository.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root")
        .to_path_buf()
}

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) {
    let mut entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        .map(|entry| entry.expect("directory entry").path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_files(&path, files);
        } else if path.is_file() {
            files.push(path);
        }
    }
}

fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn cited_ids(line: &str) -> Vec<&str> {
    let bytes = line.as_bytes();
    let mut ids = Vec::new();
    let mut start = 0;
    while start < bytes.len() {
        if !bytes[start].is_ascii_uppercase()
            || start
                .checked_sub(1)
                .is_some_and(|index| is_word(bytes[index]))
        {
            start += 1;
            continue;
        }
        let mut cursor = start;
        while cursor < bytes.len() && cursor - start < 2 && bytes[cursor].is_ascii_uppercase() {
            cursor += 1;
        }
        let digit_start = cursor;
        while cursor < bytes.len() && cursor - digit_start < 2 && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }
        if cursor == digit_start {
            start += 1;
            continue;
        }
        if bytes.get(cursor..cursor + 5) == Some(b" Rev ")
            && bytes.get(cursor + 5).is_some_and(u8::is_ascii_digit)
        {
            cursor += 6;
        }
        if cursor < bytes.len() && is_word(bytes[cursor]) {
            start += 1;
            continue;
        }
        ids.push(&line[start..cursor]);
        start = cursor;
    }
    ids
}

#[test]
fn every_cited_rule_id_resolves() {
    let root = root();
    let table_path = root.join("specs/blocks/rule-ids.txt");
    let table_source = std::fs::read_to_string(&table_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", table_path.display()));
    let mut allowed = table_source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<BTreeSet<_>>();
    let cpu_lane_path = root.join("specs/blocks/cpu-lane.md");
    let cpu_lane_source = std::fs::read_to_string(&cpu_lane_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", cpu_lane_path.display()));
    for (rule, heading) in [
        (
            "CL1",
            "**CL1 — A kernel runs on the host through its own body.**",
        ),
        ("CL2", "**CL2 — Single-threaded, so no barrier.**"),
        ("CL3", "**CL3 — Same numbers.**"),
        ("CL4", "**CL4 — The lane is a gate module.**"),
    ] {
        assert!(
            cpu_lane_source.contains(heading),
            "{rule} exemption must remain backed by the CPU-lane contract",
        );
        allowed.insert(rule);
    }
    let descriptor_kinds = [
        "U8", "U16", "U32", "U64", "I32", "F32", "F64", "L0", "L1", "L2", "L3",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let meta_rules = ["K17", "K24", "PI13", "RN16", "SC14", "TX8"]
        .into_iter()
        .collect::<BTreeSet<_>>();

    let mut files = Vec::new();
    collect_files(&root.join("crates/webgpu-gen/src"), &mut files);
    collect_files(&root.join("crates/webgpu-gen/tests"), &mut files);
    collect_files(&root.join("crates/typegpu-gen/src"), &mut files);
    collect_files(&root.join("crates/typegpu-gen/tests"), &mut files);
    collect_files(&root.join("crates/harness/src"), &mut files);
    collect_files(&root.join("crates/harness/tests"), &mut files);
    collect_files(&root.join("lib"), &mut files);
    collect_files(&root.join("programs"), &mut files);
    collect_files(&root.join("tools"), &mut files);
    files.push(root.join("crates/webgpu-gen/policy.toml"));
    files.extend([
        root.join("crates/facade/src/runtime.rs"),
        root.join("crates/facade/src/generated.rs"),
        root.join("crates/facade/subscript-typegpu.h"),
        root.join("crates/harness/src/native_symbols.generated.rs"),
    ]);

    for path in files {
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        for (index, line) in source.lines().enumerate() {
            for id in cited_ids(line) {
                let subscript_rule = id
                    .strip_prefix('Q')
                    .or_else(|| id.strip_prefix('R'))
                    .is_some_and(|digits| {
                        !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
                    });
                if descriptor_kinds.contains(id) || subscript_rule {
                    continue;
                }
                let relative = path.strip_prefix(&root).unwrap_or(&path);
                let fixture_rule = relative
                    .components()
                    .any(|part| part.as_os_str() == "fixtures")
                    && line.trim_start().starts_with("// expected-rule:");
                let generator_diagnostic = relative.starts_with("crates/typegpu-gen/src")
                    && line.contains(&format!("\"{id}\""));
                let runtime_diagnostic =
                    relative == Path::new("lib/typegpu.ts") && line.contains("print(");
                assert!(
                    !meta_rules.contains(id)
                        || !(fixture_rule || generator_diagnostic || runtime_diagnostic),
                    "{}:{}: meta-rule {id} is cited by a diagnostic or fixture",
                    relative.display(),
                    index + 1,
                );
                assert!(
                    allowed.contains(id),
                    "{}:{}: unresolved rule id {id}",
                    relative.display(),
                    index + 1,
                );
            }
        }
    }
}
