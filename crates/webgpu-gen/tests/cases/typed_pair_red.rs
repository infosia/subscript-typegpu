//! S1 typed-pair guard: policy can derive only float siblings.

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
fn typed_pair_rejects_a_non_float_element() {
    let policy = repo_file("crates/webgpu-gen/policy.toml");
    let row = "source = \"queue.write_buffer\"\nelement = \"float\"";
    assert!(policy.contains(row), "typed-pair red fixture row moved");
    let invalid = policy.replacen(
        row,
        "source = \"queue.write_buffer\"\nelement = \"uint32_t\"",
        1,
    );
    let error = subscript_typegpu_webgpu_gen::generate(
        &repo_file("third_party/webgpu-headers/webgpu.yml"),
        &invalid,
    )
    .expect_err("non-float typed pair unexpectedly generated")
    .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "policy error (invalid): `typed-pair.queue.write_buffer`: typed-pair element must be `float`, found `uint32_t`"
    );
}

#[test]
fn typed_pair_rejects_a_source_without_an_api_anchor() {
    let policy = format!(
        "{}\n[[typed_pairs]]\nsource = \"buffer.write_mapped_range\"\nelement = \"float\"\nreason = \"red fixture without an API anchor\"\n",
        repo_file("crates/webgpu-gen/policy.toml")
    );
    let error = subscript_typegpu_webgpu_gen::generate(
        &repo_file("third_party/webgpu-headers/webgpu.yml"),
        &policy,
    )
    .expect_err("typed pair without an API anchor unexpectedly generated")
    .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "policy error (invalid): `typed-pair.buffer.write_mapped_range`: typed-pair source has no synthetic API anchor `GPUBuffer.writeMappedRangeF32`"
    );
}
