//! Program suite naming and module ownership (T1).

use std::path::Path;

use crate::{collect_ts_files, repository_root};

/// Each suite module belongs to an entry with the same two-digit id (T1).
#[test]
fn every_suite_module_is_reached_by_its_program() {
    let root = repository_root();
    let directory = root.join("programs");
    let mut files = Vec::new();
    collect_ts_files(&directory, true, &mut files);
    let mut modules = std::collections::BTreeMap::<_, Vec<_>>::new();
    let mut entries = Vec::new();
    for path in &files {
        let relative = path.strip_prefix(&directory).expect("suite path");
        let first = relative
            .iter()
            .next()
            .and_then(|part| part.to_str())
            .expect("suite name");
        match first.as_bytes() {
            [b'm', a @ b'0'..=b'9', b @ b'0'..=b'9', ..] => {
                modules.entry([*a, *b]).or_default().push(relative);
            }
            [b'a' | b'b' | b'x', a, b, b'-', ..] if relative.parent() == Some(Path::new("")) => {
                entries.push(([*a, *b], path));
            }
            _ => {}
        }
    }
    for (id, entry) in entries {
        if let Some(required) = modules.get_mut(&id) {
            let loaded = subscript_typegpu_gen::load_program_files(&root.join("lib"), entry)
                .unwrap_or_else(|error| panic!("load {}: {error}", entry.display()));
            let entry_name = &loaded.last().expect("suite entry").name;
            let prefix = Path::new(entry_name).parent().expect("entry directory");
            required.retain(|path| {
                !loaded
                    .iter()
                    .any(|file| Path::new(&file.name).strip_prefix(prefix) == Ok(*path))
            });
        }
    }
    let missing: Vec<_> = modules.into_values().flatten().collect();
    assert!(
        missing.is_empty(),
        "suite modules with no matching importer: {missing:?}"
    );
}

/// The hygiene script rejects each invalid path in an isolated suite.
#[cfg(unix)]
#[test]
fn hygiene_rejects_invalid_suite_module_paths() {
    use std::fs;
    use std::process::Command;

    let directory = std::env::temp_dir().join(format!("typegpu-hygiene-{}", std::process::id()));
    fs::create_dir(&directory).expect("create temporary repository");
    fs::create_dir(directory.join("tools")).expect("create tools directory");
    fs::copy(
        repository_root().join("tools/hygiene.sh"),
        directory.join("tools/hygiene.sh"),
    )
    .expect("copy hygiene script");
    assert!(Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("initialize fixture")
        .success());
    let paths = [
        "programs/m30/Bad/util.ts",
        "programs/m30/inner/bad_name.ts",
        "programs/m30/inner/main.typegpu.ts",
        "programs/m30/inner/ok-name.ts",
    ];
    for name in paths {
        let path = directory.join(name);
        fs::create_dir_all(path.parent().expect("parent")).expect("create module directory");
        fs::write(path, "").expect("create module");
    }
    fs::create_dir_all(directory.join("crates/fixture")).expect("create crate directory");
    fs::write(directory.join("crates/fixture/Cargo.toml"), "").expect("create crate manifest");
    fs::write(directory.join("Cargo.toml"), "").expect("create manifest");
    let result = Command::new("sh")
        .arg("tools/hygiene.sh")
        .current_dir(&directory)
        .output()
        .expect("run hygiene");
    fs::remove_dir_all(directory).expect("remove temporary repository");
    let stderr = String::from_utf8(result.stderr).expect("hygiene diagnostics");
    eprint!("{stderr}");
    assert_eq!(result.status.code(), Some(1));
    let messages: std::collections::BTreeSet<_> = stderr.lines().collect();
    assert_eq!(
        messages,
        [
            "hygiene: invalid suite module: programs/m30/Bad/util.ts",
            "hygiene: invalid suite module: programs/m30/inner/bad_name.ts",
            "hygiene: generated support module exists: programs/m30/inner/main.typegpu.ts",
        ]
        .into_iter()
        .collect()
    );
}
