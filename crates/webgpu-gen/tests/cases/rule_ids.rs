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
    let allowed = table_source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<BTreeSet<_>>();
    let descriptor_kinds = ["U8", "U16", "U32", "U64", "I32", "F32", "F64"]
        .into_iter()
        .collect::<BTreeSet<_>>();

    let mut files = Vec::new();
    collect_files(&root.join("crates/webgpu-gen/src"), &mut files);
    collect_files(&root.join("crates/webgpu-gen/tests"), &mut files);
    collect_files(&root.join("crates/harness/src"), &mut files);
    collect_files(&root.join("crates/harness/tests"), &mut files);
    collect_files(&root.join("programs"), &mut files);
    collect_files(&root.join("tools"), &mut files);
    files.push(root.join("crates/webgpu-gen/policy.toml"));
    files.extend([
        root.join("crates/facade/src/runtime.rs"),
        root.join("crates/facade/src/generated.rs"),
        root.join("crates/facade/subscript-typegpu.h"),
        root.join("crates/harness/src/native_symbols.generated.rs"),
        root.join("lib/subscript-typegpu.generated.d.ts"),
        root.join("lib/wire-enum-aliases.generated.d.ts"),
        root.join("lib/webgpu.ts"),
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
                assert!(
                    allowed.contains(id),
                    "{}:{}: unresolved rule id {id}",
                    path.strip_prefix(&root).unwrap_or(&path).display(),
                    index + 1,
                );
            }
        }
    }
}
