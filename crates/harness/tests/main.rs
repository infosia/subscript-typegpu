use std::collections::BTreeSet;
use std::path::PathBuf;

mod api;
mod c_layout;
mod differential;
mod library;
mod live;
mod rejections;
mod runtime;
mod traps;
mod wgsl_goldens;

// Re-pin with specs/subscript-typegpu-project-plan.md section 5,
// "The substrate generator".
const FACADE_EXPORT_COUNT: usize = 163;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("harness crate is under the repository root")
        .to_path_buf()
}

fn generated_exports() -> BTreeSet<String> {
    let path = repository_root().join("crates/facade/src/generated.rs");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let mut names = BTreeSet::new();
    let mut saw_no_mangle = false;
    for line in source.lines() {
        let line = line.trim();
        if line == "#[no_mangle]" {
            saw_no_mangle = true;
            continue;
        }
        if !saw_no_mangle || line.is_empty() {
            continue;
        }
        let Some(signature) = line.strip_prefix("pub extern \"C\" fn ") else {
            saw_no_mangle = false;
            continue;
        };
        saw_no_mangle = false;
        let name = signature
            .split_once('(')
            .map_or(signature, |(name, _)| name)
            .trim();
        if name.starts_with("subscript_typegpu_") {
            names.insert(name.to_owned());
        }
    }
    names
}

#[test]
fn facade_symbol_table_matches_no_mangle_exports() {
    let exports = generated_exports();
    let table = subscript_typegpu_harness::native_symbols_generated::facade_symbols()
        .into_iter()
        .map(|(name, _)| name)
        .collect::<BTreeSet<_>>();
    if let Some(name) = exports.difference(&table).next() {
        panic!("facade export missing from symbol table: {name}");
    }
    if let Some(name) = table.difference(&exports).next() {
        panic!("symbol table name missing from facade exports: {name}");
    }
}
