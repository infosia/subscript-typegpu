//! WGSL shell and raw-declaration discovery.

use std::collections::BTreeSet;

use subscript_compiler::hir::{Callee, Expr, ExprKind, Function, Module, Stmt};
use subscript_compiler::{Diagnostic, Pos, RuleCode};

/// One WGSL shell: a source function whose GPU body is author WGSL (K29).
#[derive(Debug, Clone)]
pub(crate) struct Shell {
    /// The shell name, which is the shelled function's name without generic arguments.
    pub(crate) name: String,
    /// The name of the module-level function that carries the host body.
    pub(crate) function: String,
    /// The WGSL statements that become the emitted function body.
    pub(crate) body: String,
    /// The declaration position.
    pub(crate) pos: Pos,
}

/// The program's one `wgslDeclarations` call (K30).
#[derive(Debug, Clone)]
pub(crate) struct Declarations {
    /// The raw WGSL text, which precedes every generated declaration of every module.
    pub(crate) text: String,
    /// The `const`, `fn`, `struct`, and `alias` names that the text declares.
    pub(crate) names: BTreeSet<String>,
    /// The call position.
    pub(crate) pos: Pos,
}

/// The author WGSL of one program.
#[derive(Debug, Clone, Default)]
pub(crate) struct ShellProgram {
    /// The shells, in declaration order.
    pub(crate) shells: Vec<Shell>,
    /// The raw declarations, when the program calls `wgslDeclarations`.
    pub(crate) declarations: Option<Declarations>,
}

fn diagnostic(rule: &str, message: impl Into<String>, pos: Pos) -> Diagnostic {
    Diagnostic::new(
        RuleCode::S100,
        format!("{rule}: {} (author)", message.into()),
        pos,
    )
}

