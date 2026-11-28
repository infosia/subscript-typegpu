use super::{generate, validate, CALL_KERNEL};

const COUNTER: &str = r#"
import { PrivateVar, privateVar } from "./typegpu";
import { Vec2f } from "./typegpu-types";
const counter: PrivateVar<u32> = privateVar<u32>(0);
function tick(): u32 { counter.$ = counter.$ + 1; return counter.$; }
function pair(a: u32, b: u32): u32 { return 10 * a + b; }
function nextVec(): Vec2f { const x = tick() as f32; return new Vec2f(x, x); }
function edgeVec(): Vec2f { return nextVec(); }
@ValueType class Digits {
  tens: u32; ones: u32;
  constructor(ones: u32, tens: u32) { this.ones = ones; this.tens = tens; }
  plus(x: u32): u32 { return this.tens * 10 + x; }
}
function nextDigits(): Digits { return new Digits(0, tick()); }
"#;

fn check(body: &str, sequence: &[&str]) {
    let source = CALL_KERNEL
        .replace("DECLARATIONS", COUNTER)
        .replace("BODY", &format!("const k = res.output[0]; {body}"));
    let wgsl = &generate(&source).pipelines[0].1;
    assert_sequence(wgsl, sequence);
}

fn assert_sequence(wgsl: &str, sequence: &[&str]) {
    validate(wgsl);
    let mut rest = wgsl;
    for text in sequence {
        rest = rest
            .split_once(text)
            .unwrap_or_else(|| panic!("missing {text} in {rest}"))
            .1;
    }
}

#[test]
fn arguments() {
    check(
        "res.output[1] = pair(tick(), k == 0 ? tick() : 0);",
        &[
            "let _g_operand_0 = tick();",
            "if (k == 0u)",
            "_g_conditional_0 = tick();",
            "pair(_g_operand_0, _g_conditional_0)",
        ],
    );
}

#[test]
fn step() {
    check(
        "res.output[1] = nextVec().step(edgeVec()).x as u32;",
        &[
            "let _g_operand_0 = nextVec();",
            "step(edgeVec(), _g_operand_0)",
        ],
    );
}

#[test]
fn smoothstep() {
    check(
        "res.output[1] = nextVec().smoothstep(edgeVec(), edgeVec()).x as u32;",
        &[
            "let _g_operand_0 = nextVec();",
            "smoothstep(edgeVec(), edgeVec(), _g_operand_0)",
        ],
    );
}

#[test]
fn logic_and() {
    check(
        "const v = k != 0 && (k == 0 ? tick() > 0 : false); res.output[1] = v ? 1 : counter.$;",
        &[
            "var _g_logic_0: bool = k != 0u;",
            "if (_g_logic_0) {\n    var _g_conditional_",
            "if (k == 0u)",
            "tick() > 0u;",
            "_g_logic_0 = _g_conditional_0;\n  }",
        ],
    );
}

#[test]
fn logic_or() {
    check(
        "const v = k != 0 || (k != 0 ? tick() > 0 : false); res.output[1] = v ? 1 : counter.$;",
        &[
            "var _g_logic_0: bool = k != 0u;",
            "if (!_g_logic_0) {\n    var _g_conditional_",
            "if (k != 0u)",
            "tick() > 0u;",
            "_g_logic_0 = _g_conditional_0;\n  }",
        ],
    );
}

#[test]
fn schema() {
    check(
        "const d = new Digits(tick(), tick()); res.output[1] = pair(d.tens, d.ones);",
        &["let _g_operand_0 = tick();", "Digits(tick(), _g_operand_0)"],
    );
}

#[test]
fn method() {
    check(
        "res.output[1] = nextDigits().plus(k == 0 ? tick() : 0);",
        &[
            "let _g_operand_0 = nextDigits();",
            "if (k == 0u)",
            "_g_conditional_0 = tick();",
            "Digits_plus(_g_operand_0, _g_conditional_0)",
        ],
    );
}

#[test]
fn independent_operands() {
    check(
        "res.output[1] = pair(k, k == 0 ? tick() : 0) + pair(tick(), tick());",
        &[
            "let k = output[0u];\n  var _g_conditional_0",
            "pair(k, _g_conditional_0) + pair(tick(), tick())",
        ],
    );
}

#[test]
fn nested_arm() {
    check(
        "res.output[1] = pair(tick(), k == 0 ? pair(tick(), k == 0 ? tick() : 0) : 0);",
        &[
            " = tick();\n  var _g_conditional_1",
            "if (k == 0u) {\n    let _g_operand_",
            " = tick();\n    var _g_conditional_0",
            "_g_conditional_0 = tick();",
            "_g_conditional_1 = pair(_g_operand_",
            "output[1u] = pair(_g_operand_",
        ],
    );
}

#[test]
fn mutable_read_and_default() {
    let declarations = format!("{COUNTER} function add(a: u32, b: u32 = counter.$ == 0 ? tick() : 0): u32 {{ return a + b; }}");
    let source = CALL_KERNEL
        .replace("DECLARATIONS", &declarations)
        .replace("BODY", "let a = res.output[0]; res.output[1] = add(a);");
    let wgsl = &generate(&source).pipelines[0].1;
    validate(wgsl);
    assert!(
        wgsl.contains("let _g_operand_0 = a;\n  var _g_conditional_0"),
        "{wgsl}"
    );
}

