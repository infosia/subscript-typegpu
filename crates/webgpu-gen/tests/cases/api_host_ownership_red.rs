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
    "[[api.deviations]]\nmember = \"GPUDevice.@constructor\"\npattern = \"host-owned-wrapper\"\nreason = \"JavaScript exposes no GPUDevice constructor. The owning constructor and GPUHostOwnedDevice.create each acquire and cache one queue reference. Both classes implement GPUDeviceCreation and lend their cached queue. GPUHostOwnedDevice.dispose releases only the queue reference. The host owns the device and destroys it.\"\n\n"
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
        "api policy error (invalid): `GPUDevice.@constructor`: device ownership transfer requires pattern `host-owned-wrapper`, found `operation`",
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

#[test]
fn creation_interface_requires_a_policy_row() {
    let policy = repo_file("crates/webgpu-gen/policy.toml");
    let start = policy
        .find("[[api.deviations]]\nmember = \"GPUDeviceCreation.@interface\"")
        .expect("interface row");
    let end = start + policy[start..].find("\n\n").expect("row end") + 2;
    let policy = format!("{}{}", &policy[..start], &policy[end..]);
    red(&policy, "api policy error (unpoliced): `GPUDeviceCreation.@interface` is reachable from the selected IDL subset but is neither generated, deviation-rowed, nor excluded");
}

#[test]
fn creation_interface_rejects_shape_controls() {
    let policy = repo_file("crates/webgpu-gen/policy.toml").replace(
        "pattern = \"device-creation-interface\"\nreason =",
        "pattern = \"device-creation-interface\"\nnullable_return = true\nreason =",
    );
    red(&policy, "api policy error (invalid): `GPUDeviceCreation.@interface`: device-creation-interface accepts only member, pattern, and reason");
}

#[test]
fn device_wrappers_share_creation_signatures_and_cache_their_queues() {
    let api = subscript_typegpu_webgpu_gen::generate_api(
        &gpuweb(),
        &support::require_base_mirror!(),
        &repo_file("crates/webgpu-gen/policy.toml"),
    )
    .expect("generate device wrappers")
    .source;
    let creation = api
        .split("export interface GPUDeviceCreation {")
        .nth(1)
        .expect("creation interface")
        .split("\n}")
        .next()
        .expect("interface body");
    for name in ["GPUDevice", "GPUHostOwnedDevice"] {
        let class = api
            .split(&format!(
                "export class {name} implements GPUDeviceCreation {{"
            ))
            .nth(1)
            .expect("implementing class")
            .split("\n}\n")
            .next()
            .expect("class body");
        for signature in creation
            .lines()
            .map(str::trim)
            .filter(|line| line.ends_with(';'))
        {
            let signature = signature.trim_end_matches(';');
            let concrete = signature
                .replace(
                    "GPUSamplerDescriptor | null)",
                    "GPUSamplerDescriptor | null = null)",
                )
                .replace(
                    "GPUCommandEncoderDescriptor | null)",
                    "GPUCommandEncoderDescriptor | null = null)",
                );
            assert!(
                class.contains(&format!("{concrete} {{"))
                    || class.contains(&format!("async {concrete} {{")),
                "{name} lacks {signature}"
            );
        }
        assert_eq!(
            class
                .matches("subscript_typegpu_device_get_queue(this.device)")
                .count(),
            1
        );
        assert!(class.contains("get queue(): GPUQueue {\n    return this.queueValue;"));
        assert!(class.contains("[Symbol.dispose](): void {\n    this.dispose();"));
        if name == "GPUHostOwnedDevice" {
            assert!(class.contains("private queueValue: GPUQueue;"));
            assert!(class.contains("dispose(): void {\n    this.queueValue.dispose();\n  }"));
            assert!(!class.contains("subscript_typegpu_device_release"));
            assert!(!class.contains("destroy():"));
        }
    }
}
