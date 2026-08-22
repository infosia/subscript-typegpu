//! Pin canaries. Re-pin with `specs/tracking/p0-seed.md` “Pins” and
//! plan §5 “The substrate generator”.

use std::path::{Path, PathBuf};

use weedle::Parse;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root")
        .to_path_buf()
}

fn read(relative: impl AsRef<Path>) -> String {
    let path = root().join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

#[test]
fn namespace_prepass_is_exact_and_loud() {
    let document = r#"
<script type=idl>
typedef unsigned long GPUExampleFlags;
[SecureContext]
[Exposed=Window]
namespace GPUExample {
    const GPUFlagsConstant FIRST = 0x01;
    const GPUFlagsConstant NEXT_VALUE = 0x0A;
};
interface GPUDevice {};
</script>
"#;
    let extracted =
        subscript_typegpu_webgpu_gen::extract_gpuweb_idl(document).expect("rigid namespace parses");
    assert_eq!(extracted.block_count, 1);
    assert_eq!(extracted.namespace_count, 1);
    assert_eq!(extracted.namespace_constants.len(), 2);
    assert_eq!(extracted.namespace_constants[0].namespace, "GPUExample");
    assert_eq!(extracted.namespace_constants[0].name, "FIRST");
    assert_eq!(extracted.namespace_constants[0].value, 1);
    assert_eq!(extracted.namespace_constants[1].name, "NEXT_VALUE");
    assert_eq!(extracted.namespace_constants[1].value, 10);
    assert!(!extracted.weedle_source.contains("namespace GPUExample"));
    let unsupported = document.replace(
        "    const GPUFlagsConstant FIRST = 0x01;",
        "    undefined reset();",
    );
    let error = subscript_typegpu_webgpu_gen::extract_gpuweb_idl(&unsupported)
        .expect_err("new grammar must fail loud");
    assert!(error.contains("fork-and-pin weedle2"), "{error}");
}

#[test]
fn pinned_gpuweb_counts_and_namespace_constants_are_stable() {
    let document = read("third_party/gpuweb/spec/index.bs");
    let extracted =
        subscript_typegpu_webgpu_gen::extract_gpuweb_idl(&document).expect("extract pinned IDL");
    assert_eq!(extracted.block_count, 125);
    assert_eq!(extracted.namespace_count, 5);
    let expected = [
        ("GPUBufferUsage", "MAP_READ", 0x0001),
        ("GPUBufferUsage", "MAP_WRITE", 0x0002),
        ("GPUBufferUsage", "COPY_SRC", 0x0004),
        ("GPUBufferUsage", "COPY_DST", 0x0008),
        ("GPUBufferUsage", "INDEX", 0x0010),
        ("GPUBufferUsage", "VERTEX", 0x0020),
        ("GPUBufferUsage", "UNIFORM", 0x0040),
        ("GPUBufferUsage", "STORAGE", 0x0080),
        ("GPUBufferUsage", "INDIRECT", 0x0100),
        ("GPUBufferUsage", "QUERY_RESOLVE", 0x0200),
        ("GPUMapMode", "READ", 0x0001),
        ("GPUMapMode", "WRITE", 0x0002),
        ("GPUTextureUsage", "COPY_SRC", 0x01),
        ("GPUTextureUsage", "COPY_DST", 0x02),
        ("GPUTextureUsage", "TEXTURE_BINDING", 0x04),
        ("GPUTextureUsage", "STORAGE_BINDING", 0x08),
        ("GPUTextureUsage", "RENDER_ATTACHMENT", 0x10),
        ("GPUTextureUsage", "TRANSIENT_ATTACHMENT", 0x20),
        ("GPUShaderStage", "VERTEX", 0x1),
        ("GPUShaderStage", "FRAGMENT", 0x2),
        ("GPUShaderStage", "COMPUTE", 0x4),
        ("GPUColorWrite", "RED", 0x1),
        ("GPUColorWrite", "GREEN", 0x2),
        ("GPUColorWrite", "BLUE", 0x4),
        ("GPUColorWrite", "ALPHA", 0x8),
        ("GPUColorWrite", "ALL", 0xF),
    ];
    assert_eq!(extracted.namespace_constants.len(), expected.len());
    for (actual, (namespace, name, value)) in extracted.namespace_constants.iter().zip(expected) {
        assert_eq!(actual.namespace, namespace);
        assert_eq!(actual.name, name);
        assert_eq!(actual.value, value);
    }
    let (remaining, definitions) =
        weedle::Definitions::parse(&extracted.weedle_source).expect("weedle2 parses remainder");
    assert!(
        remaining.trim().is_empty(),
        "weedle2 remainder: {remaining:?}"
    );
    assert_eq!(definitions.len(), 198, "weedle2 definition count moved");
}

#[test]
fn api_accounting_and_absence_enum_members_are_stable() {
    let policy = read("crates/webgpu-gen/policy.toml");
    let mirror = crate::support::require_base_mirror!();
    let gpuweb = subscript_typegpu_webgpu_gen::GPUWEB_IDL_INPUTS
        .iter()
        .map(read)
        .collect::<Vec<_>>()
        .join("\n");
    let generated = subscript_typegpu_webgpu_gen::generate_api(&gpuweb, &mirror, &policy)
        .expect("pinned API joins");
    assert_eq!(
        (
            generated.pattern_members,
            generated.override_members,
            generated.excluded_members,
            generated.wrapper_constructs,
            generated.result_constructs,
        ),
        (526, 116, 24, 21, 4),
    );
    assert_eq!(
        generated.absence_enum_members,
        [
            "GPUTextureViewDescriptor.format",
            "GPUTextureViewDescriptor.dimension",
            "GPUSamplerDescriptor.compare",
            "GPUPrimitiveState.stripIndexFormat",
            "GPUDepthStencilState.depthCompare",
            "GPURenderBundleEncoderDescriptor.depthStencilFormat",
            "GPURenderPassDepthStencilAttachment.depthLoadOp",
            "GPURenderPassDepthStencilAttachment.depthStoreOp",
            "GPURenderPassDepthStencilAttachment.stencilLoadOp",
            "GPURenderPassDepthStencilAttachment.stencilStoreOp",
        ],
    );
}
