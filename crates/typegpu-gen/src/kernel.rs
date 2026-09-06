//! Typed-HIR to WGSL kernel emission.

use crate::TryAny;

use std::collections::{BTreeMap, BTreeSet};

use subscript_compiler::hir::{
    BinOp, Callee, Expr, ExprKind, ForOfKind, Function, Module, Stmt, UnOp,
};
use subscript_compiler::{Diagnostic, Pos, RuleCode, Type};

use crate::mapping::{self, MethodEmission};
use crate::pipeline::{
    library_class, BindingKind, Pipeline, StorageTextureAccess, TextureViewDimension,
};
use crate::render::RenderPipeline;
use crate::schema::Schema;

/// Statements that must run before an expression, each with its indent relative to the statement.
///
/// A lowered conditional puts its `if`/`else` here, so both sides keep short-circuit evaluation
/// (K9).
type Prelude = Vec<(usize, String)>;

/// One author-WGSL line range inside an emitted module (K31).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WgslSpan {
    /// `shell <name>` or `declarations`.
    pub(crate) label: String,
    /// First one-based line of the range.
    pub(crate) start_line: u32,
    /// Last one-based line of the range.
    pub(crate) end_line: u32,
}

/// One emitted WGSL module and the author-WGSL ranges inside it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EmittedWgsl {
    /// The complete module text, which the committed `.wgsl` golden holds (K16).
    pub(crate) text: String,
    /// The recorded ranges, in emission order.
    pub(crate) spans: Vec<WgslSpan>,
}

/// One emitted WGSL expression and the statements that must precede it.
///
/// The emitter parenthesizes an operand by the precedence of the emitted WGSL operator, never by
/// the subscript expression kind (K14).
#[derive(Debug, Clone)]
pub(crate) struct Snippet {
    /// The WGSL expression text.
    pub(crate) text: String,
    precedence: u8,
    prelude: Prelude,
}

impl Snippet {
    /// Builds a snippet whose text needs parentheses below `precedence`. A higher number binds
    /// tighter.
    fn new(text: String, precedence: u8) -> Self {
        Self {
            text,
            precedence,
            prelude: Vec::new(),
        }
    }

    /// Builds a snippet that never needs parentheses, such as a literal, a name, or a call.
    fn atom(text: String) -> Self {
        Self::new(text, 10)
    }
}

/// Builds one author-facing diagnostic that names `rule`, the single rule it enforces (K17).
fn diagnostic(rule: &str, message: impl Into<String>, pos: Pos) -> Diagnostic {
    Diagnostic::new(
        RuleCode::S100,
        format!("{rule}: {} (author)", message.into()),
        pos,
    )
}

/// Builds a diagnostic that names the generator as its source (K15).
///
/// The author's program passed the checker, so a reader who sees one has found a generator defect.
fn generator_diagnostic(message: impl Into<String>, pos: Pos) -> Diagnostic {
    Diagnostic::new(
        RuleCode::S100,
        format!("K15: {} (generator)", message.into()),
        pos,
    )
}

/// Returns the module-level function of this name, and `None` when the module declares none.
fn function<'a>(module: &'a Module, name: &str) -> Option<&'a Function> {
    module
        .functions
        .iter()
        .find(|function| function.name == name)
}

/// Returns the class name of a class type, and `None` for every other type.
fn class_name<'a>(module: &'a Module, ty: &Type) -> Option<&'a str> {
    let Type::Class(id) = ty else { return None };
    module.classes.get(id.0).map(|class| class.name.as_str())
}

/// Returns the source position of one statement. A block gives its first statement's position.
fn statement_pos(statement: &Stmt) -> Option<&Pos> {
    match statement {
        Stmt::Let { pos, .. }
        | Stmt::Return { pos, .. }
        | Stmt::If { pos, .. }
        | Stmt::While { pos, .. }
        | Stmt::For { pos, .. }
        | Stmt::ForOf { pos, .. }
        | Stmt::Switch { pos, .. }
        | Stmt::Break(pos)
        | Stmt::Continue(pos) => Some(pos),
        Stmt::Expr(expr) => Some(&expr.pos),
        Stmt::Block(body) => body.first().and_then(statement_pos),
    }
}

/// Reports whether the named function comes from `file`.
///
/// The generator recognizes a library function by declaring file, never by name alone (RN1). A
/// generic instantiation can carry a position outside the library, so a parameter's file and the
/// first statement's file both count.
fn function_declared_in(module: &Module, name: &str, file: &str) -> bool {
    function(module, name).is_some_and(|function| {
        function.pos.file == file
            || function.params.iter().any(|param| param.pos.file == file)
            || function
                .body
                .first()
                .and_then(statement_pos)
                .is_some_and(|pos| pos.file == file)
    })
}

/// Returns the WGSL scalar of an atomic library class, and `None` for every other type (K21).
///
/// # Errors
///
/// If the type names a class the module does not hold, returns an internal diagnostic.
fn atomic_scalar(
    module: &Module,
    ty: &Type,
    pos: &Pos,
) -> Result<Option<&'static str>, Diagnostic> {
    let Type::Class(id) = ty else {
        return Ok(None);
    };
    let class = crate::class(module, id.0, "kernel::atomic_scalar", pos)?;
    if class.pos.file != "typegpu-types.ts" {
        return Ok(None);
    }

    Ok(match class.name.as_str() {
        "AtomicU32" => Some("u32"),
        "AtomicI32" => Some("i32"),
        _ => None,
    })
}

/// Reports whether `ty` holds an atomic at any depth (K21).
///
/// An atomic cannot be copied to a local or written as a whole, so a schema that holds one is
/// restricted wherever a plain schema is not.
///
/// # Errors
///
/// If a field names a class the module does not hold, returns an internal diagnostic.
fn type_contains_atomic(module: &Module, ty: &Type, pos: &Pos) -> Result<bool, Diagnostic> {
    /// Descends one type. `seen` holds the class ids already visited, so a cycle terminates.
    fn visit(
        module: &Module,
        ty: &Type,
        seen: &mut BTreeSet<usize>,
        pos: &Pos,
    ) -> Result<bool, Diagnostic> {
        if atomic_scalar(module, ty, pos)?.is_some() {
            return Ok(true);
        }
        match ty {
            Type::FixedArray(item, _) | Type::Array(item) => visit(module, item, seen, pos),
            Type::Class(id) if seen.insert(id.0) => {
                crate::class(module, id.0, "kernel::type_contains_atomic", pos)?
                    .fields
                    .iter()
                    .try_any(|field| visit(module, &field.ty, seen, &field.pos))
            }
            _ => Ok(false),
        }
    }
    visit(module, ty, &mut BTreeSet::new(), pos)
}

/// Reports whether `ty` is the `PrivateVar<T>` wrapper that `privateVar` returns (K20).
fn is_private_var(module: &Module, ty: &Type) -> bool {
    library_class(module, ty).is_some_and(|class| class.name.starts_with("PrivateVar<"))
}

/// The WGSL form of one module-level declaration that a kernel reads (K19, K20).
#[derive(Debug, Clone)]
enum KernelGlobalKind {
    /// A `const` whose initializer the generator folds.
    Constant(Expr),
    /// A `var<private>` and its initializer expression.
    Private(Expr),
    /// A `var<workgroup>` of one value, which takes no initializer.
    WorkgroupVar,
    /// A `var<workgroup>` array of this literal length.
    WorkgroupArray(u32),
}

/// One module-level declaration that a kernel's call graph reads.
#[derive(Debug, Clone)]
struct KernelGlobal {
    /// The author's declaration name, which the emitter mangles (K14).
    name: String,
    /// The value type. A wrapper gives the `T` it carries, never the wrapper class.
    ty: Type,
    /// The WGSL form.
    kind: KernelGlobalKind,
    /// The declaration position.
    pos: Pos,
}

