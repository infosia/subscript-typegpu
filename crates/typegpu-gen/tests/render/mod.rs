use naga::valid::{Capabilities, ValidationFlags, Validator};
use subscript_compiler::SourceFile;

use crate::support;

fn generate(source: &str) -> subscript_typegpu_gen::Generated {
    let mut files = support::b01_files();
    files.pop();
    files.push(SourceFile::new("render-test.ts", source));
    subscript_typegpu_gen::generate(&files).unwrap_or_else(|diagnostics| {
        panic!(
            "render test generation failed: {}",
            diagnostics
                .iter()
                .map(|item| item.message.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        )
    })
}

fn validate(wgsl: &str) {
    let module = naga::front::wgsl::parse_str(wgsl)
        .unwrap_or_else(|error| panic!("naga parse failed: {error}"));
    let capabilities = if wgsl.starts_with("enable f16;") {
        Capabilities::SHADER_FLOAT16
    } else {
        Capabilities::empty()
    };
    Validator::new(ValidationFlags::all(), capabilities)
        .validate(&module)
        .unwrap_or_else(|error| panic!("naga validation failed: {error}"));
}

#[test]
fn render_entries_interfaces_and_layout_constants_use_typed_hir() {
    let generated = generate(
        r#"
import { FragmentInvocation, renderPipeline, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec2f, Vec3f, Vec4f } from "./typegpu-types";
@CStruct class Vertex { position: Vec2f; color: Vec3f; constructor(position: Vec2f, color: Vec3f) { this.position = position; this.color = color; } }
@CStruct class Varyings { position: Vec4f; color: Vec3f; constructor(position: Vec4f, color: Vec3f) { this.position = position; this.color = color; } }
function vert(value: Vertex, ctx: VertexInvocation): Varyings { return new Varyings(new Vec4f(value.position.x, value.position.y, 0.0, 1.0), value.color); }
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f { return new Vec4f(input.color.x, input.color.y, input.color.z, 1.0); }
export const tri: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "@location(0) position: vec2<f32>",
        "@location(1) color: vec3<f32>",
        "@builtin(position) position: vec4<f32>",
        "@vertex\nfn vert(value: Vertex) -> Varyings",
        "@fragment\nfn frag(input: Varyings) -> @location(0) vec4<f32>",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}` in:\n{wgsl}");
    }
    for expected in [
        "tri_VERTEX_ENTRY: string = \"vert\"",
        "tri_FRAGMENT_ENTRY: string = \"frag\"",
        "tri_TARGET_FORMAT: GPUTextureFormat = \"rgba8unorm\"",
        "arrayStride: 32",
        "{ format: \"float32x2\", offset: 0, shaderLocation: 0 }",
        "{ format: \"float32x3\", offset: 16, shaderLocation: 1 }",
    ] {
        assert!(
            generated.support_module.contains(expected),
            "missing `{expected}` in support module"
        );
    }
    assert!(!generated.support_module.contains("Varyings_SIZE"));
    validate(wgsl);
}

#[test]
fn instance_locations_builtins_flat_varyings_and_stage_visibility_emit() {
    let generated = generate(
        r#"
import { FragmentInvocation, renderPipelineInstanced, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec2f, Vec2u, Vec4f } from "./typegpu-types";
@CStruct class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@CStruct class Instance { offset: Vec2f; constructor(offset: Vec2f) { this.offset = offset; } }
@CStruct class Varyings { position: Vec4f; id: Vec2u; constructor(position: Vec4f, id: Vec2u) { this.position = position; this.id = id; } }
function vert(value: Vertex, instance: Instance, ctx: VertexInvocation): Varyings { return new Varyings(new Vec4f(value.position.x + instance.offset.x, value.position.y + instance.offset.y, 0.0, 1.0), new Vec2u(ctx.vertexIndex, ctx.instanceIndex)); }
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f { return new Vec4f(input.id.x as f32, input.position.y, 0.0, ctx.frontFacing ? 1.0 : 0.0); }
export const instanced: RenderPipelineSpec = renderPipelineInstanced<Vertex, Instance, Varyings>(vert, frag, { format: "rgba8unorm" });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "@location(1) offset: vec2<f32>",
        "@location(0) @interpolate(flat) id: vec2<u32>",
        "@builtin(vertex_index) vertexIndex: u32",
        "@builtin(instance_index) instanceIndex: u32",
        "@builtin(front_facing) frontFacing: bool",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}` in:\n{wgsl}");
    }
    assert!(generated
        .support_module
        .contains("instanced_VERTEX_LAYOUT1: VertexBufferLayoutSpec"));
    assert!(generated.support_module.contains("stepMode: \"instance\""));
    assert!(!wgsl.contains("fragmentPosition"));
    validate(wgsl);
}

