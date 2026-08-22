use subscript_compiler::SourceFile;

use crate::support;

fn generate(source: &str) -> subscript_typegpu_gen::Generated {
    let mut files = support::b01_files();
    files.pop();
    files.push(SourceFile::new("kernel-test.ts", source));
    subscript_typegpu_gen::generate(&files).unwrap_or_else(|diagnostics| {
        panic!(
            "kernel test generation failed: {}",
            diagnostics
                .iter()
                .map(|item| item.message.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        )
    })
}

#[test]
fn conditional_uses_control_flow_and_all_identifiers_use_one_mangler() {
    let generated = generate(
        r#"
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Word { let: f32; constructor(value: f32) { this.let = value; } }
class Layout { new!: Storage<Word>; }
function loop(res: Layout, ctx: ComputeInvocation): void {
  const item: Word = res.new[0];
  const value: f32 = ctx.globalId.x > 0 ? item.let : 1.0;
}
export const names: ComputePipelineSpec = computePipeline<Layout>(loop, { workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    assert!(wgsl.contains("struct Word {\n  let_: f32,\n}"));
    assert!(wgsl.contains("var<storage, read> new_: array<Word>;"));
    assert!(wgsl.contains("var _conditional_0: f32;\n  if ("));
    assert!(wgsl.contains("fn loop_("));
    assert!(generated
        .support_module
        .contains("names_ENTRY: string = \"loop_\""));
}

#[test]
fn half_vectors_enable_the_capability_and_binding_sizes_use_wgsl_layout() {
    let generated = generate(
        r#"
import { Vec2h } from "./typegpu-types";
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
class HalfLayout { values!: Storage<Vec2h>; }
function half(res: HalfLayout, ctx: ComputeInvocation): void { const value: Vec2h = res.values[0]; }
export const halfPipeline: ComputePipelineSpec = computePipeline<HalfLayout>(half, { workgroupSize: [2, 3, 4] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    assert!(wgsl.starts_with("enable f16;\n\n"));
    assert!(wgsl.contains("array<vec2<f16>>"));
    assert!(generated.support_module.contains("minBindingSize: 4"));
}

#[test]
fn uniform_and_two_layout_bindings_keep_group_order() {
    let generated = generate(
        r#"
import { Vec4f } from "./typegpu-types";
import { ComputeInvocation, computePipeline2, ComputePipelineSpec, Storage, Uniform } from "./typegpu";
class First { params!: Uniform<Vec4f>; }
class Second { values!: Storage<Vec4f>; }
function groups(a: First, b: Second, ctx: ComputeInvocation): void {
  const params: Vec4f = a.params.get();
  const value: Vec4f = b.values[0];
}

export const grouped: ComputePipelineSpec = computePipeline2<First, Second>(groups, { workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    assert!(wgsl.contains("@group(0) @binding(0) var<uniform> params: vec4<f32>;"));
    assert!(wgsl.contains("@group(1) @binding(0) var<storage, read> values: array<vec4<f32>>;"));
    assert!(
        generated
            .support_module
            .matches("minBindingSize: 16")
            .count()
            == 2
    );
}

#[test]
fn control_flow_operators_casts_helpers_and_builtins_emit_as_wgsl() {
    let generated = generate(
        r#"
import { clamp, fract, sign, Vec4f } from "./typegpu-types";
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage, Storage } from "./typegpu";
@CStruct class Pack { values: FixedArray<Vec4f, 2>; constructor(values: FixedArray<Vec4f, 2>) { this.values = values; } }
class Layout { input!: Storage<Pack>; output!: MutStorage<Pack>; }
function helper(value: f32): f32 { return clamp(value, 0.0, 1.0); }
function operations(res: Layout, ctx: ComputeInvocation): void {
  let total: f32 = 0.0;
  let index: u32 = 0;
  while (index < 1) { total += helper(-1.0); index += 1; }
  for (let i: u32 = 0; i < 2; i += 1) { total += i as f32; }
  const pack: Pack = res.input[0];
  for (const value of pack.values) { total += value.length(); }
  const inverted: u32 = ~index;
  const flag: boolean = !false;
  const choice: f32 = flag ? fract(total) : sign(total);
  const integer: i32 = choice as i32;
  const unsigned: u32 = integer as u32;
  res.output[0] = pack;
}

export const operationPipeline: ComputePipelineSpec = computePipeline<Layout>(operations, { workgroupSize: [4, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "fn helper(value: f32) -> f32 {",
        "while (index < 1u) {",
        "for (var i = 0u; i < 2u; i += 1u) {",
        "for (var value_index = 0u; value_index < 2u;",
        "total += length(value);",
        "let inverted = ~index;",
        "let flag = !false;",
        "var _conditional_0: f32;",
        "let integer = i32(choice);",
        "let unsigned = u32(integer);",
        "output[0u] = pack;",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}` in:\n{wgsl}");
    }
}

#[test]
fn every_k10_and_k11_mapping_reaches_the_emitter() {
    let generated = generate(
        r#"
import { clamp, fract, mix, sign, smoothstep, step, Mat4x4f, Vec3f, Vec3i, Vec3u, Vec4f } from "./typegpu-types";
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
class MappingLayout {
  floats!: Storage<Vec3f>;
  ints!: Storage<Vec3i>;
  uints!: Storage<Vec3u>;
  matrices!: Storage<Mat4x4f>;
  vectors4!: Storage<Vec4f>;
}
function mappings(res: MappingLayout, ctx: ComputeInvocation): void {
  const a: Vec3f = res.floats[0]; const b: Vec3f = res.floats[1];
  const add: Vec3f = a.add(b); const sub: Vec3f = a.sub(b); const mul: Vec3f = a.mul(b);
  const scale: Vec3f = a.scale(2.0); const dot: f32 = a.dot(b); const cross: Vec3f = a.cross(b);
  const length: f32 = a.length(); const normalized: Vec3f = a.normalize();
  const ia: Vec3i = res.ints[0]; const ib: Vec3i = res.ints[1];
  const iadd: Vec3i = ia.add(ib); const isub: Vec3i = ia.sub(ib); const imul: Vec3i = ia.mul(ib); const iscale: Vec3i = ia.scale(2);
  const ua: Vec3u = res.uints[0]; const ub: Vec3u = res.uints[1];
  const uadd: Vec3u = ua.add(ub); const usub: Vec3u = ua.sub(ub); const umul: Vec3u = ua.mul(ub); const uscale: Vec3u = ua.scale(2);
  const matrix: Mat4x4f = res.matrices[0]; const other: Mat4x4f = res.matrices[1]; const vector: Vec4f = res.vectors4[0];
  const mm: Mat4x4f = matrix.mul(other); const mv: Vec4f = matrix.mulVec(vector); const mt: Mat4x4f = matrix.transpose();
  Math.abs(-1.0); Math.min(1.0, 2.0); Math.max(1.0, 2.0); Math.floor(1.5);
  Math.ceil(1.5); Math.sqrt(4.0); Math.pow(2.0, 3.0); Math.exp(1.0);
  Math.log(1.0); Math.sin(1.0); Math.cos(1.0); Math.tan(1.0); Math.fround(1.25);
  const scalar: f32 = 0.5;
  const clamped: f32 = clamp(scalar, 0.0, 1.0); const mixed: f32 = mix(scalar, scalar, 0.5);
  const stepped: f32 = step(0.5, scalar); const smoothed: f32 = smoothstep(0.0, 1.0, scalar);
  const fractional: f32 = fract(scalar); const signed: f32 = sign(scalar);
}
export const mappingPipeline: ComputePipelineSpec = computePipeline<MappingLayout>(mappings, { workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "a + b",
        "a - b",
        "a * b",
        "a * 2.0f",
        "dot(a, b)",
        "cross(a, b)",
        "length(a)",
        "normalize(a)",
        "ia + ib",
        "ua + ub",
        "matrix * other",
        "matrix * vector",
        "transpose(matrix)",
        "abs(-1.0f)",
        "min(1.0f, 2.0f)",
        "max(1.0f, 2.0f)",
        "floor(1.5f)",
        "ceil(1.5f)",
        "sqrt(4.0f)",
        "pow(2.0f, 3.0f)",
        "exp(1.0f)",
        "log(1.0f)",
        "sin(1.0f)",
        "cos(1.0f)",
        "tan(1.0f)",
        "1.25f;",
        "clamp(scalar, 0.0f, 1.0f)",
        "mix(scalar, scalar, 0.5f)",
        "step(0.5f, scalar)",
        "smoothstep(0.0f, 1.0f, scalar)",
        "fract(scalar)",
        "sign(scalar)",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}` in:\n{wgsl}");
    }
}
