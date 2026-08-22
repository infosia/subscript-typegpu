//! Capability-query joins fail by the exact stale policy or boundary axis.

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
        .expect_err("invalid capability-query fixture unexpectedly generated")
        .to_string();
    eprintln!("{error}");
    assert_eq!(error, expected);
}

#[test]
fn result_record_boundary_exclusions_reject_duplicates() {
    let policy = repo_file("crates/webgpu-gen/policy.toml").replace(
        "boundary_field_exclusions = [\"backendType\", \"adapterType\", \"vendorID\", \"deviceID\"]",
        "boundary_field_exclusions = [\"backendType\", \"backendType\", \"adapterType\", \"vendorID\", \"deviceID\"]",
    );
    red(
        &support::require_base_mirror!(),
        &policy,
        "api policy error (duplicate): api policy lists `GPUAdapterInfo.boundary_field_exclusions.backendType` more than once",
    );
}

#[test]
fn retired_capability_exclusions_are_dead() {
    let policy = format!(
        "{}\n[[api.exclude]]\nmember = \"GPUAdapter.features\"\nreason = \"stale feature exclusion\"\n",
        repo_file("crates/webgpu-gen/policy.toml")
    );
    red(
        &support::require_base_mirror!(),
        &policy,
        "api policy error (dead): api policy entry `GPUAdapter.features` was consumed by no generation step",
    );
}

#[test]
fn result_record_boundary_exclusions_name_real_fields() {
    let policy = repo_file("crates/webgpu-gen/policy.toml").replace(
        "boundary_field_exclusions = [\"backendType\", \"adapterType\", \"vendorID\", \"deviceID\"]",
        "boundary_field_exclusions = [\"backendType\", \"adapterType\", \"vendorID\", \"deviceID\", \"missing\"]",
    );
    red(
        &support::require_base_mirror!(),
        &policy,
        "api policy error (unknown): policy names `mirror.SubscriptTypegpuAdapterInfo.missing` but the selected IDL/mirror join has no such construct",
    );
}

#[test]
fn result_record_boundary_exclusions_cannot_hide_selected_idl_fields() {
    let policy = repo_file("crates/webgpu-gen/policy.toml").replace(
        "boundary_field_exclusions = [\"backendType\", \"adapterType\", \"vendorID\", \"deviceID\"]",
        "boundary_field_exclusions = [\"vendor\", \"adapterType\", \"vendorID\", \"deviceID\"]",
    );
    red(
        &support::require_base_mirror!(),
        &policy,
        "api policy error (invalid): `GPUAdapterInfo.boundary_field_exclusions.vendor`: boundary-field exclusion names a selected IDL result attribute",
    );
}

#[test]
fn result_record_u64_fields_join_without_narrowing() {
    let mirror = support::require_base_mirror!()
        .replace("maxBufferSize: u64;", "maxBufferSize: u32;")
        .replace("maxBufferSize: u64,", "maxBufferSize: u32,");
    red(
        &mirror,
        &repo_file("crates/webgpu-gen/policy.toml"),
        "api policy error (invalid): `GPUSupportedLimits.maxBufferSize`: IDL result scalar `u64` does not match mirror field `u32`",
    );
}

#[test]
fn feature_probes_join_the_exact_feature_enum() {
    let mirror = support::require_base_mirror!().replace(
        "subscript_typegpu_adapter_has_feature(adapter: SubscriptTypegpuAdapter, feature: SubscriptTypegpuFeatureName)",
        "subscript_typegpu_adapter_has_feature(adapter: SubscriptTypegpuAdapter, feature: SubscriptTypegpuInstanceFeatureName)",
    );
    red(
        &mirror,
        &repo_file("crates/webgpu-gen/policy.toml"),
        "api policy error (invalid): `mirror.subscript_typegpu_adapter_has_feature`: parameter types are [\"SubscriptTypegpuAdapter\", \"SubscriptTypegpuInstanceFeatureName\"], expected [\"SubscriptTypegpuAdapter\", \"SubscriptTypegpuFeatureName\"]",
    );
}