#[test]
fn b06_render_fragment_module_validates_with_naga() {
    let program = support::root().join("programs/b06-render.ts");
    let generated = subscript_typegpu_gen::generate(&support::program_files(&program))
        .expect("generate b06-render");
    let wgsl = &generated
        .pipelines
        .iter()
        .find(|(name, _)| name == "tri")
        .expect("b06 tri pipeline")
        .1;
    validate(wgsl);
}

#[test]
fn render_binding_visibility_follows_each_kernel_reach() {
    let generated = generate(
        r#"
import { FragmentInvocation, renderPipelineL, RenderPipelineSpec, Storage, Uniform, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@CStruct class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@CStruct class Offset { value: Vec4f; constructor(value: Vec4f) { this.value = value; } }
@CStruct class Tint { value: Vec4f; constructor(value: Vec4f) { this.value = value; } }
@CStruct class Varyings { position: Vec4f; constructor(position: Vec4f) { this.position = position; } }
class Layout { vertexOnly!: Uniform<Offset>; fragmentOnly!: Storage<Tint>; both!: Storage<Tint>; }
function vert(res: Layout, value: Vertex, ctx: VertexInvocation): Varyings { const offset: Offset = res.vertexOnly.get(); const shared: Tint = res.both[0]; return new Varyings(new Vec4f(value.position.x + offset.value.x + shared.value.x * 0.0, value.position.y + offset.value.y, 0.0, 1.0)); }
function frag(res: Layout, input: Varyings, ctx: FragmentInvocation): Vec4f { const tint: Tint = res.fragmentOnly[0]; const shared: Tint = res.both[0]; return tint.value.add(shared.value.scale(0.0)); }
export const shifted: RenderPipelineSpec = renderPipelineL<Layout, Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
"#,
    );
    for expected in [
        "binding: 0, visibility: VERTEX_VISIBILITY",
        "binding: 1, visibility: FRAGMENT_VISIBILITY",
        "binding: 2, visibility: VERTEX_VISIBILITY + FRAGMENT_VISIBILITY",
    ] {
        assert!(
            generated.support_module.contains(expected),
            "missing `{expected}` in:\n{}",
            generated.support_module
        );
    }
    validate(&generated.pipelines[0].1);
}

#[test]
fn render_f16_capability_depends_on_referenced_module_types() {
    let varying_half = generate(
        r#"
import { FragmentInvocation, renderPipeline, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec2f, Vec2h, Vec4f } from "./typegpu-types";
@CStruct class Vertex { position: Vec2f; half: Vec2h; constructor(position: Vec2f, half: Vec2h) { this.position = position; this.half = half; } }
@CStruct class Varyings { position: Vec4f; half: Vec2h; constructor(position: Vec4f, half: Vec2h) { this.position = position; this.half = half; } }
function vert(value: Vertex, ctx: VertexInvocation): Varyings { return new Varyings(new Vec4f(value.position.x, value.position.y, 0.0, 1.0), value.half); }
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
export const pipeline: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
"#,
    );
    assert!(varying_half.pipelines[0].1.starts_with("enable f16;"));
    validate(&varying_half.pipelines[0].1);

    let unrelated_half = generate(
        r#"
import { FragmentInvocation, renderPipeline, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
import { UnusedHalf_SIZE } from "./render-test.typegpu";
@CStruct class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@CStruct class UnusedHalf { value: f16; constructor(value: f16) { this.value = value; } }
@CStruct class Varyings { position: Vec4f; constructor(position: Vec4f) { this.position = position; } }
function vert(value: Vertex, ctx: VertexInvocation): Varyings { return new Varyings(new Vec4f(value.position.x, value.position.y, 0.0, 1.0)); }
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
export const pipeline: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
"#,
    );
    assert!(!unrelated_half.pipelines[0].1.starts_with("enable f16;"));
    validate(&unrelated_half.pipelines[0].1);
}

#[test]
fn program_owned_pipeline_lookalikes_are_not_declarations() {
    let generated = generate(
        r#"
import { ComputeInvocation, ComputePipelineSpec, FragmentInvocation, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@CStruct class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@CStruct class Varyings { position: Vec4f; constructor(position: Vec4f) { this.position = position; } }
class Layout {}
function computePipeline<L>(kernel: (res: L, ctx: ComputeInvocation) => void, spec: ComputePipelineSpec): ComputePipelineSpec { return spec; }
function renderPipeline<V, O>(vertex: (value: V, ctx: VertexInvocation) => O, fragment: (input: O, ctx: FragmentInvocation) => Vec4f, spec: RenderPipelineSpec): RenderPipelineSpec { return spec; }
function kernel(res: Layout, ctx: ComputeInvocation): void {}
function vert(value: Vertex, ctx: VertexInvocation): Varyings { return new Varyings(new Vec4f(value.position.x, value.position.y, 0.0, 1.0)); }
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
export const compute: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [1, 1, 1] });
export const render: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
"#,
    );
    assert!(generated.pipelines.is_empty());
}
