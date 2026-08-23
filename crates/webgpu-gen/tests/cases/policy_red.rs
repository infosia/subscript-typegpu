//! The four policy failure classes (F18), each demonstrated red
//! against a bad fixture with the exact named error (T7).

use std::path::PathBuf;

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn repo_file(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root")
        .join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn generate_error(policy_fixture: &str) -> String {
    subscript_typegpu_webgpu_gen::generate(&fixture("mini.yml"), &fixture(policy_fixture))
        .err()
        .unwrap_or_else(|| panic!("{policy_fixture} must fail generation"))
        .to_string()
}

fn generate_fixture_error(yml_fixture: &str, policy_fixture: &str) -> String {
    subscript_typegpu_webgpu_gen::generate(&fixture(yml_fixture), &fixture(policy_fixture))
        .err()
        .unwrap_or_else(|| panic!("{yml_fixture} with {policy_fixture} must fail generation"))
        .to_string()
}

#[test]
fn unknown_policy_entry_fails() {
    assert_eq!(
        generate_error("policy-unknown.toml"),
        "policy error (unknown): policy names `gizmo.frobnicate` but webgpu.yml has no such construct"
    );
}

#[test]
fn dead_policy_entry_fails() {
    assert_eq!(
        generate_error("policy-dead.toml"),
        "policy error (dead): policy entry `doodad.ping` was consumed by no generation step"
    );
}

#[test]
fn duplicate_policy_entry_fails() {
    assert_eq!(
        generate_error("policy-duplicate.toml"),
        "policy error (duplicate): policy lists `gizmo.get_tag` more than once"
    );
}

#[test]
fn unpoliced_construct_fails() {
    assert_eq!(
        generate_error("policy-unpoliced.toml"),
        "policy error (unpoliced): `gizmo.self_destruct` is reachable from the subset but has neither a rule pattern nor a policy entry"
    );
}

#[test]
fn unpoliced_freestanding_function() {
    assert_eq!(
        generate_fixture_error(
            "policy-unpoliced-freestanding.yml",
            "policy-unpoliced-freestanding.toml",
        ),
        "policy error (unpoliced): `inspect_hub` is reachable from the subset but has neither a rule pattern nor a policy entry"
    );
}

#[test]
fn host_only_construct_cannot_also_be_excluded() {
    let yml = repo_file("third_party/webgpu-headers/webgpu.yml");
    let mut policy = repo_file("crates/webgpu-gen/policy.toml");
    policy.push_str(
        "\n[[exclude]]\nconstruct = \"instance.create_surface\"\nreason = \"conflict fixture\"\n",
    );
    let error = subscript_typegpu_webgpu_gen::generate(&yml, &policy)
        .expect_err("host_only plus exclude must fail")
        .to_string();
    assert_eq!(
        error,
        "policy error (invalid): `wgpuInstanceCreateSurface`: construct is both host_only and exclude"
    );
}
