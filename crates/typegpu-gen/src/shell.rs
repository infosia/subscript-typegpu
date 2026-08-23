//! WGSL shell and raw-declaration discovery.

use std::collections::BTreeSet;

use subscript_compiler::hir::{Callee, Expr, ExprKind, Function, Module, Stmt};
use subscript_compiler::{Diagnostic, Pos, RuleCode};

#[derive(Debug, Clone)]
pub(crate) struct Shell {
    pub(crate) name: String,
    pub(crate) function: String,
    pub(crate) body: String,
    pub(crate) pos: Pos,
}

#[derive(Debug, Clone)]
pub(crate) struct Declarations {
    pub(crate) text: String,
    pub(crate) names: BTreeSet<String>,
    pub(crate) pos: Pos,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ShellProgram {
    pub(crate) shells: Vec<Shell>,
    pub(crate) declarations: Option<Declarations>,
}

fn diagnostic(rule: &str, message: impl Into<String>, pos: Pos) -> Diagnostic {
    Diagnostic::new(
        RuleCode::S100,
        format!("{rule}: {} (author)", message.into()),
        pos,
    )
}

fn base_name(name: &str) -> &str {
    name.split('<').next().unwrap_or(name)
}

fn library_call(module: &Module, name: &str, expected: &str) -> bool {
    base_name(name) == expected
        && module
            .functions
            .iter()
            .find(|function| function.name == name)
            .is_some_and(|function| {
                function.pos.file == "typegpu.ts"
                    || function
                        .params
                        .first()
                        .is_some_and(|param| param.pos.file == "typegpu.ts")
            })
}

fn descriptor_body(module: &Module, expr: &Expr) -> Result<String, Diagnostic> {
    let ExprKind::DescriptorLit { class, fields } = &expr.kind else {
        return Err(diagnostic(
            "K29",
            "WGSL shell options must be a descriptor literal",
            expr.pos.clone(),
        ));
    };
    let descriptor = &module.classes[class.0];
    let Some(index) = descriptor
        .fields
        .iter()
        .position(|field| field.name == "body")
    else {
        return Err(diagnostic(
            "K29",
            "WgslShellSpec has no body field",
            expr.pos.clone(),
        ));
    };
    match fields.get(index).and_then(Option::as_ref) {
        Some(Expr {
            kind: ExprKind::Str(body),
            ..
        }) => Ok(body.clone()),
        Some(value) => Err(diagnostic(
            "K29",
            "WGSL shell body must be a string literal",
            value.pos.clone(),
        )),
        None => Err(diagnostic(
            "K29",
            "WGSL shell options omit body",
            expr.pos.clone(),
        )),
    }
}

fn tokens(text: &str, pos: &Pos) -> Result<Vec<String>, Diagnostic> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut index = 0;
    let mut braces = 0_i32;
    while index < bytes.len() {
        let byte = bytes[index];
        if matches!(byte, b' ' | b'\t' | b'\r' | b'\n') {
            index += 1;
            continue;
        }
        if byte == b'/' && bytes.get(index + 1) == Some(&b'/') {
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            continue;
        }
        if byte == b'/' && bytes.get(index + 1) == Some(&b'*') {
            index += 2;
            let mut depth = 1_u32;
            while index < bytes.len() && depth > 0 {
                if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'*') {
                    depth += 1;
                    index += 2;
                } else if bytes[index] == b'*' && bytes.get(index + 1) == Some(&b'/') {
                    depth -= 1;
                    index += 2;
                } else {
                    index += 1;
                }
            }
            if depth != 0 {
                return Err(diagnostic(
                    "K30",
                    "WGSL text has an unclosed comment",
                    pos.clone(),
                ));
            }
            continue;
        }
        if byte.is_ascii_alphabetic() || byte == b'_' {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            out.push(text[start..index].to_owned());
            continue;
        }
        let token = (byte as char).to_string();
        if byte == b'{' {
            braces += 1;
        } else if byte == b'}' {
            braces -= 1;
            if braces < 0 {
                return Err(diagnostic(
                    "K30",
                    "WGSL text has unbalanced braces",
                    pos.clone(),
                ));
            }
        }
        out.push(token);
        index += 1;
    }
    if braces != 0 {
        return Err(diagnostic(
            "K30",
            "WGSL text has unbalanced braces",
            pos.clone(),
        ));
    }
    for token in &out {
        if matches!(
            token.as_str(),
            "override" | "workgroupBarrier" | "storageBarrier" | "textureBarrier"
        ) {
            return Err(diagnostic(
                "K30",
                format!("WGSL text contains forbidden token `{token}`"),
                pos.clone(),
            ));
        }
    }
    for pair in out.windows(2) {
        if (pair[0] == "@" && matches!(pair[1].as_str(), "group" | "binding"))
            || (pair[0] == "var" && pair[1] == "<")
        {
            return Err(diagnostic(
                "K30",
                format!(
                    "WGSL text contains forbidden token sequence `{}{}`",
                    pair[0], pair[1]
                ),
                pos.clone(),
            ));
        }
    }
    Ok(out)
}

