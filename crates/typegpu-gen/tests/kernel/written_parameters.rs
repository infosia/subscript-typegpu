use super::{check_calls, generate, validate};

#[test]
fn scalar_statements() {
    check_calls(
        "function bump(x: u32): u32 { x = x + 1; x += 2; x++; return x; }",
        "res.output[0] = bump(3);",
        &["fn bump(_g_parameter_x: u32) -> u32 {\n  var x = _g_parameter_x;\n  x = x + 1u;\n  x += 2u;\n  x += 1u;\n  return x;\n}"],
    );
}

#[test]
fn vector_field() {
    check_calls(
        r#"import { Vec2f } from "./typegpu-types";
function bump(v: Vec2f): u32 { v.x = 4.0; return v.x as u32; }"#,
        "res.output[0] = bump(new Vec2f(1.0, 2.0));",
        &["fn bump(_g_parameter_v: vec2<f32>) -> u32 {\n  var v = _g_parameter_v;\n  v.x = 4.0f;"],
    );
}

#[test]
fn schema_fields_and_elements() {
    let module = generate(
        r#"
import { ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
@ValueType class Item { constructor(public value: u32) {} }
@ValueType class Batch { constructor(public item: Item, public items: FixedArray<Item, 2>) {} }
class Layout { constructor(public batches: MutStorage<Batch>) {} }
function edit(a: Batch, b: Batch, c: Batch, i: i32): Batch {
  a.item.value = 5;
  b.items[i] = a.item;
  c.items[i].value = b.items[i].value;
  return c;
}
function kernel(res: Layout): void { res.batches[1] = edit(res.batches[0], res.batches[0], res.batches[0], 1); }
const pipe: ComputePipelineSpec = computePipeline(kernel, { name: "pipe", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &module.pipelines[0].1;
    validate(wgsl);
    assert!(
        wgsl.contains(
            "var a = _g_parameter_a;\n  var b = _g_parameter_b;\n  var c = _g_parameter_c;"
        ),
        "{wgsl}"
    );
    for text in [
        "a.item.value = 5u;",
        "b.items[i] = a.item;",
        "c.items[i].value = b.items[i].value;",
    ] {
        assert!(wgsl.contains(text), "{wgsl}");
    }
}

#[test]
fn read_and_written_names() {
    check_calls(
        "function add(x: u32, _g_parameter_x: u32): u32 { x += _g_parameter_x; return x; }",
        "res.output[0] = add(1, 2);",
        &["fn add(_g_parameter_x: u32, _g_parameter_x_: u32) -> u32 {\n  var x = _g_parameter_x;\n  x += _g_parameter_x_;"],
    );
}

#[test]
fn declaration_order() {
    check_calls(
        "function edit(z: u32, a: u32): u32 { a++; z += a; return z; }",
        "res.output[0] = edit(2, 3);",
        &["fn edit(_g_parameter_z: u32, _g_parameter_a: u32) -> u32 {\n  var z = _g_parameter_z;\n  var a = _g_parameter_a;"],
    );
}

#[test]
fn generic_instance() {
    check_calls(
        "function replace<T>(x: T, y: T): T { x = y; return x; }",
        "res.output[0] = replace<u32>(1, 2);",
        &["fn replace_u32(_g_parameter_x: u32, y: u32) -> u32 {\n  var x = _g_parameter_x;\n  x = y;"],
    );
}

#[test]
fn method_argument() {
    check_calls(
        "@ValueType class Item { constructor(public value: u32) {} plus(x: u32): u32 { x += this.value; return x; } }",
        "res.output[0] = new Item(2).plus(3);",
        &["fn Item_plus(self_: Item, _g_parameter_x: u32) -> u32 {\n  var x = _g_parameter_x;\n  x += self_.value;"],
    );
}

#[test]
fn local_function() {
    check_calls(
        "function outer(x: u32): u32 { function inner(x: u32): u32 { x++; return x; } return inner(x); }",
        "res.output[0] = outer(3);",
        &["fn outer_inner(_g_parameter_x: u32) -> u32 {\n  var x = _g_parameter_x;", "fn outer(x: u32) -> u32 {\n  return outer_inner(x);"],
    );
}

#[test]
fn render_inputs_and_helper() {
    let module = generate(
        r#"
import { RenderPipelineSpec, renderPipeline } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@ValueType class Vertex { constructor(public xy: Vec2f) {} }
@ValueType class Varyings { constructor(public position: Vec4f) {} }
function bump(x: f32): f32 { x += 1.0; return x; }
function vert(vertex: Vertex): Varyings {
  vertex.xy.x = bump(vertex.xy.x);
  return new Varyings(new Vec4f(vertex.xy.x, vertex.xy.y, 0.0, 1.0));
}
function frag(input: Varyings): Vec4f { input.position.y = 1.0; return input.position; }
const pipe: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
"#,
    );
    let wgsl = &module.pipelines[0].1;
    validate(wgsl);
    for text in [
        "fn vert(_g_parameter_vertex: Vertex) -> Varyings {\n  var vertex = _g_parameter_vertex;",
        "fn frag(_g_parameter_input: Varyings) -> @location(0u) vec4<f32> {\n  var input = _g_parameter_input;",
        "fn bump(_g_parameter_x: f32) -> f32 {\n  var x = _g_parameter_x;",
    ] {
        assert!(wgsl.contains(text), "{wgsl}");
    }
}

#[test]
fn nested_statements() {
    check_calls(
        "function edit(x: u32, y: u32): u32 { for (let i: u32 = 0; i < 2; i++) { x++; } if (x > 1) { y += x; } return y; }",
        "res.output[0] = edit(0, 2);",
        &["var x = _g_parameter_x;\n  var y = _g_parameter_y;", "    x += 1u;", "    y += x;"],
    );
}

#[test]
fn switch_while_and_do_while() {
    check_calls(
        r#"function edit(x: u32, y: u32, z: u32): u32 {
  switch (y) { case 0: x++; break; default: break; }
  while (y < x) { y++; }
  do { z++; } while (z < y);
  return z;
}"#,
        "res.output[0] = edit(0, 0, 0);",
        &[
            "var x = _g_parameter_x;\n  var y = _g_parameter_y;\n  var z = _g_parameter_z;",
            "x += 1u;",
            "y += 1u;",
            "z += 1u;",
        ],
    );
}

#[test]
fn logical_assignment_in_entry_local_function() {
    check_calls(
        "",
        "function either(p: boolean, v: boolean): boolean { p ||= v; return p; } if (either(false, true)) { res.output[0] = 1; }",
        &["fn kernel_either(_g_parameter_p: bool, v: bool) -> bool {\n  var p = _g_parameter_p;\n  if (!(p)) {\n    p = v;"],
    );
}

#[test]
fn read_only() {
    check_calls(
        "function sum(a: u32, b: u32): u32 { return a + b; }",
        "res.output[0] = sum(2, 3);",
        &["fn sum(a: u32, b: u32) -> u32 {\n  return a + b;\n}"],
    );
}

#[test]
fn default_argument() {
    check_calls(
        "function bump(x: u32 = 7): u32 { x++; return x; }",
        "res.output[0] = bump();",
        &[
            "fn bump(_g_parameter_x: u32) -> u32 {\n  var x = _g_parameter_x;",
            "bump(7u)",
        ],
    );
}

#[test]
fn block_local_shadows_parameter() {
    check_calls(
        "function keep(x: u32): u32 { { let x: u32 = 0; x++; } return x; }",
        "res.output[0] = keep(2);",
        &["fn keep(x: u32) -> u32 {\n  {\n    var x = 0u;\n    x += 1u;\n  }\n  return x;"],
    );
}

#[test]
fn mangled_names_stay_distinct() {
    check_calls(
        "function add(abs: u32, _g_parameter_abs: u32): u32 { abs += _g_parameter_abs; return abs; }",
        "res.output[0] = add(1, 2);",
        &["fn add(_g_parameter_abs__: u32, _g_parameter_abs_: u32) -> u32 {\n  var abs_ = _g_parameter_abs__;\n  abs_ += _g_parameter_abs_;"],
    );
}
