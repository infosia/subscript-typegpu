//! Instance creation layout follows the pinned header model.

use std::path::Path;

fn repo_file(relative: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("repo root")
            .to_path_buf()
            .join(relative),
    )
    .expect("read repository fixture")
}

fn red(yml: &str, expected: &str) {
    let error =
        subscript_typegpu_webgpu_gen::generate(yml, &repo_file("crates/webgpu-gen/policy.toml"))
            .expect_err("invalid instance descriptor fixture unexpectedly generated")
            .to_string();
    eprintln!("{error}");
    assert_eq!(error, expected);
}

#[test]
fn instance_creation_requires_the_pinned_extensible_descriptor_kind() {
    let yml = repo_file("third_party/webgpu-headers/webgpu.yml").replacen(
        "  - name: instance_descriptor\n    doc: |\n      TODO\n    type: extensible",
        "  - name: instance_descriptor\n    doc: |\n      TODO\n    type: standalone",
        1,
    );
    red(
        &yml,
        "policy error (invalid): `struct.instance_descriptor`: instance creation requires an extensible instance descriptor, found `standalone`",
    );
}

#[test]
fn instance_creation_rejects_a_nonimmutable_instance_array() {
    let yml = repo_file("third_party/webgpu-headers/webgpu.yml").replacen(
        "        type: array<enum.instance_feature_name>\n        pointer: immutable",
        "        type: array<enum.instance_feature_name>\n        pointer: mutable",
        1,
    );
    red(
        &yml,
        "policy error (invalid): `struct.instance_descriptor`: instance descriptor array `required_features` must be immutable",
    );
}

#[test]
fn instance_creation_rejects_an_unmodelled_instance_member() {
    let yml = repo_file("third_party/webgpu-headers/webgpu.yml");
    let start = yml
        .find("  - name: instance_descriptor")
        .expect("instance descriptor fixture");
    let (before, instance_and_after) = yml.split_at(start);
    let yml = format!("{before}{}", instance_and_after.replacen(
        "      - name: required_features",
        "      - name: mystery\n        doc: TODO\n        type: uint32\n      - name: required_features",
        1,
    ));
    red(
        &yml,
        "policy error (invalid): `struct.instance_descriptor`: unsupported instance descriptor member `mystery` type `uint32`",
    );
}

#[test]
fn function_table_emitter_contains_no_handwritten_instance_type_name() {
    let source = repo_file("crates/webgpu-gen/src/emit_rust.rs");
    assert!(
        !source.contains("WGPUInstanceDescriptor"),
        "function table emission must take the instance descriptor name from the yml plan"
    );
}