fn declaration_names(tokens: &[String]) -> BTreeSet<String> {
    tokens
        .windows(2)
        .filter(|pair| matches!(pair[0].as_str(), "const" | "fn" | "struct" | "alias"))
        .map(|pair| pair[1].clone())
        .collect()
}

fn visit_expr(module: &Module, expr: &Expr, diagnostics: &mut Vec<Diagnostic>) {
    match &expr.kind {
        ExprKind::Call { callee, args } => {
            if let Callee::Func(name) = callee {
                if library_call(module, name, "wgslShell") {
                    diagnostics.push(diagnostic(
                        "K29",
                        "a WGSL shell declaration appears inside a function",
                        expr.pos.clone(),
                    ));
                } else if library_call(module, name, "wgslDeclarations") {
                    diagnostics.push(diagnostic(
                        "K30",
                        "wgslDeclarations appears inside a function",
                        expr.pos.clone(),
                    ));
                }
            }
            if let Callee::Value(value) = callee {
                visit_expr(module, value, diagnostics);
            }
            if let Callee::Method { recv, .. } = callee {
                visit_expr(module, recv, diagnostics);
            }
            for arg in args {
                visit_expr(module, arg, diagnostics);
            }
        }
        ExprKind::Unary { operand, .. }
        | ExprKind::Cast(operand)
        | ExprKind::Length(operand)
        | ExprKind::Field { obj: operand, .. }
        | ExprKind::JsonResultValue(operand) => visit_expr(module, operand, diagnostics),
        ExprKind::Binary { left, right, .. }
        | ExprKind::Assign {
            target: left,
            value: right,
            ..
        } => {
            visit_expr(module, left, diagnostics);
            visit_expr(module, right, diagnostics);
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => {
            for arg in args {
                visit_expr(module, arg, diagnostics);
            }
        }
        ExprKind::DescriptorLit { fields, .. } => {
            for field in fields.iter().flatten() {
                visit_expr(module, field, diagnostics);
            }
        }
        ExprKind::Index { obj, index, .. } => {
            visit_expr(module, obj, diagnostics);
            visit_expr(module, index, diagnostics);
        }
        ExprKind::Cond { cond, then, els } => {
            visit_expr(module, cond, diagnostics);
            visit_expr(module, then, diagnostics);
            visit_expr(module, els, diagnostics);
        }
        _ => {}
    }
}

fn visit_statements(module: &Module, statements: &[Stmt], diagnostics: &mut Vec<Diagnostic>) {
    for statement in statements {
        match statement {
            Stmt::Let { init, .. } | Stmt::Expr(init) => visit_expr(module, init, diagnostics),
            Stmt::Return {
                value: Some(value), ..
            } => visit_expr(module, value, diagnostics),
            Stmt::Return { value: None, .. } => {}
            Stmt::If {
                cond, then, els, ..
            } => {
                visit_expr(module, cond, diagnostics);
                visit_statements(module, then, diagnostics);
                if let Some(els) = els {
                    visit_statements(module, els, diagnostics);
                }
            }
            Stmt::While { cond, body, .. } => {
                visit_expr(module, cond, diagnostics);
                visit_statements(module, body, diagnostics);
            }
            Stmt::For {
                init,
                cond,
                step,
                body,
                ..
            } => {
                if let Some(init) = init {
                    visit_statements(module, std::slice::from_ref(init), diagnostics);
                }
                if let Some(cond) = cond {
                    visit_expr(module, cond, diagnostics);
                }
                if let Some(step) = step {
                    visit_expr(module, step, diagnostics);
                }
                visit_statements(module, body, diagnostics);
            }
            Stmt::ForOf { subject, body, .. } => {
                visit_expr(module, subject, diagnostics);
                visit_statements(module, body, diagnostics);
            }
            Stmt::Switch { disc, cases, .. } => {
                visit_expr(module, disc, diagnostics);
                for case in cases {
                    visit_statements(module, &case.body, diagnostics);
                }
            }
            Stmt::Block(body) => visit_statements(module, body, diagnostics),
            _ => {}
        }
    }
}

pub(crate) fn discover(module: &Module) -> Result<ShellProgram, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    for function in &module.functions {
        if function.pos.file != "typegpu.ts" {
            visit_statements(module, &function.body, &mut diagnostics);
        }
    }
    let mut shells = Vec::new();
    for global in &module.globals {
        let ExprKind::Call {
            callee: Callee::Func(callee),
            args,
        } = &global.init.kind
        else {
            continue;
        };
        if !library_call(module, callee, "wgslShell") {
            continue;
        }
        let Some(Expr {
            kind: ExprKind::FuncRef(function),
            ..
        }) = args.first()
        else {
            diagnostics.push(diagnostic(
                "K29",
                "WGSL shell function must be a named function",
                global.init.pos.clone(),
            ));
            continue;
        };
        let Some(options) = args.get(1) else {
            diagnostics.push(diagnostic(
                "K29",
                "WGSL shell declaration omits options",
                global.init.pos.clone(),
            ));
            continue;
        };
        match descriptor_body(module, options) {
            Ok(body) => match tokens(&body, &options.pos) {
                Ok(_) => shells.push(Shell {
                    name: function.split('<').next().unwrap_or(function).to_owned(),
                    function: function.clone(),
                    body,
                    pos: global.pos.clone(),
                }),
                Err(error) => diagnostics.push(error),
            },
            Err(error) => diagnostics.push(error),
        }
    }

    let mut declarations = None;
    for statement in &module.top_level {
        let Stmt::Expr(Expr {
            kind:
                ExprKind::Call {
                    callee: Callee::Func(callee),
                    args,
                },
            pos,
            ..
        }) = statement
        else {
            continue;
        };
        if !library_call(module, callee, "wgslDeclarations") {
            continue;
        }
        if declarations.is_some() {
            diagnostics.push(diagnostic(
                "K30",
                "a program has a second wgslDeclarations call",
                pos.clone(),
            ));
            continue;
        }
        let Some(Expr {
            kind: ExprKind::Str(text),
            ..
        }) = args.first()
        else {
            diagnostics.push(diagnostic(
                "K30",
                "wgslDeclarations text must be a string literal",
                pos.clone(),
            ));
            continue;
        };
        match tokens(text, pos) {
            Ok(raw_tokens) => {
                declarations = Some(Declarations {
                    text: text.clone(),
                    names: declaration_names(&raw_tokens),
                    pos: pos.clone(),
                });
            }
            Err(error) => diagnostics.push(error),
        }
    }
    if diagnostics.is_empty() {
        Ok(ShellProgram {
            shells,
            declarations,
        })
    } else {
        Err(diagnostics)
    }
}

