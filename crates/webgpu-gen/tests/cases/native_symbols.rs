//! Native symbol generation follows the resolved facade plan.

use std::path::{Path, PathBuf};

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn repo_file(relative: impl AsRef<Path>) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root")
        .join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

#[test]
fn mini_symbol_table_follows_plan_order() {
    let generated =
        subscript_typegpu_webgpu_gen::generate(&fixture("mini.yml"), &fixture("mini.policy.toml"))
            .expect("mini fixture generates");
    assert_eq!(
        generated.export_names,
        [
            "subscript_typegpu_make_hub",
            "subscript_typegpu_hub_pump",
            "subscript_typegpu_hub_release",
            "subscript_typegpu_hub_fetch_gizmo",
            "subscript_typegpu_future_status",
            "subscript_typegpu_future_drop",
            "subscript_typegpu_fetch_gizmo_take",
            "subscript_typegpu_gizmo_get_tag",
            "subscript_typegpu_gizmo_create_part",
            "subscript_typegpu_gizmo_write_part",
            "subscript_typegpu_part_release",
            "subscript_typegpu_gizmo_release",
        ]
    );
}

#[test]
fn symbol_source_has_the_required_table_shape() {
    let generated =
        subscript_typegpu_webgpu_gen::generate(&fixture("mini.yml"), &fixture("mini.policy.toml"))
            .expect("mini fixture generates");
    assert!(generated
        .native_symbols
        .contains("pub fn facade_symbols() -> Vec<(String, *const u8)>"));
    assert!(generated
        .native_symbols
        .contains("pub fn facade_counting_symbols() -> Vec<(String, *const u8)>"));
    assert!(generated
        .native_symbols
        .contains("pub fn facade_export_names() -> &'static [&'static str]"));
    assert!(generated
        .native_symbols
        .contains("use subscript_typegpu_facade as facade;"));
    for (index, name) in generated.export_names.into_iter().enumerate() {
        assert!(generated.native_symbols.contains(&format!(
            "(\"{name}\".to_owned(), facade::{name} as *const u8)"
        )));
        assert!(generated.native_symbols.contains(&format!(
            "(\"{name}\".to_owned(), coverage_{index} as *const u8)"
        )));
        assert!(generated
            .native_symbols
            .contains(&format!("super::coverage_hit({index});")));
    }
}

#[test]
fn full_plan_export_names_are_unique() {
    let generated = subscript_typegpu_webgpu_gen::generate(
        &repo_file("third_party/webgpu-headers/webgpu.yml"),
        &repo_file("crates/webgpu-gen/policy.toml"),
    )
    .expect("pinned facade generates");
    assert!(
        !generated.header.contains("\n\n\n"),
        "export exclusions left a blank declaration gap",
    );
    let mut names = generated.export_names;
    let before = names.len();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), before);
}

#[test]
fn export_exclusion_removes_every_public_and_backend_symbol() {
    let mut policy = fixture("mini.policy.toml");
    policy.push_str(
        "\n[[export_exclude]]\nname = \"subscript_typegpu_gizmo_get_tag\"\nreason = \"the fixture has no public API consumer\"\n",
    );
    let generated = subscript_typegpu_webgpu_gen::generate(&fixture("mini.yml"), &policy)
        .expect("mini fixture with an export exclusion generates");
    assert!(!generated
        .export_names
        .iter()
        .any(|name| name == "subscript_typegpu_gizmo_get_tag"));
    assert!(!generated.header.contains("subscript_typegpu_gizmo_get_tag"));
    assert!(!generated.rust.contains("subscript_typegpu_gizmo_get_tag"));
    assert!(!generated.rust.contains("wgpuGizmoGetTag"));
    assert!(!generated
        .native_symbols
        .contains("subscript_typegpu_gizmo_get_tag"));
}
