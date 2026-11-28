use super::{check_calls, generate, validate};

const MATRIX: &str = r#"
import { Mat2x2f, Vec2f } from "./typegpu-types";
function makeMatrix(): Mat2x2f { return new Mat2x2f(new Vec2f(1.0, 2.0), new Vec2f(3.0, 4.0)); }
"#;

fn check_reads(width: usize, components: &str) {
    let vector = format!("Vec{width}f");
    let matrix = format!("Mat{width}x{width}f");
    let columns = vec!["v"; width].join(", ");
    let last = width - 1;
    check_calls(
        &format!(r#"import {{ {matrix}, {vector} }} from "./typegpu-types";"#),
        &format!(
            "const v = new {vector}({components}); let m = new {matrix}({columns});
             const first = m.c0; const y = m.c1.y; const last = m.c{last};
             res.output[0] = (first.x + y + last.x) as u32;"
        ),
        &[
            "first = m[0u];",
            "y = m[1u].y;",
            &format!("last = m[{last}u];"),
        ],
    );
}

#[test]
fn mat2_reads() {
    check_reads(2, "1.0, 2.0");
}

#[test]
fn mat3_reads() {
    check_reads(3, "1.0, 2.0, 3.0");
}

#[test]
fn mat4_reads() {
    check_reads(4, "1.0, 2.0, 3.0, 4.0");
}

#[test]
fn local_writes() {
    check_calls(
        MATRIX,
        "let n = makeMatrix(); const v = new Vec2f(5.0, 6.0); const s: f32 = 9.0;
         n.c0 = v; n.c1.x = s; n.c1.y += s; res.output[0] = n.c1.x as u32;",
        &["n[0u] = v;", "n[1u].x = s;", "n[1u].y += s;"],
    );
}

#[test]
fn written_parameter() {
    check_calls(
        &format!("{MATRIX} function edit(m: Mat2x2f): u32 {{ m.c0.x = 9.0; return m.c0.x as u32; }}"),
        "res.output[0] = edit(makeMatrix());",
        &["fn edit(_g_parameter_m: mat2x2<f32>) -> u32 {\n  var m = _g_parameter_m;\n  m[0u].x = 9.0f;"],
    );
}

#[test]
fn schema_field() {
    check_calls(
        &format!("{MATRIX} @ValueType class Item {{ constructor(public transform: Mat2x2f) {{}} }}"),
        "let s = new Item(makeMatrix()); s.transform.c1.x = 9.0; res.output[0] = s.transform.c1.x as u32;",
        &["s.transform[1u].x = 9.0f;", "u32(s.transform[1u].x)"],
    );
}

#[test]
fn schema_array_element() {
    let module = generate(
        r#"
import { ComputePipelineSpec, MutStorage, Storage, computePipeline } from "./typegpu";
import { Mat2x2f } from "./typegpu-types";
@ValueType class Item { constructor(public ms: FixedArray<Mat2x2f, 2>) {} }
class Layout { constructor(public items: Storage<Item>, public output: MutStorage<f32>) {} }
function kernel(res: Layout): void { const a = res.items[0]; res.output[0] = a.ms[1].c0.x; }
const pipe: ComputePipelineSpec = computePipeline(kernel, { name: "pipe", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &module.pipelines[0].1;
    validate(wgsl);
    assert!(wgsl.contains("output[0u] = a.ms[1i][0u].x;"), "{wgsl}");
}

#[test]
fn binding_reads() {
    let module = generate(
        r#"
import { ComputePipelineSpec, MutStorage, Storage, Uniform, computePipeline } from "./typegpu";
import { Mat2x2f, Mat4x4f } from "./typegpu-types";
class Layout { constructor(public transform: Uniform<Mat4x4f>, public mats: Storage<Mat2x2f>, public output: MutStorage<f32>) {} }
function kernel(res: Layout): void { res.output[0] = res.transform.$.c3.w + res.mats[1].c0.x; }
const pipe: ComputePipelineSpec = computePipeline(kernel, { name: "pipe", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &module.pipelines[0].1;
    validate(wgsl);
    assert!(wgsl.contains("transform[3u].w + mats[1u][0u].x"), "{wgsl}");
}

#[test]
fn module_constant() {
    check_calls(
        r#"import { Mat2x2f, Vec2f } from "./typegpu-types";
const K: Mat2x2f = new Mat2x2f(new Vec2f(1.0, 2.0), new Vec2f(3.0, 4.0));"#,
        "res.output[0] = K.c1.x as u32;",
        &["u32(K[1u].x)"],
    );
}

#[test]
fn private_variable() {
    check_calls(
        r#"import { PrivateVar, privateVar } from "./typegpu";
import { Mat2x2f, Vec2f } from "./typegpu-types";
const state: PrivateVar<Mat2x2f> = privateVar<Mat2x2f>(new Mat2x2f(new Vec2f(1.0, 2.0), new Vec2f(3.0, 4.0)));"#,
        "res.output[0] = state.$.c1.x as u32;",
        &["u32(state[1u].x)"],
    );
}

#[test]
fn workgroup_variable() {
    check_calls(
        r#"import { WorkgroupVar, workgroupVar } from "./typegpu";
import { Mat2x2f } from "./typegpu-types";
const sharedMatrix: WorkgroupVar<Mat2x2f> = workgroupVar<Mat2x2f>();"#,
        "res.output[0] = sharedMatrix.$.c1.x as u32;",
        &["u32(sharedMatrix[1u].x)"],
    );
}

#[test]
fn call_result() {
    check_calls(
        MATRIX,
        "const v = makeMatrix().c1; res.output[0] = v.x as u32;",
        &["v = makeMatrix()[1u];"],
    );
}

#[test]
fn matrix_product() {
    check_calls(
        MATRIX,
        "const m = makeMatrix(); res.output[0] = m.mul(m).c0.x as u32;",
        &["u32((m * m)[0u].x)"],
    );
}

#[test]
fn capture_before_conflict() {
    check_calls(
        &format!(
            "{MATRIX} function first(v: Vec2f, y: u32): u32 {{ return v.x as u32 + y; }}
                   function tick(): u32 {{ return 1; }}"
        ),
        "let m = makeMatrix(); res.output[0] = first(m.c1, res.output[0] == 0 ? tick() : 0);",
        &[
            "let _g_operand_0 = m[1u];\n  var _g_conditional_0",
            "first(_g_operand_0, _g_conditional_0)",
        ],
    );
}

#[test]
fn user_field() {
    check_calls(
        "@ValueType class ColumnData { constructor(public c0: u32) {} }",
        "let m = new ColumnData(1); m.c0 = 2; res.output[0] = m.c0;",
        &["m.c0 = 2u;", "output[0u] = m.c0;"],
    );
}

#[test]
fn fragment_column() {
    let module = generate(
        r#"
import { RenderPipelineSpec, renderPipeline } from "./typegpu";
import { Mat4x4f, Vec2f, Vec4f } from "./typegpu-types";
@ValueType class Vertex { constructor(public xy: Vec2f) {} }
@ValueType class Varyings { constructor(public position: Vec4f) {} }
function vert(vertex: Vertex): Varyings { return new Varyings(new Vec4f(vertex.xy.x, vertex.xy.y, 0.0, 1.0)); }
function frag(input: Varyings): Vec4f {
  const v = input.position;
  const m = new Mat4x4f(v, v, v, v);
  return m.c2;
}
const pipe: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
"#,
    );
    let wgsl = &module.pipelines[0].1;
    validate(wgsl);
    assert!(wgsl.contains("return m[2u];"), "{wgsl}");
}