#[test]
fn indexed_store() {
    check(
        "res.output[tick()] = k == 0 ? tick() : 0;",
        &[
            "let _g_operand_0 = tick();",
            "if (k == 0u)",
            "output[_g_operand_0] = _g_conditional_0;",
        ],
    );
}

#[test]
fn compound_field() {
    check(
        "let d = new Digits(0, 1); d.tens += k == 0 ? tick() : 0; res.output[1] = d.tens;",
        &[
            "let _g_operand_0 = d.tens;",
            "if (k == 0u)",
            "d.tens = _g_operand_0 + _g_conditional_0;",
        ],
    );
}

#[test]
fn capture_before_a_reordered_capture() {
    check("const d = new Digits(tick(), tick()); res.output[1] = pair(tick(), new Digits(tick(), tick()).tens);",
        &[" = tick();\n  let _g_operand_", " = tick();\n  output[1u] = pair(_g_operand_", ", Digits(tick(), _g_operand_"]);
}

#[test]
fn pure_prelude_before_effectful_text() {
    check(
        "res.output[1] = pair(tick(), pair(k == 0 ? 1 : 2, tick()));",
        &[
            "let k = output[0u];\n  var _g_conditional_0",
            "pair(tick(), pair(_g_conditional_0, tick()))",
        ],
    );
}

fn binding_kernel(declarations: &str, fields: &str, body: &str) -> String {
    let source = CALL_KERNEL
        .replace("DECLARATIONS", &format!("{COUNTER} {declarations}"))
        .replace("public output: MutStorage<u32>", fields)
        .replace("BODY", body);
    generate(&source).pipelines.remove(0).1
}

#[test]
fn sample_level_keeps_texture_and_sampler_bindings_inline() {
    let wgsl = binding_kernel(
        r#"import { Texture2d, Sampler } from "./typegpu";"#,
        "public source: Texture2d<f32>, public nearest: Sampler, public output: MutStorage<u32>",
        "const k = res.output[0]; res.output[1] = res.source.sampleLevel(res.nearest, new Vec2f(k == 0 ? (tick() as f32) : 0.0, 0.0), 0.0).x as u32;",
    );
    for line in wgsl
        .lines()
        .map(str::trim_start)
        .filter(|line| line.starts_with("let "))
    {
        assert!(
            !line.ends_with("= source;") && !line.ends_with("= nearest;"),
            "{line}"
        );
    }
    assert_sequence(
        &wgsl,
        &[
            "if (k == 0u)",
            "f32(tick())",
            "textureSampleLevel(source, nearest,",
        ],
    );
}

#[test]
fn atomic_place_index_precedes_argument_prelude() {
    let wgsl = binding_kernel(
        r#"import { AtomicU32 } from "./typegpu-types";
@ValueType class Cell { constructor(public value: AtomicU32) {} }"#,
        "public cells: MutStorage<Cell>, public output: MutStorage<u32>",
        "const k = res.output[0]; res.cells[tick()].value.add(k == 0 ? tick() : 0);",
    );
    assert_sequence(
        &wgsl,
        &[
            "let _g_operand_0 = tick();",
            "if (k == 0u)",
            "_g_conditional_0 = tick();",
            "atomicAdd(&cells[_g_operand_0].value, _g_conditional_0);",
        ],
    );
}

#[test]
fn texture_store_coordinates_precede_value_prelude() {
    let wgsl = binding_kernel(
        r#"import { StorageTexture2d, Rgba8unorm } from "./typegpu";
import { Vec2i, Vec4f } from "./typegpu-types";"#,
        "public target: StorageTexture2d<Rgba8unorm>, public output: MutStorage<u32>",
        "const k = res.output[0]; res.target.store(new Vec2i(tick() as i32, 0), new Vec4f(k == 0 ? tick() as f32 : 0.0, 0.0, 0.0, 1.0));",
    );
    assert_sequence(
        &wgsl,
        &[
            "let _g_operand_0 = vec2<i32>(i32(tick()), 0i);",
            "if (k == 0u)",
            "_g_conditional_0 = f32(tick());",
            "textureStore(target_, _g_operand_0, vec4<f32>(_g_conditional_0,",
        ],
    );
}

#[test]
fn while_condition_evaluates_operands_inside_loop() {
    check(
        "while (pair(tick(), k == 0 ? tick() : 0) < 10) { res.output[1] = k; }",
        &[
            "loop {",
            "let _g_operand_0 = tick();",
            "if (k == 0u)",
            "_g_conditional_0 = tick();",
            "if (!(pair(_g_operand_0, _g_conditional_0) < 10u))",
            "break;",
            "output[1u] = k;",
        ],
    );
}

#[test]
fn for_step_evaluates_operands_in_continuing_block() {
    check(
        "for (let i: u32 = 0; i < 10; i = pair(tick(), k == 0 ? tick() : 0)) { res.output[1] = i; }",
        &[
            "loop {",
            "output[1u] = i;",
            "continuing {",
            "let _g_operand_0 = tick();",
            "if (k == 0u)",
            "_g_conditional_0 = tick();",
            "i = pair(_g_operand_0, _g_conditional_0);",
        ],
    );
}

#[test]
fn switch_discriminant_evaluates_operands_before_selection() {
    check(
        "switch (pair(tick(), k == 0 ? tick() : 0)) { case 1: res.output[1] = k; break; default: break; }",
        &[
            "let _g_operand_0 = tick();",
            "if (k == 0u)",
            "_g_conditional_0 = tick();",
            "switch (pair(_g_operand_0, _g_conditional_0))",
            "output[1u] = k;",
        ],
    );
}
