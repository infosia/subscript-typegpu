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
fn area2_struct_patterns_and_pair_rename_are_generated() {
    let generated = subscript_typegpu_webgpu_gen::generate(
        &repo_file("third_party/webgpu-headers/webgpu.yml"),
        &repo_file("crates/webgpu-gen/policy.toml"),
    )
    .expect("area 2 policy generates");
    assert!(generated.header.contains(
        "size_t viewFormatsCount;\n    const SubscriptTypegpuTextureFormat* viewFormats;"
    ));
    assert!(!generated.header.contains("viewFormatCount"));
    assert!(generated.header.contains("SubscriptTypegpuExtent3D size;"));
    assert!(generated.header.contains("float lodMinClamp;"));
    assert!(generated.header.contains("uint16_t maxAnisotropy;"));
    assert!(generated.header.contains(
        "const SubscriptTypegpuExtent3D* extent, size_t dataCount, const uint8_t* data);"
    ));
    assert!(generated.rust.contains(
        "if descriptor.is_null() {\n        // SAFETY: webgpu.yml marks this descriptor optional."
    ));
}

#[test]
fn pair_count_rename_must_match_pointer_name_exactly() {
    let policy = repo_file("crates/webgpu-gen/policy.toml")
        .replace("to = \"viewFormatsCount\"", "to = \"viewFormatCount\"");
    let error = subscript_typegpu_webgpu_gen::generate(
        &repo_file("third_party/webgpu-headers/webgpu.yml"),
        &policy,
    )
    .expect_err("mismatched pair count rename must fail")
    .to_string();
    assert_eq!(
        error,
        "policy error (invalid): `texture_descriptor.view_format_count`: pair count must be pointer-field name + Count exactly (`viewFormatsCount`)"
    );
}
