//! Device-descriptor joins fail on every policy, IDL, or mirror axis.

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

fn red(mirror: &str, policy: &str, expected: &str) {
    red_with_idl(&gpuweb(), mirror, policy, expected);
}

fn red_with_idl(gpuweb: &str, mirror: &str, policy: &str, expected: &str) {
    let error = subscript_typegpu_webgpu_gen::generate_api(gpuweb, mirror, policy)
        .expect_err("invalid device-descriptor fixture unexpectedly generated")
        .to_string();
    eprintln!("{error}");
    assert_eq!(error, expected);
}

#[test]
fn required_limits_require_the_exact_idl_record_shape() {
    let idl = gpuweb().replace(
        "record<DOMString, (GPUSize64 or undefined)> requiredLimits = {};",
        "record<USVString, (GPUSize64 or undefined)> requiredLimits = {};",
    );
    red_with_idl(
        &idl,
        &support::require_base_mirror!(),
        &repo_file("crates/webgpu-gen/policy.toml"),
        "api policy error (invalid): `GPUDeviceDescriptor.requiredLimits`: dictionary-required-limits requires record<DOMString, (GPUSize64 or undefined)>",
    );
}

#[test]
fn required_limits_require_a_nullable_boundary_aggregate() {
    let mirror = support::require_base_mirror!()
        .replace(
            "requiredLimits: SubscriptTypegpuLimits | null;",
            "requiredLimits: SubscriptTypegpuLimits;",
        )
        .replace(
            "requiredLimits: SubscriptTypegpuLimits | null,",
            "requiredLimits: SubscriptTypegpuLimits,",
        );
    red(
        &mirror,
        &repo_file("crates/webgpu-gen/policy.toml"),
        "api policy error (invalid): `GPUDeviceDescriptor.requiredLimits`: dictionary-required-limits requires a nullable mirror aggregate, found `SubscriptTypegpuLimits`",
    );
}

#[test]
fn required_limits_source_must_join_the_boundary_aggregate() {
    let policy = repo_file("crates/webgpu-gen/policy.toml").replace(
        "required_limits_source = \"GPUSupportedLimits\"",
        "required_limits_source = \"GPUAdapterInfo\"",
    );
    red(
        &support::require_base_mirror!(),
        &policy,
        "api policy error (invalid): `GPUDeviceDescriptor.requiredLimits`: required-limits source `GPUAdapterInfo` joins `SubscriptTypegpuAdapterInfo`, but the mirror field declares `SubscriptTypegpuLimits | null`",
    );
}

#[test]
fn required_limits_pin_the_u32_unspecified_sentinel() {
    let policy = repo_file("crates/webgpu-gen/policy.toml").replace(
        "required_limits_u32_unspecified = 4294967295",
        "required_limits_u32_unspecified = 0",
    );
    red(
        &support::require_base_mirror!(),
        &policy,
        "api policy error (invalid): `GPUDeviceDescriptor.requiredLimits`: required-limits u32 unspecified sentinel is `0`, expected `4294967295`",
    );
}

#[test]
fn required_limits_controls_are_complete() {
    let policy = repo_file("crates/webgpu-gen/policy.toml")
        .replace("required_limits_source = \"GPUSupportedLimits\"\n", "");
    red(
        &support::require_base_mirror!(),
        &policy,
        "api policy error (invalid): `GPUDeviceDescriptor.requiredLimits`: dictionary deviation fields do not match pattern `dictionary-required-limits`",
    );
}

#[test]
fn optional_async_descriptors_require_nullable_mirror_parameters() {
    let mirror = support::require_base_mirror!().replace(
        "descriptor: SubscriptTypegpuDeviceDescriptor | null): SubscriptTypegpuFutureId;",
        "descriptor: SubscriptTypegpuDeviceDescriptor): SubscriptTypegpuFutureId;",
    );
    red(
        &mirror,
        &repo_file("crates/webgpu-gen/policy.toml"),
        "api policy error (invalid): `GPUAdapter.requestDevice.argument.descriptor`: optional async descriptor `GPUDeviceDescriptor` requires nullable mirror parameter `SubscriptTypegpuDeviceDescriptor`",
    );
}

#[test]
fn optional_async_descriptors_require_the_empty_dictionary_default() {
    let idl = gpuweb().replace(
        "requestDevice(optional GPUDeviceDescriptor descriptor = {});",
        "requestDevice(optional GPUDeviceDescriptor descriptor);",
    );
    red_with_idl(
        &idl,
        &support::require_base_mirror!(),
        &repo_file("crates/webgpu-gen/policy.toml"),
        "api policy error (invalid): `GPUAdapter.requestDevice.argument.descriptor`: optional async descriptors require the pinned empty-dictionary default",
    );
}

#[test]
fn required_limits_source_fields_must_be_direct_numeric_scalars() {
    let idl = gpuweb().replace(
        "readonly attribute unsigned long maxBindGroups;",
        "readonly attribute GPUFeatureName maxBindGroups;",
    );
    let mirror = support::require_base_mirror!()
        .replace(
            "maxBindGroups: u32;",
            "maxBindGroups: SubscriptTypegpuFeatureName;",
        )
        .replace(
            "maxBindGroups: u32,",
            "maxBindGroups: SubscriptTypegpuFeatureName,",
        );
    red_with_idl(
        &idl,
        &mirror,
        &repo_file("crates/webgpu-gen/policy.toml"),
        "api policy error (invalid): `GPUSupportedLimits.maxBindGroups`: required-limits fields must be direct u32 or u64 scalars, found `GPUFeatureName`",
    );
}

#[test]
fn optional_operation_descriptors_require_nullable_mirror_parameters() {
    let mirror = support::require_base_mirror!().replace(
        "subscript_typegpu_device_create_sampler(device: SubscriptTypegpuDevice, descriptor: SubscriptTypegpuSamplerDescriptor | null)",
        "subscript_typegpu_device_create_sampler(device: SubscriptTypegpuDevice, descriptor: SubscriptTypegpuSamplerDescriptor)",
    );
    red(
        &mirror,
        &repo_file("crates/webgpu-gen/policy.toml"),
        "api policy error (invalid): `GPUDevice.createSampler.argument.descriptor`: optional operation descriptor `GPUSamplerDescriptor` requires nullable mirror parameter `SubscriptTypegpuSamplerDescriptor`",
    );
}