/// Collects the module-level declaration names that `expr` reads.
fn global_names_expr(expr: &Expr, out: &mut BTreeSet<String>) {
    if let ExprKind::Global(name) = &expr.kind {
        out.insert(name.clone());
    }
    match &expr.kind {
        ExprKind::AbsenceTest { value: operand, .. }
        | ExprKind::Unary { operand, .. }
        | ExprKind::Cast(operand)
        | ExprKind::Length(operand)
        | ExprKind::Field { obj: operand, .. }
        | ExprKind::JsonResultValue(operand) => global_names_expr(operand, out),
        ExprKind::Binary { left, right, .. }
        | ExprKind::Assign {
            target: left,
            value: right,
            ..
        } => {
            global_names_expr(left, out);
            global_names_expr(right, out);
        }
        ExprKind::Call { callee, args } => {
            if let Callee::Value(value) = callee {
                global_names_expr(value, out);
            }
            if let Callee::Method { recv, .. } = callee {
                global_names_expr(recv, out);
            }
            for arg in args {
                global_names_expr(arg, out);
            }
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => {
            for arg in args {
                global_names_expr(arg, out);
            }
        }
        ExprKind::DescriptorLit { fields, .. } => {
            for value in fields.iter().flatten() {
                global_names_expr(value, out);
            }
        }
        ExprKind::Index { obj, index, .. } => {
            global_names_expr(obj, out);
            global_names_expr(index, out);
        }
        ExprKind::Cond { cond, then, els } => {
            global_names_expr(cond, out);
            global_names_expr(then, out);
            global_names_expr(els, out);
        }
        _ => {}
    }
}

/// Runs `global_names_expr` over every expression the statement holds, nested bodies included.
fn global_names_stmt(statement: &Stmt, out: &mut BTreeSet<String>) {
    match statement {
        Stmt::Let { init, .. } | Stmt::Expr(init) => global_names_expr(init, out),
        Stmt::Return {
            value: Some(value), ..
        } => global_names_expr(value, out),
        Stmt::Return { value: None, .. } => {}
        Stmt::If {
            cond, then, els, ..
        } => {
            global_names_expr(cond, out);
            for statement in then {
                global_names_stmt(statement, out);
            }
            if let Some(els) = els {
                for statement in els {
                    global_names_stmt(statement, out);
                }
            }
        }
        Stmt::While { cond, body, .. } => {
            global_names_expr(cond, out);
            for statement in body {
                global_names_stmt(statement, out);
            }
        }
        Stmt::For {
            init,
            cond,
            step,
            body,
            ..
        } => {
            if let Some(init) = init {
                global_names_stmt(init, out);
            }
            if let Some(cond) = cond {
                global_names_expr(cond, out);
            }
            if let Some(step) = step {
                global_names_expr(step, out);
            }
            for statement in body {
                global_names_stmt(statement, out);
            }
        }
        Stmt::ForOf { subject, body, .. } => {
            global_names_expr(subject, out);
            for statement in body {
                global_names_stmt(statement, out);
            }
        }
        Stmt::Switch { disc, cases, .. } => {
            global_names_expr(disc, out);
            for case in cases {
                if let Some(test) = &case.test {
                    global_names_expr(test, out);
                }
                for statement in &case.body {
                    global_names_stmt(statement, out);
                }
            }
        }
        Stmt::Block(body) => {
            for statement in body {
                global_names_stmt(statement, out);
            }
        }
        Stmt::Break(_) | Stmt::Continue(_) => {}
    }
}

/// Returns the `T` that a library wrapper class carries in its host-body field.
///
/// `field_name` is that field, `value` for one value and `values` for an array. A `T[]` field
/// gives the element type. The result is `None` when the class holds no such field.
///
/// # Errors
///
/// If the type names a class the module does not hold, returns an internal diagnostic.
fn wrapper_item_type(
    module: &Module,
    ty: &Type,
    field_name: &str,
    pos: &Pos,
) -> Result<Option<Type>, Diagnostic> {
    let Type::Class(id) = ty else { return Ok(None) };
    let class = crate::class(module, id.0, "kernel::wrapper_item_type", pos)?;
    Ok(class
        .fields
        .iter()
        .find(|field| field.name == field_name)
        .map(|field| match &field.ty {
            Type::Array(item) => (**item).clone(),
            item => item.clone(),
        }))
}

/// Returns the module-level declarations that one kernel's call graph reads (K19, K20).
///
/// The result follows the module's declaration order, which the emitter writes them in (K14).
///
/// # Errors
///
/// A kernel that reads a mutable global gives a K19 diagnostic. A variable wrapper with no value
/// type or a non-literal length gives a K20 diagnostic. A cycle in the call graph gives a K2
/// diagnostic.
fn kernel_globals(
    module: &Module,
    kernel: &Function,
    shells: &crate::shell::ShellProgram,
) -> Result<Vec<KernelGlobal>, Diagnostic> {
    let helpers = dependencies(module, kernel, shells)?;
    let mut reached = BTreeSet::new();
    for statement in &kernel.body {
        global_names_stmt(statement, &mut reached);
    }
    for helper in helpers {
        if let Some(function) = function(module, &helper) {
            for statement in &function.body {
                global_names_stmt(statement, &mut reached);
            }
        }
    }
    // A reached constant's initializer can read another constant, which the kernel never names.
    // The set grows until it stops, so the emitted module declares every name it uses (K19).
    loop {
        let before = reached.len();
        for global in &module.globals {
            if reached.contains(&global.name) {
                global_names_expr(&global.init, &mut reached);
            }
        }
        if reached.len() == before {
            break;
        }
    }

    let mut globals = Vec::new();
    for global in module
        .globals
        .iter()
        .filter(|global| reached.contains(&global.name))
    {
        if global.mutable {
            return Err(diagnostic(
                "K19",
                format!("mutable global `{}` is read by a kernel", global.name),
                global.pos.clone(),
            ));
        }
        // The three library factories decide the address space (K20). Every other initializer is
        // a module constant, which the emitter folds (K19).
        let wrapper = match &global.init.kind {
            ExprKind::Call {
                callee: Callee::Func(name),
                args,
            } if function_declared_in(module, name, "typegpu.ts") => {
                Some((crate::base_name(name), args.as_slice()))
            }
            _ => None,
        };
        let (ty, kind) = match wrapper {
            Some(("privateVar", [init])) => (
                wrapper_item_type(module, &global.ty, "value", &global.pos)?.ok_or_else(|| {
                    diagnostic(
                        "K20",
                        "private variable has no value type",
                        global.pos.clone(),
                    )
                })?,
                KernelGlobalKind::Private(init.clone()),
            ),
            Some(("workgroupVar", [])) => (
                wrapper_item_type(module, &global.ty, "values", &global.pos)?.ok_or_else(|| {
                    diagnostic(
                        "K20",
                        "workgroup variable has no value type",
                        global.pos.clone(),
                    )
                })?,
                KernelGlobalKind::WorkgroupVar,
            ),
            Some(("workgroupArray", [length])) => {
                let ExprKind::Int(length) = length.kind else {
                    return Err(diagnostic(
                        "K20",
                        "workgroup array length is not an integer literal",
                        length.pos.clone(),
                    ));
                };
                let length = u32::try_from(length).map_err(|_| {
                    diagnostic(
                        "K20",
                        "workgroup array length is outside u32",
                        global.pos.clone(),
                    )
                })?;
                (
                    wrapper_item_type(module, &global.ty, "values", &global.pos)?.ok_or_else(
                        || {
                            diagnostic(
                                "K20",
                                "workgroup array has no item type",
                                global.pos.clone(),
                            )
                        },
                    )?,
                    KernelGlobalKind::WorkgroupArray(length),
                )
            }
            _ => (
                global.ty.clone(),
                KernelGlobalKind::Constant(global.init.clone()),
            ),
        };
        globals.push(KernelGlobal {
            name: global.name.clone(),
            ty,
            kind,
            pos: global.pos.clone(),
        });
    }
    Ok(globals)
}

/// Returns the declarations that both render entry points read, with no repeat (RN9).
///
/// One module carries both entry points, so the union reaches it once, in the module's declaration
/// order (K14).
///
/// # Errors
///
/// Returns the diagnostics of `kernel_globals`.
fn render_kernel_globals(
    module: &Module,
    kernels: [&Function; 2],
    shells: &crate::shell::ShellProgram,
) -> Result<Vec<KernelGlobal>, Diagnostic> {
    let mut reached = BTreeMap::new();
    for kernel in kernels {
        for global in kernel_globals(module, kernel, shells)? {
            reached.insert(global.name.clone(), global);
        }
    }
    Ok(module
        .globals
        .iter()
        .filter_map(|global| reached.remove(&global.name))
        .collect())
}

/// Returns the module constants and variables that the kernel's call graph reads (K19, K20).
///
/// # Errors
///
/// Returns a K2 diagnostic for a cycle in the call graph. A reached global with no WGSL form
/// gives a K19 or K20 diagnostic.
pub(crate) fn reached_global_names(
    module: &Module,
    pipeline: &Pipeline,
    shells: &crate::shell::ShellProgram,
) -> Result<BTreeSet<String>, Diagnostic> {
    let kernel = function(module, &pipeline.entry)
        .ok_or_else(|| generator_diagnostic("kernel disappeared from HIR", pipeline.pos.clone()))?;
    Ok(kernel_globals(module, kernel, shells)?
        .into_iter()
        .map(|global| global.name)
        .collect())
}

/// Returns the module constants and variables that both render entry points read (K19, K20).
///
/// # Errors
///
/// Returns a K2 diagnostic for a cycle in a call graph. A reached global with no WGSL form
/// gives a K19 or K20 diagnostic.
pub(crate) fn reached_render_global_names(
    module: &Module,
    pipeline: &RenderPipeline,
    shells: &crate::shell::ShellProgram,
) -> Result<BTreeSet<String>, Diagnostic> {
    let vertex = function(module, &pipeline.vertex_entry).ok_or_else(|| {
        generator_diagnostic("vertex kernel disappeared from HIR", pipeline.pos.clone())
    })?;
    let fragment = function(module, &pipeline.fragment_entry).ok_or_else(|| {
        generator_diagnostic("fragment kernel disappeared from HIR", pipeline.pos.clone())
    })?;
    Ok(render_kernel_globals(module, [vertex, fragment], shells)?
        .into_iter()
        .map(|global| global.name)
        .collect())
}

/// Reports whether `expr` holds a construct that sequential host simulation cannot keep (CL2).
///
/// A barrier call, an atomic method, and a write to a private variable each give `true`. The host
/// lane runs invocations in sequence, which is not the GPU's per-invocation state.
///
/// # Errors
///
/// If a receiver type names a class the module does not hold, returns an internal diagnostic.
fn expression_blocks_host(module: &Module, expression: &Expr) -> Result<bool, Diagnostic> {
    Ok(match &expression.kind {
        ExprKind::AbsenceTest { value: operand, .. }
        | ExprKind::Unary { operand, .. }
        | ExprKind::Cast(operand)
        | ExprKind::Length(operand)
        | ExprKind::Field { obj: operand, .. }
        | ExprKind::JsonResultValue(operand) => expression_blocks_host(module, operand)?,
        ExprKind::Binary { left, right, .. }
        | ExprKind::Assign {
            target: left,
            value: right,
            ..
        } => expression_blocks_host(module, left)? || expression_blocks_host(module, right)?,
        ExprKind::Call { callee, args } => {
            let callee_blocks = match callee {
                Callee::Func(name) => {
                    matches!(
                        crate::base_name(name),
                        "workgroupBarrier" | "storageBarrier"
                    ) && function_declared_in(module, name, "typegpu.ts")
                }
                Callee::Method { recv, name } => {
                    atomic_scalar(module, &recv.ty, &expression.pos)?.is_some()
                        || (name == "$=" && is_private_var(module, &recv.ty))
                        || expression_blocks_host(module, recv)?
                }
                Callee::Value(value) => expression_blocks_host(module, value)?,
                _ => false,
            };
            callee_blocks
                || args
                    .iter()
                    .try_any(|arg| expression_blocks_host(module, arg))?
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => args
            .iter()
            .try_any(|arg| expression_blocks_host(module, arg))?,
        ExprKind::DescriptorLit { fields, .. } => fields
            .iter()
            .flatten()
            .try_any(|value| expression_blocks_host(module, value))?,
        ExprKind::Index { obj, index, .. } => {
            expression_blocks_host(module, obj)? || expression_blocks_host(module, index)?
        }
        ExprKind::Cond { cond, then, els } => {
            expression_blocks_host(module, cond)?
                || expression_blocks_host(module, then)?
                || expression_blocks_host(module, els)?
        }
        _ => false,
    })
}

/// Runs `expression_blocks_host` over every expression the statements hold, nested bodies
/// included.
///
/// # Errors
///
/// Returns the internal diagnostics of `expression_blocks_host`.
fn statements_block_host(module: &Module, statements: &[Stmt]) -> Result<bool, Diagnostic> {
    statements.iter().try_any(|statement| {
        Ok(match statement {
            Stmt::Let { init, .. } | Stmt::Expr(init) => expression_blocks_host(module, init)?,
            Stmt::Return { value, .. } => value
                .as_ref()
                .into_iter()
                .try_any(|value| expression_blocks_host(module, value))?,
            Stmt::If {
                cond, then, els, ..
            } => {
                expression_blocks_host(module, cond)?
                    || statements_block_host(module, then)?
                    || els
                        .as_ref()
                        .into_iter()
                        .try_any(|items| statements_block_host(module, items))?
            }
            Stmt::While { cond, body, .. } => {
                expression_blocks_host(module, cond)? || statements_block_host(module, body)?
            }
            Stmt::For {
                init,
                cond,
                step,
                body,
                ..
            } => {
                init.as_deref()
                    .into_iter()
                    .try_any(|item| statements_block_host(module, std::slice::from_ref(item)))?
                    || cond
                        .as_ref()
                        .into_iter()
                        .try_any(|value| expression_blocks_host(module, value))?
                    || step
                        .as_ref()
                        .into_iter()
                        .try_any(|value| expression_blocks_host(module, value))?
                    || statements_block_host(module, body)?
            }
            Stmt::ForOf { subject, body, .. } => {
                expression_blocks_host(module, subject)? || statements_block_host(module, body)?
            }
            Stmt::Switch { disc, cases, .. } => {
                expression_blocks_host(module, disc)?
                    || cases.iter().try_any(|case| {
                        Ok({
                            case.test
                                .as_ref()
                                .into_iter()
                                .try_any(|test| expression_blocks_host(module, test))?
                                || statements_block_host(module, &case.body)?
                        })
                    })?
            }
            Stmt::Block(body) => statements_block_host(module, body)?,
            Stmt::Break(_) | Stmt::Continue(_) => false,
        })
    })
}

/// Reports whether sequential host simulation keeps the kernel's behavior (CL2).
///
/// A barrier, a workgroup variable, an atomic method, and a write to a private variable each
/// give `false`. The support module exports the result as `<name>_HOST_RUNNABLE`.
///
/// # Errors
///
/// Returns a K2 diagnostic for a cycle in the call graph.
pub(crate) fn host_runnable(
    module: &Module,
    kernel: &Function,
    shells: &crate::shell::ShellProgram,
) -> Result<bool, Diagnostic> {
    let globals = kernel_globals(module, kernel, shells)?;
    if globals.iter().any(|global| {
        matches!(
            global.kind,
            KernelGlobalKind::WorkgroupVar | KernelGlobalKind::WorkgroupArray(_)
        )
    }) {
        return Ok(false);
    }
    if statements_block_host(module, &kernel.body)? {
        return Ok(false);
    }
    for helper in dependencies(module, kernel, shells)? {
        if crate::shell::function_is_shell(shells, &helper) {
            continue;
        }
        if function(module, &helper)
            .into_iter()
            .try_any(|function| statements_block_host(module, &function.body))?
        {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Reports whether the kernel or a helper it reaches calls a barrier (K22).
///
/// A shell never counts, because the lexical fence rejects a barrier token (K30). A guarded
/// declaration whose kernel reaches a barrier is a PI15 diagnostic.
///
/// # Errors
///
/// Returns a K2 diagnostic for a cycle in the call graph.
pub(crate) fn reaches_barrier(
    module: &Module,
    kernel: &Function,
    shells: &crate::shell::ShellProgram,
) -> Result<bool, Diagnostic> {
    if contains_barrier(module, &kernel.body) {
        return Ok(true);
    }
    for helper in dependencies(module, kernel, shells)? {
        if crate::shell::function_is_shell(shells, &helper) {
            continue;
        }
        if function(module, &helper)
            .is_some_and(|function| contains_barrier(module, &function.body))
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Returns the WGSL spelling of one kernel value type (K4).
///
/// A schema class and a varyings class give the mangled struct name (K14).
///
/// # Errors
///
/// Returns a K4 diagnostic for `f16`, and a K5 diagnostic for a reference class and for every
/// other type outside the kernel value set.
pub(crate) fn wgsl_type(module: &Module, ty: &Type, pos: &Pos) -> Result<String, Diagnostic> {
    Ok(match ty {
        Type::F32 => "f32".to_owned(),
        Type::I32 => "i32".to_owned(),
        Type::U32 => "u32".to_owned(),
        Type::Bool => "bool".to_owned(),
        Type::F16 => {
            return Err(diagnostic(
                "K4",
                "f16 is storage-only in kernels",
                pos.clone(),
            ))
        }
        Type::FixedArray(item, length) => {
            format!(
                "array<{}, {}>",
                wgsl_type(module, item, pos)?,
                crate::wgsl_u32_literal(length)
            )
        }
        Type::Class(id) => {
            let class = crate::class(module, id.0, "kernel::wgsl_type", pos)?;
            if !class.is_value {
                return Err(diagnostic(
                    "K5",
                    format!(
                        "reference class `{}` is not a kernel value type",
                        class.name
                    ),
                    pos.clone(),
                ));
            }
            if class.pos.file != "typegpu-types.ts" {
                return Ok(mapping::ident(&class.name));
            }
            match class.name.as_str() {
                "Vec2f" => "vec2<f32>".to_owned(),
                "Vec3f" => "vec3<f32>".to_owned(),
                "Vec4f" => "vec4<f32>".to_owned(),
                "Vec2i" => "vec2<i32>".to_owned(),
                "Vec3i" => "vec3<i32>".to_owned(),
                "Vec4i" => "vec4<i32>".to_owned(),
                "Vec2u" => "vec2<u32>".to_owned(),
                "Vec3u" => "vec3<u32>".to_owned(),
                "Vec4u" => "vec4<u32>".to_owned(),
                "Vec2b" => "vec2<bool>".to_owned(),
                "Vec3b" => "vec3<bool>".to_owned(),
                "Vec4b" => "vec4<bool>".to_owned(),
                "Vec2h" => "vec2<f16>".to_owned(),
                "Vec3h" => "vec3<f16>".to_owned(),
                "Vec4h" => "vec4<f16>".to_owned(),
                "Mat2x2f" => "mat2x2<f32>".to_owned(),
                "Mat3x3f" => "mat3x3<f32>".to_owned(),
                "Mat4x4f" => "mat4x4<f32>".to_owned(),
                "AtomicU32" => "atomic<u32>".to_owned(),
                "AtomicI32" => "atomic<i32>".to_owned(),
                name => mapping::ident(name),
            }
        }
        _ => {
            return Err(diagnostic(
                "K5",
                format!(
                    "type `{}` is not allowed in a kernel",
                    type_name(module, ty, pos)?
                ),
                pos.clone(),
            ))
        }
    })
}

/// Returns the `@group @binding var` declaration line of one binding (PI5, TX1).
///
/// The emitter writes these in group and binding order, which K14 fixes.
///
/// # Errors
///
/// Returns the diagnostics of `wgsl_type` for a buffer item type outside the kernel value set.
fn binding_declaration(
    module: &Module,
    group: u32,
    binding: &crate::pipeline::Binding,
) -> Result<String, Diagnostic> {
    let name = mapping::ident(&binding.name);
    let declaration = match binding.kind {
        BindingKind::Uniform => format!(
            "var<uniform> {name}: {};",
            wgsl_type(module, &binding.item_ty, &binding.pos)?,
        ),
        BindingKind::Guard => format!("var<uniform> {name}: vec3<u32>;"),
        BindingKind::Storage | BindingKind::MutStorage => format!(
            "var<{}> {name}: array<{}>;",
            binding.kind.wgsl(),
            wgsl_type(module, &binding.item_ty, &binding.pos)?,
        ),
        BindingKind::Texture(sample, dimension) => {
            format!(
                "var {name}: texture_{}<{}>;",
                dimension.wgsl(),
                sample.wgsl()
            )
        }
        BindingKind::StorageTexture(format, access, dimension) => {
            format!(
                "var {name}: texture_storage_{}<{}, {}>;",
                dimension.wgsl(),
                format.wgsl(),
                access.wgsl(),
            )
        }
        BindingKind::Sampler => format!("var {name}: sampler;"),
    };
    Ok(format!(
        "@group({}) @binding({}) {declaration}\n",
        crate::wgsl_u32_literal(group),
        crate::wgsl_u32_literal(binding.index),
    ))
}

/// Renders one type the way the checker prints it, for a diagnostic message.
///
/// # Errors
///
/// If the type names a class, an enum, or a string alias the module does not hold, returns an
/// internal diagnostic.
fn type_name(module: &Module, ty: &Type, pos: &Pos) -> Result<String, Diagnostic> {
    crate::pipeline::type_name(module, ty, pos)
}

/// Returns the WGSL spelling of one binary operator, and `None` for an operator outside K9.
fn binop(op: BinOp) -> Option<&'static str> {
    Some(match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        // Measured at ba6aa2e: a u32 / u32 expression has Type::U32 in typed HIR.
        BinOp::Div => "/",
        BinOp::Rem => "%",
        BinOp::Eq => "==",
        BinOp::Ne => "!=",
        BinOp::Lt => "<",
        BinOp::Le => "<=",
        BinOp::Gt => ">",
        BinOp::Ge => ">=",
        BinOp::And => "&&",
        BinOp::Or => "||",
        BinOp::BitAnd => "&",
        BinOp::BitOr => "|",
        _ => return None,
    })
}

/// Returns the precedence of the emitted WGSL operator. A higher number binds tighter.
///
/// The emitter judges parentheses on the emitted operator, never on the subscript expression kind
/// (K14). An operator outside the binary set gives 0.
fn binary_precedence(op: BinOp) -> u8 {
    match op {
        BinOp::Or => 1,
        BinOp::And => 2,
        BinOp::BitOr => 3,
        BinOp::BitAnd => 4,
        BinOp::Eq | BinOp::Ne => 5,
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => 6,
        BinOp::Add | BinOp::Sub => 7,
        BinOp::Mul | BinOp::Div | BinOp::Rem => 8,
        _ => 0,
    }
}

fn is_bitwise_operator(op: BinOp) -> bool {
    matches!(
        op,
        BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr | BinOp::UShr
    )
}

fn is_arithmetic_or_comparison_operator(op: BinOp) -> bool {
    matches!(
        op,
        BinOp::Add
            | BinOp::Sub
            | BinOp::Mul
            | BinOp::Div
            | BinOp::Rem
            | BinOp::Eq
            | BinOp::Ne
            | BinOp::Lt
            | BinOp::Le
            | BinOp::Gt
            | BinOp::Ge
    )
}

/// Reports whether `operand` needs parentheses inside a `parent` bitwise expression (K14).
///
/// WGSL gives two different bitwise operators no relative precedence, and Tint refuses a bitwise
/// operator mixed with an arithmetic or a comparison operator. `naga` accepts both forms, so the
/// emitter parenthesizes on the stricter side.
fn mixed_bitwise_chain(parent: BinOp, operand: &Expr) -> bool {
    let ExprKind::Binary { op: child, .. } = &operand.kind else {
        return false;
    };
    is_bitwise_operator(parent)
        && (is_arithmetic_or_comparison_operator(*child)
            || (is_bitwise_operator(*child) && parent != *child))
}

/// Returns one operand's text, parenthesized where the emitted WGSL needs it (K14).
///
/// `parent` is the enclosing operator's precedence and `right` marks the right operand, which
/// takes parentheses at equal precedence so that the emitted tree keeps the source's grouping.
/// `mixed_bitwise` comes from `mixed_bitwise_chain`. A mixed `&&` and `||` chain also takes them,
/// because Tint requires them and `naga` does not.
fn binary_operand(value: &Snippet, parent: u8, right: bool, mixed_bitwise: bool) -> String {
    let mixed_logical = matches!((parent, value.precedence), (1, 2) | (2, 1));
    if mixed_logical
        || mixed_bitwise
        || value.precedence < parent
        || (right && value.precedence == parent)
    {
        format!("({})", value.text)
    } else {
        value.text.clone()
    }
}

/// Returns the WGSL spelling of one literal, with the suffix its HIR type gives (K6, K14).
///
/// # Errors
///
/// Returns a K5 diagnostic for an `f64` literal, and a K6 diagnostic when the literal and its type
/// do not pair.
fn literal(expr: &Expr) -> Result<String, Diagnostic> {
    match (&expr.kind, &expr.ty) {
        (ExprKind::Int(value), Type::U32) => Ok(crate::wgsl_u32_literal(value)),
        (ExprKind::Int(value), Type::I32) => Ok(crate::wgsl_i32_literal(*value)),
        (ExprKind::Float(value), Type::F32) => Ok(f32_literal(*value)),
        (ExprKind::Float(_), Type::F64) => Err(diagnostic(
            "K5",
            "f64 is outside the kernel value types",
            expr.pos.clone(),
        )),
        (ExprKind::Bool(value), Type::Bool) => Ok(value.to_string()),
        _ => Err(diagnostic(
            "K6",
            "literal has no WGSL spelling for its HIR type",
            expr.pos.clone(),
        )),
    }
}

/// Returns the WGSL `f32` spelling of a value, with a fraction part and the `f` suffix (K14).
fn f32_literal(value: f64) -> String {
    let mut text = value.to_string();
    if !text.contains('.') && !text.contains('e') && !text.contains('E') {
        text.push_str(".0");
    }
    format!("{text}f")
}

/// Reports whether `ty` can reach a kernel as a WGSL `const` (K19).
///
/// The set is a scalar, a library vector or matrix, and a `FixedArray` of a foldable type. An
/// atomic is outside it.
///
/// # Errors
///
/// If the type names a class the module does not hold, returns an internal diagnostic.
fn constant_type(module: &Module, ty: &Type, pos: &Pos) -> Result<bool, Diagnostic> {
    Ok(match ty {
        Type::F32 | Type::I32 | Type::U32 | Type::Bool => true,
        Type::FixedArray(item, _) => constant_type(module, item, pos)?,
        Type::Class(id) => {
            let class = crate::class(module, id.0, "kernel::constant_type", pos)?;
            class.pos.file == "typegpu-types.ts"
                && (class.name.starts_with("Vec") || class.name.starts_with("Mat"))
                && atomic_scalar(module, ty, pos)?.is_none()
        }
        _ => false,
    })
}

/// The value of one folded module constant initializer (K19).
#[derive(Clone)]
enum FoldedConstant {
    Bool(bool),
    I32(i32),
    U32(u32),
    F32(f32),
    /// A vector, a matrix, or an array, which folds its arguments and keeps the call form.
    Construct {
        /// The emitted WGSL type spelling, such as `vec3<f32>`.
        constructor: String,
        /// The folded arguments, in order.
        args: Vec<FoldedConstant>,
    },
}

impl FoldedConstant {
    /// Returns the WGSL text of this value.
    ///
    /// A negative `i32` takes the precedence of a unary negation, so an enclosing operator
    /// parenthesizes it (K14).
    fn snippet(&self) -> Snippet {
        match self {
            Self::Bool(value) => Snippet::atom(value.to_string()),
            Self::I32(value) => {
                if *value < 0 {
                    Snippet::new(crate::wgsl_i32_literal(i64::from(*value)), 9)
                } else {
                    Snippet::atom(crate::wgsl_i32_literal(i64::from(*value)))
                }
            }
            Self::U32(value) => Snippet::atom(crate::wgsl_u32_literal(value)),
            Self::F32(value) => {
                let mut text = value.to_string();
                if !text.contains('.') && !text.contains('e') && !text.contains('E') {
                    text.push_str(".0");
                }
                Snippet::atom(format!("{text}f"))
            }
            Self::Construct { constructor, args } => Snippet::atom(format!(
                "{constructor}({})",
                args.iter()
                    .map(|arg| arg.snippet().text)
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }
}

/// Builds the K19 diagnostic for constant arithmetic that leaves the constant's type.
fn constant_overflow(ty: &str, pos: &Pos) -> Diagnostic {
    diagnostic(
        "K19",
        format!("module constant arithmetic overflows {ty}"),
        pos.clone(),
    )
}

/// Folds one binary operation over two constant values (K19).
///
/// The arithmetic is checked in the constant's own type, so the folded value is the value the GPU
/// reads.
///
/// # Errors
///
/// Returns a K19 diagnostic for an overflow, a division by zero, an operator the type does not
/// carry, and a pair of operands of different types.
fn fold_binary(
    op: BinOp,
    left: FoldedConstant,
    right: FoldedConstant,
    pos: &Pos,
) -> Result<FoldedConstant, Diagnostic> {
    macro_rules! checked_integer {
        ($left:expr, $right:expr, $ty:literal, $variant:ident) => {{
            let left = $left;
            let right = $right;
            let value = match op {
                BinOp::Add => left.checked_add(right),
                BinOp::Sub => left.checked_sub(right),
                BinOp::Mul => left.checked_mul(right),
                BinOp::Div if right == 0 => {
                    return Err(diagnostic(
                        "K19",
                        "module constant divides by zero",
                        pos.clone(),
                    ));
                }
                BinOp::Div => left.checked_div(right),
                BinOp::Rem if right == 0 => {
                    return Err(diagnostic(
                        "K19",
                        "module constant divides by zero",
                        pos.clone(),
                    ));
                }
                BinOp::Rem => left.checked_rem(right),
                BinOp::BitAnd => Some(left & right),
                BinOp::BitOr => Some(left | right),
                BinOp::Eq => return Ok(FoldedConstant::Bool(left == right)),
                BinOp::Ne => return Ok(FoldedConstant::Bool(left != right)),
                BinOp::Lt => return Ok(FoldedConstant::Bool(left < right)),
                BinOp::Le => return Ok(FoldedConstant::Bool(left <= right)),
                BinOp::Gt => return Ok(FoldedConstant::Bool(left > right)),
                BinOp::Ge => return Ok(FoldedConstant::Bool(left >= right)),
                _ => None,
            }
            .ok_or_else(|| constant_overflow($ty, pos))?;
            Ok(FoldedConstant::$variant(value))
        }};
    }

    match (left, right) {
        (FoldedConstant::I32(left), FoldedConstant::I32(right)) => {
            checked_integer!(left, right, "i32", I32)
        }
        (FoldedConstant::U32(left), FoldedConstant::U32(right)) => {
            checked_integer!(left, right, "u32", U32)
        }
        (FoldedConstant::F32(left), FoldedConstant::F32(right)) => {
            let value = match op {
                BinOp::Add => left + right,
                BinOp::Sub => left - right,
                BinOp::Mul => left * right,
                BinOp::Div if right == 0.0 => {
                    return Err(diagnostic(
                        "K19",
                        "module constant divides by zero",
                        pos.clone(),
                    ));
                }
                BinOp::Div => left / right,
                BinOp::Rem if right == 0.0 => {
                    return Err(diagnostic(
                        "K19",
                        "module constant divides by zero",
                        pos.clone(),
                    ));
                }
                BinOp::Rem => left % right,
                BinOp::Eq => return Ok(FoldedConstant::Bool(left == right)),
                BinOp::Ne => return Ok(FoldedConstant::Bool(left != right)),
                BinOp::Lt => return Ok(FoldedConstant::Bool(left < right)),
                BinOp::Le => return Ok(FoldedConstant::Bool(left <= right)),
                BinOp::Gt => return Ok(FoldedConstant::Bool(left > right)),
                BinOp::Ge => return Ok(FoldedConstant::Bool(left >= right)),
                _ => {
                    return Err(diagnostic(
                        "K19",
                        "module constant uses an unsupported binary operator",
                        pos.clone(),
                    ));
                }
            };
            if !value.is_finite() {
                return Err(constant_overflow("f32", pos));
            }
            Ok(FoldedConstant::F32(value))
        }
        (FoldedConstant::Bool(left), FoldedConstant::Bool(right)) => match op {
            BinOp::And => Ok(FoldedConstant::Bool(left && right)),
            BinOp::Or => Ok(FoldedConstant::Bool(left || right)),
            BinOp::Eq => Ok(FoldedConstant::Bool(left == right)),
            BinOp::Ne => Ok(FoldedConstant::Bool(left != right)),
            _ => Err(diagnostic(
                "K19",
                "module constant uses an unsupported boolean operator",
                pos.clone(),
            )),
        },
        _ => Err(diagnostic(
            "K19",
            "module constant binary operands are not foldable scalars",
            pos.clone(),
        )),
    }
}

/// Folds one module constant initializer expression to a value (K19).
///
/// The admitted forms are a literal, a unary or binary expression over folded values, another
/// module constant, a vector factory, a value-class construction, and an array literal. `cache`
/// holds the constants already folded and `visiting` holds the ones on the current path.
///
/// # Errors
///
/// Returns a K19 diagnostic for every other initializer, and the diagnostics of `fold_binary`.
fn fold_constant_expr(
    module: &Module,
    globals: &BTreeMap<String, KernelGlobal>,
    cache: &mut BTreeMap<String, FoldedConstant>,
    visiting: &mut BTreeSet<String>,
    expr: &Expr,
) -> Result<FoldedConstant, Diagnostic> {
    match &expr.kind {
        ExprKind::Int(value) => match expr.ty {
            Type::I32 => i32::try_from(*value)
                .map(FoldedConstant::I32)
                .map_err(|_| constant_overflow("i32", &expr.pos)),
            Type::U32 => u32::try_from(*value)
                .map(FoldedConstant::U32)
                .map_err(|_| constant_overflow("u32", &expr.pos)),
            _ => Err(diagnostic(
                "K19",
                "module constant integer has an unsupported type",
                expr.pos.clone(),
            )),
        },
        ExprKind::Float(value) if expr.ty == Type::F32 => {
            let value = *value as f32;
            if value.is_finite() {
                Ok(FoldedConstant::F32(value))
            } else {
                Err(constant_overflow("f32", &expr.pos))
            }
        }
        ExprKind::Bool(value) => Ok(FoldedConstant::Bool(*value)),
        ExprKind::Global(name) => fold_global_constant(module, globals, cache, visiting, name),
        ExprKind::Unary { op, operand } => {
            let operand = fold_constant_expr(module, globals, cache, visiting, operand)?;
            match (op, operand) {
                (UnOp::Neg, FoldedConstant::I32(value)) => value
                    .checked_neg()
                    .map(FoldedConstant::I32)
                    .ok_or_else(|| constant_overflow("i32", &expr.pos)),
                (UnOp::Neg, FoldedConstant::F32(value)) => Ok(FoldedConstant::F32(-value)),
                (UnOp::Not, FoldedConstant::Bool(value)) => Ok(FoldedConstant::Bool(!value)),
                (UnOp::BitNot, FoldedConstant::I32(value)) => Ok(FoldedConstant::I32(!value)),
                (UnOp::BitNot, FoldedConstant::U32(value)) => Ok(FoldedConstant::U32(!value)),
                _ => Err(diagnostic(
                    "K19",
                    "module constant uses an unsupported unary operator",
                    expr.pos.clone(),
                )),
            }
        }
        ExprKind::Binary { op, left, right } => fold_binary(
            *op,
            fold_constant_expr(module, globals, cache, visiting, left)?,
            fold_constant_expr(module, globals, cache, visiting, right)?,
            &expr.pos,
        ),
        ExprKind::Call {
            callee: Callee::Func(name),
            args,
        } if function_declared_in(module, name, "typegpu-types.ts") => {
            let constructor = mapping::free_function(name).ok_or_else(|| {
                diagnostic(
                    "K19",
                    format!("module constant calls unsupported function `{name}`"),
                    expr.pos.clone(),
                )
            })?;
            if !constructor.starts_with("vec") {
                return Err(diagnostic(
                    "K19",
                    format!("module constant calls non-vector factory `{name}`"),
                    expr.pos.clone(),
                ));
            }
            Ok(FoldedConstant::Construct {
                constructor: constructor.to_owned(),
                args: args
                    .iter()
                    .map(|arg| fold_constant_expr(module, globals, cache, visiting, arg))
                    .collect::<Result<Vec<_>, _>>()?,
            })
        }
        ExprKind::New { class, args } => {
            let class = crate::class(module, class.0, "kernel::fold_constant_expr", &expr.pos)?;
            if !class.is_value {
                return Err(diagnostic(
                    "K19",
                    format!(
                        "module constant constructs reference class `{}`",
                        class.name
                    ),
                    expr.pos.clone(),
                ));
            }
            Ok(FoldedConstant::Construct {
                constructor: wgsl_type(module, &expr.ty, &expr.pos)?,
                args: args
                    .iter()
                    .map(|arg| fold_constant_expr(module, globals, cache, visiting, arg))
                    .collect::<Result<Vec<_>, _>>()?,
            })
        }
        ExprKind::ArrayLit(args) => Ok(FoldedConstant::Construct {
            constructor: wgsl_type(module, &expr.ty, &expr.pos)?,
            args: args
                .iter()
                .map(|arg| fold_constant_expr(module, globals, cache, visiting, arg))
                .collect::<Result<Vec<_>, _>>()?,
        }),
        _ => Err(diagnostic(
            "K19",
            "module constant initializer is not evaluable",
            expr.pos.clone(),
        )),
    }
}

/// Folds one named module constant, through the cache (K19).
///
/// # Errors
///
/// Returns a K19 diagnostic when the name is a variable rather than a constant, and when its type
/// is outside `constant_type`. A cycle gives a generator diagnostic: subscript rejects a module
/// initializer that reads a declaration after it, so a cycle never reaches a checked program.
fn fold_global_constant(
    module: &Module,
    globals: &BTreeMap<String, KernelGlobal>,
    cache: &mut BTreeMap<String, FoldedConstant>,
    visiting: &mut BTreeSet<String>,
    name: &str,
) -> Result<FoldedConstant, Diagnostic> {
    if let Some(value) = cache.get(name) {
        return Ok(value.clone());
    }
    let global = globals.get(name).ok_or_else(|| {
        generator_diagnostic(
            format!("global `{name}` disappeared from typed HIR"),
            Pos::new("", 1, 1),
        )
    })?;
    let KernelGlobalKind::Constant(init) = &global.kind else {
        return Err(diagnostic(
            "K19",
            format!("module constant initializer reads variable `{name}`"),
            global.pos.clone(),
        ));
    };
    if !constant_type(module, &global.ty, &global.pos)? {
        return Err(diagnostic(
            "K19",
            format!(
                "module constant `{name}` has unsupported type `{}`",
                type_name(module, &global.ty, &global.pos)?
            ),
            global.pos.clone(),
        ));
    }
    if !visiting.insert(name.to_owned()) {
        return Err(generator_diagnostic(
            format!("module constant cycle includes `{name}`"),
            global.pos.clone(),
        ));
    }
    let value = fold_constant_expr(module, globals, cache, visiting, init);
    visiting.remove(name);
    let value = value?;
    cache.insert(name.to_owned(), value.clone());
    Ok(value)
}

/// Emits one constant expression as WGSL text, with no fold (K20).
///
/// A private variable's initializer keeps its expression form, so a named constant stays a name.
/// The parentheses follow the emitted operator, as in a kernel body (K14).
///
/// # Errors
///
/// Returns a K19 diagnostic for an operator or a call outside the constant set, and for a read of
/// a variable.
fn constant_snippet(
    module: &Module,
    globals: &BTreeMap<String, KernelGlobal>,
    expr: &Expr,
) -> Result<Snippet, Diagnostic> {
    match &expr.kind {
        ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_) => {
            Ok(Snippet::atom(literal(expr)?))
        }
        ExprKind::Global(name) => {
            let global = globals.get(name).ok_or_else(|| {
                generator_diagnostic(
                    format!("global `{name}` disappeared from typed HIR"),
                    expr.pos.clone(),
                )
            })?;
            if !matches!(global.kind, KernelGlobalKind::Constant(_)) {
                return Err(diagnostic(
                    "K19",
                    format!("module constant initializer reads variable `{name}`"),
                    expr.pos.clone(),
                ));
            }
            Ok(Snippet::atom(mapping::ident(name)))
        }
        ExprKind::Unary { op, operand } => {
            let operand = constant_snippet(module, globals, operand)?;
            let spelling = match op {
                UnOp::Neg => "-",
                UnOp::Not => "!",
                UnOp::BitNot => "~",
                _ => {
                    return Err(diagnostic(
                        "K19",
                        "module constant uses an unsupported unary operator",
                        expr.pos.clone(),
                    ))
                }
            };
            let text = if operand.precedence <= 9 {
                format!("({})", operand.text)
            } else {
                operand.text
            };
            Ok(Snippet::new(format!("{spelling}{text}"), 9))
        }
        ExprKind::Binary { op, left, right } => {
            let spelling = binop(*op).ok_or_else(|| {
                diagnostic(
                    "K19",
                    "module constant uses an unsupported binary operator",
                    expr.pos.clone(),
                )
            })?;
            let left_mixed = mixed_bitwise_chain(*op, left);
            let right_mixed = mixed_bitwise_chain(*op, right);
            let left = constant_snippet(module, globals, left)?;
            let right = constant_snippet(module, globals, right)?;
            let precedence = binary_precedence(*op);
            Ok(Snippet::new(
                format!(
                    "{} {spelling} {}",
                    binary_operand(&left, precedence, false, left_mixed),
                    binary_operand(&right, precedence, true, right_mixed)
                ),
                precedence,
            ))
        }
        ExprKind::Call {
            callee: Callee::Func(name),
            args,
        } if function_declared_in(module, name, "typegpu-types.ts") => {
            let Some(factory) = mapping::free_function(name) else {
                return Err(diagnostic(
                    "K19",
                    format!("module constant calls unsupported function `{name}`"),
                    expr.pos.clone(),
                ));
            };
            if !factory.starts_with("vec") {
                return Err(diagnostic(
                    "K19",
                    format!("module constant calls non-vector factory `{name}`"),
                    expr.pos.clone(),
                ));
            }
            let args = args
                .iter()
                .map(|arg| constant_snippet(module, globals, arg).map(|value| value.text))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Snippet::atom(format!("{factory}({})", args.join(", "))))
        }
        ExprKind::New { class, args } => {
            let class = crate::class(module, class.0, "kernel::constant_snippet", &expr.pos)?;
            if !class.is_value {
                return Err(diagnostic(
                    "K19",
                    format!(
                        "module constant constructs reference class `{}`",
                        class.name
                    ),
                    expr.pos.clone(),
                ));
            }
            let constructor = wgsl_type(module, &expr.ty, &expr.pos)?;
            let args = args
                .iter()
                .map(|arg| constant_snippet(module, globals, arg).map(|value| value.text))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Snippet::atom(format!("{constructor}({})", args.join(", "))))
        }
        _ => Err(diagnostic(
            "K19",
            "module constant initializer is not evaluable",
            expr.pos.clone(),
        )),
    }
}

/// Emits the module constant, private, and workgroup declarations, in declaration order (K14).
///
/// The text ends with one blank line when it holds a declaration, and is empty otherwise.
///
/// # Errors
///
/// A constant type or an initializer outside the fold gives a K19 diagnostic. A private
/// initializer that does not evaluate gives a K20 diagnostic, as does a zero-length workgroup
/// array. An atomic in the private address space gives a K21 diagnostic.
fn emit_kernel_globals(module: &Module, globals: &[KernelGlobal]) -> Result<String, Diagnostic> {
    let by_name = globals
        .iter()
        .map(|global| (global.name.clone(), global.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut folded = BTreeMap::new();
    let mut visiting = BTreeSet::new();
    let mut out = String::new();
    for global in globals {
        let name = mapping::ident(&global.name);
        match &global.kind {
            KernelGlobalKind::Constant(_) => {
                if !constant_type(module, &global.ty, &global.pos)? {
                    return Err(diagnostic(
                        "K19",
                        format!(
                            "module constant `{}` has unsupported type `{}`",
                            global.name,
                            type_name(module, &global.ty, &global.pos)?
                        ),
                        global.pos.clone(),
                    ));
                }
                let ty = wgsl_type(module, &global.ty, &global.pos)?;
                let value = fold_global_constant(
                    module,
                    &by_name,
                    &mut folded,
                    &mut visiting,
                    &global.name,
                )?
                .snippet();
                out.push_str(&format!("const {name}: {ty} = {};\n", value.text));
            }
            KernelGlobalKind::Private(init) => {
                if type_contains_atomic(module, &global.ty, &global.pos)? {
                    return Err(diagnostic(
                        "K21",
                        "an atomic value cannot use private address space",
                        global.pos.clone(),
                    ));
                }
                let ty = wgsl_type(module, &global.ty, &global.pos)?;
                // A private variable's initializer keeps its expression form, so a K19 failure
                // here belongs to the K20 declaration the author wrote.
                let value = constant_snippet(module, &by_name, init).map_err(|_| {
                    diagnostic(
                        "K20",
                        format!(
                            "private variable `{}` initializer is not evaluable",
                            global.name
                        ),
                        init.pos.clone(),
                    )
                })?;
                out.push_str(&format!("var<private> {name}: {ty} = {};\n", value.text));
            }
            KernelGlobalKind::WorkgroupVar => {
                let ty = wgsl_type(module, &global.ty, &global.pos)?;
                out.push_str(&format!("var<workgroup> {name}: {ty};\n"));
            }
            KernelGlobalKind::WorkgroupArray(length) => {
                if *length == 0 {
                    return Err(diagnostic(
                        "K20",
                        "workgroup array length is zero",
                        global.pos.clone(),
                    ));
                }
                let ty = wgsl_type(module, &global.ty, &global.pos)?;
                out.push_str(&format!(
                    "var<workgroup> {name}: array<{ty}, {}>;\n",
                    crate::wgsl_u32_literal(length)
                ));
            }
        }
    }
    if !out.is_empty() {
        out.push('\n');
    }
    Ok(out)
}

/// Returns the barrier name when `expr` calls `workgroupBarrier` or `storageBarrier` (K22).
///
/// The declaring file and the empty argument list both must match, so a program's own function of
/// the same name is not a barrier.
fn barrier_call<'a>(module: &Module, expr: &'a Expr) -> Option<&'a str> {
    let ExprKind::Call {
        callee: Callee::Func(name),
        args,
    } = &expr.kind
    else {
        return None;
    };
    if !args.is_empty() || !function_declared_in(module, name, "typegpu.ts") {
        return None;
    }
    let base = crate::base_name(name);
    matches!(base, "workgroupBarrier" | "storageBarrier").then_some(base)
}

// K18 permits continue because WGSL targets the enclosing loop through a switch.
/// Reports whether a switch case body ends with `break`, `continue`, or `return` (K18).
fn case_terminates(body: &[Stmt]) -> bool {
    match body.last() {
        Some(Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return { .. }) => true,
        Some(Stmt::Block(body)) => case_terminates(body),
        _ => false,
    }
}

/// Rejects a statement outside the kernel statement set, before emission (K7).
///
/// # Errors
///
/// Returns a K7 diagnostic for a `for...of` over anything but a `FixedArray`. The emitter rejects
/// the remaining statements as it writes them.
fn validate_statement_subset(statements: &[Stmt]) -> Result<(), Diagnostic> {
    for statement in statements {
        match statement {
            Stmt::ForOf { kind, pos, .. } if *kind != ForOfKind::FixedArrayValues => {
                return Err(diagnostic(
                    "K7",
                    "statement is outside the current kernel subset",
                    pos.clone(),
                ));
            }
            Stmt::If { then, els, .. } => {
                validate_statement_subset(then)?;
                if let Some(els) = els {
                    validate_statement_subset(els)?;
                }
            }
            Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::ForOf { body, .. } => {
                validate_statement_subset(body)?;
            }
            Stmt::Switch { cases, .. } => {
                for case in cases {
                    validate_statement_subset(&case.body)?;
                }
            }
            Stmt::Block(body) => validate_statement_subset(body)?,
            _ => {}
        }
    }
    Ok(())
}

/// One binding as the emitter needs it: the emitted name, the kind, and the item type.
#[derive(Debug, Clone)]
struct BindingRef {
    /// The mangled WGSL variable name (K14).
    name: String,
    /// The address space and the resource kind, which decides the legal methods.
    kind: BindingKind,
    /// The wrapper's item type `T`.
    item_ty: Type,
}

/// Collects the local names that the statements declare, in declaration order.
///
/// A block-scoped local and a `for` variable both count, because WGSL and subscript scope them
/// differently and the emitted names must stay distinct (K14).
fn local_declarations(statements: &[Stmt], out: &mut Vec<String>) {
    for statement in statements {
        match statement {
            Stmt::Let { name, .. } => out.push(name.clone()),
            Stmt::If { then, els, .. } => {
                local_declarations(then, out);
                if let Some(els) = els {
                    local_declarations(els, out);
                }
            }
            Stmt::While { body, .. } => {
                local_declarations(body, out);
            }
            Stmt::ForOf { name, body, .. } => {
                out.push(name.clone());
                local_declarations(body, out);
            }
            Stmt::For { init, body, .. } => {
                if let Some(init) = init {
                    if let Stmt::Let { name, .. } = init.as_ref() {
                        out.push(name.clone());
                    }
                }
                local_declarations(body, out);
            }
            Stmt::Switch { cases, .. } => {
                for case in cases {
                    local_declarations(&case.body, out);
                }
            }
            Stmt::Block(body) => local_declarations(body, out),
            _ => {}
        }
    }
}

/// Maps each parameter and local of one function to the name the emitter writes (K14).
///
/// `module_names` is the module-scope name set, fixed before any body is emitted. A local whose
/// mangled name is in that set, or that another local already took, gains a `_` until it is free.
/// A binding read therefore never resolves to a local.
fn local_names(function: &Function, module_names: &BTreeSet<String>) -> BTreeMap<String, String> {
    let mut originals = function
        .params
        .iter()
        .map(|param| param.name.clone())
        .collect::<Vec<_>>();
    local_declarations(&function.body, &mut originals);
    let mut used = module_names.clone();
    let mut names = BTreeMap::new();
    for original in originals {
        if names.contains_key(&original) {
            continue;
        }
        let mut emitted = mapping::ident(&original);
        while used.contains(&emitted) {
            emitted.push('_');
        }
        used.insert(emitted.clone());
        names.insert(original, emitted);
    }
    names
}

/// Emits the WGSL of one function body, entry point or helper.
struct Emitter<'a> {
    module: &'a Module,
    /// Each layout parameter name and its group index, so a field access on one is a binding.
    layout_params: BTreeMap<String, usize>,
    /// The layout class names, which a kernel local must never take (PI6).
    layout_names: BTreeSet<String>,
    /// The invocation parameter name, whose field reads are builtins (PI4, RN3).
    invocation_param: String,
    /// The entry point kind, which decides the legal builtins and the legal texture methods.
    invocation_kind: InvocationKind,
    /// Every binding, by group and field name.
    bindings: BTreeMap<(usize, String), BindingRef>,
    /// Every module-level declaration the call graph reads, by name.
    globals: BTreeMap<String, KernelGlobal>,
    /// The emitted name of each parameter and local (K14).
    local_names: BTreeMap<String, String>,
    /// The builtins the body reads. The entry point declares these parameters and no other (PI4).
    used_builtins: BTreeSet<String>,
    /// The counter behind each lowered conditional's placeholder name (K9).
    conditional_index: u32,
    /// The enclosing loop count, which `break` and `continue` need (K18).
    loop_depth: u32,
    /// The enclosing switch count, which `break` needs (K18).
    switch_depth: u32,
    /// Whether this body is a helper, where a barrier is illegal (K22).
    in_helper: bool,
}

/// The entry point kind that a body belongs to. A helper belongs to none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InvocationKind {
    None,
    Compute,
    Vertex,
    Fragment,
}

impl<'a> Emitter<'a> {
    /// Builds the emitter of one entry point.
    ///
    /// `invocation_index` is the invocation parameter's position, which follows the layouts and,
    /// for a vertex kernel, the vertex and instance values (PI2, RN2). `module_names` fixes the
    /// module-scope names before any body is emitted (K14).
    ///
    /// # Errors
    ///
    /// If the kernel has fewer parameters than the declaration promises, returns an internal
    /// diagnostic. Pipeline discovery already checked the count.
    fn entry(
        module: &'a Module,
        layouts: &'a [crate::pipeline::Layout],
        kernel: &Function,
        invocation_index: usize,
        invocation_kind: InvocationKind,
        globals: &[KernelGlobal],
        module_names: &BTreeSet<String>,
    ) -> Result<Self, Diagnostic> {
        // Pipeline discovery validates each layout parameter and the invocation parameter.

        let mut layout_params = BTreeMap::new();
        let mut bindings = BTreeMap::new();
        for (group, layout) in layouts.iter().enumerate() {
            layout_params.insert(
                kernel
                    .params
                    .get(group)
                    .ok_or_else(|| {
                        crate::internal(
                            "kernel::Emitter::entry",
                            "missing layout parameter",
                            &kernel.pos,
                        )
                    })?
                    .name
                    .clone(),
                group,
            );
            for binding in &layout.bindings {
                bindings.insert(
                    (group, binding.name.clone()),
                    BindingRef {
                        name: mapping::ident(&binding.name),
                        kind: binding.kind,
                        item_ty: binding.item_ty.clone(),
                    },
                );
            }
        }
        let local_names = local_names(kernel, module_names);

        Ok(Self {
            module,
            layout_params,
            layout_names: layouts.iter().map(|layout| layout.name.clone()).collect(),
            invocation_param: kernel
                .params
                .get(invocation_index)
                .ok_or_else(|| {
                    crate::internal(
                        "kernel::Emitter::entry",
                        "missing invocation parameter",
                        &kernel.pos,
                    )
                })?
                .name
                .clone(),
            invocation_kind,
            bindings,
            globals: globals
                .iter()
                .map(|global| (global.name.clone(), global.clone()))
                .collect(),
            local_names,
            used_builtins: BTreeSet::new(),
            conditional_index: 0,
            loop_depth: 0,
            switch_depth: 0,
            in_helper: false,
        })
    }

    /// Builds the emitter of one helper body.
    ///
    /// A helper takes no layout class and no invocation, so it reaches no binding and no builtin
    /// (K2). It reads the module declarations like an entry point.
    fn helper(
        module: &'a Module,
        helper: &Function,
        globals: &[KernelGlobal],
        module_names: &BTreeSet<String>,
    ) -> Self {
        Self {
            module,
            layout_params: BTreeMap::new(),
            layout_names: BTreeSet::new(),
            invocation_param: String::new(),
            invocation_kind: InvocationKind::None,
            bindings: BTreeMap::new(),
            globals: globals
                .iter()
                .map(|global| (global.name.clone(), global.clone()))
                .collect(),
            local_names: local_names(helper, module_names),
            used_builtins: BTreeSet::new(),
            conditional_index: 0,
            loop_depth: 0,
            switch_depth: 0,
            in_helper: true,
        }
    }

    /// Returns the emitted name of one local or parameter (K14).
    fn local_name(&self, name: &str) -> String {
        self.local_names
            .get(name)
            .cloned()
            .unwrap_or_else(|| mapping::ident(name))
    }

    /// Returns the binding that `expr` names, when `expr` is a field access on a layout parameter.
    ///
    /// `res.particles` gives the binding. Any other expression gives `None` (PI6).
    fn binding_ref(&self, expr: &Expr) -> Option<BindingRef> {
        let ExprKind::Field { obj, name } = &expr.kind else {
            return None;
        };
        let ExprKind::Local(param) = &obj.kind else {
            return None;
        };
        let group = *self.layout_params.get(param)?;
        self.bindings.get(&(group, name.clone())).cloned()
    }

    /// Returns the binding at the root of a place expression.
    ///
    /// `res.particles[i].value` and `res.particles.get(i)` both give the binding, so a nested place
    /// keeps the address space of the binding it lives in (K21, K22).
    fn binding_root(&self, expr: &Expr) -> Option<BindingRef> {
        if let Some(binding) = self.binding_ref(expr) {
            return Some(binding);
        }
        match &expr.kind {
            ExprKind::Field { obj, .. } | ExprKind::Index { obj, .. } => self.binding_root(obj),
            ExprKind::Call {
                callee: Callee::Method { recv, .. },
                ..
            } => self.binding_root(recv),
            _ => None,
        }
    }

    /// Returns the module-level declaration at the root of a place expression.
    ///
    /// The accessor calls `$` and `get` pass through, because they are the authored access forms
    /// of a private or workgroup variable (K20).
    fn global_root(&self, expr: &Expr) -> Option<KernelGlobal> {
        match &expr.kind {
            ExprKind::Global(name) => self.globals.get(name).cloned(),
            ExprKind::Field { obj, .. } | ExprKind::Index { obj, .. } => self.global_root(obj),
            ExprKind::Call {
                callee: Callee::Method { recv, name },
                args,
            } if (name == "$" && args.is_empty()) || (name == "get" && args.len() == 1) => {
                self.global_root(recv)
            }
            _ => None,
        }
    }

    /// Returns the private or workgroup variable that `expr` names directly (K20).
    ///
    /// A module constant gives `None`, because a constant carries no accessor methods.
    fn wrapper_ref(&self, expr: &Expr) -> Option<KernelGlobal> {
        let ExprKind::Global(name) = &expr.kind else {
            return None;
        };
        self.globals.get(name).and_then(|global| {
            (!matches!(global.kind, KernelGlobalKind::Constant(_))).then(|| global.clone())
        })
    }

    /// Returns the emitted place of an atomic method receiver, which the builtin takes by pointer.
    ///
    /// # Errors
    ///
    /// Returns a K21 diagnostic when the receiver sits behind a uniform or read-only storage
    /// binding. A receiver that is neither a storage place nor a workgroup place gives the same
    /// diagnostic, because an atomic on a local has no address to take.
    fn atomic_place(&mut self, recv: &Expr) -> Result<Snippet, Diagnostic> {
        let binding = self.binding_root(recv);
        if binding
            .as_ref()
            .is_some_and(|binding| binding.kind != BindingKind::MutStorage)
        {
            return Err(diagnostic(
                "K21",
                "atomic method receiver is behind a uniform or read-only storage binding",
                recv.pos.clone(),
            ));
        }
        let storage = binding.is_some();
        let workgroup = self.global_root(recv).is_some_and(|global| {
            matches!(
                global.kind,
                KernelGlobalKind::WorkgroupVar | KernelGlobalKind::WorkgroupArray(_)
            )
        });
        if !storage && !workgroup {
            return Err(diagnostic(
                "K21",
                "atomic method receiver is not a storage or workgroup place",
                recv.pos.clone(),
            ));
        }
        if let ExprKind::Call {
            callee: Callee::Method { recv, name },
            args,
        } = &recv.kind
        {
            if name == "$" && args.is_empty() {
                if let Some(global) = self.wrapper_ref(recv) {
                    return Ok(Snippet::atom(mapping::ident(&global.name)));
                }
            }
        }
        self.snippet(recv)
    }

    /// Emits every argument and returns their texts with one merged prelude, in argument order.
    ///
    /// # Errors
    ///
    /// Returns the first diagnostic that an argument gives.
    fn snippets(&mut self, args: &[Expr]) -> Result<(Vec<String>, Prelude), Diagnostic> {
        let mut texts = Vec::with_capacity(args.len());
        let mut prelude = Vec::new();
        for arg in args {
            let value = self.snippet(arg)?;
            prelude.extend(value.prelude);
            texts.push(value.text);
        }
        Ok((texts, prelude))
    }

    /// Emits the argument of a `Math.fround` call, which lowers to the argument alone (K11).
    ///
    /// The checker types a JavaScript `Math` argument as `f64`, so this walk spells the literals
    /// and the operators as `f32`. `f64` stays outside the kernel value types (K5).
    ///
    /// # Errors
    ///
    /// Returns a K11 diagnostic for an operator outside K11, and the diagnostics of `snippet`.
    fn fround_argument(&mut self, expr: &Expr) -> Result<Snippet, Diagnostic> {
        match (&expr.kind, &expr.ty) {
            (ExprKind::Float(value), Type::F64) => Ok(Snippet::atom(f32_literal(*value))),
            (ExprKind::Binary { op, left, right }, Type::F64) => {
                let Some(spelling) = binop(*op) else {
                    return Err(diagnostic(
                        "K11",
                        "Math.fround argument uses an operator outside K11",
                        expr.pos.clone(),
                    ));
                };
                let left_mixed = mixed_bitwise_chain(*op, left);
                let right_mixed = mixed_bitwise_chain(*op, right);
                let left = self.fround_argument(left)?;
                let right = self.fround_argument(right)?;
                let precedence = binary_precedence(*op);
                let text = format!(
                    "{} {spelling} {}",
                    binary_operand(&left, precedence, false, left_mixed),
                    binary_operand(&right, precedence, true, right_mixed)
                );
                let mut prelude = left.prelude;
                prelude.extend(right.prelude);
                Ok(Snippet {
                    text,
                    precedence,
                    prelude,
                })
            }
            _ => self.snippet(expr),
        }
    }

    /// Emits one expression as a WGSL snippet (K9).
    ///
    /// The returned prelude holds the statements that must precede the expression, which only a
    /// lowered conditional produces. The caller writes them at the place the expression is
    /// evaluated, so both sides keep short-circuit evaluation.
    ///
    /// # Errors
    ///
    /// An expression outside the set gives a K9 diagnostic. A string and a reference class give a
    /// K5 diagnostic. A cast outside `f32`, `i32`, and `u32` gives a K12 diagnostic. A whole-value
    /// write to an atomic gives a K21 diagnostic.
    fn snippet(&mut self, expr: &Expr) -> Result<Snippet, Diagnostic> {
        if let ExprKind::Cast(value) = &expr.kind {
            let fround_to_f32 = matches!(
                &value.kind,
                ExprKind::Call { callee: Callee::Math(function), .. }
                    if mapping::math(*function) == Some("") && expr.ty == Type::F32
            );
            if !fround_to_f32
                && (!matches!(expr.ty, Type::F32 | Type::I32 | Type::U32)
                    || !matches!(value.ty, Type::F32 | Type::I32 | Type::U32))
            {
                return Err(diagnostic(
                    "K12",
                    "cast is not among f32, i32, and u32",
                    expr.pos.clone(),
                ));
            }
        }
        let ty = if matches!(
            expr.kind,
            ExprKind::Cast(_) | ExprKind::New { .. } | ExprKind::Cond { .. }
        ) {
            wgsl_type(self.module, &expr.ty, &expr.pos)?
        } else {
            String::new()
        };
        let local = |text: String, _ty: String| Snippet::atom(text);
        match &expr.kind {
            ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_) => {
                Ok(local(literal(expr)?, ty))
            }
            ExprKind::Str(_) => Err(diagnostic(
                "K5",
                "string local or expression in kernel",
                expr.pos.clone(),
            )),
            ExprKind::AbsenceTest { .. } => Err(diagnostic(
                "K5",
                "string local or expression in kernel",
                expr.pos.clone(),
            )),
            ExprKind::Local(name) => Ok(Snippet::atom(self.local_name(name))),
            ExprKind::Global(name) => {
                if !self.globals.contains_key(name) {
                    return Err(generator_diagnostic(
                        format!("global `{name}` has no kernel declaration"),
                        expr.pos.clone(),
                    ));
                }
                Ok(Snippet::atom(mapping::ident(name)))
            }
            ExprKind::Unary { op, operand } => {
                let value = self.snippet(operand)?;
                let spelling = match op {
                    UnOp::Neg => "-",
                    UnOp::Not => "!",
                    UnOp::BitNot => "~",
                    _ => {
                        return Err(diagnostic(
                            "K9",
                            "unary operator is outside K9",
                            expr.pos.clone(),
                        ))
                    }
                };
                let text = if value.precedence <= 9 {
                    format!("({})", value.text)
                } else {
                    value.text.clone()
                };
                Ok(Snippet {
                    text: format!("{spelling}{text}"),
                    precedence: 9,
                    prelude: value.prelude,
                })
            }
            ExprKind::Binary { op, left, right } => {
                let Some(spelling) = binop(*op) else {
                    return Err(diagnostic(
                        "K9",
                        "this binary operator is outside K9",
                        expr.pos.clone(),
                    ));
                };
                let left_mixed = mixed_bitwise_chain(*op, left);
                let right_mixed = mixed_bitwise_chain(*op, right);
                let left = self.snippet(left)?;
                let right = self.snippet(right)?;
                let precedence = binary_precedence(*op);
                let text = format!(
                    "{} {spelling} {}",
                    binary_operand(&left, precedence, false, left_mixed),
                    binary_operand(&right, precedence, true, right_mixed)
                );
                let mut prelude = left.prelude;
                prelude.extend(right.prelude);
                Ok(Snippet {
                    text,
                    precedence,
                    prelude,
                })
            }
            ExprKind::Assign { op, target, value } => {
                if (self.binding_root(target).is_some() || self.global_root(target).is_some())
                    && type_contains_atomic(self.module, &target.ty, &target.pos)?
                {
                    return Err(diagnostic(
                        "K21",
                        "an atomic value or schema cannot be written as a whole",
                        target.pos.clone(),
                    ));
                }
                let target = self.snippet(target)?;
                let value = self.snippet(value)?;
                let spelling = match op {
                    None => "=".to_owned(),
                    Some(op) => format!(
                        "{}=",
                        binop(*op).ok_or_else(|| diagnostic(
                            "K9",
                            "assignment operator is outside K9",
                            expr.pos.clone()
                        ))?
                    ),
                };
                let mut prelude = target.prelude;
                prelude.extend(value.prelude);
                Ok(Snippet {
                    text: format!("{} {spelling} {}", target.text, value.text),
                    precedence: 0,
                    prelude,
                })
            }
            ExprKind::Cast(value) => {
                if let ExprKind::Call {
                    callee: Callee::Math(function),
                    args,
                } = &value.kind
                {
                    if mapping::math(*function) == Some("") && expr.ty == Type::F32 {
                        return self.call(value, &Callee::Math(*function), args);
                    }
                }
                let value = self.snippet(value)?;
                Ok(Snippet {
                    text: format!("{}({})", ty, value.text),
                    precedence: 10,
                    prelude: value.prelude,
                })
            }
            ExprKind::Field { obj, name } => {
                if let Some(binding) = self.binding_ref(expr) {
                    return Ok(Snippet::atom(binding.name));
                }
                if matches!(&obj.kind, ExprKind::Local(name) if name == &self.invocation_param) {
                    let builtin = match (self.invocation_kind, name.as_str()) {
                        (InvocationKind::Compute, "globalId") => "globalId",
                        (InvocationKind::Compute, "localId") => "localId",
                        (InvocationKind::Compute, "workgroupId") => "workgroupId",
                        (InvocationKind::Compute, "numWorkgroups") => "numWorkgroups",
                        (InvocationKind::Compute, "localIndex") => "localIndex",
                        (InvocationKind::Vertex, "vertexIndex") => "vertexIndex",
                        (InvocationKind::Vertex, "instanceIndex") => "instanceIndex",
                        (InvocationKind::Fragment, "frontFacing") => "frontFacing",
                        _ => {
                            return Err(generator_diagnostic(
                                format!("unknown invocation field `{name}`"),
                                expr.pos.clone(),
                            ))
                        }
                    };
                    self.used_builtins.insert(builtin.to_owned());
                    return Ok(Snippet::atom(builtin.to_owned()));
                }
                if obj.ty == Type::F16 || expr.ty == Type::F16 {
                    return Err(diagnostic(
                        "K4",
                        "f16 field access is not admitted in a kernel",
                        expr.pos.clone(),
                    ));
                }
                let object = self.snippet(obj)?;
                let object_text = if object.precedence < 10 {
                    format!("({})", object.text)
                } else {
                    object.text
                };
                Ok(Snippet {
                    text: format!("{object_text}.{}", mapping::ident(name)),
                    precedence: 10,
                    prelude: object.prelude,
                })
            }
            ExprKind::Index { obj, index, .. } => {
                let object = self.snippet(obj)?;
                let index = self.snippet(index)?;
                let object_text = if object.precedence < 10 {
                    format!("({})", object.text)
                } else {
                    object.text.clone()
                };
                let mut prelude = object.prelude;
                prelude.extend(index.prelude);
                Ok(Snippet {
                    text: format!("{object_text}[{}]", index.text),
                    precedence: 10,
                    prelude,
                })
            }
            ExprKind::Call { callee, args } => self.call(expr, callee, args),
            ExprKind::New { class, args } => {
                let class = crate::class(self.module, class.0, "kernel::snippet", &expr.pos)?;
                if !class.is_value {
                    return Err(diagnostic(
                        "K5",
                        format!("new reference class `{}` in kernel", class.name),
                        expr.pos.clone(),
                    ));
                }
                let args = args
                    .iter()
                    .map(|arg| self.snippet(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                let mut prelude = Vec::new();
                let texts = args
                    .into_iter()
                    .map(|arg| {
                        prelude.extend(arg.prelude);
                        arg.text
                    })
                    .collect::<Vec<_>>();
                Ok(Snippet {
                    text: format!("{}({})", ty, texts.join(", ")),
                    precedence: 10,
                    prelude,
                })
            }
            ExprKind::Cond { cond, then, els } => {
                let cond = self.snippet(cond)?;
                let then = self.snippet(then)?;
                let els = self.snippet(els)?;
                // The conditional lowers to a `var` and an `if`/`else` in the prelude, which the
                // caller writes where the expression is evaluated. Both sides then keep
                // short-circuit evaluation (K9). The `_g_` prefix mangles, so no author name
                // collides with the placeholder (K14).
                let result = format!("_g_conditional_{}", self.conditional_index);
                self.conditional_index += 1;
                let mut prelude = cond.prelude;
                prelude.push((0, format!("var {result}: {ty};")));
                prelude.push((0, format!("if ({}) {{", cond.text)));
                prelude.extend(
                    then.prelude
                        .into_iter()
                        .map(|(relative, text)| (relative + 1, text)),
                );
                prelude.push((1, format!("{result} = {};", then.text)));
                prelude.push((0, "} else {".to_owned()));
                prelude.extend(
                    els.prelude
                        .into_iter()
                        .map(|(relative, text)| (relative + 1, text)),
                );
                prelude.push((1, format!("{result} = {};", els.text)));
                prelude.push((0, "}".to_owned()));
                Ok(Snippet {
                    text: result,
                    precedence: 10,
                    prelude,
                })
            }
            ExprKind::Template(_) => Err(diagnostic(
                "K9",
                "template string in kernel",
                expr.pos.clone(),
            )),
            ExprKind::Lambda { .. } => {
                Err(diagnostic("K9", "a lambda is outside K9", expr.pos.clone()))
            }
            ExprKind::AsyncSuspend | ExprKind::AsyncCall { .. } => {
                Err(diagnostic("K9", "await is outside K9", expr.pos.clone()))
            }
            ExprKind::Length(_) => Err(diagnostic(
                "K9",
                "Length of a T[] is outside the kernel subset",
                expr.pos.clone(),
            )),
            ExprKind::ArrayLit(_) => Err(diagnostic(
                "K9",
                "an array literal is outside K9",
                expr.pos.clone(),
            )),
            _ => Err(diagnostic(
                "K9",
                "this expression construct is outside K9",
                expr.pos.clone(),
            )),
        }
    }

    /// Emits one call (K10, K11, K21, PI6, TX3).
    ///
    /// The arms run in check order. An atomic receiver comes first, then a private or workgroup
    /// variable, then a binding, then a library method through the K10 table. A helper call falls
    /// through to the plain call form.
    ///
    /// # Errors
    ///
    /// A `Math` member outside K11 gives a K11 diagnostic. A barrier in an expression gives a K22
    /// diagnostic. An atomic method outside K21 gives a K21 diagnostic. A variable method that the
    /// wrapper does not carry gives a K20 diagnostic. A texture method the access forbids gives a
    /// TX3 or a TX11 diagnostic. A method with no table row gives a K10 diagnostic.
    fn call(&mut self, expr: &Expr, callee: &Callee, args: &[Expr]) -> Result<Snippet, Diagnostic> {
        match callee {
            Callee::Math(function) => {
                let Some(name) = mapping::math(*function) else {
                    return Err(diagnostic(
                        "K11",
                        format!("Math.{} is outside K11", function.name()),
                        expr.pos.clone(),
                    ));
                };
                if name.is_empty() {
                    if args.len() != 1 {
                        return Err(diagnostic(
                            "K11",
                            "Math.fround requires one argument",
                            expr.pos.clone(),
                        ));
                    }
                    return self.fround_argument(args.first().ok_or_else(|| {
                        crate::internal("kernel::call", "missing fround argument", &expr.pos)
                    })?);
                }
                let mut texts = Vec::with_capacity(args.len());
                let mut prelude = Vec::new();
                for arg in args {
                    let value = match (&arg.kind, &arg.ty) {
                        // JavaScript Math arguments are typed as f64 by the checker; K11 maps
                        // those literal arguments to WGSL f32 without admitting f64 elsewhere.
                        (ExprKind::Float(value), Type::F64) => Snippet::atom(f32_literal(*value)),
                        _ => self.snippet(arg)?,
                    };
                    prelude.extend(value.prelude);
                    texts.push(value.text);
                }
                Ok(Snippet {
                    text: format!("{name}({})", texts.join(", ")),
                    precedence: 10,
                    prelude,
                })
            }
            Callee::Func(name) => {
                let base = crate::base_name(name);
                if matches!(base, "workgroupBarrier" | "storageBarrier")
                    && function_declared_in(self.module, name, "typegpu.ts")
                {
                    return Err(diagnostic(
                        "K22",
                        format!("`{base}` is legal only as a statement"),
                        expr.pos.clone(),
                    ));
                }
                let is_library = function(self.module, name)
                    .is_some_and(|function| function.pos.file == "typegpu-types.ts");
                let mapped = is_library.then(|| mapping::free_function(name)).flatten();
                let called = mapped.unwrap_or_else(|| crate::base_name(name));
                let called = if mapped.is_some() {
                    called.to_owned()
                } else {
                    mapping::ident(called)
                };
                let (args, prelude) = self.snippets(args)?;
                Ok(Snippet {
                    text: format!("{called}({})", args.join(", ")),
                    precedence: 10,
                    prelude,
                })
            }
            Callee::Method { recv, name } => {
                if atomic_scalar(self.module, &recv.ty, &expr.pos)?.is_some() {
                    let place = self.atomic_place(recv)?;
                    let (args, args_prelude) = self.snippets(args)?;
                    let mut prelude = place.prelude;
                    prelude.extend(args_prelude);
                    let receiver = class_name(self.module, &recv.ty).ok_or_else(|| {
                        crate::internal(
                            "kernel::call",
                            "atomic scalar receiver has no class name",
                            &recv.pos,
                        )
                    })?;
                    let builtin = match mapping::method(receiver, name) {
                        Some(MethodEmission::Atomic(builtin)) => builtin,
                        _ => {
                            return Err(diagnostic(
                                "K21",
                                format!("atomic method `{name}` is outside K21"),
                                expr.pos.clone(),
                            ))
                        }
                    };
                    let text = match args.as_slice() {
                        [] if name == "load" => format!("{builtin}(&{})", place.text),
                        [value] if name != "load" => {
                            format!("{builtin}(&{}, {value})", place.text)
                        }
                        _ => {
                            return Err(diagnostic(
                                "K21",
                                format!("atomic method `{name}` has an invalid arity"),
                                expr.pos.clone(),
                            ))
                        }
                    };
                    return Ok(Snippet {
                        text,
                        precedence: 10,
                        prelude,
                    });
                }
                if let Some(global) = self.wrapper_ref(recv) {
                    let (args, prelude) = self.snippets(args)?;
                    let target = mapping::ident(&global.name);
                    let text = match (&global.kind, name.as_str(), args.as_slice()) {
                        (KernelGlobalKind::Private(_), "$", [])
                        | (KernelGlobalKind::WorkgroupVar, "$", []) => target,
                        (KernelGlobalKind::Private(_), "$=", [value])
                        | (KernelGlobalKind::WorkgroupVar, "$=", [value]) => {
                            if type_contains_atomic(self.module, &global.ty, &expr.pos)? {
                                return Err(diagnostic(
                                    "K21",
                                    "an atomic value or schema cannot be written as a whole",
                                    expr.pos.clone(),
                                ));
                            }
                            format!("{target} = {value}")
                        }
                        (KernelGlobalKind::WorkgroupArray(_), "get", [index]) => {
                            format!("{target}[{index}]")
                        }
                        (KernelGlobalKind::WorkgroupArray(_), "set", [index, value]) => {
                            if type_contains_atomic(self.module, &global.ty, &expr.pos)? {
                                return Err(diagnostic(
                                    "K21",
                                    "an atomic value or schema cannot be written as a whole",
                                    expr.pos.clone(),
                                ));
                            }
                            format!("{target}[{index}] = {value}")
                        }
                        (KernelGlobalKind::WorkgroupArray(length), "length", []) => {
                            format!("{length}u")
                        }
                        _ => {
                            return Err(diagnostic(
                                "K20",
                                format!("variable method `{name}` is not valid for this wrapper"),
                                expr.pos.clone(),
                            ))
                        }
                    };
                    let precedence = if matches!(name.as_str(), "$=" | "set") {
                        0
                    } else {
                        10
                    };
                    return Ok(Snippet {
                        text,
                        precedence,
                        prelude,
                    });
                }
                if let Some(binding) = self.binding_ref(recv) {
                    let (args, prelude) = self.snippets(args)?;
                    let text = match (binding.kind, name.as_str(), args.as_slice()) {
                        (BindingKind::Uniform, "$", []) => binding.name,
                        (BindingKind::Storage | BindingKind::MutStorage, "get", [index]) => {
                            format!("{}[{index}]", binding.name)
                        }
                        (BindingKind::MutStorage, "set", [index, value]) => {
                            if type_contains_atomic(self.module, &binding.item_ty, &expr.pos)? {
                                return Err(diagnostic(
                                    "K21",
                                    "an atomic schema cannot be written as a whole",
                                    expr.pos.clone(),
                                ));
                            }
                            format!("{}[{index}] = {value}", binding.name)
                        }
                        (BindingKind::Storage | BindingKind::MutStorage, "length", []) => {
                            format!("arrayLength(&{})", binding.name)
                        }
                        (BindingKind::Texture(_, _), "dimensions", []) => {
                            format!("textureDimensions({})", binding.name)
                        }
                        (
                            BindingKind::Texture(_, TextureViewDimension::TwoD),
                            "load",
                            [coords, level],
                        ) => {
                            format!("textureLoad({}, {coords}, {level})", binding.name)
                        }
                        (
                            BindingKind::Texture(_, TextureViewDimension::TwoDArray),
                            "load",
                            [coords, layer, level],
                        ) => {
                            format!("textureLoad({}, {coords}, {layer}, {level})", binding.name)
                        }
                        (
                            BindingKind::Texture(_, TextureViewDimension::TwoD),
                            "sampleLevel",
                            [sampler, uv, level],
                        ) => {
                            format!(
                                "textureSampleLevel({}, {sampler}, {uv}, {level})",
                                binding.name
                            )
                        }
                        (
                            BindingKind::Texture(_, TextureViewDimension::TwoD),
                            "sample",
                            [sampler, uv],
                        ) => {
                            if self.invocation_kind != InvocationKind::Fragment {
                                return Err(diagnostic(
                                    "TX3",
                                    "Texture2d.sample is legal only in a fragment kernel",
                                    expr.pos.clone(),
                                ));
                            }
                            format!("textureSample({}, {sampler}, {uv})", binding.name)
                        }
                        (
                            BindingKind::Texture(_, TextureViewDimension::TwoDArray),
                            "sample" | "sampleLevel",
                            _,
                        ) => {
                            return Err(diagnostic(
                                "TX3",
                                "sampling is not legal on an array sampled texture",
                                expr.pos.clone(),
                            ));
                        }
                        (BindingKind::Texture(_, _), "store", _) => {
                            return Err(diagnostic(
                                "TX3",
                                "store is not legal on a sampled texture",
                                expr.pos.clone(),
                            ));
                        }
                        (BindingKind::StorageTexture(_, access, _), "dimensions", [])
                            if access.can_read() =>
                        {
                            format!("textureDimensions({})", binding.name)
                        }
                        (
                            BindingKind::StorageTexture(_, access, TextureViewDimension::TwoD),
                            "load",
                            [coords],
                        ) if access.can_read() => {
                            format!("textureLoad({}, {coords})", binding.name)
                        }
                        (
                            BindingKind::StorageTexture(_, access, TextureViewDimension::TwoDArray),
                            "load",
                            [coords, layer],
                        ) if access.can_read() => {
                            format!("textureLoad({}, {coords}, {layer})", binding.name)
                        }
                        (
                            BindingKind::StorageTexture(_, access, TextureViewDimension::TwoD),
                            "store",
                            [coords, value],
                        ) if access.can_write() => {
                            format!("textureStore({}, {coords}, {value})", binding.name)
                        }
                        (
                            BindingKind::StorageTexture(_, access, TextureViewDimension::TwoDArray),
                            "store",
                            [coords, layer, value],
                        ) if access.can_write() => {
                            format!("textureStore({}, {coords}, {layer}, {value})", binding.name)
                        }
                        (
                            BindingKind::StorageTexture(
                                _,
                                StorageTextureAccess::Write,
                                TextureViewDimension::TwoD,
                            ),
                            "load",
                            _,
                        ) => {
                            return Err(diagnostic(
                                "TX11",
                                "load is not legal on a write-only storage texture",
                                expr.pos.clone(),
                            ));
                        }
                        (
                            BindingKind::StorageTexture(
                                _,
                                StorageTextureAccess::Write,
                                TextureViewDimension::TwoDArray,
                            ),
                            "load",
                            _,
                        ) => {
                            return Err(diagnostic(
                                "TX13",
                                "load is not legal on a write-only array storage texture",
                                expr.pos.clone(),
                            ));
                        }
                        (
                            BindingKind::StorageTexture(
                                _,
                                StorageTextureAccess::Read,
                                TextureViewDimension::TwoDArray,
                            ),
                            "store",
                            _,
                        ) => {
                            return Err(diagnostic(
                                "TX13",
                                "store is not legal on a read-only array storage texture",
                                expr.pos.clone(),
                            ));
                        }
                        _ => {
                            return Err(generator_diagnostic(
                                format!("binding method `{name}` is not valid for this wrapper"),
                                expr.pos.clone(),
                            ))
                        }
                    };
                    let precedence = if matches!(name.as_str(), "set" | "store") {
                        0
                    } else {
                        10
                    };
                    return Ok(Snippet {
                        text,
                        precedence,
                        prelude,
                    });
                }
                let recv_value = self.snippet(recv)?;
                let receiver = class_name(self.module, &recv.ty).ok_or_else(|| {
                    diagnostic(
                        "K10",
                        "a method receiver is not a library class",
                        expr.pos.clone(),
                    )
                })?;
                let library = matches!(&recv.ty, Type::Class(id) if crate::class(self.module, id.0, "kernel::call", &expr.pos)?.pos.file == "typegpu-types.ts");
                if !library {
                    return Err(diagnostic(
                        "K10",
                        format!("method `{receiver}.{name}` is outside K10"),
                        expr.pos.clone(),
                    ));
                }
                let Some(emission) = mapping::method(receiver, name) else {
                    return Err(diagnostic(
                        "K10",
                        format!("method `{receiver}.{name}` is outside K10"),
                        expr.pos.clone(),
                    ));
                };
                let arg_values = args
                    .iter()
                    .map(|arg| self.snippet(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                let (text, precedence) = match emission {
                    MethodEmission::Binary(op) if arg_values.len() == 1 => {
                        let precedence = match op {
                            "+" | "-" => 7,
                            "*" => 8,
                            "<" | "<=" | ">" | ">=" | "==" | "!=" => 5,
                            _ => 0,
                        };
                        let recv = binary_operand(&recv_value, precedence, false, false);
                        let arg = binary_operand(
                            arg_values.first().ok_or_else(|| {
                                crate::internal("kernel::call", "missing binary operand", &expr.pos)
                            })?,
                            precedence,
                            true,
                            false,
                        );
                        (format!("{recv} {op} {arg}"), precedence)
                    }
                    MethodEmission::Builtin(builtin) if arg_values.is_empty() => {
                        (format!("{builtin}({})", recv_value.text), 10)
                    }
                    MethodEmission::Builtin(builtin) => {
                        let args = arg_values
                            .iter()
                            .map(|value| value.text.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        (format!("{builtin}({}, {args})", recv_value.text), 10)
                    }
                    MethodEmission::BuiltinReceiverLast(builtin) => {
                        let args = arg_values
                            .iter()
                            .map(|value| value.text.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        (format!("{builtin}({args}, {})", recv_value.text), 10)
                    }
                    MethodEmission::Unary(op) if arg_values.is_empty() => {
                        let recv = if recv_value.precedence <= 9 {
                            format!("({})", recv_value.text)
                        } else {
                            recv_value.text.clone()
                        };
                        (format!("{op}{recv}"), 9)
                    }
                    MethodEmission::Swizzle(fields) if arg_values.is_empty() => {
                        let recv = if recv_value.precedence < 10 {
                            format!("({})", recv_value.text)
                        } else {
                            recv_value.text.clone()
                        };
                        (format!("{recv}.{fields}"), 10)
                    }
                    _ => {
                        return Err(diagnostic(
                            "K10",
                            format!("wrong arity for `{receiver}.{name}`"),
                            expr.pos.clone(),
                        ))
                    }
                };
                let mut prelude = recv_value.prelude;
                for arg in arg_values {
                    prelude.extend(arg.prelude);
                }
                Ok(Snippet {
                    text,
                    precedence,
                    prelude,
                })
            }
            Callee::Value(_) => Err(diagnostic(
                "K9",
                "function value or lambda call in kernel",
                expr.pos.clone(),
            )),
            _ => Err(diagnostic(
                "K9",
                "call target is outside the kernel subset",
                expr.pos.clone(),
            )),
        }
    }

    /// Emits a statement list at `indent`, in order.
    ///
    /// # Errors
    ///
    /// Returns the first diagnostic that a statement gives.
    fn statements(
        &mut self,
        statements: &[Stmt],
        indent: usize,
        out: &mut String,
    ) -> Result<(), Diagnostic> {
        for statement in statements {
            self.statement(statement, indent, out)?;
        }
        Ok(())
    }

    /// Appends one line with two-space indentation and no trailing space (K14).
    fn line(out: &mut String, indent: usize, text: &str) {
        out.push_str(&"  ".repeat(indent));
        out.push_str(text);
        out.push('\n');
    }

    /// Appends a prelude's lines, each at `indent` plus its own relative indent.
    fn emit_prelude(out: &mut String, indent: usize, prelude: Prelude) {
        for (relative, text) in prelude {
            Self::line(out, indent + relative, &text);
        }
    }

    /// Emits one statement at `indent` (K7, K8, K18).
    ///
    /// A `let` binding emits a WGSL `let`, and a mutable binding or a value-class local emits a
    /// `var`, because the kernel assigns its fields (K8).
    ///
    /// # Errors
    ///
    /// A `using` declaration gives a K5 diagnostic. A layout class as a local gives a PI6
    /// diagnostic. A copy of a schema that holds an atomic gives a K21 diagnostic. A barrier in a
    /// helper gives a K22 diagnostic. `switch`, `break`, and `continue` give K18 diagnostics.
    fn statement(
        &mut self,
        statement: &Stmt,
        indent: usize,
        out: &mut String,
    ) -> Result<(), Diagnostic> {
        match statement {
            Stmt::Let {
                name,
                ty,
                mutable,
                init,
                dispose,
                pos,
            } => {
                if *dispose {
                    return Err(diagnostic(
                        "K5",
                        "`using` declaration in kernel",
                        pos.clone(),
                    ));
                }
                if class_name(self.module, ty)
                    .is_some_and(|class| self.layout_names.contains(class))
                {
                    return Err(diagnostic(
                        "PI6",
                        "a layout class is used as a kernel local",
                        pos.clone(),
                    ));
                }
                if atomic_scalar(self.module, ty, pos)?.is_none()
                    && type_contains_atomic(self.module, ty, pos)?
                {
                    return Err(diagnostic(
                        "K21",
                        "a schema that contains an atomic cannot be copied to a local",
                        pos.clone(),
                    ));
                }
                let value = self.snippet(init)?;
                let _ = wgsl_type(self.module, ty, pos)?;
                Self::emit_prelude(out, indent, value.prelude);
                let value_class = matches!(ty, Type::Class(id) if crate::class(self.module, id.0, "kernel::statement", pos)?.is_value);
                let declaration = if *mutable || value_class {
                    "var"
                } else {
                    "let"
                };
                Self::line(
                    out,
                    indent,
                    &format!("{declaration} {} = {};", self.local_name(name), value.text),
                );
            }
            Stmt::Expr(expr) => {
                if let Some(barrier) = barrier_call(self.module, expr) {
                    if self.in_helper {
                        return Err(diagnostic(
                            "K22",
                            format!("`{barrier}` is not legal in a helper"),
                            expr.pos.clone(),
                        ));
                    }
                    Self::line(out, indent, &format!("{barrier}();"));
                    return Ok(());
                }
                let value = self.snippet(expr)?;
                Self::emit_prelude(out, indent, value.prelude);
                Self::line(out, indent, &format!("{};", value.text));
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    let value = self.snippet(value)?;
                    Self::emit_prelude(out, indent, value.prelude);
                    Self::line(out, indent, &format!("return {};", value.text));
                } else {
                    Self::line(out, indent, "return;");
                }
            }
            Stmt::If {
                cond, then, els, ..
            } => {
                let cond = self.snippet(cond)?;
                Self::emit_prelude(out, indent, cond.prelude);
                Self::line(out, indent, &format!("if ({}) {{", cond.text));
                self.statements(then, indent + 1, out)?;
                if let Some(els) = els {
                    Self::line(out, indent, "} else {");
                    self.statements(els, indent + 1, out)?;
                }
                Self::line(out, indent, "}");
            }
            Stmt::While { cond, body, .. } => {
                // A condition with a prelude must run its statements on every iteration, and a
                // WGSL `while` header holds no statements. The `loop` form with an explicit
                // `break` gives the same semantics.
                let cond = self.snippet(cond)?;
                if cond.prelude.is_empty() {
                    Self::line(out, indent, &format!("while ({}) {{", cond.text));
                    self.loop_depth += 1;
                    let result = self.statements(body, indent + 1, out);
                    self.loop_depth -= 1;
                    result?;
                    Self::line(out, indent, "}");
                } else {
                    Self::line(out, indent, "loop {");
                    Self::emit_prelude(out, indent + 1, cond.prelude);
                    Self::line(out, indent + 1, &format!("if (!({})) {{", cond.text));
                    Self::line(out, indent + 2, "break;");
                    Self::line(out, indent + 1, "}");
                    self.loop_depth += 1;
                    let result = self.statements(body, indent + 1, out);
                    self.loop_depth -= 1;
                    result?;
                    Self::line(out, indent, "}");
                }
            }
            Stmt::For {
                init,
                cond,
                step,
                body,
                pos,
            } => {
                let (init, init_prelude) = match init.as_deref() {
                    Some(Stmt::Let {
                        name,
                        ty,
                        mutable,
                        init,
                        dispose,
                        pos,
                    }) => {
                        if *dispose {
                            return Err(diagnostic(
                                "K5",
                                "`using` declaration in kernel",
                                pos.clone(),
                            ));
                        }
                        if class_name(self.module, ty)
                            .is_some_and(|class| self.layout_names.contains(class))
                        {
                            return Err(diagnostic(
                                "PI6",
                                "a layout class is used as a for-loop local",
                                pos.clone(),
                            ));
                        }
                        let value = self.snippet(init)?;
                        let value_class = matches!(ty, Type::Class(id) if crate::class(self.module, id.0, "kernel::statement", pos)?.is_value);
                        let declaration = if *mutable || value_class {
                            "var"
                        } else {
                            "let"
                        };
                        let _ = wgsl_type(self.module, ty, pos)?;
                        (
                            format!("{} {} = {}", declaration, self.local_name(name), value.text),
                            value.prelude,
                        )
                    }
                    Some(Stmt::Expr(expr)) => {
                        let value = self.snippet(expr)?;
                        (value.text, value.prelude)
                    }
                    None => (String::new(), Vec::new()),
                    _ => {
                        return Err(diagnostic(
                            "K7",
                            "unsupported for-loop initializer",
                            pos.clone(),
                        ))
                    }
                };
                let cond = cond.as_ref().map(|value| self.snippet(value)).transpose()?;
                let step = step.as_ref().map(|value| self.snippet(value)).transpose()?;
                // A `for` header holds no statements either, so a condition or a step with a
                // prelude takes the `loop` form. The outer block scopes the initializer.
                let loop_prelude = cond.as_ref().is_some_and(|value| !value.prelude.is_empty())
                    || step.as_ref().is_some_and(|value| !value.prelude.is_empty());
                if loop_prelude {
                    Self::line(out, indent, "{");
                    Self::emit_prelude(out, indent + 1, init_prelude);
                    if !init.is_empty() {
                        Self::line(out, indent + 1, &format!("{init};"));
                    }
                    Self::line(out, indent + 1, "loop {");
                    if let Some(cond) = cond {
                        Self::emit_prelude(out, indent + 2, cond.prelude);
                        Self::line(out, indent + 2, &format!("if (!({})) {{", cond.text));
                        Self::line(out, indent + 3, "break;");
                        Self::line(out, indent + 2, "}");
                    }
                    self.loop_depth += 1;
                    let result = self.statements(body, indent + 2, out);
                    self.loop_depth -= 1;
                    result?;
                    if let Some(step) = step {
                        Self::emit_prelude(out, indent + 2, step.prelude);
                        Self::line(out, indent + 2, &format!("{};", step.text));
                    }
                    Self::line(out, indent + 1, "}");
                    Self::line(out, indent, "}");
                } else {
                    Self::emit_prelude(out, indent, init_prelude);
                    let cond = cond.map_or_else(String::new, |value| value.text);
                    let step = step.map_or_else(String::new, |value| value.text);
                    Self::line(out, indent, &format!("for ({init}; {cond}; {step}) {{"));
                    self.loop_depth += 1;
                    let result = self.statements(body, indent + 1, out);
                    self.loop_depth -= 1;
                    result?;
                    Self::line(out, indent, "}");
                }
            }
            Stmt::ForOf {
                name,
                ty,
                subject,
                kind: ForOfKind::FixedArrayValues,
                body,
                pos,
            } => {
                let Type::FixedArray(_, length) = subject.ty else {
                    return Err(diagnostic(
                        "K7",
                        "for...of subject is not a FixedArray",
                        pos.clone(),
                    ));
                };
                let subject = self.snippet(subject)?;
                Self::emit_prelude(out, indent, subject.prelude);
                let index = format!("_g_{}_index", self.local_name(name));
                Self::line(
                    out,
                    indent,
                    &format!(
                        "for (var {index} = 0u; {index} < {length}u; {index} = {index} + 1u) {{"
                    ),
                );
                Self::line(
                    out,
                    indent + 1,
                    &format!("let {} = {}[{index}];", self.local_name(name), subject.text),
                );
                let _ = wgsl_type(self.module, ty, pos)?;
                self.loop_depth += 1;
                let result = self.statements(body, indent + 1, out);
                self.loop_depth -= 1;
                result?;
                Self::line(out, indent, "}");
            }
            Stmt::Switch { disc, cases, pos } => {
                if !matches!(disc.ty, Type::I32 | Type::U32) {
                    return Err(diagnostic(
                        "K18",
                        "switch discriminant is not i32 or u32",
                        disc.pos.clone(),
                    ));
                }
                if !cases.iter().any(|case| case.test.is_none()) {
                    return Err(diagnostic("K18", "switch has no default case", pos.clone()));
                }
                let disc = self.snippet(disc)?;
                Self::emit_prelude(out, indent, disc.prelude);
                Self::line(out, indent, &format!("switch ({}) {{", disc.text));
                // An empty case shares the next case's body, which WGSL writes as one selector
                // with several values. The labels accumulate until a body arrives (K18).
                let mut labels = Vec::new();
                for case in cases {
                    let label = if let Some(test) = &case.test {
                        let value = self.snippet(test)?;
                        if !value.prelude.is_empty() {
                            return Err(diagnostic(
                                "K18",
                                "switch case label is not constant",
                                test.pos.clone(),
                            ));
                        }
                        value.text
                    } else {
                        "default".to_owned()
                    };
                    labels.push(label);
                    if case.body.is_empty() {
                        continue;
                    }
                    if !case.body.is_empty() && !case_terminates(&case.body) {
                        return Err(diagnostic(
                            "K18",
                            "switch case falls through with statements",
                            case.pos.clone(),
                        ));
                    }
                    let selector = if matches!(labels.as_slice(), [label] if label == "default") {
                        "default".to_owned()
                    } else {
                        let mut ordered = labels
                            .iter()
                            .filter(|label| label.as_str() != "default")
                            .cloned()
                            .collect::<Vec<_>>();
                        if labels.iter().any(|label| label == "default") {
                            ordered.push("default".to_owned());
                        }
                        format!("case {}", ordered.join(", "))
                    };
                    Self::line(out, indent + 1, &format!("{selector}: {{"));
                    self.switch_depth += 1;
                    let result = self.statements(&case.body, indent + 2, out);
                    self.switch_depth -= 1;
                    result?;
                    Self::line(out, indent + 1, "}");
                    labels.clear();
                }
                // Labels remain when the last case is empty, so no body follows to share.
                if !labels.is_empty() {
                    return Err(diagnostic(
                        "K18",
                        "an empty switch case has no following body",
                        pos.clone(),
                    ));
                }
                Self::line(out, indent, "}");
            }
            Stmt::Block(body) => {
                Self::line(out, indent, "{");
                self.statements(body, indent + 1, out)?;
                Self::line(out, indent, "}");
            }
            Stmt::ForOf { pos, .. } => {
                return Err(diagnostic(
                    "K7",
                    "statement is outside the current kernel subset",
                    pos.clone(),
                ));
            }
            Stmt::Break(pos) => {
                if self.loop_depth == 0 && self.switch_depth == 0 {
                    return Err(diagnostic(
                        "K18",
                        "break is outside a loop or switch",
                        pos.clone(),
                    ));
                }
                Self::line(out, indent, "break;");
            }
            Stmt::Continue(pos) => {
                if self.loop_depth == 0 {
                    return Err(diagnostic("K18", "continue is outside a loop", pos.clone()));
                }
                Self::line(out, indent, "continue;");
            }
        }
        Ok(())
    }
}

/// Whether a value is uniform across the workgroup, and the reason when it is not (K22).
#[derive(Debug, Clone, PartialEq, Eq)]
enum UniformityTaint {
    Uniform,
    /// The reason, which the diagnostic quotes so that the author finds the value.
    NonUniform(String),
}

impl UniformityTaint {
    /// Combines two taints. One non-uniform side makes the result non-uniform and keeps its
    /// reason.
    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::NonUniform(reason), _) | (_, Self::NonUniform(reason)) => {
                Self::NonUniform(reason)
            }
            (Self::Uniform, Self::Uniform) => Self::Uniform,
        }
    }

    /// Returns the non-uniform reason, and `None` when the taint is uniform.
    fn reason(&self) -> Option<&str> {
        match self {
            Self::Uniform => None,
            Self::NonUniform(reason) => Some(reason),
        }
    }
}

/// One enclosing statement that a `break` or a `continue` can target (K18, K22).
#[derive(Debug, Clone, Copy)]
enum UniformityTarget {
    /// A loop, and whether its body holds a barrier.
    Loop { has_barrier: bool },
    /// A switch, which `break` targets and `continue` passes through.
    Switch,
}

/// Checks barrier placement in one kernel body (K22).
///
/// The analysis is a conservative taint: it rejects some uniform programs and accepts no
/// non-uniform barrier. `naga` did not reject a barrier after a non-uniform early return, and
/// both backends refuse such a module at shader-module creation.
struct BarrierValidator<'emitter, 'module> {
    /// The emitter of the same kernel, which resolves bindings, globals, and the invocation.
    emitter: &'emitter Emitter<'module>,
    /// The taint of each local, merged over every assignment the collect pass found.
    locals: BTreeMap<String, UniformityTaint>,
    /// The line and column of the last barrier in the body, which bounds the `return` check.
    last_barrier: Option<(u32, u32)>,
}

impl<'emitter, 'module> BarrierValidator<'emitter, 'module> {
    /// Builds the validator of one kernel body.
    fn new(emitter: &'emitter Emitter<'module>, kernel: &Function) -> Self {
        Self {
            emitter,
            locals: BTreeMap::new(),
            last_barrier: last_barrier_position(emitter.module, &kernel.body),
        }
    }

    /// Runs the two passes over one kernel body and returns the first violation (K22).
    ///
    /// The collect pass must precede the check pass: a local's taint depends on every assignment
    /// in the body. An assignment that follows the barrier in source order counts too.
    ///
    /// # Errors
    ///
    /// Returns a K22 diagnostic that names the statement and the non-uniform value.
    fn validate(mut self, kernel: &Function) -> Result<(), Diagnostic> {
        self.collect_statements(&kernel.body, UniformityTaint::Uniform);
        self.validate_statements(
            &kernel.body,
            UniformityTaint::Uniform,
            true,
            &mut Vec::new(),
        )
    }

    /// Returns the taint of one expression (K22).
    ///
    /// A literal, a module constant, a `Uniform<T>` read, and `length()` of a binding are uniform.
    /// A builtin, a storage read, a variable read, and a helper result are non-uniform. A local
    /// takes the taint the collect pass recorded, and an unknown local is non-uniform.
    fn expression(&self, expr: &Expr) -> UniformityTaint {
        match &expr.kind {
            ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_) => UniformityTaint::Uniform,
            ExprKind::Local(name) => self.locals.get(name).cloned().unwrap_or_else(|| {
                UniformityTaint::NonUniform(format!("local or parameter `{name}`"))
            }),
            ExprKind::Global(name) => match self.emitter.globals.get(name) {
                Some(KernelGlobal {
                    kind: KernelGlobalKind::Constant(_),
                    ..
                }) => UniformityTaint::Uniform,
                _ => UniformityTaint::NonUniform(format!("global variable `{name}`")),
            },
            ExprKind::Field { obj, name } if matches!(&obj.kind, ExprKind::Local(param) if param == &self.emitter.invocation_param) => {
                UniformityTaint::NonUniform(format!(
                    "builtin `{}.{name}`",
                    self.emitter.invocation_param
                ))
            }
            ExprKind::AbsenceTest { value: operand, .. }
            | ExprKind::Unary { operand, .. }
            | ExprKind::Cast(operand)
            | ExprKind::JsonResultValue(operand) => self.expression(operand),
            ExprKind::Length(operand) => {
                if self.emitter.binding_root(operand).is_some() {
                    UniformityTaint::Uniform
                } else {
                    self.expression(operand)
                }
            }
            ExprKind::Field { obj, .. } => {
                if let Some(binding) = self.emitter.binding_ref(expr) {
                    UniformityTaint::NonUniform(format!("binding `{}`", binding.name))
                } else {
                    self.expression(obj)
                }
            }
            ExprKind::Binary { left, right, .. } => {
                self.expression(left).merge(self.expression(right))
            }
            ExprKind::Assign { op, target, value } => {
                let value = self.expression(value);
                if op.is_some() {
                    self.expression(target).merge(value)
                } else {
                    assignment_target_taint(self, target).merge(value)
                }
            }
            ExprKind::Call { callee, args } => match callee {
                Callee::Func(name)
                    if function_declared_in(self.emitter.module, name, "typegpu.ts")
                        || function_declared_in(self.emitter.module, name, "typegpu-types.ts") =>
                {
                    self.expressions(args)
                }
                Callee::Func(name) => UniformityTaint::NonUniform(format!(
                    "helper result `{}`",
                    crate::base_name(name)
                )),
                Callee::Method { recv, name }
                    if args.is_empty()
                        && name == "$"
                        && self
                            .emitter
                            .binding_ref(recv)
                            .is_some_and(|binding| binding.kind == BindingKind::Uniform) =>
                {
                    UniformityTaint::Uniform
                }
                Callee::Method { recv, name }
                    if args.is_empty()
                        && name == "length"
                        && self.emitter.binding_root(recv).is_some() =>
                {
                    UniformityTaint::Uniform
                }
                Callee::Method { recv, .. } => {
                    if let Some(binding) = self.emitter.binding_root(recv) {
                        UniformityTaint::NonUniform(format!("binding `{}`", binding.name))
                            .merge(self.expressions(args))
                    } else {
                        self.expression(recv).merge(self.expressions(args))
                    }
                }
                Callee::Math(_) => self.expressions(args),
                Callee::Value(value) => self.expression(value).merge(self.expressions(args)),
                _ => UniformityTaint::NonUniform("call result".to_owned()),
            },
            ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => self.expressions(args),
            ExprKind::DescriptorLit { fields, .. } => {
                self.expressions(&fields.iter().flatten().cloned().collect::<Vec<_>>())
            }
            ExprKind::Index { obj, index, .. } => {
                if let Some(binding) = self.emitter.binding_root(obj) {
                    UniformityTaint::NonUniform(format!("binding `{}`", binding.name))
                        .merge(self.expression(index))
                } else {
                    self.expression(obj).merge(self.expression(index))
                }
            }
            ExprKind::Cond { cond, then, els } => self
                .expression(cond)
                .merge(self.expression(then))
                .merge(self.expression(els)),
            ExprKind::EnumMember { .. } | ExprKind::Zero => UniformityTaint::Uniform,
            _ => UniformityTaint::NonUniform("expression value".to_owned()),
        }
    }

    /// Returns the merged taint of a list of expressions.
    fn expressions(&self, expressions: &[Expr]) -> UniformityTaint {
        expressions
            .iter()
            .fold(UniformityTaint::Uniform, |taint, expr| {
                taint.merge(self.expression(expr))
            })
    }

    /// Merges `value` into the recorded taint of the local that `target` writes.
    ///
    /// A taint never falls back to uniform, so a local assigned once under a non-uniform condition
    /// stays non-uniform for the complete body.
    fn record_assignment(&mut self, target: &Expr, value: UniformityTaint) {
        let Some(name) = assigned_local(target) else {
            return;
        };
        let prior = self
            .locals
            .get(name)
            .cloned()
            .unwrap_or(UniformityTaint::Uniform);
        self.locals.insert(name.to_owned(), prior.merge(value));
    }

    /// Records one assignment expression's effect on its target local.
    ///
    /// `control` is the taint of the enclosing conditions, so an assignment under a non-uniform
    /// branch taints its target.
    fn collect_assignment(&mut self, expr: &Expr, control: UniformityTaint) {
        let ExprKind::Assign { op, target, value } = &expr.kind else {
            return;
        };
        let mut taint = self.expression(value).merge(control);
        if op.is_some() {
            taint = self.expression(target).merge(taint);
        } else {
            taint = assignment_target_taint(self, target).merge(taint);
        }
        self.record_assignment(target, taint);
    }

    /// Returns the taint of every early exit from one loop body (K22).
    ///
    /// `nested_loops` and `switches` count the statements between here and the loop, so only a
    /// `break` or a `continue` that targets this loop counts. A `for...of` body carries its own
    /// non-uniform control. The caller taints every local the loop writes with the result.
    fn loop_exit_taint(
        &self,
        statements: &[Stmt],
        control: UniformityTaint,
        nested_loops: usize,
        switches: usize,
    ) -> UniformityTaint {
        let mut result = UniformityTaint::Uniform;
        for statement in statements {
            let candidate = match statement {
                Stmt::If {
                    cond, then, els, ..
                } => {
                    let branch = control.clone().merge(self.expression(cond));
                    let mut value =
                        self.loop_exit_taint(then, branch.clone(), nested_loops, switches);
                    if let Some(els) = els {
                        value =
                            value.merge(self.loop_exit_taint(els, branch, nested_loops, switches));
                    }
                    value
                }
                Stmt::While { cond, body, .. } => self.loop_exit_taint(
                    body,
                    control.clone().merge(self.expression(cond)),
                    nested_loops + 1,
                    switches,
                ),
                Stmt::For { cond, body, .. } => self.loop_exit_taint(
                    body,
                    control.clone().merge(
                        cond.as_ref()
                            .map_or(UniformityTaint::Uniform, |expr| self.expression(expr)),
                    ),
                    nested_loops + 1,
                    switches,
                ),
                Stmt::ForOf { body, .. } => self.loop_exit_taint(
                    body,
                    UniformityTaint::NonUniform("`for...of` control".to_owned()),
                    nested_loops + 1,
                    switches,
                ),
                Stmt::Switch { disc, cases, .. } => {
                    let branch = control.clone().merge(self.expression(disc));
                    cases.iter().fold(UniformityTaint::Uniform, |value, case| {
                        value.merge(self.loop_exit_taint(
                            &case.body,
                            branch.clone(),
                            nested_loops,
                            switches + 1,
                        ))
                    })
                }
                Stmt::Block(body) => {
                    self.loop_exit_taint(body, control.clone(), nested_loops, switches)
                }
                Stmt::Break(_) if nested_loops == 0 && switches == 0 => control.clone(),
                Stmt::Continue(_) if nested_loops == 0 => control.clone(),
                _ => UniformityTaint::Uniform,
            };
            result = result.merge(candidate);
        }
        result
    }

    /// Taints every local that the loop writes with `taint`, and does nothing when it is uniform.
    ///
    /// A `break` or a `continue` under a non-uniform condition makes every local the loop writes
    /// non-uniform, because the iteration count then differs between invocations (K22).
    fn taint_loop_writes(&mut self, body: &[Stmt], step: Option<&Expr>, taint: UniformityTaint) {
        if taint == UniformityTaint::Uniform {
            return;
        }
        let mut written = BTreeSet::new();
        written_locals(body, &mut written);
        if let Some(step) = step {
            written_locals_expr(step, &mut written);
        }
        for name in written {
            let prior = self
                .locals
                .get(&name)
                .cloned()
                .unwrap_or(UniformityTaint::Uniform);
            self.locals.insert(name, prior.merge(taint.clone()));
        }
    }

    /// Records the taint of every local the statements write (K22).
    ///
    /// `control` is the taint of the enclosing conditions. A loop body repeats until no taint
    /// changes, because a later iteration can taint a local an earlier one read.
    fn collect_statements(&mut self, statements: &[Stmt], control: UniformityTaint) {
        for statement in statements {
            match statement {
                Stmt::Let { name, init, .. } => {
                    let value = self.expression(init).merge(control.clone());
                    let prior = self
                        .locals
                        .get(name)
                        .cloned()
                        .unwrap_or(UniformityTaint::Uniform);
                    self.locals.insert(name.clone(), prior.merge(value));
                }
                Stmt::Expr(expr) => self.collect_assignment(expr, control.clone()),
                Stmt::If {
                    cond, then, els, ..
                } => {
                    let branch = control.clone().merge(self.expression(cond));
                    self.collect_statements(then, branch.clone());
                    if let Some(els) = els {
                        self.collect_statements(els, branch);
                    }
                }
                Stmt::While { cond, body, .. } => loop {
                    let before = self.locals.clone();
                    let body_control = control.clone().merge(self.expression(cond));
                    self.collect_statements(body, body_control.clone());
                    let exit = self.loop_exit_taint(body, body_control, 0, 0);
                    self.taint_loop_writes(body, None, exit);
                    if self.locals == before {
                        break;
                    }
                },
                Stmt::For {
                    init,
                    cond,
                    step,
                    body,
                    ..
                } => {
                    if let Some(init) = init {
                        self.collect_statements(std::slice::from_ref(init), control.clone());
                    }
                    loop {
                        let before = self.locals.clone();
                        let condition = cond
                            .as_ref()
                            .map_or(UniformityTaint::Uniform, |expr| self.expression(expr));
                        let loop_control = control.clone().merge(condition);
                        self.collect_statements(body, loop_control.clone());
                        if let Some(step) = step {
                            self.collect_assignment(step, loop_control.clone());
                        }
                        let exit = self.loop_exit_taint(body, loop_control, 0, 0);
                        self.taint_loop_writes(body, step.as_ref(), exit);
                        if self.locals == before {
                            break;
                        }
                    }
                }
                Stmt::ForOf {
                    name,
                    subject,
                    body,
                    ..
                } => {
                    let subject = self.expression(subject).merge(control.clone());
                    self.locals.insert(name.clone(), subject.clone());
                    self.collect_statements(body, subject);
                }
                Stmt::Switch { disc, cases, .. } => {
                    let branch = control.clone().merge(self.expression(disc));
                    for case in cases {
                        self.collect_statements(&case.body, branch.clone());
                    }
                }
                Stmt::Block(body) => self.collect_statements(body, control.clone()),
                Stmt::Return { .. } | Stmt::Break(_) | Stmt::Continue(_) => {}
            }
        }
    }

    /// Checks every barrier and every early exit against the recorded taints (K22).
    ///
    /// `control` is the taint of the enclosing conditions. `barrier_scope_allowed` is false inside
    /// a `switch` and a `for...of`, where a barrier is illegal whatever the taint. `targets` is
    /// the stack of enclosing loops and switches, so a `break` or a `continue` resolves to the
    /// statement it leaves.
    ///
    /// # Errors
    ///
    /// Returns a K22 diagnostic in three cases. A barrier runs under non-uniform control. A
    /// `return` precedes a barrier under a non-uniform taint. A `break` or a `continue` leaves a
    /// loop that holds a barrier, under non-uniform control.
    fn validate_statements(
        &self,
        statements: &[Stmt],
        control: UniformityTaint,
        barrier_scope_allowed: bool,
        targets: &mut Vec<UniformityTarget>,
    ) -> Result<(), Diagnostic> {
        for statement in statements {
            match statement {
                Stmt::Expr(expr) => {
                    if let Some(barrier) = barrier_call(self.emitter.module, expr) {
                        let reason = if barrier_scope_allowed {
                            control.reason()
                        } else {
                            Some("`switch` or `for...of` control")
                        };
                        if let Some(reason) = reason {
                            return Err(diagnostic(
                                "K22",
                                format!(
                                    "`{barrier}` barrier statement is under non-uniform {reason}"
                                ),
                                expr.pos.clone(),
                            ));
                        }
                    }
                }
                Stmt::Return { value, pos } => {
                    let before_later_barrier = self
                        .last_barrier
                        .is_some_and(|barrier| (pos.line, pos.col) < barrier);
                    let taint = control.clone().merge(
                        value
                            .as_ref()
                            .map_or(UniformityTaint::Uniform, |value| self.expression(value)),
                    );
                    let leaves_loop_with_barrier = taint.reason().is_some()
                        && targets.iter().rev().any(|target| {
                            matches!(target, UniformityTarget::Loop { has_barrier: true })
                        });
                    if before_later_barrier || leaves_loop_with_barrier {
                        let reason = taint.reason().unwrap_or("return path");
                        return Err(diagnostic(
                            "K22",
                            format!(
                                "`return` statement precedes a barrier with non-uniform {reason}"
                            ),
                            pos.clone(),
                        ));
                    }
                }
                Stmt::If {
                    cond, then, els, ..
                } => {
                    let branch = control.clone().merge(self.expression(cond));
                    self.validate_statements(then, branch.clone(), barrier_scope_allowed, targets)?;
                    if let Some(els) = els {
                        self.validate_statements(els, branch, barrier_scope_allowed, targets)?;
                    }
                }
                Stmt::While { cond, body, .. } => {
                    let loop_control = control.clone().merge(self.expression(cond));
                    targets.push(UniformityTarget::Loop {
                        has_barrier: contains_barrier(self.emitter.module, body),
                    });
                    let result = self.validate_statements(
                        body,
                        loop_control,
                        barrier_scope_allowed,
                        targets,
                    );
                    targets.pop();
                    result?;
                }
                Stmt::For { cond, body, .. } => {
                    let condition = cond
                        .as_ref()
                        .map_or(UniformityTaint::Uniform, |expr| self.expression(expr));
                    let loop_control = control.clone().merge(condition);
                    targets.push(UniformityTarget::Loop {
                        has_barrier: contains_barrier(self.emitter.module, body),
                    });
                    let result = self.validate_statements(
                        body,
                        loop_control,
                        barrier_scope_allowed,
                        targets,
                    );
                    targets.pop();
                    result?;
                }
                Stmt::ForOf { body, .. } => {
                    let loop_control = control
                        .clone()
                        .merge(UniformityTaint::NonUniform("`for...of` control".to_owned()));
                    targets.push(UniformityTarget::Loop {
                        has_barrier: contains_barrier(self.emitter.module, body),
                    });
                    let result = self.validate_statements(body, loop_control, false, targets);
                    targets.pop();
                    result?;
                }
                Stmt::Switch { disc, cases, .. } => {
                    let switch_control = control.clone().merge(self.expression(disc));
                    targets.push(UniformityTarget::Switch);
                    for case in cases {
                        self.validate_statements(
                            &case.body,
                            switch_control.clone(),
                            false,
                            targets,
                        )?;
                    }
                    targets.pop();
                }
                Stmt::Block(body) => {
                    self.validate_statements(
                        body,
                        control.clone(),
                        barrier_scope_allowed,
                        targets,
                    )?;
                }
                Stmt::Break(pos) => {
                    if matches!(
                        targets.last(),
                        Some(UniformityTarget::Loop { has_barrier: true })
                    ) {
                        if let Some(reason) = control.reason() {
                            return Err(diagnostic(
                                "K22",
                                format!(
                                    "`break` statement leaves a loop with a barrier under non-uniform {reason}"
                                ),
                                pos.clone(),
                            ));
                        }
                    }
                }
                Stmt::Continue(pos) => {
                    let loop_has_barrier = targets.iter().rev().find_map(|target| match target {
                        UniformityTarget::Loop { has_barrier } => Some(*has_barrier),
                        UniformityTarget::Switch => None,
                    });
                    if loop_has_barrier == Some(true) {
                        if let Some(reason) = control.reason() {
                            return Err(diagnostic(
                                "K22",
                                format!(
                                    "`continue` statement leaves a loop with a barrier under non-uniform {reason}"
                                ),
                                pos.clone(),
                            ));
                        }
                    }
                }
                Stmt::Let { .. } => {}
            }
        }
        Ok(())
    }
}

/// Returns the local name at the root of an assignment target, and `None` for every other target.
fn assigned_local(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Local(name) => Some(name),
        ExprKind::Field { obj, .. } | ExprKind::Index { obj, .. } => assigned_local(obj),
        _ => None,
    }
}

/// Adds the local that `expr` assigns to `out`, when `expr` is an assignment.
fn written_locals_expr(expr: &Expr, out: &mut BTreeSet<String>) {
    if let ExprKind::Assign { target, .. } = &expr.kind {
        if let Some(name) = assigned_local(target) {
            out.insert(name.to_owned());
        }
    }
}

/// Collects every local that the statements assign, nested bodies included.
fn written_locals(statements: &[Stmt], out: &mut BTreeSet<String>) {
    for statement in statements {
        match statement {
            Stmt::Expr(expr) => written_locals_expr(expr, out),
            Stmt::If { then, els, .. } => {
                written_locals(then, out);
                if let Some(els) = els {
                    written_locals(els, out);
                }
            }
            Stmt::While { body, .. } | Stmt::ForOf { body, .. } => {
                written_locals(body, out);
            }
            Stmt::For { step, body, .. } => {
                written_locals(body, out);
                if let Some(step) = step {
                    written_locals_expr(step, out);
                }
            }
            Stmt::Switch { cases, .. } => {
                for case in cases {
                    written_locals(&case.body, out);
                }
            }
            Stmt::Block(body) => written_locals(body, out),
            _ => {}
        }
    }
}

/// Returns the taint that an assignment target contributes, apart from the assigned value (K22).
///
/// A plain write replaces the local's value, so the target's own place is uniform. An index
/// expression still contributes the index's taint, because it decides which element the write
/// reaches.
fn assignment_target_taint(validator: &BarrierValidator<'_, '_>, target: &Expr) -> UniformityTaint {
    match &target.kind {
        ExprKind::Local(_) => UniformityTaint::Uniform,
        ExprKind::Field { obj, .. } => assignment_target_taint(validator, obj),
        ExprKind::Index { obj, index, .. } => {
            assignment_target_taint(validator, obj).merge(validator.expression(index))
        }
        _ => UniformityTaint::Uniform,
    }
}

/// Reports whether the statements hold a barrier call at any depth (K22).
fn contains_barrier(module: &Module, statements: &[Stmt]) -> bool {
    statements.iter().any(|statement| match statement {
        Stmt::Expr(expr) => barrier_call(module, expr).is_some(),
        Stmt::If { then, els, .. } => {
            contains_barrier(module, then)
                || els
                    .as_ref()
                    .is_some_and(|statements| contains_barrier(module, statements))
        }
        Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::ForOf { body, .. } => {
            contains_barrier(module, body)
        }
        Stmt::Switch { cases, .. } => cases
            .iter()
            .any(|case| contains_barrier(module, &case.body)),
        Stmt::Block(body) => contains_barrier(module, body),
        _ => false,
    })
}

/// Returns the line and column of the last barrier in the statements, at any depth (K22).
///
/// A `return` before that position ends some invocations while others reach the barrier, which
/// the check rejects.
fn last_barrier_position(module: &Module, statements: &[Stmt]) -> Option<(u32, u32)> {
    let mut last = None;
    for statement in statements {
        let candidate = match statement {
            Stmt::Expr(expr) if barrier_call(module, expr).is_some() => {
                Some((expr.pos.line, expr.pos.col))
            }
            Stmt::If { then, els, .. } => last_barrier_position(module, then)
                .into_iter()
                .chain(
                    els.as_ref()
                        .and_then(|statements| last_barrier_position(module, statements)),
                )
                .max(),
            Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::ForOf { body, .. } => {
                last_barrier_position(module, body)
            }
            Stmt::Switch { cases, .. } => cases
                .iter()
                .filter_map(|case| last_barrier_position(module, &case.body))
                .max(),
            Stmt::Block(body) => last_barrier_position(module, body),
            _ => None,
        };
        last = last.into_iter().chain(candidate).max();
    }
    last
}

/// Collects the names of the functions that `expr` calls directly.
fn called_functions_expr(expr: &Expr, out: &mut BTreeSet<String>) {
    match &expr.kind {
        ExprKind::Call { callee, args } => {
            if let Callee::Func(name) = callee {
                out.insert(name.clone());
            }
            if let Callee::Value(value) = callee {
                called_functions_expr(value, out);
            }
            if let Callee::Method { recv, .. } = callee {
                called_functions_expr(recv, out);
            }
            for arg in args {
                called_functions_expr(arg, out);
            }
        }
        ExprKind::AbsenceTest { value: operand, .. }
        | ExprKind::Unary { operand, .. }
        | ExprKind::Cast(operand)
        | ExprKind::Length(operand)
        | ExprKind::Field { obj: operand, .. }
        | ExprKind::JsonResultValue(operand) => called_functions_expr(operand, out),
        ExprKind::Binary { left, right, .. } => {
            called_functions_expr(left, out);
            called_functions_expr(right, out);
        }
        ExprKind::Assign { target, value, .. } => {
            called_functions_expr(target, out);
            called_functions_expr(value, out);
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => {
            for arg in args {
                called_functions_expr(arg, out);
            }
        }
        ExprKind::DescriptorLit { fields, .. } => {
            for field in fields.iter().flatten() {
                called_functions_expr(field, out);
            }
        }
        ExprKind::Index { obj, index, .. } => {
            called_functions_expr(obj, out);
            called_functions_expr(index, out);
        }
        ExprKind::Cond { cond, then, els } => {
            called_functions_expr(cond, out);
            called_functions_expr(then, out);
            called_functions_expr(els, out);
        }
        _ => {}
    }
}

/// Runs `called_functions_expr` over every expression the statement holds, nested bodies included.
fn called_functions_stmt(stmt: &Stmt, out: &mut BTreeSet<String>) {
    match stmt {
        Stmt::Let { init, .. } | Stmt::Expr(init) => called_functions_expr(init, out),
        Stmt::Return {
            value: Some(value), ..
        } => called_functions_expr(value, out),
        Stmt::Return { value: None, .. } => {}
        Stmt::If {
            cond, then, els, ..
        } => {
            called_functions_expr(cond, out);
            for stmt in then {
                called_functions_stmt(stmt, out);
            }
            if let Some(els) = els {
                for stmt in els {
                    called_functions_stmt(stmt, out);
                }
            }
        }
        Stmt::While { cond, body, .. } => {
            called_functions_expr(cond, out);
            for stmt in body {
                called_functions_stmt(stmt, out);
            }
        }
        Stmt::For {
            init,
            cond,
            step,
            body,
            ..
        } => {
            if let Some(init) = init {
                called_functions_stmt(init, out);
            }
            if let Some(cond) = cond {
                called_functions_expr(cond, out);
            }
            if let Some(step) = step {
                called_functions_expr(step, out);
            }
            for stmt in body {
                called_functions_stmt(stmt, out);
            }
        }
        Stmt::ForOf { subject, body, .. } => {
            called_functions_expr(subject, out);
            for stmt in body {
                called_functions_stmt(stmt, out);
            }
        }
        Stmt::Switch { disc, cases, .. } => {
            called_functions_expr(disc, out);
            for stmt in cases.iter().flat_map(|case| &case.body) {
                called_functions_stmt(stmt, out);
            }
        }
        Stmt::Block(body) => {
            for stmt in body {
                called_functions_stmt(stmt, out);
            }
        }
        Stmt::Break(_) | Stmt::Continue(_) => {}
    }
}

/// Appends the schema name that `ty` names to `out`, once, in first-use order (K14).
///
/// A library class carries its own WGSL spelling and declares no struct, so it never enters.
/// `seen` holds the names already appended.
///
/// # Errors
///
/// If the type names a class the module does not hold, returns an internal diagnostic.
fn collect_schema_type(
    module: &Module,
    ty: &Type,
    seen: &mut BTreeSet<String>,
    out: &mut Vec<String>,
    pos: &Pos,
) -> Result<(), Diagnostic> {
    match ty {
        Type::Class(id) => {
            let class = crate::class(module, id.0, "kernel::collect_schema_type", pos)?;
            if class.is_value
                && class.pos.file != "typegpu-types.ts"
                && class.pos.file != "typegpu.ts"
                && seen.insert(class.name.clone())
            {
                out.push(class.name.clone());
            }
        }
        Type::FixedArray(item, _) => collect_schema_type(module, item, seen, out, pos)?,
        _ => {}
    }

    Ok(())
}

/// Appends the schema names that `expr` and its sub-expressions carry, in first-use order.
///
/// Every expression carries its own type, so the walk reads the type first and then descends.
///
/// # Errors
///
/// Returns the internal diagnostics of `collect_schema_type`.
fn collect_schema_expr(
    module: &Module,
    expr: &Expr,
    seen: &mut BTreeSet<String>,
    out: &mut Vec<String>,
) -> Result<(), Diagnostic> {
    collect_schema_type(module, &expr.ty, seen, out, &expr.pos)?;
    match &expr.kind {
        ExprKind::AbsenceTest { value: operand, .. }
        | ExprKind::Unary { operand, .. }
        | ExprKind::Cast(operand)
        | ExprKind::Length(operand)
        | ExprKind::Field { obj: operand, .. }
        | ExprKind::JsonResultValue(operand) => {
            collect_schema_expr(module, operand, seen, out)?;
        }
        ExprKind::Binary { left, right, .. } => {
            collect_schema_expr(module, left, seen, out)?;
            collect_schema_expr(module, right, seen, out)?;
        }
        ExprKind::Assign { target, value, .. } => {
            collect_schema_expr(module, target, seen, out)?;
            collect_schema_expr(module, value, seen, out)?;
        }
        ExprKind::Call { callee, args } => {
            if let Callee::Value(value) = callee {
                collect_schema_expr(module, value, seen, out)?;
            }
            if let Callee::Method { recv, .. } = callee {
                collect_schema_expr(module, recv, seen, out)?;
            }
            for arg in args {
                collect_schema_expr(module, arg, seen, out)?;
            }
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => {
            for arg in args {
                collect_schema_expr(module, arg, seen, out)?;
            }
        }
        ExprKind::DescriptorLit { fields, .. } => {
            for field in fields.iter().flatten() {
                collect_schema_expr(module, field, seen, out)?;
            }
        }
        ExprKind::Index { obj, index, .. } => {
            collect_schema_expr(module, obj, seen, out)?;
            collect_schema_expr(module, index, seen, out)?;
        }
        ExprKind::Cond { cond, then, els } => {
            collect_schema_expr(module, cond, seen, out)?;
            collect_schema_expr(module, then, seen, out)?;
            collect_schema_expr(module, els, seen, out)?;
        }
        ExprKind::Template(parts) => {
            for part in parts {
                if let subscript_compiler::hir::TplPart::Expr(value) = part {
                    collect_schema_expr(module, value, seen, out)?;
                }
            }
        }
        ExprKind::Lambda { body, .. } => {
            for stmt in body {
                collect_schema_stmt(module, stmt, seen, out)?;
            }
        }
        _ => {}
    }

    Ok(())
}

/// Appends the schema names that one statement carries, its declared types included.
///
/// # Errors
///
/// Returns the internal diagnostics of `collect_schema_type`.
fn collect_schema_stmt(
    module: &Module,
    stmt: &Stmt,
    seen: &mut BTreeSet<String>,
    out: &mut Vec<String>,
) -> Result<(), Diagnostic> {
    match stmt {
        Stmt::Let { ty, init, .. } => {
            collect_schema_type(module, ty, seen, out, &init.pos)?;
            collect_schema_expr(module, init, seen, out)?;
        }
        Stmt::Expr(expr) => collect_schema_expr(module, expr, seen, out)?,
        Stmt::Return {
            value: Some(value), ..
        } => collect_schema_expr(module, value, seen, out)?,
        Stmt::Return { value: None, .. } => {}
        Stmt::If {
            cond, then, els, ..
        } => {
            collect_schema_expr(module, cond, seen, out)?;
            for stmt in then {
                collect_schema_stmt(module, stmt, seen, out)?;
            }
            if let Some(els) = els {
                for stmt in els {
                    collect_schema_stmt(module, stmt, seen, out)?;
                }
            }
        }
        Stmt::While { cond, body, .. } => {
            collect_schema_expr(module, cond, seen, out)?;
            for stmt in body {
                collect_schema_stmt(module, stmt, seen, out)?;
            }
        }
        Stmt::For {
            init,
            cond,
            step,
            body,
            ..
        } => {
            if let Some(init) = init {
                collect_schema_stmt(module, init, seen, out)?;
            }
            if let Some(cond) = cond {
                collect_schema_expr(module, cond, seen, out)?;
            }
            if let Some(step) = step {
                collect_schema_expr(module, step, seen, out)?;
            }
            for stmt in body {
                collect_schema_stmt(module, stmt, seen, out)?;
            }
        }
        Stmt::ForOf {
            ty, subject, body, ..
        } => {
            collect_schema_type(module, ty, seen, out, &subject.pos)?;
            collect_schema_expr(module, subject, seen, out)?;
            for stmt in body {
                collect_schema_stmt(module, stmt, seen, out)?;
            }
        }
        Stmt::Switch { disc, cases, .. } => {
            collect_schema_expr(module, disc, seen, out)?;
            for stmt in cases.iter().flat_map(|case| &case.body) {
                collect_schema_stmt(module, stmt, seen, out)?;
            }
        }
        Stmt::Block(body) => {
            for stmt in body {
                collect_schema_stmt(module, stmt, seen, out)?;
            }
        }
        Stmt::Break(_) | Stmt::Continue(_) => {}
    }

    Ok(())
}

/// Returns the schema names that one compute pipeline's WGSL module declares.
///
/// The order is first use: the bindings, then the reached globals, then the helper signatures
/// and bodies. K14 fixes that order for the emitted structs.
///
/// # Errors
///
/// Returns a K2 diagnostic for a cycle in the call graph. A reached global with no WGSL form
/// gives a K19 or K20 diagnostic.
pub(crate) fn referenced_schema_names(
    module: &Module,
    pipeline: &Pipeline,
    shells: &crate::shell::ShellProgram,
) -> Result<Vec<String>, Diagnostic> {
    let kernel = function(module, &pipeline.entry).ok_or_else(|| {
        generator_diagnostic(
            "the kernel disappeared from typed HIR",
            pipeline.pos.clone(),
        )
    })?;
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for layout in &pipeline.layouts {
        for binding in &layout.bindings {
            collect_schema_type(module, &binding.item_ty, &mut seen, &mut out, &binding.pos)?;
        }
    }
    for global in kernel_globals(module, kernel, shells)? {
        collect_schema_type(module, &global.ty, &mut seen, &mut out, &global.pos)?;
    }
    for name in dependencies(module, kernel, shells)? {
        let helper = function(module, &name).ok_or_else(|| {
            generator_diagnostic("a helper disappeared from typed HIR", pipeline.pos.clone())
        })?;
        for param in &helper.params {
            collect_schema_type(module, &param.ty, &mut seen, &mut out, &param.pos)?;
        }
        collect_schema_type(module, &helper.ret, &mut seen, &mut out, &helper.pos)?;
        if !crate::shell::function_is_shell(shells, &name) {
            for stmt in &helper.body {
                collect_schema_stmt(module, stmt, &mut seen, &mut out)?;
            }
        }
    }
    for stmt in &kernel.body {
        collect_schema_stmt(module, stmt, &mut seen, &mut out)?;
    }
    Ok(out)
}

/// Returns the functions that one kernel's call graph reaches, in dependency order (K2).
///
/// A callee precedes its caller, so the emitter writes each helper after the helpers it calls. A
/// shell is a leaf: the emitter never walks its subscript body (K29). A library function is not
/// GPU code and does not enter.
///
/// # Errors
///
/// Returns a K2 diagnostic that names the cycle when the graph is not acyclic.
fn dependencies(
    module: &Module,
    kernel: &Function,
    shells: &crate::shell::ShellProgram,
) -> Result<Vec<String>, Diagnostic> {
    /// Visits one function and appends it after the functions it calls. `stack` holds the current
    /// path, which detects a cycle, and `done` holds the functions already appended.
    fn visit(
        module: &Module,
        name: &str,
        shells: &crate::shell::ShellProgram,
        stack: &mut Vec<String>,
        done: &mut BTreeSet<String>,
        order: &mut Vec<String>,
        pos: &Pos,
    ) -> Result<(), Diagnostic> {
        if done.contains(name) {
            return Ok(());
        }
        if crate::shell::function_is_shell(shells, name) {
            done.insert(name.to_owned());
            order.push(name.to_owned());
            return Ok(());
        }
        if let Some(start) = stack.iter().position(|item| item == name) {
            let mut cycle = stack.iter().skip(start).cloned().collect::<Vec<_>>();
            cycle.push(name.to_owned());
            return Err(diagnostic(
                "K2",
                format!("recursive helper cycle: {}", cycle.join(" -> ")),
                pos.clone(),
            ));
        }
        let Some(function) = function(module, name) else {
            return Ok(());
        };
        if matches!(
            function.pos.file.as_str(),
            "typegpu-types.ts" | "typegpu.ts"
        ) {
            return Ok(());
        }
        stack.push(name.to_owned());
        let mut calls = BTreeSet::new();
        for stmt in &function.body {
            called_functions_stmt(stmt, &mut calls);
        }
        for called in calls {
            visit(module, &called, shells, stack, done, order, &function.pos)?;
        }
        stack.pop();
        done.insert(name.to_owned());
        order.push(name.to_owned());
        Ok(())
    }
    let mut calls = BTreeSet::new();
    for stmt in &kernel.body {
        called_functions_stmt(stmt, &mut calls);
    }
    let mut order = Vec::new();
    let mut done = BTreeSet::new();
    for called in calls {
        visit(
            module,
            &called,
            shells,
            &mut Vec::new(),
            &mut done,
            &mut order,
            &kernel.pos,
        )?;
    }
    Ok(order)
}

/// Returns the `@builtin` parameter declaration of one invocation field (PI4, RN3).
///
/// An entry point declares the parameters its body reads and no other. A name outside the set
/// gives the empty string.
fn builtin_parameter(name: &str) -> &'static str {
    match name {
        "globalId" => "@builtin(global_invocation_id) globalId: vec3<u32>",
        "localId" => "@builtin(local_invocation_id) localId: vec3<u32>",
        "workgroupId" => "@builtin(workgroup_id) workgroupId: vec3<u32>",
        "numWorkgroups" => "@builtin(num_workgroups) numWorkgroups: vec3<u32>",
        "localIndex" => "@builtin(local_invocation_index) localIndex: u32",
        "vertexIndex" => "@builtin(vertex_index) vertexIndex: u32",
        "instanceIndex" => "@builtin(instance_index) instanceIndex: u32",
        "frontFacing" => "@builtin(front_facing) frontFacing: bool",
        _ => "",
    }
}

/// Returns every name the emitted module declares at module scope, in emitted spelling (K14).
///
/// The set is fixed before any body is emitted, so a local that collides with one of these names
/// gains a `_` and shadows nothing. The builtin parameter names join the set, because an entry
/// point declares them beside the module declarations.
fn module_scope_names(
    structs: &[(String, String)],
    layouts: &[crate::pipeline::Layout],
    globals: &[KernelGlobal],
    helpers: &[String],
    entries: &[&str],
    declaration_names: Option<&BTreeSet<String>>,
) -> BTreeSet<String> {
    let mut names = structs
        .iter()
        .map(|(name, _)| mapping::ident(name))
        .collect::<BTreeSet<_>>();
    names.extend(
        layouts
            .iter()
            .flat_map(|layout| &layout.bindings)
            .map(|binding| mapping::ident(&binding.name)),
    );
    names.extend(globals.iter().map(|global| mapping::ident(&global.name)));
    names.extend(
        helpers
            .iter()
            .map(|name| mapping::ident(crate::base_name(name))),
    );
    names.extend(entries.iter().map(|name| mapping::ident(name)));
    names.extend(declaration_names.into_iter().flatten().cloned());
    names.extend(
        [
            "globalId",
            "localId",
            "workgroupId",
            "numWorkgroups",
            "localIndex",
            "vertexIndex",
            "instanceIndex",
            "frontFacing",
        ]
        .into_iter()
        .map(str::to_owned),
    );
    names
}

/// Returns the one-based line number that the next appended character starts on.
fn next_line(out: &str) -> u32 {
    out.bytes().filter(|byte| *byte == b'\n').count() as u32 + 1
}

/// Appends author WGSL and records its line range under `label` (K31).
///
/// The harness reads the ranges to attribute a `naga` error to the shell or to the declarations.
/// A line outside every range stays a generator defect (K15).
fn append_recorded_text(out: &mut String, text: &str, label: String, spans: &mut Vec<WgslSpan>) {
    let start_line = next_line(out);
    out.push_str(text);
    if !text.ends_with('\n') {
        out.push('\n');
    }
    let end_line = next_line(out).saturating_sub(1).max(start_line);
    spans.push(WgslSpan {
        label,
        start_line,
        end_line,
    });
}

/// Emits one WGSL shell: the `fn` line from the typed signature and the author body (K29).
///
/// The body keeps its own relative indentation. The generator removes the common leading
/// blankspace of the non-empty lines and adds one level. A body inside a template literal then
/// lands at the function's indent.
///
/// # Errors
///
/// Returns the K29 diagnostics of `validate_signature`, and the diagnostics of `wgsl_type` for a
/// parameter or a return type outside the kernel value set.
fn emit_shell(
    module: &Module,
    shell: &crate::shell::Shell,
    layouts: &[crate::pipeline::Layout],
    out: &mut String,
    spans: &mut Vec<WgslSpan>,
) -> Result<(), Diagnostic> {
    let function = crate::shell::validate_signature(module, shell, layouts)?;
    let params = function
        .params
        .iter()
        .map(|param| {
            Ok(format!(
                "{}: {}",
                mapping::ident(&param.name),
                wgsl_type(module, &param.ty, &param.pos)?
            ))
        })
        .collect::<Result<Vec<_>, Diagnostic>>()?;
    let result = if function.ret == Type::Void {
        String::new()
    } else {
        format!(" -> {}", wgsl_type(module, &function.ret, &function.pos)?)
    };
    out.push_str(&format!(
        "fn {}({}){result} {{\n",
        mapping::ident(&shell.name),
        params.join(", ")
    ));
    let lines = shell.body.split('\n').collect::<Vec<_>>();
    let common_indent = lines
        .iter()
        .filter_map(|line| {
            let line = line.trim_end_matches(crate::shell::is_wgsl_blankspace);
            (!line.is_empty()).then(|| {
                line.chars()
                    .take_while(|ch| crate::shell::is_wgsl_blankspace(*ch))
                    .count()
            })
        })
        .min()
        .unwrap_or(0);
    let mut body = String::new();
    for line in lines {
        let line = line.trim_end_matches(crate::shell::is_wgsl_blankspace);
        let start = line
            .char_indices()
            .nth(common_indent)
            .map_or(line.len(), |(index, _)| index);
        let line = line.get(start..).ok_or_else(|| {
            crate::internal("kernel::emit_shell", "invalid indent boundary", &shell.pos)
        })?;
        if line.is_empty() {
            body.push('\n');
            continue;
        }
        body.push_str("  ");
        body.push_str(line);
        body.push('\n');
    }
    append_recorded_text(out, &body, format!("shell {}", shell.name), spans);
    out.push_str("}\n\n");
    Ok(())
}

/// Emits the complete WGSL module of one compute pipeline (K14).
///
/// `structs` holds the schema struct texts in first-use order, and `uses_f16` adds the `enable`
/// directive. The result is the text that the committed `.wgsl` golden holds (K16).
///
/// # Errors
///
/// Returns one diagnostic. The cases include a statement outside K7, an expression outside K9,
/// and a type outside K4. A method outside K10, a non-uniform barrier (K22), and an atomic
/// outside read-write storage (K21) also give one.
pub(crate) fn emit(
    module: &Module,
    pipeline: &Pipeline,
    structs: &[(String, String)],
    uses_f16: bool,
    shells: &crate::shell::ShellProgram,
) -> Result<EmittedWgsl, Diagnostic> {
    let kernel = function(module, &pipeline.entry)
        .ok_or_else(|| generator_diagnostic("kernel disappeared from HIR", pipeline.pos.clone()))?;
    let dependencies = dependencies(module, kernel, shells)?;
    let helpers = dependencies
        .iter()
        .filter(|name| !crate::shell::function_is_shell(shells, name))
        .cloned()
        .collect::<Vec<_>>();
    let reached_shells = dependencies
        .iter()
        .filter_map(|name| crate::shell::shell_for_function(shells, name))
        .collect::<Vec<_>>();
    let globals = kernel_globals(module, kernel, shells)?;
    let module_names = module_scope_names(
        structs,
        &pipeline.layouts,
        &globals,
        &helpers,
        &[&pipeline.entry],
        shells.declarations.as_ref().map(|item| &item.names),
    );
    validate_statement_subset(&kernel.body)?;
    for binding in pipeline.layouts.iter().flat_map(|layout| &layout.bindings) {
        if binding.kind.is_buffer()
            && binding.kind != BindingKind::MutStorage
            && type_contains_atomic(module, &binding.item_ty, &binding.pos)?
        {
            return Err(diagnostic(
                "K21",
                format!(
                    "binding `{}` places an atomic schema in uniform or read-only storage",
                    binding.name
                ),
                binding.pos.clone(),
            ));
        }
    }
    let mut emitter = Emitter::entry(
        module,
        &pipeline.layouts,
        kernel,
        pipeline.layouts.len(),
        InvocationKind::Compute,
        &globals,
        &module_names,
    )?;
    let mut helper_text = String::new();
    for name in &helpers {
        let helper = function(module, name).ok_or_else(|| {
            generator_diagnostic(
                format!("helper `{name}` disappeared from typed HIR"),
                pipeline.pos.clone(),
            )
        })?;
        if helper.is_async || helper.is_generator {
            return Err(diagnostic(
                "K2",
                format!("helper `{name}` is async or a generator"),
                helper.pos.clone(),
            ));
        }
        validate_statement_subset(&helper.body)?;
        for param in &helper.params {
            let takes_layout = class_name(module, &param.ty)
                .is_some_and(|name| pipeline.layouts.iter().any(|layout| layout.name == name));
            if takes_layout || class_name(module, &param.ty) == Some("ComputeInvocation") {
                return Err(diagnostic(
                    "K2",
                    format!("helper `{name}` takes a layout class or ComputeInvocation"),
                    param.pos.clone(),
                ));
            }
            let _ = wgsl_type(module, &param.ty, &param.pos)?;
        }
        let mut helper_emitter = Emitter::helper(module, helper, &globals, &module_names);
        let params = helper
            .params
            .iter()
            .map(|param| {
                Ok(format!(
                    "{}: {}",
                    helper_emitter.local_name(&param.name),
                    wgsl_type(module, &param.ty, &param.pos)?
                ))
            })
            .collect::<Result<Vec<_>, Diagnostic>>()?;
        let result = if helper.ret == Type::Void {
            String::new()
        } else {
            format!(" -> {}", wgsl_type(module, &helper.ret, &helper.pos)?)
        };
        helper_text.push_str(&format!(
            "fn {}({}){result} {{\n",
            mapping::ident(crate::base_name(name)),
            params.join(", ")
        ));
        helper_emitter.statements(&helper.body, 1, &mut helper_text)?;
        helper_text.push_str("}\n\n");
    }
    // The body is emitted before the module text, because the walk records which builtins the
    // entry point declares and which locals took a mangled name (K14).
    let mut entry_body = String::new();
    if pipeline.guarded {
        // The guard reads the global invocation id whether the author's body does or not (PI15).
        emitter.used_builtins.insert("globalId".to_owned());
        let guard = pipeline
            .layouts
            .last()
            .and_then(|layout| {
                layout
                    .bindings
                    .iter()
                    .find(|binding| binding.kind == BindingKind::Guard)
            })
            .ok_or_else(|| {
                generator_diagnostic(
                    "guarded pipeline lost its guard binding",
                    pipeline.pos.clone(),
                )
            })?;
        entry_body.push_str(&format!(
            "  if (globalId.x < {guard}.x && globalId.y < {guard}.y && globalId.z < {guard}.z) {{\n",
            guard = mapping::ident(&guard.name),
        ));
        emitter.statements(&kernel.body, 2, &mut entry_body)?;
        entry_body.push_str("  }\n");
    } else {
        emitter.statements(&kernel.body, 1, &mut entry_body)?;
    }
    // The check resolves bindings and globals through the emitter, so it reads the same maps the
    // emitted text did (K22).
    BarrierValidator::new(&emitter, kernel).validate(kernel)?;

    // The order below is K14's, the same for every module. It runs from the `enable` directives
    // to the raw declarations, the schema structs, the shells, the bindings, the module variables,
    // the helpers, and the entry point.
    let mut out = String::new();
    let mut spans = Vec::new();
    if uses_f16
        || pipeline
            .layouts
            .iter()
            .flat_map(|layout| &layout.bindings)
            .any(|binding| {
                class_name(module, &binding.item_ty)
                    .is_some_and(|name| matches!(name, "Vec2h" | "Vec3h" | "Vec4h"))
            })
    {
        out.push_str("enable f16;\n\n");
    }
    if let Some(declarations) = &shells.declarations {
        append_recorded_text(
            &mut out,
            &declarations.text,
            "declarations".to_owned(),
            &mut spans,
        );
        out.push('\n');
    }
    for (_, structure) in structs {
        out.push_str(structure);
        out.push('\n');
    }
    for shell in reached_shells {
        emit_shell(module, shell, &pipeline.layouts, &mut out, &mut spans)?;
    }
    for layout in &pipeline.layouts {
        for binding in &layout.bindings {
            out.push_str(&binding_declaration(module, layout.group, binding)?);
        }
    }
    out.push('\n');
    out.push_str(&emit_kernel_globals(module, &globals)?);
    out.push_str(&helper_text);
    // The entry point declares the builtins the body reads and no other (PI4). The order is this
    // list's, not the order the body reads them in, so the emitted signature is deterministic.
    let parameters = [
        "globalId",
        "localId",
        "workgroupId",
        "numWorkgroups",
        "localIndex",
    ]
    .into_iter()
    .filter(|name| emitter.used_builtins.contains(*name))
    .map(builtin_parameter)
    .collect::<Vec<_>>();
    out.push_str(&format!(
        "@compute @workgroup_size({}, {}, {})\n",
        crate::wgsl_u32_literal(pipeline.workgroup[0]),
        crate::wgsl_u32_literal(pipeline.workgroup[1]),
        crate::wgsl_u32_literal(pipeline.workgroup[2])
    ));
    out.push_str(&format!(
        "fn {}({}) {{\n",
        mapping::ident(&pipeline.entry),
        parameters.join(", ")
    ));
    out.push_str(&entry_body);
    out.push_str("}\n");
    Ok(EmittedWgsl { text: out, spans })
}

/// Returns the schema names that one render pipeline's WGSL module declares.
///
/// The order is first use: the bindings, then the vertex call graph, then the fragment call
/// graph. K14 fixes that order for the emitted structs.
///
/// # Errors
///
/// Returns a K2 diagnostic for a cycle in a call graph. An entry point or a helper that the
/// module does not declare gives a generator diagnostic.
pub(crate) fn referenced_render_schema_names(
    module: &Module,
    pipeline: &RenderPipeline,
    shells: &crate::shell::ShellProgram,
) -> Result<Vec<String>, Diagnostic> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for layout in &pipeline.layouts {
        for binding in &layout.bindings {
            collect_schema_type(module, &binding.item_ty, &mut seen, &mut out, &binding.pos)?;
        }
    }
    for entry in [&pipeline.vertex_entry, &pipeline.fragment_entry] {
        let kernel = function(module, entry).ok_or_else(|| {
            generator_diagnostic(
                "a render kernel disappeared from typed HIR",
                pipeline.pos.clone(),
            )
        })?;
        for name in dependencies(module, kernel, shells)? {
            let helper = function(module, &name).ok_or_else(|| {
                generator_diagnostic(
                    "a render helper disappeared from typed HIR",
                    pipeline.pos.clone(),
                )
            })?;
            for param in &helper.params {
                collect_schema_type(module, &param.ty, &mut seen, &mut out, &param.pos)?;
            }
            collect_schema_type(module, &helper.ret, &mut seen, &mut out, &helper.pos)?;
            if !crate::shell::function_is_shell(shells, &name) {
                for stmt in &helper.body {
                    collect_schema_stmt(module, stmt, &mut seen, &mut out)?;
                }
            }
        }
        for stmt in &kernel.body {
            collect_schema_stmt(module, stmt, &mut seen, &mut out)?;
        }
    }
    let vertex = function(module, &pipeline.vertex_entry).ok_or_else(|| {
        generator_diagnostic(
            "the vertex kernel disappeared from typed HIR",
            pipeline.pos.clone(),
        )
    })?;
    let fragment = function(module, &pipeline.fragment_entry).ok_or_else(|| {
        generator_diagnostic(
            "the fragment kernel disappeared from typed HIR",
            pipeline.pos.clone(),
        )
    })?;
    for global in render_kernel_globals(module, [vertex, fragment], shells)? {
        collect_schema_type(module, &global.ty, &mut seen, &mut out, &global.pos)?;
    }
    // A vertex schema and the varyings class reach the module as attributed structs, which
    // `render_interface_structs` writes (RN4, RN7). A plain struct for either repeats the
    // name.
    let interface_names = pipeline
        .vertex_buffers
        .iter()
        .map(|buffer| buffer.schema.as_str())
        .chain(std::iter::once(pipeline.varyings_name.as_str()))
        .collect::<BTreeSet<_>>();
    out.retain(|name| !interface_names.contains(name.as_str()));
    Ok(out)
}

/// Emits the vertex input structs and the varyings struct of one render pipeline (RN4, RN7).
///
/// A vertex field takes `@location(n)` from the buffer's attribute list. The varyings `position`
/// field takes `@builtin(position)`, and an integer varying takes `@interpolate(flat)`.
///
/// # Errors
///
/// If a vertex schema is absent from the module, returns a generator diagnostic. A field type
/// outside the kernel value set gives the diagnostics of `wgsl_type`.
fn render_interface_structs(
    module: &Module,
    pipeline: &RenderPipeline,
) -> Result<String, Diagnostic> {
    let mut out = String::new();
    for buffer in &pipeline.vertex_buffers {
        let class = module
            .classes
            .iter()
            .find(|class| class.name == buffer.schema && class.pos.file == pipeline.pos.file)
            .or_else(|| {
                module
                    .classes
                    .iter()
                    .find(|class| class.name == buffer.schema)
            })
            .ok_or_else(|| {
                generator_diagnostic(
                    format!(
                        "vertex schema `{}` disappeared from typed HIR",
                        buffer.schema
                    ),
                    pipeline.pos.clone(),
                )
            })?;
        out.push_str(&format!("struct {} {{\n", mapping::ident(&buffer.schema)));
        for (field, attribute) in class.fields.iter().zip(&buffer.attributes) {
            out.push_str(&format!(
                "  @location({}) {}: {},\n",
                crate::wgsl_u32_literal(attribute.location),
                mapping::ident(&field.name),
                wgsl_type(module, &field.ty, &field.pos)?
            ));
        }
        out.push_str("}\n\n");
    }
    out.push_str(&format!(
        "struct {} {{\n",
        mapping::ident(&pipeline.varyings_name)
    ));
    for varying in &pipeline.varyings {
        let attribute = if varying.builtin_position {
            "@builtin(position)".to_owned()
        } else if varying.flat {
            format!(
                "@location({}) @interpolate(flat)",
                crate::wgsl_u32_literal(varying.location.unwrap_or(0))
            )
        } else {
            format!(
                "@location({})",
                crate::wgsl_u32_literal(varying.location.unwrap_or(0))
            )
        };
        out.push_str(&format!(
            "  {attribute} {}: {},\n",
            mapping::ident(&varying.name),
            wgsl_type(module, &varying.ty, &pipeline.pos)?
        ));
    }
    out.push_str("}\n\n");
    Ok(out)
}

/// Emits the helpers that both render entry points reach, once each, in dependency order (RN9).
///
/// The vertex graph comes first, then the fragment graph, so a helper both entry points call
/// keeps the vertex position.
///
/// # Errors
///
/// Returns a K2 diagnostic for a helper that is `async` or a generator, and for one that takes a
/// layout class or an invocation class. A body gives the emitter's own diagnostics.
fn render_helpers(
    module: &Module,
    pipeline: &RenderPipeline,
    kernels: [&Function; 2],
    globals: &[KernelGlobal],
    module_names: &BTreeSet<String>,
    shells: &crate::shell::ShellProgram,
) -> Result<String, Diagnostic> {
    let mut names = Vec::new();
    let mut seen = BTreeSet::new();
    for kernel in kernels {
        for name in dependencies(module, kernel, shells)? {
            if crate::shell::function_is_shell(shells, &name) {
                continue;
            }
            if seen.insert(name.clone()) {
                names.push(name);
            }
        }
    }
    let mut out = String::new();
    for name in names {
        let helper = function(module, &name).ok_or_else(|| {
            generator_diagnostic(
                format!("helper `{name}` disappeared from typed HIR"),
                pipeline.pos.clone(),
            )
        })?;
        if helper.is_async || helper.is_generator {
            return Err(diagnostic(
                "K2",
                format!("helper `{name}` is async or a generator"),
                helper.pos.clone(),
            ));
        }
        for param in &helper.params {
            let takes_layout = class_name(module, &param.ty)
                .is_some_and(|name| pipeline.layouts.iter().any(|layout| layout.name == name));
            if takes_layout
                || matches!(
                    class_name(module, &param.ty),
                    Some("VertexInvocation" | "FragmentInvocation")
                )
            {
                return Err(diagnostic(
                    "K2",
                    format!("helper `{name}` takes a layout class or invocation class"),
                    param.pos.clone(),
                ));
            }
            let _ = wgsl_type(module, &param.ty, &param.pos)?;
        }
        let mut emitter = Emitter::helper(module, helper, globals, module_names);
        let params = helper
            .params
            .iter()
            .map(|param| {
                Ok(format!(
                    "{}: {}",
                    emitter.local_name(&param.name),
                    wgsl_type(module, &param.ty, &param.pos)?
                ))
            })
            .collect::<Result<Vec<_>, Diagnostic>>()?;
        let result = if helper.ret == Type::Void {
            String::new()
        } else {
            format!(" -> {}", wgsl_type(module, &helper.ret, &helper.pos)?)
        };
        out.push_str(&format!(
            "fn {}({}){result} {{\n",
            mapping::ident(crate::base_name(&name)),
            params.join(", ")
        ));
        emitter.statements(&helper.body, 1, &mut out)?;
        out.push_str("}\n\n");
    }
    Ok(out)
}

/// Emits the complete WGSL module of one render pipeline, with both entry points (RN9).
///
/// `schemas` decides the `enable f16;` directive, and `structs` holds the struct texts in
/// first-use order. The vertex entry precedes the fragment entry, which K14 fixes.
///
/// # Errors
///
/// Returns one diagnostic. A vertex kernel that writes a storage binding (RN9) and every
/// compute-side kernel rejection are the cases.
pub(crate) fn emit_render(
    module: &Module,
    pipeline: &RenderPipeline,
    structs: &[(String, String)],
    schemas: &[Schema],
    shells: &crate::shell::ShellProgram,
) -> Result<EmittedWgsl, Diagnostic> {
    crate::render::reject_vertex_storage_writes(module, pipeline)?;
    let vertex = function(module, &pipeline.vertex_entry).ok_or_else(|| {
        generator_diagnostic("vertex kernel disappeared from HIR", pipeline.pos.clone())
    })?;
    let fragment = function(module, &pipeline.fragment_entry).ok_or_else(|| {
        generator_diagnostic("fragment kernel disappeared from HIR", pipeline.pos.clone())
    })?;
    let globals = render_kernel_globals(module, [vertex, fragment], shells)?;
    let mut helper_names = Vec::new();
    let mut seen_helpers = BTreeSet::new();
    for kernel in [vertex, fragment] {
        for name in dependencies(module, kernel, shells)? {
            if seen_helpers.insert(name.clone()) {
                helper_names.push(name);
            }
        }
    }
    let mut module_names = module_scope_names(
        structs,
        &pipeline.layouts,
        &globals,
        &helper_names,
        &[&pipeline.vertex_entry, &pipeline.fragment_entry],
        shells.declarations.as_ref().map(|item| &item.names),
    );
    module_names.insert(mapping::ident(&pipeline.varyings_name));
    let layout_count = pipeline.layouts.len();
    let vertex_value_count = pipeline.vertex_buffers.len();
    let mut vertex_emitter = Emitter::entry(
        module,
        &pipeline.layouts,
        vertex,
        layout_count + vertex_value_count,
        InvocationKind::Vertex,
        &globals,
        &module_names,
    )?;
    let mut fragment_emitter = Emitter::entry(
        module,
        &pipeline.layouts,
        fragment,
        layout_count + 1,
        InvocationKind::Fragment,
        &globals,
        &module_names,
    )?;
    // Both bodies are emitted before the module text, because each walk records the builtins its
    // entry point declares (RN3).
    let mut vertex_body = String::new();
    vertex_emitter.statements(&vertex.body, 1, &mut vertex_body)?;
    let mut fragment_body = String::new();
    fragment_emitter.statements(&fragment.body, 1, &mut fragment_body)?;

    let selected_names = structs
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<BTreeSet<_>>();
    // `enable f16;` depends on every type this module names: a schema struct, a binding item, a
    // vertex attribute, and a varying (RN9, LY15).
    let uses_f16 = schemas
        .iter()
        .filter(|schema| selected_names.contains(schema.name.as_str()))
        .any(|schema| crate::emit::uses_f16(&schema.tree))
        || pipeline
            .layouts
            .iter()
            .flat_map(|layout| &layout.bindings)
            .try_any(|binding| {
                crate::render::type_uses_f16(module, &binding.item_ty, &binding.pos)
            })?
        || pipeline
            .vertex_buffers
            .iter()
            .flat_map(|buffer| &buffer.attributes)
            .any(|attribute| attribute.format.starts_with("float16"))
        || pipeline
            .varyings
            .iter()
            .try_any(|varying| crate::render::type_uses_f16(module, &varying.ty, &pipeline.pos))?;
    let mut out = String::new();
    let mut spans = Vec::new();
    if uses_f16 {
        out.push_str("enable f16;\n\n");
    }
    if let Some(declarations) = &shells.declarations {
        append_recorded_text(
            &mut out,
            &declarations.text,
            "declarations".to_owned(),
            &mut spans,
        );
        out.push('\n');
    }
    for (_, structure) in structs {
        out.push_str(structure);
        out.push('\n');
    }
    out.push_str(&render_interface_structs(module, pipeline)?);
    for name in &helper_names {
        if let Some(shell) = crate::shell::shell_for_function(shells, name) {
            emit_shell(module, shell, &pipeline.layouts, &mut out, &mut spans)?;
        }
    }
    for layout in &pipeline.layouts {
        for binding in &layout.bindings {
            out.push_str(&binding_declaration(module, layout.group, binding)?);
        }
    }
    if !pipeline.layouts.is_empty() {
        out.push('\n');
    }
    out.push_str(&emit_kernel_globals(module, &globals)?);
    out.push_str(&render_helpers(
        module,
        pipeline,
        [vertex, fragment],
        &globals,
        &module_names,
        shells,
    )?);

    let mut vertex_parameters = vertex
        .params
        .get(layout_count..layout_count + vertex_value_count)
        .ok_or_else(|| {
            crate::internal(
                "kernel::emit_render",
                "missing vertex parameters",
                &vertex.pos,
            )
        })?
        .iter()
        .map(|param| {
            Ok(format!(
                "{}: {}",
                vertex_emitter.local_name(&param.name),
                wgsl_type(module, &param.ty, &param.pos)?
            ))
        })
        .collect::<Result<Vec<_>, Diagnostic>>()?;
    vertex_parameters.extend(
        ["vertexIndex", "instanceIndex"]
            .into_iter()
            .filter(|name| vertex_emitter.used_builtins.contains(*name))
            .map(builtin_parameter)
            .map(str::to_owned),
    );
    // The vertex entry precedes the fragment entry, which K14 fixes for every render module.
    out.push_str("@vertex\n");
    out.push_str(&format!(
        "fn {}({}) -> {} {{\n",
        mapping::ident(&pipeline.vertex_entry),
        vertex_parameters.join(", "),
        mapping::ident(&pipeline.varyings_name)
    ));
    out.push_str(&vertex_body);
    out.push_str("}\n\n");

    let input = fragment.params.get(layout_count).ok_or_else(|| {
        crate::internal(
            "kernel::emit_render",
            "missing fragment input",
            &fragment.pos,
        )
    })?;
    let mut fragment_parameters = vec![format!(
        "{}: {}",
        fragment_emitter.local_name(&input.name),
        wgsl_type(module, &input.ty, &input.pos)?
    )];
    fragment_parameters.extend(
        ["frontFacing"]
            .into_iter()
            .filter(|name| fragment_emitter.used_builtins.contains(*name))
            .map(builtin_parameter)
            .map(str::to_owned),
    );
    out.push_str("@fragment\n");
    let fragment_result = "@location(0u) vec4<f32>";
    out.push_str(&format!(
        "fn {}({}) -> {fragment_result} {{\n",
        mapping::ident(&pipeline.fragment_entry),
        fragment_parameters.join(", ")
    ));
    out.push_str(&fragment_body);
    out.push_str("}\n");
    Ok(EmittedWgsl { text: out, spans })
}
