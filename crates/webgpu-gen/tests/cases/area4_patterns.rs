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
    .expect("area 4 policy generates")
}

#[test]
fn area4_chain_descriptors_async_and_scalar_patterns_are_generated() {
    let generated = generate(&repo_file("crates/webgpu-gen/policy.toml"));
    assert!(generated.header.contains(
        "typedef struct SubscriptTypegpuShaderModuleDescriptor {\n    SubscriptTypegpuStringView label;\n    SubscriptTypegpuStringView code;"
    ));
    for forbidden in ["WGPUChainedStruct", "WGPUShaderSourceWGSL", "nextInChain"] {
        assert!(
            !generated.header.contains(forbidden),
            "private chain spelling leaked into subscript-typegpu.h: {forbidden}"
        );
    }
    assert!(generated.rust.contains("struct WGPUShaderSourceWGSL"));
    assert!(generated
        .rust
        .contains("s_type: WGPUSType_ShaderSourceWGSL"));
    assert!(generated
        .header
        .contains("SubscriptTypegpuPipelineLayout _Nullable layout;"));
    assert!(generated
        .header
        .contains("const SubscriptTypegpuDepthStencilState* depthStencil;"));
    assert!(generated
        .header
        .contains("const SubscriptTypegpuFragmentState* fragment;"));
    assert!(generated
        .header
        .contains("const SubscriptTypegpuBlendState* blend;"));
    assert!(generated
        .header
        .contains("SubscriptTypegpuColorWriteMask writeMask;"));
    assert!(generated.header.contains(
        "subscript_typegpu_device_create_compute_pipeline_async_begin(SubscriptTypegpuInstance instance, SubscriptTypegpuDevice device, const SubscriptTypegpuComputePipelineDescriptor* descriptor)"
    ));
    assert!(generated.header.contains(
        "subscript_typegpu_create_compute_pipeline_async_take(SubscriptTypegpuInstance instance, SubscriptTypegpuFutureId future)"
    ));
    assert!(generated.header.contains(
        "subscript_typegpu_device_create_render_pipeline_async_begin(SubscriptTypegpuInstance instance, SubscriptTypegpuDevice device, const SubscriptTypegpuRenderPipelineDescriptor* descriptor)"
    ));
    assert!(generated
        .header
        .contains("subscript_typegpu_create_render_pipeline_async_take(SubscriptTypegpuInstance instance, SubscriptTypegpuFutureId future)"));
    assert!(generated.header.contains(
        "subscript_typegpu_compute_pipeline_get_bind_group_layout(SubscriptTypegpuComputePipeline computePipeline, uint32_t groupIndex)"
    ));
    assert_eq!(
        generated
            .rust
            .matches("const WGPUCreatePipelineAsyncStatus_Success: i32")
            .count(),
        1,
        "compute and render callbacks share one backend status enum"
    );
}

#[test]
fn each_area4_pair_count_rename_must_match_pointer_name_exactly() {
    let policy = repo_file("crates/webgpu-gen/policy.toml");
    for (construct, valid, invalid, expected) in [
        (
            "compute_state.constant_count",
            "construct = \"compute_state.constant_count\"\nto = \"constantsCount\"",
            "construct = \"compute_state.constant_count\"\nto = \"constantCount\"",
            "constantsCount",
        ),
        (
            "vertex_buffer_layout.attribute_count",
            "construct = \"vertex_buffer_layout.attribute_count\"\nto = \"attributesCount\"",
            "construct = \"vertex_buffer_layout.attribute_count\"\nto = \"attributeCount\"",
            "attributesCount",
        ),
        (
            "vertex_state.constant_count",
            "construct = \"vertex_state.constant_count\"\nto = \"constantsCount\"",
            "construct = \"vertex_state.constant_count\"\nto = \"constantCount\"",
            "constantsCount",
        ),
        (
            "vertex_state.buffer_count",
            "construct = \"vertex_state.buffer_count\"\nto = \"buffersCount\"",
            "construct = \"vertex_state.buffer_count\"\nto = \"bufferCount\"",
            "buffersCount",
        ),
        (
            "fragment_state.constant_count",
            "construct = \"fragment_state.constant_count\"\nto = \"constantsCount\"",
            "construct = \"fragment_state.constant_count\"\nto = \"constantCount\"",
            "constantsCount",
        ),
        (
            "fragment_state.target_count",
            "construct = \"fragment_state.target_count\"\nto = \"targetsCount\"",
            "construct = \"fragment_state.target_count\"\nto = \"targetCount\"",
            "targetsCount",
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

#[test]
fn shader_chain_flattening_requires_a_policy_row() {
    let policy = repo_file("crates/webgpu-gen/policy.toml").replace(
        "\n[[chain_flattenings]]\nconstruct = \"shader_module_descriptor.shader_source_WGSL\"\nfields = [\"code\"]\nreason = \"PL2/F12 exposes WGSL code on the base descriptor while constructing the extension chain internally\"\n",
        "\n",
    );
    let error = subscript_typegpu_webgpu_gen::generate(
        &repo_file("third_party/webgpu-headers/webgpu.yml"),
        &policy,
    )
    .expect_err("unlisted extension flattening must fail")
    .to_string();
    assert_eq!(
        error,
        "policy error (unpoliced): `shader_module_descriptor.shader_source_WGSL` is reachable from the subset but has neither a rule pattern nor a policy entry"
    );
}

#[test]
fn double_underscore_in_yml_keeps_a_literal_enum_name_underscore() {
    let generated = generate(&repo_file("crates/webgpu-gen/policy.toml"));
    assert!(generated
        .header
        .contains("SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UNORM10_10_10_2 = 0x00000028"));
    assert!(!generated
        .header
        .contains("SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UNORM1010102"));
}

#[test]
fn mixed_case_yml_enum_words_titlecase_the_first_character() {
    let generated = generate(&repo_file("crates/webgpu-gen/policy.toml"));
    assert!(generated
        .header
        .contains("SUBSCRIPT_TYPEGPU_BACKEND_TYPE_OPEN_GL = 0x00000007"));
    assert!(generated
        .header
        .contains("SUBSCRIPT_TYPEGPU_BACKEND_TYPE_OPEN_GLES = 0x00000008"));
    assert!(!generated
        .header
        .contains("SUBSCRIPT_TYPEGPU_BACKEND_TYPE_OPENGL ="));
    assert!(!generated
        .header
        .contains("SUBSCRIPT_TYPEGPU_BACKEND_TYPE_OPENGLES ="));
}
