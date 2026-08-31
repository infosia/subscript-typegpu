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
export const pipeline: ComputePipelineSpec = computePipeline2<Textures, Settings>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "@group(0u) @binding(0u) var source: texture_2d<f32>;",
        "@group(0u) @binding(1u) var nearest: sampler;",
        "texture_storage_2d<rgba8unorm, write>",
        "@group(1u) @binding(0u) var<uniform>",
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
fn float_texture_wrappers_and_storage_formats_are_declared_by_library_identity() {
    let generated = generate(
        r#"
import { ComputeInvocation, ComputePipelineSpec, R32float, Rgba16float, Rgba32float, Rgba8unorm, Sampler, StorageTexture2d, Texture2d, computePipeline } from "./typegpu";
import { Vec2f, Vec2i, Vec4f } from "./typegpu-types";
class Layout {
  floats!: Texture2d<f32>;
  filtering!: Sampler;
  rgba8unorm!: StorageTexture2d<Rgba8unorm>;
  rgba16float!: StorageTexture2d<Rgba16float>;
  r32float!: StorageTexture2d<R32float>;
  rgba32float!: StorageTexture2d<Rgba32float>;
}
function kernel(res: Layout, ctx: ComputeInvocation): void {
  const dimensions = res.floats.dimensions();
  const loaded = res.floats.load(new Vec2i(0, 0), 0);
  const sampled = res.floats.sampleLevel(res.filtering, new Vec2f(0.25, 0.25), 0.0);
  res.rgba8unorm.store(new Vec2i(0, 0), loaded.add(sampled));
  res.rgba16float.store(new Vec2i(0, 0), loaded);
  res.r32float.store(new Vec2i(0, 0), sampled);
  res.rgba32float.store(new Vec2i(dimensions.x as i32, 0), new Vec4f(0.0, 0.0, 0.0, 1.0));
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "texture_2d<f32>",
        "var filtering: sampler;",
        "texture_storage_2d<rgba8unorm, write>",
        "texture_storage_2d<rgba16float, write>",
        "texture_storage_2d<r32float, write>",
        "texture_storage_2d<rgba32float, write>",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}` in:\n{wgsl}");
    }
    validate(wgsl);
}

#[test]
fn read_access_storage_textures_emit_methods_layout_access_and_resources() {
    let generated = generate(
        r#"
import { ComputeInvocation, ComputePipelineSpec, R32float, ReadStorageTexture2d, ReadWriteStorageTexture2d, computePipeline } from "./typegpu";
import { Vec2i } from "./typegpu-types";
class Layout {
  source!: ReadStorageTexture2d<R32float>;
  target!: ReadWriteStorageTexture2d<R32float>;
}
function kernel(res: Layout, ctx: ComputeInvocation): void {
  const coords = new Vec2i(ctx.globalId.x as i32, ctx.globalId.y as i32);
  const size = res.source.dimensions();
  const source = res.source.load(coords);
  const target = res.target.load(coords);
  if (ctx.globalId.x < size.x && ctx.globalId.y < res.target.dimensions().y) {
    res.target.store(coords, source.add(target));
  }
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "texture_storage_2d<r32float, read>",
        "texture_storage_2d<r32float, read_write>",
        "textureDimensions(source)",
        "textureDimensions(target_)",
        "textureLoad(source, coords)",
        "textureLoad(target_, coords)",
        "textureStore(target_, coords, source_ + target__)",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}` in:\n{wgsl}");
    }
    for expected in [
        "kind: \"storageTexture\", minBindingSize: 0, format: \"r32float\", access: \"read-only\"",
        "kind: \"storageTexture\", minBindingSize: 0, format: \"r32float\", access: \"read-write\"",
        "source!: GPUTextureView",
        "target!: GPUTextureView",
        "textureResource(resources.source)",
        "textureResource(resources.target)",
    ] {
        assert!(
            generated.support_module.contains(expected),
            "missing `{expected}` in support module:\n{}",
            generated.support_module,
        );
    }
    validate(wgsl);
}

#[test]
fn array_textures_emit_layered_calls_dimensions_and_layout_access() {
    let generated = generate(
        r#"
import { ComputeInvocation, ComputePipelineSpec, ReadStorageTexture2dArray, Rgba16float, Texture2dArray, WriteStorageTexture2dArray, computePipeline } from "./typegpu";
import { Vec2i } from "./typegpu-types";
class Layout {
  sampled!: Texture2dArray<f32>;
  source!: ReadStorageTexture2dArray<Rgba16float>;
  target!: WriteStorageTexture2dArray<Rgba16float>;
}
function kernel(res: Layout, ctx: ComputeInvocation): void {
  const coords = new Vec2i(ctx.globalId.x as i32, ctx.globalId.y as i32);
  const layer = ctx.globalId.z as i32;
  const sampled = res.sampled.load(coords, layer, 0);
  const source = res.source.load(coords, layer);
  res.target.store(coords, layer, sampled.add(source));
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "texture_2d_array<f32>",
        "texture_storage_2d_array<rgba16float, read>",
        "texture_storage_2d_array<rgba16float, write>",
        "textureLoad(sampled, coords, layer, 0u)",
        "textureLoad(source, coords, layer)",
        "textureStore(target_, coords, layer, sampled_ + source_)",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}` in:\n{wgsl}");
    }
    for expected in [
        "kind: \"texture\", minBindingSize: 0, sampleType: \"float\", viewDimension: \"2d-array\"",
        "kind: \"storageTexture\", minBindingSize: 0, format: \"rgba16float\", access: \"read-only\", viewDimension: \"2d-array\"",
        "kind: \"storageTexture\", minBindingSize: 0, format: \"rgba16float\", access: \"write-only\", viewDimension: \"2d-array\"",
    ] {
        assert!(
            generated.support_module.contains(expected),
            "missing `{expected}` in support module:\n{}",
            generated.support_module,
        );
    }
    validate(wgsl);
}
