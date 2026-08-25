//! EG11 gate for binding-wrapper method calls in authored programs.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use subscript_compiler::hir::{
    AsyncCallee, Callee, Expr, ExprKind, Function, Module, Stmt, TplPart,
};
use subscript_compiler::{CheckOptions, Type};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("harness crate is under the repository root")
        .to_path_buf()
}

fn collect_ts_files(directory: &Path, recursive: bool, files: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()));
    for entry in entries {
        let path = entry.expect("read source entry").path();
        if recursive && path.is_dir() {
            collect_ts_files(&path, true, files);
        } else if path.is_file() && path.extension() == Some(OsStr::new("ts")) {
            files.push(path);
        }
    }
}

fn authored_sources() -> Vec<PathBuf> {
    let root = repository_root();
    let mut files = Vec::new();
    collect_ts_files(&root.join("programs"), false, &mut files);
    collect_ts_files(&root.join("examples"), true, &mut files);
    files.sort();
    assert!(!files.is_empty(), "authored source list is empty");
    files
}

fn wrapper_name<'a>(module: &'a Module, receiver: &Expr) -> Option<&'a str> {
    let Type::Class(id) = &receiver.ty else {
        return None;
    };
    let class = module.classes.get(id.0)?;
    if class.pos.file != "typegpu.ts"
        || !matches!(
            class.name.split_once('<').map(|(name, _)| name),
            Some("Storage" | "MutStorage" | "WorkgroupArray")
        )
    {
        return None;
    }
    Some(class.name.as_str())
}

fn is_authored_method(source: &str, expression: &Expr, name: &str) -> bool {
    let Some(line) = expression
        .pos
        .line
        .checked_sub(1)
        .and_then(|line| source.lines().nth(line as usize))
    else {
        return false;
    };
    let Some(suffix) = expression
        .pos
        .col
        .checked_sub(1)
        .and_then(|column| line.get(column as usize..))
    else {
        return false;
    };
    let needle = format!(".{name}(");
    let Some(method) = suffix.find(&needle) else {
        return false;
    };
    suffix.find('[').map_or(true, |index| method < index)
}

fn inspect_expr(
    module: &Module,
    source: &str,
    expression: &Expr,
    source_name: &str,
    display_name: &str,
    failures: &mut Vec<String>,
) {
    if expression.pos.file == source_name {
        if let ExprKind::Call {
            callee: Callee::Method { recv, name },
            ..
        } = &expression.kind
        {
            if matches!(name.as_str(), "get" | "set")
                && is_authored_method(source, expression, name)
            {
                if let Some(wrapper) = wrapper_name(module, recv) {
                    let replacement = if name == "get" {
                        "`x[i]`"
                    } else {
                        "`x[i] = v`"
                    };
                    failures.push(format!(
                        "{display_name}:{}: EG11: use {replacement} for {wrapper} instead of `{name}`",
                        expression.pos.line
                    ));
                }
            }
        }
    }

    macro_rules! visit {
        ($child:expr) => {
            inspect_expr(module, source, $child, source_name, display_name, failures)
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
            inspect_statements(module, source, body, source_name, display_name, failures);
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
    source: &str,
    statements: &[Stmt],
    source_name: &str,
    display_name: &str,
    failures: &mut Vec<String>,
) {
    for statement in statements {
        match statement {
            Stmt::Let { init, .. } | Stmt::Expr(init) => {
                inspect_expr(module, source, init, source_name, display_name, failures);
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    inspect_expr(module, source, value, source_name, display_name, failures);
                }
            }
            Stmt::If {
                cond, then, els, ..
            } => {
                inspect_expr(module, source, cond, source_name, display_name, failures);
                inspect_statements(module, source, then, source_name, display_name, failures);
                if let Some(els) = els {
                    inspect_statements(module, source, els, source_name, display_name, failures);
                }
            }
            Stmt::While { cond, body, .. } => {
                inspect_expr(module, source, cond, source_name, display_name, failures);
                inspect_statements(module, source, body, source_name, display_name, failures);
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
                        source,
                        std::slice::from_ref(init.as_ref()),
                        source_name,
                        display_name,
                        failures,
                    );
                }
                if let Some(cond) = cond {
                    inspect_expr(module, source, cond, source_name, display_name, failures);
                }
                if let Some(step) = step {
                    inspect_expr(module, source, step, source_name, display_name, failures);
                }
                inspect_statements(module, source, body, source_name, display_name, failures);
            }
            Stmt::ForOf { subject, body, .. } => {
                inspect_expr(module, source, subject, source_name, display_name, failures);
                inspect_statements(module, source, body, source_name, display_name, failures);
            }
            Stmt::Switch { disc, cases, .. } => {
                inspect_expr(module, source, disc, source_name, display_name, failures);
                for case in cases {
                    if let Some(test) = &case.test {
                        inspect_expr(module, source, test, source_name, display_name, failures);
                    }
                    inspect_statements(
                        module,
                        source,
                        &case.body,
                        source_name,
                        display_name,
                        failures,
                    );
                }
            }
            Stmt::Block(body) => {
                inspect_statements(module, source, body, source_name, display_name, failures);
            }
            Stmt::Break(_) | Stmt::Continue(_) => {}
            _ => {}
        }
    }
}

