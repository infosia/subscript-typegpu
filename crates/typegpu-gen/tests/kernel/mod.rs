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
    assert!(wgsl.contains("var _g_conditional_0: f32;\n  if ("));
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
        "for (var _g_value_index = 0u; _g_value_index < 2u;",
        "total += length(value);",
        "let inverted = ~index;",
        "let flag = !false;",
        "var _g_conditional_0: f32;",
        "let integer = i32(choice);",
        "let unsigned = u32(integer);",
        "output[0u] = pack;",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}` in:\n{wgsl}");
    }
}

#[test]
fn lowered_operators_preserve_emitted_wgsl_precedence() {
    let generated = generate(
        r#"
import { Vec3f } from "./typegpu-types";
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
class Layout { values!: Storage<Vec3f>; }
function precedence(res: Layout, ctx: ComputeInvocation): void {
  const a: Vec3f = res.values[0];
  const b: Vec3f = res.values[1];
  const s: f32 = 2.0;
  const first: Vec3f = a.add(b).scale(s);
  const second: Vec3f = a.scale(s).add(b);
  const third: f32 = a.dot(b) * 2.0;
  const fourth: f32 = -(a.x + b.x);
  const rounded: f32 = (Math.fround(1.0 + 2.0) as f32) * s;
}

export const precedencePipeline: ComputePipelineSpec = computePipeline<Layout>(precedence, { workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "var first = (a + b) * s;",
        "var second = a * s + b;",
        "let third = dot(a, b) * 2.0f;",
        "let fourth = -(a.x + b.x);",
        "let rounded = (1.0f + 2.0f) * s;",
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

#[test]
fn conditional_preludes_stay_at_their_runtime_evaluation_site() {
    let generated = generate(
        r#"
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Pack {
  left: FixedArray<f32, 2>;
  right: FixedArray<f32, 2>;
  constructor(left: FixedArray<f32, 2>, right: FixedArray<f32, 2>) { this.left = left; this.right = right; }
}
class Layout { packs!: Storage<Pack>; }
function control(res: Layout, ctx: ComputeInvocation): void {
  let i: u32 = 0;
  const flag: boolean = ctx.globalId.x === (0 as u32);
  while (i < (2 as u32) ? true : false) {
    const chosen: u32 = flag ? (i > (0 as u32) ? (2 as u32) : (1 as u32)) : (0 as u32);
    i += chosen;
  }
  for (let j: u32 = 0; j < (flag ? (2 as u32) : (1 as u32)); j += 1) { i += j; }
  const pack: Pack = res.packs[0];
  for (const value of (flag ? pack.left : pack.right)) { i += value as u32; }
}
export const controlPipeline: ComputePipelineSpec = computePipeline<Layout>(control, { workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    let loop_start = wgsl
        .find("loop {")
        .expect("conditional while becomes a loop");
    let while_cond = wgsl
        .find("var _g_conditional_0: bool;")
        .expect("while conditional prelude exists");
    assert!(
        while_cond > loop_start,
        "while prelude escaped its loop:\n{wgsl}"
    );
    assert!(
        wgsl.contains("if (flag) {\n      var _g_conditional_1: u32;\n      if (i > u32(0i)) {"),
        "nested branch prelude escaped its branch:\n{wgsl}"
    );
    let for_loop = wgsl
        .find("var _g_conditional_3: u32;")
        .expect("for condition prelude exists");
    assert!(for_loop > wgsl.find("var j = 0u;").expect("for initializer"));
    let for_of_prelude = wgsl
        .find("var _g_conditional_4: array<f32, 2>;")
        .expect("for-of subject prelude exists");
    let for_of = wgsl
        .find("for (var _g_value_index")
        .expect("for-of loop exists");
    assert!(
        for_of_prelude < for_of,
        "for-of prelude was not flushed:\n{wgsl}"
    );
}
