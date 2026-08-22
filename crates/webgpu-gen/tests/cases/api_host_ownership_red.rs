//! J13's host-owned device seam is policy-required and fail-loud.

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

fn gpuweb() -> String {
    subscript_typegpu_webgpu_gen::GPUWEB_IDL_INPUTS
        .iter()
        .map(|relative| repo_file(relative))
        .collect::<Vec<_>>()
        .join("\n")
}

fn red(policy: &str, expected: &str) {
    let error = subscript_typegpu_webgpu_gen::generate_api(
        &gpuweb(),
        &support::require_base_mirror!(),
        policy,
    )
    .expect_err("invalid host-ownership fixture unexpectedly generated")
    .to_string();
    eprintln!("{error}");
    assert_eq!(error, expected);
}

fn constructor_row() -> &'static str {
    "[[api.deviations]]\nmember = \"GPUDevice.@constructor\"\npattern = \"host-owned-wrapper\"\nreason = \"JavaScript exposes no GPUDevice constructor; static methods are unavailable, so this API keeps a public owning constructor that accepts private raw-handle fields, acquires and caches one queue reference through subscript_typegpu_device_get_queue, and is paired with hostOwnedGPUDevice, whose GPUHostOwnedDevice wraps a host-owned device, exposes the same creation methods but neither dispose nor destroy, and returns a new owned queue wrapper from each queue call\"\n\n"
}

#[test]
fn cached_queue_constructor_requires_an_ownership_deviation() {
    let policy = repo_file("crates/webgpu-gen/policy.toml").replace(constructor_row(), "");
    red(
        &policy,
        "api policy error (unpoliced): `GPUDevice.@constructor` is reachable from the selected IDL subset but is neither generated, deviation-rowed, nor excluded",
    );
}

#[test]
fn device_ownership_deviation_requires_the_host_owned_wrapper_pattern() {
    let policy = repo_file("crates/webgpu-gen/policy.toml").replace(
        "member = \"GPUDevice.@constructor\"\npattern = \"host-owned-wrapper\"",
        "member = \"GPUDevice.@constructor\"\npattern = \"operation\"",
    );
    red(
        &policy,
        "api policy error (invalid): `GPUDevice.@constructor`: device ownership handoff requires pattern `host-owned-wrapper`, found `operation`",
    );
}

#[test]
fn device_ownership_deviation_rejects_unconsumed_controls() {
    let policy = repo_file("crates/webgpu-gen/policy.toml").replace(
        "pattern = \"host-owned-wrapper\"\nreason =",
        "pattern = \"host-owned-wrapper\"\nnullable_return = true\nreason =",
    );
    red(
        &policy,
        "api policy error (invalid): `GPUDevice.@constructor`: host-owned-wrapper accepts only member, pattern, and reason",
    );
}
