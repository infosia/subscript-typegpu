//! J9's API policy is checked in both directions.
//! Bad policy cannot name nothing, classify twice, go unused, omit a reachable member,
//! reuse a retired pattern, or misclassify a generated pattern.

use std::path::Path;

use crate::support;

fn repo_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repo root")
        .to_path_buf()
        .join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

fn generate_error(policy: &str) -> Option<String> {
    let gpuweb = format!(
        "{}\n{}",
        repo_file("third_party/gpuweb/spec/index.bs"),
        repo_file("third_party/gpuweb/spec/sections/copies.bs"),
    );
    let mirror = support::base_mirror()?;
    Some(
        subscript_typegpu_webgpu_gen::generate_api(&gpuweb, mirror, policy)
            .expect_err("invalid API policy unexpectedly generated")
            .to_string(),
    )
}

fn without(policy: &str, block: &str) -> String {
    assert!(policy.contains(block), "red fixture block moved");
    policy.replacen(block, "", 1)
}

#[test]
fn api_policy_rejects_unknown_and_retired_entries() {
    let policy = format!(
        "{}\n[[api.generate]]\nmember = \"GPU.notReal\"\npattern = \"operation\"\n",
        repo_file("crates/webgpu-gen/policy.toml")
    );
    let Some(error) = generate_error(&policy) else {
        return;
    };
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (unknown): policy names `GPU.notReal` but the selected IDL/mirror join has no such construct"
    );

    let policy = repo_file("crates/webgpu-gen/policy.toml").replacen(
        "member = \"GPUBuffer.size\"\npattern = \"attribute-accessor\"",
        "member = \"GPUBuffer.size\"\npattern = \"attribute-method\"",
        1,
    );
    let Some(error) = generate_error(&policy) else {
        return;
    };
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUBuffer.size`: pattern `attribute-method` does not match the IDL member kind"
    );
}

#[test]
fn api_policy_rejects_generated_patterns_on_deviation_rows() {
    let policy = repo_file("crates/webgpu-gen/policy.toml").replacen(
        "[[api.generate]]\nmember = \"GPUBuffer.size\"\npattern = \"attribute-accessor\"",
        "[[api.deviations]]\nmember = \"GPUBuffer.size\"\npattern = \"attribute-accessor\"\nreason = \"generated pattern red fixture\"",
        1,
    );
    let Some(error) = generate_error(&policy) else {
        return;
    };
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUBuffer.size`: pattern `attribute-accessor` does not match the IDL member kind"
    );
}

#[test]
fn api_policy_rejects_dead_entries() {
    let policy = format!(
        "{}\n[[api.exclude]]\nmember = \"GPUBuffer.destroy\"\nreason = \"dead red fixture\"\n",
        repo_file("crates/webgpu-gen/policy.toml")
    );
    let Some(error) = generate_error(&policy) else {
        return;
    };
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (dead): api policy entry `GPUBuffer.destroy` was consumed by no generation step"
    );
}

#[test]
fn api_policy_rejects_duplicate_entries() {
    let policy = format!(
        "{}\n[[api.generate]]\nmember = \"GPUBuffer.destroy\"\npattern = \"operation\"\n",
        repo_file("crates/webgpu-gen/policy.toml")
    );
    let Some(error) = generate_error(&policy) else {
        return;
    };
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (duplicate): api policy lists `GPUBuffer.destroy` more than once"
    );
}

#[test]
fn api_policy_rejects_unpoliced_members() {
    let policy = without(
        &repo_file("crates/webgpu-gen/policy.toml"),
        "[[api.generate]]\nmember = \"GPUBuffer.destroy\"\npattern = \"operation\"\n\n",
    );
    let Some(error) = generate_error(&policy) else {
        return;
    };
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (unpoliced): `GPUBuffer.destroy` is reachable from the selected IDL subset but is neither generated, deviation-rowed, nor excluded"
    );
}
