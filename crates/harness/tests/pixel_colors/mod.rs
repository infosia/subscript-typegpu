//! RN14 and RN21 gates for pixel-oracle color literals.
//! The margin exceeds f32 representation error near conversion boundaries.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use subscript_compiler::hir::{
    AsyncCallee, Callee, Expr, ExprKind, Function, Module, Stmt, TplPart,
};
use subscript_compiler::{CheckOptions, Type};

const FORMATS: &[&str] = &[
    "rgba8unorm",
    "rgba8unorm-srgb",
    "bgra8unorm",
    "bgra8unorm-srgb",
];
const ALLOW_LIST: &[(&str, f64, &str)] = &[];

#[derive(Clone, Copy)]
struct ColorLiteral {
    line: u32,
    col: u32,
    value: f64,
}

#[derive(Default)]
struct Inspection {
    pixel_oracle: bool,
    blend: bool,
    colors: BTreeMap<(u32, u32, u64), ColorLiteral>,
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("harness crate is under the repository root")
        .to_path_buf()
}

fn is_x_program(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let Some(stem) = name.strip_suffix(".ts") else {
        return false;
    };
    let bytes = stem.as_bytes();
    bytes.len() >= 5
        && bytes[0] == b'x'
        && bytes[1].is_ascii_digit()
        && bytes[2].is_ascii_digit()
        && bytes[3] == b'-'
}

fn programs() -> Vec<PathBuf> {
    let directory = repository_root().join("programs");
    let mut programs = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        .map(|entry| entry.expect("program entry").path())
        .filter(|path| is_x_program(path))
        .collect::<Vec<_>>();
    programs.sort();
    assert!(!programs.is_empty(), "pixel color program list is empty");
    programs
}

fn string_member<'a>(module: &'a Module, expression: &'a Expr) -> Option<&'a str> {
    if let ExprKind::Str(value) = &expression.kind {
        return Some(value);
    }
    let (ExprKind::Int(value), Type::StringAlias(alias)) = (&expression.kind, &expression.ty)
    else {
        return None;
    };
    let definition = module.string_aliases.get(alias.0)?;
    definition
        .members
        .iter()
        .enumerate()
        .position(|(index, _member)| definition.member_discriminant(index) == Some(*value))
        .and_then(|index| definition.members.get(index))
        .map(String::as_str)
}

fn descriptor_sets(module: &Module, expression: &Expr, class_name: &str, field: &str) -> bool {
    let ExprKind::DescriptorLit { class, fields } = &expression.kind else {
        return false;
    };
    let definition = &module.classes[class.0];
    definition.name == class_name
        && definition.pos.file == "typegpu.ts"
        && definition
            .fields
            .iter()
            .position(|candidate| candidate.name == field)
            .and_then(|index| fields.get(index))
            .is_some_and(Option::is_some)
}

fn number_literal(expression: &Expr) -> Option<f64> {
    match expression.kind {
        ExprKind::Float(value) => Some(value),
        ExprKind::Int(value) => Some(value as f64),
        _ => None,
    }
}

fn inspect_expr(
    module: &Module,
    expression: &Expr,
    program_name: &str,
    inspection: &mut Inspection,
) {
    if expression.pos.file == program_name {
        if string_member(module, expression).is_some_and(|value| FORMATS.contains(&value)) {
            inspection.pixel_oracle = true;
        }
        if matches!(
            &expression.kind,
            ExprKind::Call {
                callee: Callee::Method { name, .. },
                ..
            } if name == "copyTextureToBuffer"
        ) {
            inspection.pixel_oracle = true;
        }
        if descriptor_sets(module, expression, "RenderPipelineSpec", "blend") {
            inspection.blend = true;
        }
        if let ExprKind::New { class, args } = &expression.kind {
            let definition = &module.classes[class.0];
            if definition.pos.file == "typegpu-types.ts"
                && matches!(definition.name.as_str(), "Vec3f" | "Vec4f")
                && args
                    .iter()
                    .all(|argument| number_literal(argument).is_some())
            {
                for argument in args {
                    let value = number_literal(argument).expect("literal vector argument");
                    let literal = ColorLiteral {
                        line: argument.pos.line,
                        col: argument.pos.col,
                        value,
                    };
                    inspection.colors.insert(
                        (literal.line, literal.col, literal.value.to_bits()),
                        literal,
                    );
                }
            }
        }
    }

    macro_rules! visit {
        ($child:expr) => {
            inspect_expr(module, $child, program_name, inspection)
        };
    }
    match &expression.kind {
        ExprKind::Unary { operand, .. }
        | ExprKind::Cast(operand)
        | ExprKind::Length(operand)
        | ExprKind::Field { obj: operand, .. }
        | ExprKind::JsonResultValue(operand) => visit!(operand),
        ExprKind::Binary { left, right, .. }
        | ExprKind::Assign {
            target: left,
            value: right,
            ..
        } => {
            visit!(left);
            visit!(right);
        }
        ExprKind::Call { callee, args } => {
            match callee {
                Callee::Value(value) => visit!(value),
                Callee::Method { recv, .. } => visit!(recv),
                _ => {}
            }
            for argument in args {
                visit!(argument);
            }
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => {
            for argument in args {
                visit!(argument);
            }
        }
        ExprKind::DescriptorLit { fields, .. } => {
            for value in fields.iter().flatten() {
                visit!(value);
            }
        }
        ExprKind::Index { obj, index, .. } => {
            visit!(obj);
            visit!(index);
        }
        ExprKind::ArraySpreadLit(items) => {
            for item in items {
                visit!(&item.expr);
            }
        }
        ExprKind::Template(parts) => {
            for part in parts {
                if let TplPart::Expr(value) = part {
                    visit!(value);
                }
            }
        }
        ExprKind::Lambda { body, .. } => {
            inspect_statements(module, body, program_name, inspection);
        }
        ExprKind::Yield(value) => {
            if let Some(value) = value {
                visit!(value);
            }
        }
        ExprKind::AsyncCall { callee, args } => {
            if let AsyncCallee::Method { receiver, .. } = callee {
                visit!(receiver);
            }
            for argument in args {
                visit!(argument);
            }
        }
        ExprKind::Cond { cond, then, els } => {
            visit!(cond);
            visit!(then);
            visit!(els);
        }
        _ => {}
    }
}

fn inspect_statements(
    module: &Module,
    statements: &[Stmt],
    program_name: &str,
    inspection: &mut Inspection,
) {
    for statement in statements {
        match statement {
            Stmt::Let { init, .. } | Stmt::Expr(init) => {
                inspect_expr(module, init, program_name, inspection);
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    inspect_expr(module, value, program_name, inspection);
                }
            }
            Stmt::If {
                cond, then, els, ..
            } => {
                inspect_expr(module, cond, program_name, inspection);
                inspect_statements(module, then, program_name, inspection);
                if let Some(els) = els {
                    inspect_statements(module, els, program_name, inspection);
                }
            }
            Stmt::While { cond, body, .. } => {
                inspect_expr(module, cond, program_name, inspection);
                inspect_statements(module, body, program_name, inspection);
            }
            Stmt::For {
                init,
                cond,
                step,
                body,
                ..
            } => {
                if let Some(init) = init {
                    inspect_statements(
                        module,
                        std::slice::from_ref(init.as_ref()),
                        program_name,
                        inspection,
                    );
                }
                if let Some(cond) = cond {
                    inspect_expr(module, cond, program_name, inspection);
                }
                if let Some(step) = step {
                    inspect_expr(module, step, program_name, inspection);
                }
                inspect_statements(module, body, program_name, inspection);
            }
            Stmt::ForOf { subject, body, .. } => {
                inspect_expr(module, subject, program_name, inspection);
                inspect_statements(module, body, program_name, inspection);
            }
            Stmt::Switch { disc, cases, .. } => {
                inspect_expr(module, disc, program_name, inspection);
                for case in cases {
                    if let Some(test) = &case.test {
                        inspect_expr(module, test, program_name, inspection);
                    }
                    inspect_statements(module, &case.body, program_name, inspection);
                }
            }
            Stmt::Block(body) => {
                inspect_statements(module, body, program_name, inspection);
            }
            Stmt::Break(_) | Stmt::Continue(_) => {}
            _ => {}
        }
    }
}

