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
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f { return new Vec4f(input.id.x as f32, ctx.position.y, 0.0, ctx.frontFacing ? 1.0 : 0.0); }
export const instanced: RenderPipelineSpec = renderPipelineInstanced<Vertex, Instance, Varyings>(vert, frag, { format: "rgba8unorm" });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "@location(1) offset: vec2<f32>",
        "@location(0) @interpolate(flat) id: vec2<u32>",
        "@builtin(vertex_index) vertexIndex: u32",
        "@builtin(instance_index) instanceIndex: u32",
        "@builtin(position) fragmentPosition: vec4<f32>",
        "@builtin(front_facing) frontFacing: bool",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}` in:\n{wgsl}");
    }
    assert!(generated
        .support_module
        .contains("instanced_VERTEX_LAYOUT1: VertexBufferLayoutSpec"));
    assert!(generated.support_module.contains("stepMode: \"instance\""));
}
