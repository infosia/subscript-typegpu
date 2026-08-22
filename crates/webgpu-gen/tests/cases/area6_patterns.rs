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
fn area6_records_callbacks_and_error_scope_future_are_generated() {
    let generated = subscript_typegpu_webgpu_gen::generate(
        &repo_file("third_party/webgpu-headers/webgpu.yml"),
        &repo_file("crates/webgpu-gen/policy.toml"),
    )
    .expect("area 6 policy generates");

    assert!(generated.header.contains(
        "bool subscript_typegpu_pop_error_scope_take(SubscriptTypegpuInstance instance, SubscriptTypegpuFutureId future, SubscriptTypegpuErrorRecord* out);"
    ));
    assert!(generated
        .header
        .contains("bool subscript_typegpu_device_next_uncaptured_error(SubscriptTypegpuDevice device, SubscriptTypegpuErrorRecord* out);"));
    assert!(generated
        .header
        .contains("bool subscript_typegpu_device_lost_info(SubscriptTypegpuDevice device, SubscriptTypegpuLostRecord* out);"));
    assert!(!generated.header.contains("CallbackInfo"));
    assert!(generated.rust.contains("WGPUDeviceDescriptor"));
    assert!(generated.rust.contains("uncaptured_error_callback"));
    assert!(generated.rust.contains("device_lost_callback"));
    assert!(generated.rust.contains("complete_record_from_callback"));
}

#[test]
fn device_events_reshape_requires_a_reason() {
    let policy = repo_file("crates/webgpu-gen/policy.toml").replace(
        "reason = \"G1-G4/F6/F11/F14 pop-scope record future plus creation-time device event routing\"\n",
        "",
    );

    let error = subscript_typegpu_webgpu_gen::generate(
        &repo_file("third_party/webgpu-headers/webgpu.yml"),
        &policy,
    )
    .expect_err("device-events without a policy reason must fail");

    assert_eq!(
        error.to_string(),
        "policy error (invalid): `device.pop_error_scope`: reshape rows require a reason"
    );
}