fn inspect_function(
    module: &Module,
    source: &str,
    function: &Function,
    source_name: &str,
    display_name: &str,
    failures: &mut Vec<String>,
) {
    for parameter in &function.params {
        if let Some(default) = &parameter.default {
            inspect_expr(module, source, default, source_name, display_name, failures);
        }
    }
    inspect_statements(
        module,
        source,
        &function.body,
        source_name,
        display_name,
        failures,
    );
}

fn inspect_module(
    module: &Module,
    source: &str,
    source_name: &str,
    display_name: &str,
    failures: &mut Vec<String>,
) {
    for global in &module.globals {
        inspect_expr(
            module,
            source,
            &global.init,
            source_name,
            display_name,
            failures,
        );
    }
    for function in &module.functions {
        inspect_function(
            module,
            source,
            function,
            source_name,
            display_name,
            failures,
        );
    }
    for class in &module.classes {
        for field in &class.fields {
            if let Some(initializer) = &field.init {
                inspect_expr(
                    module,
                    source,
                    initializer,
                    source_name,
                    display_name,
                    failures,
                );
            }
        }
        if let Some(constructor) = &class.ctor {
            inspect_function(
                module,
                source,
                constructor,
                source_name,
                display_name,
                failures,
            );
        }
        for method in &class.methods {
            inspect_function(module, source, method, source_name, display_name, failures);
        }
    }
    inspect_statements(
        module,
        source,
        &module.top_level,
        source_name,
        display_name,
        failures,
    );
}

fn failures(source: &Path) -> Vec<String> {
    let source_text = std::fs::read_to_string(source)
        .unwrap_or_else(|error| panic!("read {}: {error}", source.display()));
    let files = subscript_typegpu_harness::program_files(source)
        .unwrap_or_else(|error| panic!("load {}: {error}", source.display()));
    let module = subscript_compiler::check_program_with(&files, &CheckOptions::default())
        .unwrap_or_else(|diagnostics| panic!("check {}: {diagnostics:?}", source.display()));
    let source_name = source
        .file_name()
        .and_then(|name| name.to_str())
        .expect("UTF-8 source name");
    let root = repository_root();
    let display_name = source
        .strip_prefix(&root)
        .unwrap_or(source)
        .to_string_lossy();
    let mut failures = Vec::new();
    inspect_module(
        &module,
        &source_text,
        source_name,
        &display_name,
        &mut failures,
    );
    failures
}

#[test]
fn every_authored_binding_access_uses_the_index_form() {
    let failures = subscript_typegpu_harness::run_program_pool(authored_sources(), failures)
        .into_iter()
        .flat_map(|(_, failures)| failures)
        .collect::<Vec<_>>();
    assert!(
        failures.is_empty(),
        "authored binding access failures:\n{}",
        failures.join("\n")
    );
}
