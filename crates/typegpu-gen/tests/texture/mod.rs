use subscript_compiler::SourceFile;

use crate::support;

fn generate(source: &str) -> subscript_typegpu_gen::Generated {
    let mut files = support::b01_files();
    files.pop();
    files.push(SourceFile::new("texture-test.ts", source));
    subscript_typegpu_gen::generate(&files).unwrap_or_else(|diagnostics| {
        panic!(
            "texture test generation failed: {}",
            diagnostics
                .iter()
                .map(|item| item.message.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        )
    })
}

fn validate(wgsl: &str) {
    let module = naga::front::wgsl::parse_str(wgsl).expect("parse texture WGSL");
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .expect("validate texture WGSL");
}

#[test]
fn texture_bindings_calls_groups_and_layout_entries_emit() {
    let generated = generate(
        r#"
import { ComputeInvocation, ComputePipelineSpec, Rgba8unorm, Sampler, StorageTexture2d, Texture2d, Uniform, computePipeline2 } from "./typegpu";
import { Vec2f, Vec2i, Vec4f } from "./typegpu-types";
@CStruct class Params { width: u32; constructor(width: u32) { this.width = width; } }
class Textures { source!: Texture2d<f32>; nearest!: Sampler; target!: StorageTexture2d<Rgba8unorm>; }
class Settings { params!: Uniform<Params>; }
function kernel(textures: Textures, settings: Settings, ctx: ComputeInvocation): void {
  const size = textures.source.dimensions();
  const loaded = textures.source.load(new Vec2i(0, 0), 0);
  const sampled = textures.source.sampleLevel(textures.nearest, new Vec2f(0.5, 0.5), 0.0);
  textures.target.store(new Vec2i(size.x as i32, 0), loaded.add(sampled));
}
export const pipeline: ComputePipelineSpec = computePipeline2<Textures, Settings>(kernel, { workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "@group(0) @binding(0) var source: texture_2d<f32>;",
        "@group(0) @binding(1) var nearest: sampler;",
        "texture_storage_2d<rgba8unorm, write>",
        "@group(1) @binding(0) var<uniform>",
        "textureDimensions(source)",
        "textureLoad(source, vec2<i32>(0i, 0i), 0u)",
        "textureSampleLevel(source, nearest",
        "textureStore(target_",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}` in:\n{wgsl}");
    }
    for expected in [
        "kind: \"texture\", minBindingSize: 0, sampleType: \"float\"",
        "kind: \"sampler\", minBindingSize: 0, samplerType: \"filtering\"",
        "kind: \"storageTexture\", minBindingSize: 0, format: \"rgba8unorm\"",
        "pipeline_LAYOUT1",
    ] {
        assert!(
            generated.support_module.contains(expected),
            "missing `{expected}` in support module"
        );
    }
    validate(wgsl);
}

#[test]
fn fragment_sample_emits_and_validates() {
    let program = support::root().join("programs/x11-live-fragment-sample.ts");
    let generated = subscript_typegpu_gen::generate(&support::program_files(&program))
        .expect("generate fragment texture program");
    let wgsl = &generated
        .pipelines
        .iter()
        .find(|(name, _)| name == "fragmentSample")
        .expect("fragmentSample pipeline")
        .1;
    assert!(wgsl.contains("textureSample(source, nearest, input.uv)"));
    assert!(generated.support_module.contains("FRAGMENT_VISIBILITY"));
    validate(wgsl);
}

#[test]
fn every_texture_wrapper_and_storage_format_is_declared_by_library_identity() {
    let generated = generate(
        r#"
import { ComparisonSampler, ComputeInvocation, ComputePipelineSpec, R32float, Rgba16float, Rgba32float, Rgba8uint, Rgba8unorm, Sampler, StorageTexture2d, Texture2d, computePipeline } from "./typegpu";
class Layout {
  floats!: Texture2d<f32>;
  signed!: Texture2d<i32>;
  unsigned!: Texture2d<u32>;
  filtering!: Sampler;
  comparison!: ComparisonSampler;
  rgba8unorm!: StorageTexture2d<Rgba8unorm>;
  rgba8uint!: StorageTexture2d<Rgba8uint>;
  rgba16float!: StorageTexture2d<Rgba16float>;
  r32float!: StorageTexture2d<R32float>;
  rgba32float!: StorageTexture2d<Rgba32float>;
}
function kernel(res: Layout, ctx: ComputeInvocation): void {}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "texture_2d<f32>",
        "texture_2d<i32>",
        "texture_2d<u32>",
        "var filtering: sampler;",
        "var comparison: sampler_comparison;",
        "texture_storage_2d<rgba8unorm, write>",
        "texture_storage_2d<rgba8uint, write>",
        "texture_storage_2d<rgba16float, write>",
        "texture_storage_2d<r32float, write>",
        "texture_storage_2d<rgba32float, write>",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}` in:\n{wgsl}");
    }
    assert!(generated.support_module.contains("sampleType: \"sint\""));
    assert!(generated.support_module.contains("sampleType: \"uint\""));
    assert!(generated
        .support_module
        .contains("samplerType: \"comparison\""));
    validate(wgsl);
}
