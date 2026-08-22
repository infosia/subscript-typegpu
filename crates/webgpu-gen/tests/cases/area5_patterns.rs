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

fn generate(policy: &str) -> subscript_typegpu_webgpu_gen::Generated {
    subscript_typegpu_webgpu_gen::generate(
        &repo_file("third_party/webgpu-headers/webgpu.yml"),
        policy,
    )
    .expect("area 5 policy generates")
}

#[test]
fn area5_descriptors_nullable_parameters_and_arrays_are_generated() {
    let generated = generate(&repo_file("crates/webgpu-gen/policy.toml"));
    for declaration in [
        "void subscript_typegpu_queue_submit(SubscriptTypegpuQueue queue, size_t commandsCount, const SubscriptTypegpuCommandBuffer* commands);",
        "SubscriptTypegpuBindGroup _Nullable group, size_t dynamicOffsetsCount, const uint32_t* dynamicOffsets",
        "SubscriptTypegpuBuffer _Nullable buffer, uint64_t offset, uint64_t size",
        "size_t bundlesCount, const SubscriptTypegpuRenderBundle* bundles",
        "size_t colorAttachmentsCount;",
        "size_t colorFormatsCount;",
        "const SubscriptTypegpuColor* color",
    ] {
        assert!(
            generated.header.contains(declaration),
            "missing area-5 declaration fragment: {declaration}"
        );
    }
    assert!(generated
        .header
        .contains("SubscriptTypegpuColor clearValue;"));
    assert!(generated
        .header
        .contains("SubscriptTypegpuQuerySet _Nullable occlusionQuerySet;"));
    assert!(generated.rust.contains("convert_render_pass_descriptor"));
}

#[test]
fn each_area5_pair_count_rename_must_match_pointer_name_exactly() {
    let policy = repo_file("crates/webgpu-gen/policy.toml");
    for (construct, valid, invalid, expected) in [
        (
            "render_bundle_encoder_descriptor.color_format_count",
            "construct = \"render_bundle_encoder_descriptor.color_format_count\"\nto = \"colorFormatsCount\"",
            "construct = \"render_bundle_encoder_descriptor.color_format_count\"\nto = \"colorFormatCount\"",
            "colorFormatsCount",
        ),
        (
            "render_pass_descriptor.color_attachment_count",
            "construct = \"render_pass_descriptor.color_attachment_count\"\nto = \"colorAttachmentsCount\"",
            "construct = \"render_pass_descriptor.color_attachment_count\"\nto = \"colorAttachmentCount\"",
            "colorAttachmentsCount",
        ),
        (
            "queue.submit.command_count",
            "construct = \"queue.submit.command_count\"\nto = \"commandsCount\"",
            "construct = \"queue.submit.command_count\"\nto = \"commandCount\"",
            "commandsCount",
        ),
        (
            "compute_pass_encoder.set_bind_group.dynamic_offset_count",
            "construct = \"compute_pass_encoder.set_bind_group.dynamic_offset_count\"\nto = \"dynamicOffsetsCount\"",
            "construct = \"compute_pass_encoder.set_bind_group.dynamic_offset_count\"\nto = \"dynamicOffsetCount\"",
            "dynamicOffsetsCount",
        ),
        (
            "render_pass_encoder.set_bind_group.dynamic_offset_count",
            "construct = \"render_pass_encoder.set_bind_group.dynamic_offset_count\"\nto = \"dynamicOffsetsCount\"",
            "construct = \"render_pass_encoder.set_bind_group.dynamic_offset_count\"\nto = \"dynamicOffsetCount\"",
            "dynamicOffsetsCount",
        ),
        (
            "render_bundle_encoder.set_bind_group.dynamic_offset_count",
            "construct = \"render_bundle_encoder.set_bind_group.dynamic_offset_count\"\nto = \"dynamicOffsetsCount\"",
            "construct = \"render_bundle_encoder.set_bind_group.dynamic_offset_count\"\nto = \"dynamicOffsetCount\"",
            "dynamicOffsetsCount",
        ),
        (
            "render_pass_encoder.execute_bundles.bundle_count",
            "construct = \"render_pass_encoder.execute_bundles.bundle_count\"\nto = \"bundlesCount\"",
            "construct = \"render_pass_encoder.execute_bundles.bundle_count\"\nto = \"bundleCount\"",
            "bundlesCount",
        ),
    ] {
        let error = subscript_typegpu_webgpu_gen::generate(
            &repo_file("third_party/webgpu-headers/webgpu.yml"),
            &policy.replace(valid, invalid),
        )
        .expect_err("mismatched pair count rename must fail")
        .to_string();
        assert_eq!(
            error,
            format!(
                "policy error (invalid): `{construct}`: pair count must be pointer-field name + Count exactly (`{expected}`)"
            )
        );
    }
}
