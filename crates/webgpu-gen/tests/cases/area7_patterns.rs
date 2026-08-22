use std::path::Path;

fn repo_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repo root")
        .to_path_buf()
        .join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

#[test]
fn area7_fills_features_and_device_descriptor_are_generated() {
    let generated = subscript_typegpu_webgpu_gen::generate(
        &repo_file("third_party/webgpu-headers/webgpu.yml"),
        &repo_file("crates/webgpu-gen/policy.toml"),
    )
    .expect("area 7 policy generates");

    for declaration in [
        "int32_t subscript_typegpu_get_instance_limits(SubscriptTypegpuInstanceLimits* out);",
        "bool subscript_typegpu_has_instance_feature(SubscriptTypegpuInstanceFeatureName feature);",
        "int32_t subscript_typegpu_adapter_get_limits(SubscriptTypegpuAdapter adapter, SubscriptTypegpuLimits* out);",
        "bool subscript_typegpu_adapter_get_info(SubscriptTypegpuAdapter adapter, SubscriptTypegpuAdapterInfo* out);",
        "bool subscript_typegpu_adapter_has_feature(SubscriptTypegpuAdapter adapter, SubscriptTypegpuFeatureName feature);",
        "int32_t subscript_typegpu_device_get_limits(SubscriptTypegpuDevice device, SubscriptTypegpuLimits* out);",
        "bool subscript_typegpu_device_get_adapter_info(SubscriptTypegpuDevice device, SubscriptTypegpuAdapterInfo* out);",
        "bool subscript_typegpu_device_has_feature(SubscriptTypegpuDevice device, SubscriptTypegpuFeatureName feature);",
        "SubscriptTypegpuFutureId subscript_typegpu_adapter_request_device(SubscriptTypegpuInstance instance, SubscriptTypegpuAdapter adapter);",
        "SubscriptTypegpuFutureId subscript_typegpu_adapter_request_device_with_descriptor(SubscriptTypegpuInstance instance, SubscriptTypegpuAdapter adapter, const SubscriptTypegpuDeviceDescriptor* descriptor);",
    ] {
        assert!(generated.header.contains(declaration), "missing {declaration}");
    }
    assert!(generated.header.contains("size_t requiredFeaturesCount;"));
    assert!(generated
        .header
        .contains("const SubscriptTypegpuLimits* requiredLimits;"));
    assert!(!generated.header.contains("CallbackInfo"));
    assert!(!generated
        .header
        .contains("subscript_typegpu_adapter_get_features"));
    assert!(!generated
        .header
        .contains("subscript_typegpu_device_get_features"));
    assert!(!generated
        .header
        .contains("subscript_typegpu_get_instance_features"));

    assert!(generated.rust.contains("wgpuAdapterInfoFreeMembers(info)"));
    assert!(generated
        .rust
        .contains("runtime::store_adapter_info_strings"));
    assert!(generated
        .rust
        .contains("convert_limits_max_uniform_buffer_binding_size_zero_rule"));
    assert!(generated
        .rust
        .contains("convert_limits_max_storage_buffer_binding_size_zero_rule"));
    assert!(generated
        .rust
        .contains("convert_limits_max_buffer_size_zero_rule"));
}

#[test]
fn retired_freestanding_limits_exclusion_is_dead() {
    let policy = repo_file("crates/webgpu-gen/policy.toml").replace(
        "# --- F18 exclusion rows reachable from the",
        "[[exclude]]\nconstruct = \"get_instance_limits\"\nreason = \"stale generator deferral\"\n\n# --- F18 exclusion rows reachable from the",
    );
    let error = subscript_typegpu_webgpu_gen::generate(
        &repo_file("third_party/webgpu-headers/webgpu.yml"),
        &policy,
    )
    .expect_err("a generated freestanding fill cannot retain its exclusion row");
    assert_eq!(
        error.to_string(),
        "policy error (dead): policy entry `get_instance_limits` was consumed by no generation step"
    );
}