pub(crate) fn validate_collisions(
    program: &ShellProgram,
    generated_names: &BTreeSet<String>,
) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    for shell in &program.shells {
        if generated_names.contains(&shell.name) {
            diagnostics.push(diagnostic(
                "K30",
                format!(
                    "WGSL shell name `{}` collides with a generated declaration",
                    shell.name
                ),
                shell.pos.clone(),
            ));
        }
    }
    if let Some(declarations) = &program.declarations {
        for name in declarations.names.intersection(generated_names) {
            diagnostics.push(diagnostic(
                "K30",
                format!("WGSL declaration name `{name}` collides with a generated declaration"),
                declarations.pos.clone(),
            ));
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

pub(crate) fn function_is_shell(program: &ShellProgram, name: &str) -> bool {
    program.shells.iter().any(|shell| shell.function == name)
}

pub(crate) fn shell_for_function<'a>(program: &'a ShellProgram, name: &str) -> Option<&'a Shell> {
    program.shells.iter().find(|shell| shell.function == name)
}

pub(crate) fn validate_signature<'a>(
    module: &'a Module,
    shell: &Shell,
    layouts: &[crate::pipeline::Layout],
) -> Result<&'a Function, Diagnostic> {
    let function = module
        .functions
        .iter()
        .find(|function| function.name == shell.function)
        .ok_or_else(|| diagnostic("K29", "WGSL shell function is absent", shell.pos.clone()))?;
    if function.is_async || function.is_generator {
        return Err(diagnostic(
            "K29",
            format!("WGSL shell `{}` is async or a generator", shell.name),
            function.pos.clone(),
        ));
    }
    for param in &function.params {
        if crate::pipeline::class_name(module, &param.ty).is_some_and(|name| {
            name == "ComputeInvocation" || layouts.iter().any(|layout| layout.name == name)
        }) {
            return Err(diagnostic(
                "K29",
                format!(
                    "WGSL shell `{}` takes a layout or ComputeInvocation",
                    shell.name
                ),
                param.pos.clone(),
            ));
        }
        let _ = crate::kernel::wgsl_type(module, &param.ty, &param.pos)?;
    }
    if function.ret != subscript_compiler::Type::Void {
        let _ = crate::kernel::wgsl_type(module, &function.ret, &function.pos)?;
    }
    Ok(function)
}