fn library_call(module: &Module, name: &str, expected: &str) -> bool {
    crate::base_name(name) == expected
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
    let ExprKind::DescriptorLit { .. } = &expr.kind else {
        return Err(diagnostic(
            "K29",
            "WGSL shell options must be a descriptor literal",
            expr.pos.clone(),
        ));
    };
    let Some(field) = crate::descriptor_field(module, expr, "body")? else {
        return Err(diagnostic(
            "K29",
            "WgslShellSpec has no body field",
            expr.pos.clone(),
        ));
    };
    match field {
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

/// Reports whether the character is in WGSL's blankspace set, which the fence tokenizer skips.
pub(crate) fn is_wgsl_blankspace(ch: char) -> bool {
    matches!(
        ch,
        '\u{0020}' | '\u{0009}' | '\u{000A}'
            ..='\u{000D}' | '\u{0085}' | '\u{200E}' | '\u{200F}' | '\u{2028}' | '\u{2029}'
    )
}

fn tokens(text: &str, pos: &Pos) -> Result<Vec<String>, Diagnostic> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut index = 0;
    let mut braces = 0_i32;
    // Tokens end at character boundaries. Comments end at ASCII delimiters or the end of text.
    while let Some(&byte) = bytes.get(index) {
        let ch = text
            .get(index..)
            .ok_or_else(|| crate::internal("shell::tokens", "invalid character boundary", pos))?
            .chars()
            .next()
            .ok_or_else(|| crate::internal("shell::tokens", "missing character", pos))?;
        if is_wgsl_blankspace(ch) {
            index += ch.len_utf8();
            continue;
        }
        if byte == b'/' && bytes.get(index + 1) == Some(&b'/') {
            index += 2;
            while bytes.get(index).is_some_and(|byte| *byte != b'\n') {
                index += 1;
            }
            continue;
        }
        if byte == b'/' && bytes.get(index + 1) == Some(&b'*') {
            index += 2;
            let mut depth = 1_u32;
            while index < bytes.len() && depth > 0 {
                if bytes.get(index) == Some(&b'/') && bytes.get(index + 1) == Some(&b'*') {
                    depth += 1;
                    index += 2;
                } else if bytes.get(index) == Some(&b'*') && bytes.get(index + 1) == Some(&b'/') {
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
            while bytes
                .get(index)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            {
                index += 1;
            }
            out.push(
                text.get(start..index)
                    .ok_or_else(|| crate::internal("shell::tokens", "invalid token range", pos))?
                    .to_owned(),
            );
            continue;
        }
        let token = ch.to_string();
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
        index += ch.len_utf8();
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
    for (first, second) in out.iter().zip(out.iter().skip(1)) {
        if (*first == "@" && matches!(second.as_str(), "group" | "binding"))
            || (*first == "var" && *second == "<")
        {
            return Err(diagnostic(
                "K30",
                format!(
                    "WGSL text contains forbidden token sequence `{}{}`",
                    *first, *second
                ),
                pos.clone(),
            ));
        }
    }
    Ok(out)
}

fn declaration_names(tokens: &[String]) -> BTreeSet<String> {
    tokens
        .iter()
        .zip(tokens.iter().skip(1))
        .filter(|(first, _)| matches!(first.as_str(), "const" | "fn" | "struct" | "alias"))
        .map(|(_, second)| second.clone())
        .collect()
}

fn visit_expr(module: &Module, expr: &Expr, diagnostics: &mut Vec<Diagnostic>, location: &str) {
    match &expr.kind {
        ExprKind::Call { callee, args } => {
            if let Callee::Func(name) = callee {
                if library_call(module, name, "wgslShell") {
                    diagnostics.push(diagnostic(
                        "K29",
                        format!("a WGSL shell declaration appears {location}"),
                        expr.pos.clone(),
                    ));
                } else if library_call(module, name, "wgslDeclarations") {
                    diagnostics.push(diagnostic(
                        "K30",
                        format!("wgslDeclarations appears {location}"),
                        expr.pos.clone(),
                    ));
                }
            }
            if let Callee::Value(value) = callee {
                visit_expr(module, value, diagnostics, location);
            }
            if let Callee::Method { recv, .. } = callee {
                visit_expr(module, recv, diagnostics, location);
            }
            for arg in args {
                visit_expr(module, arg, diagnostics, location);
            }
        }
        ExprKind::Unary { operand, .. }
        | ExprKind::Cast(operand)
        | ExprKind::Length(operand)
        | ExprKind::Field { obj: operand, .. }
        | ExprKind::JsonResultValue(operand) => visit_expr(module, operand, diagnostics, location),
        ExprKind::Binary { left, right, .. }
        | ExprKind::Assign {
            target: left,
            value: right,
            ..
        } => {
            visit_expr(module, left, diagnostics, location);
            visit_expr(module, right, diagnostics, location);
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => {
            for arg in args {
                visit_expr(module, arg, diagnostics, location);
            }
        }
        ExprKind::DescriptorLit { fields, .. } => {
            for field in fields.iter().flatten() {
                visit_expr(module, field, diagnostics, location);
            }
        }
        ExprKind::Index { obj, index, .. } => {
            visit_expr(module, obj, diagnostics, location);
            visit_expr(module, index, diagnostics, location);
        }
        ExprKind::Cond { cond, then, els } => {
            visit_expr(module, cond, diagnostics, location);
            visit_expr(module, then, diagnostics, location);
            visit_expr(module, els, diagnostics, location);
        }
        _ => {}
    }
}

fn visit_statements(
    module: &Module,
    statements: &[Stmt],
    diagnostics: &mut Vec<Diagnostic>,
    location: &str,
) {
    for statement in statements {
        match statement {
            Stmt::Let { init, .. } | Stmt::Expr(init) => {
                visit_expr(module, init, diagnostics, location)
            }
            Stmt::Return {
                value: Some(value), ..
            } => visit_expr(module, value, diagnostics, location),
            Stmt::Return { value: None, .. } => {}
            Stmt::If {
                cond, then, els, ..
            } => {
                visit_expr(module, cond, diagnostics, location);
                visit_statements(module, then, diagnostics, location);
                if let Some(els) = els {
                    visit_statements(module, els, diagnostics, location);
                }
            }
            Stmt::While { cond, body, .. } => {
                visit_expr(module, cond, diagnostics, location);
                visit_statements(module, body, diagnostics, location);
            }
            Stmt::For {
                init,
                cond,
                step,
                body,
                ..
            } => {
                if let Some(init) = init {
                    visit_statements(module, std::slice::from_ref(init), diagnostics, location);
                }
                if let Some(cond) = cond {
                    visit_expr(module, cond, diagnostics, location);
                }
                if let Some(step) = step {
                    visit_expr(module, step, diagnostics, location);
                }
                visit_statements(module, body, diagnostics, location);
            }
            Stmt::ForOf { subject, body, .. } => {
                visit_expr(module, subject, diagnostics, location);
                visit_statements(module, body, diagnostics, location);
            }
            Stmt::Switch { disc, cases, .. } => {
                visit_expr(module, disc, diagnostics, location);
                for case in cases {
                    visit_statements(module, &case.body, diagnostics, location);
                }
            }
            Stmt::Block(body) => visit_statements(module, body, diagnostics, location),
            _ => {}
        }
    }
}

/// Collects the shells and the raw declarations of one program (K29, K30).
///
/// Every shell body and the declaration text pass the lexical fence before this returns, so a
/// later caller reads checked text.
///
/// # Errors
///
/// Returns every K29 and K30 violation. The cases include a declaration inside a function and a
/// body that is not a string literal. A forbidden token, unbalanced braces, and a second
/// `wgslDeclarations` call also give one.
pub(crate) fn discover(module: &Module) -> Result<ShellProgram, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    for function in &module.functions {
        if function.pos.file != "typegpu.ts" {
            visit_statements(
                module,
                &function.body,
                &mut diagnostics,
                "inside a function",
            );
        }
    }
    for global in &module.globals {
        let direct_shell = matches!(
            &global.init.kind,
            ExprKind::Call {
                callee: Callee::Func(callee),
                ..
            } if library_call(module, callee, "wgslShell")
        );
        if !direct_shell {
            visit_expr(
                module,
                &global.init,
                &mut diagnostics,
                "inside a top-level statement",
            );
        }
    }
    for statement in &module.top_level {
        let direct_declarations = matches!(
            statement,
            Stmt::Expr(Expr {
                kind: ExprKind::Call {
                    callee: Callee::Func(callee),
                    ..
                },
                ..
            }) if library_call(module, callee, "wgslDeclarations")
        );
        if !direct_declarations {
            visit_statements(
                module,
                std::slice::from_ref(statement),
                &mut diagnostics,
                "inside a top-level statement",
            );
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
                    name: crate::base_name(function).to_owned(),
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

/// Rejects a shell or declaration name that collides with a generated declaration (K30).
///
/// `generated_names` holds author spellings. The comparison mangles them first, so a collision
/// cannot hide behind a name that the emitter rewrites.
///
/// # Errors
///
/// Returns one K30 diagnostic for each name that collides.
pub(crate) fn validate_collisions(
    program: &ShellProgram,
    generated_names: &BTreeSet<String>,
) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let emitted_names = generated_names
        .iter()
        .map(|name| crate::mapping::ident(name))
        .collect::<BTreeSet<_>>();
    for shell in &program.shells {
        if emitted_names.contains(&shell.name) {
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
        for name in declarations.names.intersection(&emitted_names) {
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

/// Reports whether the named function is a shell, whose subscript body the emitter never walks.
pub(crate) fn function_is_shell(program: &ShellProgram, name: &str) -> bool {
    program.shells.iter().any(|shell| shell.function == name)
}

/// Returns the shell of the named function, or `None` when the function is not a shell.
pub(crate) fn shell_for_function<'a>(program: &'a ShellProgram, name: &str) -> Option<&'a Shell> {
    program.shells.iter().find(|shell| shell.function == name)
}

/// Checks one shell's signature against the K2 helper rules and returns the function.
///
/// The emitter writes the WGSL `fn` line from this signature, so the parameter and return types
/// must map to WGSL.
///
/// # Errors
///
/// Returns a K29 diagnostic when the function is absent, `async`, a generator, or takes a layout
/// class or a `ComputeInvocation`. A parameter or return type outside K4 returns its own
/// diagnostic.
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

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented,
        clippy::indexing_slicing
    )]
    use super::tokens;
    use subscript_compiler::Pos;

    #[test]
    fn fence_uses_the_wgsl_blankspace_set() {
        let pos = Pos::new("blankspace.ts", 1, 1);
        for blank in [
            '\u{0020}', '\u{0009}', '\u{000A}', '\u{000B}', '\u{000C}', '\u{000D}', '\u{0085}',
            '\u{200E}', '\u{200F}', '\u{2028}', '\u{2029}',
        ] {
            assert_eq!(
                tokens(&format!("left{blank}right"), &pos).expect("WGSL blankspace"),
                ["left", "right"],
                "U+{:04X}",
                blank as u32,
            );
        }
        assert_eq!(
            tokens("left\u{0008}right", &pos).expect("non-blank token"),
            ["left", "\u{0008}", "right"],
        );
    }
}
