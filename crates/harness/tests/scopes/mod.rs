//! PI14 validation-scope gates over checked program HIR.

use std::path::{Path, PathBuf};

use subscript_compiler::hir::{AsyncCallee, Callee, Expr, ExprKind, Module, Stmt, TplPart};
use subscript_compiler::CheckOptions;
use subscript_compiler::Type;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScopeCall {
    Push,
    Pop,
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("harness crate is under the repository root")
        .to_path_buf()
}

fn is_program(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let Some(stem) = name.strip_suffix(".ts") else {
        return false;
    };
    let bytes = stem.as_bytes();
    bytes.len() >= 5
        && matches!(bytes[0], b'a' | b'b' | b'x')
        && bytes[1].is_ascii_digit()
        && bytes[2].is_ascii_digit()
        && bytes[3] == b'-'
}

fn programs() -> Vec<PathBuf> {
    let directory = repository_root().join("programs");
    let mut programs = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        .map(|entry| entry.expect("program entry").path())
        .filter(|path| is_program(path))
        .collect::<Vec<_>>();
    programs.sort();
    assert!(!programs.is_empty(), "scope program list is empty");
    programs
}

fn is_validation_filter(module: &Module, expression: &Expr) -> bool {
    if matches!(&expression.kind, ExprKind::Str(filter) if filter == "validation") {
        return true;
    }
    let (ExprKind::Int(value), Type::StringAlias(alias)) = (&expression.kind, &expression.ty)
    else {
        return false;
    };
    let Some(definition) = module.string_aliases.get(alias.0) else {
        return false;
    };
    definition
        .members
        .iter()
        .position(|member| member == "validation")
        .and_then(|index| definition.member_discriminant(index))
        == Some(*value)
}

fn visit_expr(module: &Module, expression: &Expr, program_name: &str, calls: &mut Vec<ScopeCall>) {
    if expression.pos.file == program_name {
        match &expression.kind {
            ExprKind::Call {
                callee: Callee::Method { name, .. },
                args,
            } if name == "pushErrorScope"
                && matches!(args.as_slice(), [filter] if is_validation_filter(module, filter)) =>
            {
                calls.push(ScopeCall::Push);
            }
            ExprKind::AsyncCall {
                callee: AsyncCallee::Method { name, .. },
                ..
            } if name == "popErrorScope" => calls.push(ScopeCall::Pop),
            _ => {}
        }
    }

    macro_rules! visit {
        ($child:expr) => {
            visit_expr(module, $child, program_name, calls)
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
            for arg in args {
                visit!(arg);
            }
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => {
            for arg in args {
                visit!(arg);
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
        ExprKind::Lambda { body, .. } => visit_statements(module, body, program_name, calls),
        ExprKind::Yield(value) => {
            if let Some(value) = value {
                visit!(value);
            }
        }
        ExprKind::AsyncCall { callee, args } => {
            if let AsyncCallee::Method { receiver, .. } = callee {
                visit!(receiver);
            }
            for arg in args {
                visit!(arg);
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

fn visit_statements(
    module: &Module,
    statements: &[Stmt],
    program_name: &str,
    calls: &mut Vec<ScopeCall>,
) {
    for statement in statements {
        match statement {
            Stmt::Let { init, .. } | Stmt::Expr(init) => {
                visit_expr(module, init, program_name, calls)
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    visit_expr(module, value, program_name, calls);
                }
            }
            Stmt::If {
                cond, then, els, ..
            } => {
                visit_expr(module, cond, program_name, calls);
                visit_statements(module, then, program_name, calls);
                if let Some(els) = els {
                    visit_statements(module, els, program_name, calls);
                }
            }
            Stmt::While { cond, body, .. } => {
                visit_expr(module, cond, program_name, calls);
                visit_statements(module, body, program_name, calls);
            }
            Stmt::For {
                init,
                cond,
                step,
                body,
                ..
            } => {
                if let Some(init) = init {
                    visit_statements(
                        module,
                        std::slice::from_ref(init.as_ref()),
                        program_name,
                        calls,
                    );
                }
                if let Some(cond) = cond {
                    visit_expr(module, cond, program_name, calls);
                }
                if let Some(step) = step {
                    visit_expr(module, step, program_name, calls);
                }
                visit_statements(module, body, program_name, calls);
            }
            Stmt::ForOf { subject, body, .. } => {
                visit_expr(module, subject, program_name, calls);
                visit_statements(module, body, program_name, calls);
            }
            Stmt::Switch { disc, cases, .. } => {
                visit_expr(module, disc, program_name, calls);
                for case in cases {
                    if let Some(test) = &case.test {
                        visit_expr(module, test, program_name, calls);
                    }
                    visit_statements(module, &case.body, program_name, calls);
                }
            }
            Stmt::Block(body) => visit_statements(module, body, program_name, calls),
            Stmt::Break(_) | Stmt::Continue(_) => {}
            _ => {}
        }
    }
}

fn scope_failure(program: &Path) -> Option<String> {
    let files = subscript_typegpu_harness::program_files(program)
        .unwrap_or_else(|error| panic!("load {}: {error}", program.display()));
    let module = subscript_compiler::check_program_with(&files, &CheckOptions::default())
        .unwrap_or_else(|diagnostics| panic!("check {}: {diagnostics:?}", program.display()));
    let program_name = program
        .file_name()
        .and_then(|name| name.to_str())
        .expect("UTF-8 program name");
    let Some(main) = module
        .functions
        .iter()
        .find(|function| function.name == "main" && function.pos.file == program_name)
    else {
        return Some(format!("{program_name}: no program main in checked HIR"));
    };
    let mut calls = Vec::new();
    visit_statements(&module, &main.body, program_name, &mut calls);
    (calls != [ScopeCall::Push, ScopeCall::Pop]).then(|| {
        format!(
            "{program_name}: expected one pushErrorScope(\"validation\") before one popErrorScope, found {calls:?}"
        )
    })
}

#[test]
fn every_program_wraps_pipeline_creation_in_a_validation_scope() {
    let failures = programs()
        .iter()
        .filter_map(|program| scope_failure(program))
        .collect::<Vec<_>>();
    assert!(
        failures.is_empty(),
        "PI14 scope failures:\n{}",
        failures.join("\n")
    );
}