fn inspect_function(
    module: &Module,
    function: &Function,
    program_name: &str,
    inspection: &mut Inspection,
) {
    for parameter in &function.params {
        if let Some(default) = &parameter.default {
            inspect_expr(module, default, program_name, inspection);
        }
    }
    inspect_statements(module, &function.body, program_name, inspection);
}

fn inspect_module(module: &Module, program_name: &str) -> Inspection {
    let mut inspection = Inspection::default();
    for global in &module.globals {
        inspect_expr(module, &global.init, program_name, &mut inspection);
    }
    for function in &module.functions {
        inspect_function(module, function, program_name, &mut inspection);
    }
    for class in &module.classes {
        for field in &class.fields {
            if let Some(initializer) = &field.init {
                inspect_expr(module, initializer, program_name, &mut inspection);
            }
        }
        if let Some(constructor) = &class.ctor {
            inspect_function(module, constructor, program_name, &mut inspection);
        }
        for method in &class.methods {
            inspect_function(module, method, program_name, &mut inspection);
        }
    }
    inspect_statements(module, &module.top_level, program_name, &mut inspection);
    inspection
}

fn is_allowed(program_name: &str, literal: ColorLiteral) -> bool {
    ALLOW_LIST.iter().any(|(program, value, _reason)| {
        *program == program_name && value.to_bits() == literal.value.to_bits()
    })
}

fn failure(program: &Path) -> Vec<String> {
    let files = subscript_typegpu_harness::program_files(program)
        .unwrap_or_else(|error| panic!("load {}: {error}", program.display()));
    let module = subscript_compiler::check_program_with(&files, &CheckOptions::default())
        .unwrap_or_else(|diagnostics| panic!("check {}: {diagnostics:?}", program.display()));
    let program_name = program
        .file_name()
        .and_then(|name| name.to_str())
        .expect("UTF-8 program name");
    let inspection = inspect_module(&module, program_name);
    if !inspection.pixel_oracle {
        return Vec::new();
    }

    let mut failures = Vec::new();
    for literal in inspection.colors.values().copied() {
        if is_allowed(program_name, literal) {
            continue;
        }
        let product = (literal.value as f32) * 255.0f32;
        let half_distance = (product.rem_euclid(1.0) - 0.5).abs();
        if half_distance < 0.01 {
            failures.push(format!(
                "{program_name}:{}: literal {} has product {product} with 255 and violates RN14",
                literal.line, literal.value
            ));
        }
        let integer_distance = (product - product.round()).abs();
        if inspection.blend && integer_distance > 0.01 {
            failures.push(format!(
                "{program_name}:{}: literal {} has product {product} with 255 and violates RN21",
                literal.line, literal.value
            ));
        }
    }
    failures
}

#[test]
fn every_pixel_oracle_color_literal_has_stable_unorm_conversion() {
    let failures = programs()
        .iter()
        .flat_map(|program| failure(program))
        .collect::<Vec<_>>();
    assert!(
        failures.is_empty(),
        "pixel color failures:\n{}",
        failures.join("\n")
    );
}
