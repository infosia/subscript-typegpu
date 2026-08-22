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
    .expect("area 3 policy generates")
}

#[test]
fn area3_struct_handle_array_nullable_and_sentinel_patterns_are_generated() {
    let policy = repo_file("crates/webgpu-gen/policy.toml");
    let generated = generate(&policy);
    assert!(generated
        .header
        .contains("#pragma clang diagnostic ignored \"-Wnullability-completeness\""));
    assert!(generated
        .header
        .contains("SubscriptTypegpuBuffer _Nullable buffer;"));
    assert!(generated
        .header
        .contains("SubscriptTypegpuSampler _Nullable sampler;"));
    assert!(generated
        .header
        .contains("SubscriptTypegpuTextureView _Nullable textureView;"));
    assert!(generated.header.contains(
        "size_t entriesCount;\n    const SubscriptTypegpuBindGroupLayoutEntry* entries;"
    ));
    assert!(generated.header.contains(
        "size_t bindGroupLayoutsCount;\n    const SubscriptTypegpuBindGroupLayout* bindGroupLayouts;"
    ));
    assert!(generated
        .rust
        .contains(".map(convert_bind_group_layout_entry)"));
    assert!(generated
        .rust
        .contains("bind_group_layouts: source.bind_group_layouts.cast()"));
    assert!(generated
        .rust
        .contains("if value == 0 { WGPU_WHOLE_SIZE } else { value }"));
}

#[test]
fn each_area3_pair_count_rename_must_match_pointer_name_exactly() {
    let policy = repo_file("crates/webgpu-gen/policy.toml");
    for (construct, valid, invalid, expected) in [
        (
            "bind_group_layout_descriptor.entry_count",
            "construct = \"bind_group_layout_descriptor.entry_count\"\nto = \"entriesCount\"",
            "construct = \"bind_group_layout_descriptor.entry_count\"\nto = \"entryCount\"",
            "entriesCount",
        ),
        (
            "bind_group_descriptor.entry_count",
            "construct = \"bind_group_descriptor.entry_count\"\nto = \"entriesCount\"",
            "construct = \"bind_group_descriptor.entry_count\"\nto = \"entryCount\"",
            "entriesCount",
        ),
        (
            "pipeline_layout_descriptor.bind_group_layout_count",
            "construct = \"pipeline_layout_descriptor.bind_group_layout_count\"\nto = \"bindGroupLayoutsCount\"",
            "construct = \"pipeline_layout_descriptor.bind_group_layout_count\"\nto = \"bindGroupLayoutCount\"",
            "bindGroupLayoutsCount",
        ),
    ] {
        let bad = policy.replace(valid, invalid);
        let error = subscript_typegpu_webgpu_gen::generate(
            &repo_file("third_party/webgpu-headers/webgpu.yml"),
            &bad,
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
fn reachable_large_sentinel_requires_a_policy_row() {
    let policy = repo_file("crates/webgpu-gen/policy.toml").replace(
        "\n[[sentinels]]\nconstruct = \"bind_group_entry.size\"\nzero_maps_to = \"constant.whole_size\"\nreason = \"C3/F15 maps facade size 0 to the unrepresentable WGPU_WHOLE_SIZE sentinel\"\n",
        "\n",
    );
    let error = subscript_typegpu_webgpu_gen::generate(
        &repo_file("third_party/webgpu-headers/webgpu.yml"),
        &policy,
    )
    .expect_err("reachable whole-size sentinel without policy must fail")
    .to_string();
    assert_eq!(
        error,
        "policy error (unpoliced): `bind_group_entry.size` is reachable from the subset but has neither a rule pattern nor a policy entry"
    );
}