#[test]
fn feature_probes_require_the_supported_features_attribute_type() {
    let idl = gpuweb().replace(
        "readonly attribute GPUSupportedFeatures features;",
        "readonly attribute GPUSupportedLimits features;",
    );
    red_with_idl(
        &idl,
        &support::require_base_mirror!(),
        &repo_file("crates/webgpu-gen/policy.toml"),
        "api policy error (invalid): `GPUAdapter.features`: feature-probe attribute type is `GPUSupportedLimits`, expected `GPUSupportedFeatures`",
    );
}

#[test]
fn feature_probes_require_boolean_results() {
    let mirror = support::require_base_mirror!().replace(
        "subscript_typegpu_adapter_has_feature(adapter: SubscriptTypegpuAdapter, feature: SubscriptTypegpuFeatureName): boolean;",
        "subscript_typegpu_adapter_has_feature(adapter: SubscriptTypegpuAdapter, feature: SubscriptTypegpuFeatureName): i32;",
    );
    red(
        &mirror,
        &repo_file("crates/webgpu-gen/policy.toml"),
        "api policy error (invalid): `mirror.subscript_typegpu_adapter_has_feature`: feature probe must return boolean",
    );
}

#[test]
fn result_record_fills_join_the_exact_output_record() {
    let mirror = support::require_base_mirror!().replace(
        "subscript_typegpu_adapter_get_limits(adapter: SubscriptTypegpuAdapter, out: SubscriptTypegpuLimits | null)",
        "subscript_typegpu_adapter_get_limits(adapter: SubscriptTypegpuAdapter, out: SubscriptTypegpuAdapterInfo | null)",
    );
    red(
        &mirror,
        &repo_file("crates/webgpu-gen/policy.toml"),
        "api policy error (invalid): `mirror.subscript_typegpu_adapter_get_limits`: parameter types are [\"SubscriptTypegpuAdapter\", \"SubscriptTypegpuAdapterInfo | null\"], expected [\"SubscriptTypegpuAdapter\", \"SubscriptTypegpuLimits | null\"]",
    );
}

#[test]
fn result_record_fills_require_a_supported_status_type() {
    let mirror = support::require_base_mirror!().replace(
        "declare function subscript_typegpu_adapter_get_limits(adapter: SubscriptTypegpuAdapter, out: SubscriptTypegpuLimits | null): i32;",
        "declare function subscript_typegpu_adapter_get_limits(adapter: SubscriptTypegpuAdapter, out: SubscriptTypegpuLimits | null): u32;",
    );
    red(
        &mirror,
        &repo_file("crates/webgpu-gen/policy.toml"),
        "api policy error (invalid): `mirror.subscript_typegpu_adapter_get_limits`: result-record fill returns `u32`, expected boolean or i32 status",
    );
}

#[test]
fn excluded_boundary_fields_require_seedable_types() {
    let mirror = support::require_base_mirror!()
        .replace("vendorID: u32;", "vendorID: SubscriptTypegpuBuffer;")
        .replace("vendorID: u32,", "vendorID: SubscriptTypegpuBuffer,");
    red(
        &mirror,
        &repo_file("crates/webgpu-gen/policy.toml"),
        "api policy error (invalid): `mirror.seed.vendorID`: excluded result-record boundary field has unsupported type `SubscriptTypegpuBuffer`",
    );
}

#[test]
fn excluded_boundary_enums_seed_from_the_lowest_value() {
    let generated = subscript_typegpu_webgpu_gen::generate_api(
        &gpuweb(),
        &support::require_base_mirror!(),
        &repo_file("crates/webgpu-gen/policy.toml"),
    )
    .expect("valid capability-query inputs generate");
    assert!(generated.source.contains(
        "new SubscriptTypegpuAdapterInfo(\"\", \"\", \"\", \"\", SubscriptTypegpuBackendType.SUBSCRIPT_TYPEGPU_BACKEND_TYPE_UNDEFINED, SubscriptTypegpuAdapterType.SUBSCRIPT_TYPEGPU_ADAPTER_TYPE_DISCRETE_GPU, 0, 0)"
    ));
}
