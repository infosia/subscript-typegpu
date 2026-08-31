//! Typed-HIR to WGSL kernel emission.

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

type Prelude = Vec<(usize, String)>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WgslSpan {
    pub(crate) label: String,
    pub(crate) start_line: u32,
    pub(crate) end_line: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EmittedWgsl {
    pub(crate) text: String,
    pub(crate) spans: Vec<WgslSpan>,
}

#[derive(Debug, Clone)]
pub(crate) struct Snippet {
    pub(crate) text: String,
    precedence: u8,
    prelude: Prelude,
}

impl Snippet {
    fn new(text: String, precedence: u8) -> Self {
        Self {
            text,
            precedence,
            prelude: Vec::new(),
        }
    }

    fn atom(text: String) -> Self {
        Self::new(text, 10)
    }
}

fn diagnostic(rule: &str, message: impl Into<String>, pos: Pos) -> Diagnostic {
    Diagnostic::new(
        RuleCode::S100,
        format!("{rule}: {} (author)", message.into()),
        pos,
    )
}

fn generator_diagnostic(message: impl Into<String>, pos: Pos) -> Diagnostic {
    Diagnostic::new(
        RuleCode::S100,
        format!("K15: {} (generator)", message.into()),
        pos,
    )
}

fn function<'a>(module: &'a Module, name: &str) -> Option<&'a Function> {
    module
        .functions
        .iter()
        .find(|function| function.name == name)
}

fn class_name<'a>(module: &'a Module, ty: &Type) -> Option<&'a str> {
    let Type::Class(id) = ty else { return None };
    module.classes.get(id.0).map(|class| class.name.as_str())
}

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

fn atomic_scalar(module: &Module, ty: &Type) -> Option<&'static str> {
    let Type::Class(id) = ty else { return None };
    let class = &module.classes[id.0];
    if class.pos.file != "typegpu-types.ts" {
        return None;
    }
    match class.name.as_str() {
        "AtomicU32" => Some("u32"),
        "AtomicI32" => Some("i32"),
        _ => None,
    }
}

fn type_contains_atomic(module: &Module, ty: &Type) -> bool {
    fn visit(module: &Module, ty: &Type, seen: &mut BTreeSet<usize>) -> bool {
        if atomic_scalar(module, ty).is_some() {
            return true;
        }
        match ty {
            Type::FixedArray(item, _) | Type::Array(item) => visit(module, item, seen),
            Type::Class(id) if seen.insert(id.0) => module.classes[id.0]
                .fields
                .iter()
                .any(|field| visit(module, &field.ty, seen)),
            _ => false,
        }
    }
    visit(module, ty, &mut BTreeSet::new())
}

fn is_private_var(module: &Module, ty: &Type) -> bool {
    library_class(module, ty).is_some_and(|class| class.name.starts_with("PrivateVar<"))
}

#[derive(Debug, Clone)]
enum KernelGlobalKind {
    Constant(Expr),
    Private(Expr),
    WorkgroupVar,
    WorkgroupArray(u32),
}

#[derive(Debug, Clone)]
struct KernelGlobal {
    name: String,
    ty: Type,
    kind: KernelGlobalKind,
    pos: Pos,
}

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

fn wrapper_item_type(module: &Module, ty: &Type, field_name: &str) -> Option<Type> {
    let Type::Class(id) = ty else { return None };
    let field = module.classes[id.0]
        .fields
        .iter()
        .find(|field| field.name == field_name)?;
    match &field.ty {
        Type::Array(item) => Some((**item).clone()),
        item => Some(item.clone()),
    }
}

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
                wrapper_item_type(module, &global.ty, "value").ok_or_else(|| {
                    diagnostic(
                        "K20",
                        "private variable has no value type",
                        global.pos.clone(),
                    )
                })?,
                KernelGlobalKind::Private(init.clone()),
            ),
            Some(("workgroupVar", [])) => (
                wrapper_item_type(module, &global.ty, "values").ok_or_else(|| {
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
                    wrapper_item_type(module, &global.ty, "values").ok_or_else(|| {
                        diagnostic(
                            "K20",
                            "workgroup array has no item type",
                            global.pos.clone(),
                        )
                    })?,
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

fn expression_blocks_host(module: &Module, expression: &Expr) -> bool {
    match &expression.kind {
        ExprKind::AbsenceTest { value: operand, .. }
        | ExprKind::Unary { operand, .. }
        | ExprKind::Cast(operand)
        | ExprKind::Length(operand)
        | ExprKind::Field { obj: operand, .. }
        | ExprKind::JsonResultValue(operand) => expression_blocks_host(module, operand),
        ExprKind::Binary { left, right, .. }
        | ExprKind::Assign {
            target: left,
            value: right,
            ..
        } => expression_blocks_host(module, left) || expression_blocks_host(module, right),
        ExprKind::Call { callee, args } => {
            let callee_blocks = match callee {
                Callee::Func(name) => {
                    matches!(
                        crate::base_name(name),
                        "workgroupBarrier" | "storageBarrier"
                    ) && function_declared_in(module, name, "typegpu.ts")
                }
                Callee::Method { recv, name } => {
                    atomic_scalar(module, &recv.ty).is_some()
                        || (name == "$=" && is_private_var(module, &recv.ty))
                        || expression_blocks_host(module, recv)
                }
                Callee::Value(value) => expression_blocks_host(module, value),
                _ => false,
            };
            callee_blocks || args.iter().any(|arg| expression_blocks_host(module, arg))
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => {
            args.iter().any(|arg| expression_blocks_host(module, arg))
        }
        ExprKind::DescriptorLit { fields, .. } => fields
            .iter()
            .flatten()
            .any(|value| expression_blocks_host(module, value)),
        ExprKind::Index { obj, index, .. } => {
            expression_blocks_host(module, obj) || expression_blocks_host(module, index)
        }
        ExprKind::Cond { cond, then, els } => {
            expression_blocks_host(module, cond)
                || expression_blocks_host(module, then)
                || expression_blocks_host(module, els)
        }
        _ => false,
    }
}

fn statements_block_host(module: &Module, statements: &[Stmt]) -> bool {
    statements.iter().any(|statement| match statement {
        Stmt::Let { init, .. } | Stmt::Expr(init) => expression_blocks_host(module, init),
        Stmt::Return { value, .. } => value
            .as_ref()
            .is_some_and(|value| expression_blocks_host(module, value)),
        Stmt::If {
            cond, then, els, ..
        } => {
            expression_blocks_host(module, cond)
                || statements_block_host(module, then)
                || els
                    .as_ref()
                    .is_some_and(|items| statements_block_host(module, items))
        }
        Stmt::While { cond, body, .. } => {
            expression_blocks_host(module, cond) || statements_block_host(module, body)
        }
        Stmt::For {
            init,
            cond,
            step,
            body,
            ..
        } => {
            init.as_deref()
                .is_some_and(|item| statements_block_host(module, std::slice::from_ref(item)))
                || cond
                    .as_ref()
                    .is_some_and(|value| expression_blocks_host(module, value))
                || step
                    .as_ref()
                    .is_some_and(|value| expression_blocks_host(module, value))
                || statements_block_host(module, body)
        }
        Stmt::ForOf { subject, body, .. } => {
            expression_blocks_host(module, subject) || statements_block_host(module, body)
        }
        Stmt::Switch { disc, cases, .. } => {
            expression_blocks_host(module, disc)
                || cases.iter().any(|case| {
                    case.test
                        .as_ref()
                        .is_some_and(|test| expression_blocks_host(module, test))
                        || statements_block_host(module, &case.body)
                })
        }
        Stmt::Block(body) => statements_block_host(module, body),
        Stmt::Break(_) | Stmt::Continue(_) => false,
    })
}

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
    if statements_block_host(module, &kernel.body) {
        return Ok(false);
    }
    for helper in dependencies(module, kernel, shells)? {
        if crate::shell::function_is_shell(shells, &helper) {
            continue;
        }
        if function(module, &helper)
            .is_some_and(|function| statements_block_host(module, &function.body))
        {
            return Ok(false);
        }
    }
    Ok(true)
}

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
            let class = &module.classes[id.0];
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
                    type_name(module, ty)
                ),
                pos.clone(),
            ))
        }
    })
}

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

fn type_name(module: &Module, ty: &Type) -> String {
    subscript_compiler::types::display_type(
        ty,
        &|id| module.classes[id.0].name.clone(),
        &|id| module.enums[id.0].name.clone(),
        &|id| module.string_aliases[id.0].name.clone(),
    )
}

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

fn mixed_bitwise_chain(parent: BinOp, operand: &Expr) -> bool {
    let ExprKind::Binary { op: child, .. } = &operand.kind else {
        return false;
    };
    is_bitwise_operator(parent)
        && (is_arithmetic_or_comparison_operator(*child)
            || (is_bitwise_operator(*child) && parent != *child))
}

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

fn f32_literal(value: f64) -> String {
    let mut text = value.to_string();
    if !text.contains('.') && !text.contains('e') && !text.contains('E') {
        text.push_str(".0");
    }
    format!("{text}f")
}

fn constant_type(module: &Module, ty: &Type) -> bool {
    match ty {
        Type::F32 | Type::I32 | Type::U32 | Type::Bool => true,
        Type::FixedArray(item, _) => constant_type(module, item),
        Type::Class(id) => {
            let class = &module.classes[id.0];
            class.pos.file == "typegpu-types.ts"
                && (class.name.starts_with("Vec") || class.name.starts_with("Mat"))
                && atomic_scalar(module, ty).is_none()
        }
        _ => false,
    }
}

#[derive(Clone)]
enum FoldedConstant {
    Bool(bool),
    I32(i32),
    U32(u32),
    F32(f32),
    Construct {
        constructor: String,
        args: Vec<FoldedConstant>,
    },
}

impl FoldedConstant {
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

fn constant_overflow(ty: &str, pos: &Pos) -> Diagnostic {
    diagnostic(
        "K19",
        format!("module constant arithmetic overflows {ty}"),
        pos.clone(),
    )
}

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
            let class = &module.classes[class.0];
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
    if !constant_type(module, &global.ty) {
        return Err(diagnostic(
            "K19",
            format!(
                "module constant `{name}` has unsupported type `{}`",
                type_name(module, &global.ty)
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
            let class = &module.classes[class.0];
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
                if !constant_type(module, &global.ty) {
                    return Err(diagnostic(
                        "K19",
                        format!(
                            "module constant `{}` has unsupported type `{}`",
                            global.name,
                            type_name(module, &global.ty)
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
                if type_contains_atomic(module, &global.ty) {
                    return Err(diagnostic(
                        "K21",
                        "an atomic value cannot use private address space",
                        global.pos.clone(),
                    ));
                }
                let ty = wgsl_type(module, &global.ty, &global.pos)?;
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
fn case_terminates(body: &[Stmt]) -> bool {
    match body.last() {
        Some(Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return { .. }) => true,
        Some(Stmt::Block(body)) => case_terminates(body),
        _ => false,
    }
}

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

#[derive(Debug, Clone)]
struct BindingRef {
    name: String,
    kind: BindingKind,
    item_ty: Type,
}

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

struct Emitter<'a> {
    module: &'a Module,
    layout_params: BTreeMap<String, usize>,
    layout_names: BTreeSet<String>,
    invocation_param: String,
    invocation_kind: InvocationKind,
    bindings: BTreeMap<(usize, String), BindingRef>,
    globals: BTreeMap<String, KernelGlobal>,
    local_names: BTreeMap<String, String>,
    used_builtins: BTreeSet<String>,
    conditional_index: u32,
    loop_depth: u32,
    switch_depth: u32,
    in_helper: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InvocationKind {
    None,
    Compute,
    Vertex,
    Fragment,
}

impl<'a> Emitter<'a> {
    fn entry(
        module: &'a Module,
        layouts: &'a [crate::pipeline::Layout],
        kernel: &Function,
        invocation_index: usize,
        invocation_kind: InvocationKind,
        globals: &[KernelGlobal],
        module_names: &BTreeSet<String>,
    ) -> Self {
        let mut layout_params = BTreeMap::new();
        let mut bindings = BTreeMap::new();
        for (group, layout) in layouts.iter().enumerate() {
            layout_params.insert(kernel.params[group].name.clone(), group);
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
        Self {
            module,
            layout_params,
            layout_names: layouts.iter().map(|layout| layout.name.clone()).collect(),
            invocation_param: kernel.params[invocation_index].name.clone(),
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
        }
    }

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

    fn local_name(&self, name: &str) -> String {
        self.local_names
            .get(name)
            .cloned()
            .unwrap_or_else(|| mapping::ident(name))
    }

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

    fn wrapper_ref(&self, expr: &Expr) -> Option<KernelGlobal> {
        let ExprKind::Global(name) = &expr.kind else {
            return None;
        };
        self.globals.get(name).and_then(|global| {
            (!matches!(global.kind, KernelGlobalKind::Constant(_))).then(|| global.clone())
        })
    }

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
                    && type_contains_atomic(self.module, &target.ty)
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
                let class = &self.module.classes[class.0];
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
                    return self.fround_argument(&args[0]);
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
                if atomic_scalar(self.module, &recv.ty).is_some() {
                    let place = self.atomic_place(recv)?;
                    let (args, args_prelude) = self.snippets(args)?;
                    let mut prelude = place.prelude;
                    prelude.extend(args_prelude);
                    let receiver = class_name(self.module, &recv.ty)
                        .expect("atomic scalar receiver has a class name");
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
                            if type_contains_atomic(self.module, &global.ty) {
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
                            if type_contains_atomic(self.module, &global.ty) {
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
                            if type_contains_atomic(self.module, &binding.item_ty) {
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
                        (BindingKind::Texture(_, _), "sampleLevel", [sampler, uv, level]) => {
                            format!(
                                "textureSampleLevel({}, {sampler}, {uv}, {level})",
                                binding.name
                            )
                        }
                        (BindingKind::Texture(_, _), "sample", [sampler, uv]) => {
                            if self.invocation_kind != InvocationKind::Fragment {
                                return Err(diagnostic(
                                    "TX3",
                                    "Texture2d.sample is legal only in a fragment kernel",
                                    expr.pos.clone(),
                                ));
                            }
                            format!("textureSample({}, {sampler}, {uv})", binding.name)
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
                                "TX11",
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
                                "TX11",
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
                let library = matches!(&recv.ty, Type::Class(id) if self.module.classes[id.0].pos.file == "typegpu-types.ts");
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
                        let arg = binary_operand(&arg_values[0], precedence, true, false);
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

    fn line(out: &mut String, indent: usize, text: &str) {
        out.push_str(&"  ".repeat(indent));
        out.push_str(text);
        out.push('\n');
    }

    fn emit_prelude(out: &mut String, indent: usize, prelude: Prelude) {
        for (relative, text) in prelude {
            Self::line(out, indent + relative, &text);
        }
    }

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
                if atomic_scalar(self.module, ty).is_none() && type_contains_atomic(self.module, ty)
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
                let value_class =
                    matches!(ty, Type::Class(id) if self.module.classes[id.0].is_value);
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
                        let value_class =
                            matches!(ty, Type::Class(id) if self.module.classes[id.0].is_value);
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
                    let selector = if labels.len() == 1 && labels[0] == "default" {
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum UniformityTaint {
    Uniform,
    NonUniform(String),
}

impl UniformityTaint {
    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::NonUniform(reason), _) | (_, Self::NonUniform(reason)) => {
                Self::NonUniform(reason)
            }
            (Self::Uniform, Self::Uniform) => Self::Uniform,
        }
    }

    fn reason(&self) -> Option<&str> {
        match self {
            Self::Uniform => None,
            Self::NonUniform(reason) => Some(reason),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum UniformityTarget {
    Loop { has_barrier: bool },
    Switch,
}

// K22 uses a conservative taint analysis that accepts no non-uniform barrier placement.
struct BarrierValidator<'emitter, 'module> {
    emitter: &'emitter Emitter<'module>,
    locals: BTreeMap<String, UniformityTaint>,
    last_barrier: Option<(u32, u32)>,
}

impl<'emitter, 'module> BarrierValidator<'emitter, 'module> {
    fn new(emitter: &'emitter Emitter<'module>, kernel: &Function) -> Self {
        Self {
            emitter,
            locals: BTreeMap::new(),
            last_barrier: last_barrier_position(emitter.module, &kernel.body),
        }
    }

    fn validate(mut self, kernel: &Function) -> Result<(), Diagnostic> {
        self.collect_statements(&kernel.body, UniformityTaint::Uniform);
        self.validate_statements(
            &kernel.body,
            UniformityTaint::Uniform,
            true,
            &mut Vec::new(),
        )
    }

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

    fn expressions(&self, expressions: &[Expr]) -> UniformityTaint {
        expressions
            .iter()
            .fold(UniformityTaint::Uniform, |taint, expr| {
                taint.merge(self.expression(expr))
            })
    }

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

fn assigned_local(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Local(name) => Some(name),
        ExprKind::Field { obj, .. } | ExprKind::Index { obj, .. } => assigned_local(obj),
        _ => None,
    }
}

fn written_locals_expr(expr: &Expr, out: &mut BTreeSet<String>) {
    if let ExprKind::Assign { target, .. } = &expr.kind {
        if let Some(name) = assigned_local(target) {
            out.insert(name.to_owned());
        }
    }
}

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

fn collect_schema_type(
    module: &Module,
    ty: &Type,
    seen: &mut BTreeSet<String>,
    out: &mut Vec<String>,
) {
    match ty {
        Type::Class(id) => {
            let class = &module.classes[id.0];
            if class.is_value
                && class.pos.file != "typegpu-types.ts"
                && class.pos.file != "typegpu.ts"
                && seen.insert(class.name.clone())
            {
                out.push(class.name.clone());
            }
        }
        Type::FixedArray(item, _) => collect_schema_type(module, item, seen, out),
        _ => {}
    }
}

fn collect_schema_expr(
    module: &Module,
    expr: &Expr,
    seen: &mut BTreeSet<String>,
    out: &mut Vec<String>,
) {
    collect_schema_type(module, &expr.ty, seen, out);
    match &expr.kind {
        ExprKind::AbsenceTest { value: operand, .. }
        | ExprKind::Unary { operand, .. }
        | ExprKind::Cast(operand)
        | ExprKind::Length(operand)
        | ExprKind::Field { obj: operand, .. }
        | ExprKind::JsonResultValue(operand) => {
            collect_schema_expr(module, operand, seen, out);
        }
        ExprKind::Binary { left, right, .. } => {
            collect_schema_expr(module, left, seen, out);
            collect_schema_expr(module, right, seen, out);
        }
        ExprKind::Assign { target, value, .. } => {
            collect_schema_expr(module, target, seen, out);
            collect_schema_expr(module, value, seen, out);
        }
        ExprKind::Call { callee, args } => {
            if let Callee::Value(value) = callee {
                collect_schema_expr(module, value, seen, out);
            }
            if let Callee::Method { recv, .. } = callee {
                collect_schema_expr(module, recv, seen, out);
            }
            for arg in args {
                collect_schema_expr(module, arg, seen, out);
            }
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => {
            for arg in args {
                collect_schema_expr(module, arg, seen, out);
            }
        }
        ExprKind::DescriptorLit { fields, .. } => {
            for field in fields.iter().flatten() {
                collect_schema_expr(module, field, seen, out);
            }
        }
        ExprKind::Index { obj, index, .. } => {
            collect_schema_expr(module, obj, seen, out);
            collect_schema_expr(module, index, seen, out);
        }
        ExprKind::Cond { cond, then, els } => {
            collect_schema_expr(module, cond, seen, out);
            collect_schema_expr(module, then, seen, out);
            collect_schema_expr(module, els, seen, out);
        }
        ExprKind::Template(parts) => {
            for part in parts {
                if let subscript_compiler::hir::TplPart::Expr(value) = part {
                    collect_schema_expr(module, value, seen, out);
                }
            }
        }
        ExprKind::Lambda { body, .. } => {
            for stmt in body {
                collect_schema_stmt(module, stmt, seen, out);
            }
        }
        _ => {}
    }
}

fn collect_schema_stmt(
    module: &Module,
    stmt: &Stmt,
    seen: &mut BTreeSet<String>,
    out: &mut Vec<String>,
) {
    match stmt {
        Stmt::Let { ty, init, .. } => {
            collect_schema_type(module, ty, seen, out);
            collect_schema_expr(module, init, seen, out);
        }
        Stmt::Expr(expr) => collect_schema_expr(module, expr, seen, out),
        Stmt::Return {
            value: Some(value), ..
        } => collect_schema_expr(module, value, seen, out),
        Stmt::Return { value: None, .. } => {}
        Stmt::If {
            cond, then, els, ..
        } => {
            collect_schema_expr(module, cond, seen, out);
            for stmt in then {
                collect_schema_stmt(module, stmt, seen, out);
            }
            if let Some(els) = els {
                for stmt in els {
                    collect_schema_stmt(module, stmt, seen, out);
                }
            }
        }
        Stmt::While { cond, body, .. } => {
            collect_schema_expr(module, cond, seen, out);
            for stmt in body {
                collect_schema_stmt(module, stmt, seen, out);
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
                collect_schema_stmt(module, init, seen, out);
            }
            if let Some(cond) = cond {
                collect_schema_expr(module, cond, seen, out);
            }
            if let Some(step) = step {
                collect_schema_expr(module, step, seen, out);
            }
            for stmt in body {
                collect_schema_stmt(module, stmt, seen, out);
            }
        }
        Stmt::ForOf {
            ty, subject, body, ..
        } => {
            collect_schema_type(module, ty, seen, out);
            collect_schema_expr(module, subject, seen, out);
            for stmt in body {
                collect_schema_stmt(module, stmt, seen, out);
            }
        }
        Stmt::Switch { disc, cases, .. } => {
            collect_schema_expr(module, disc, seen, out);
            for stmt in cases.iter().flat_map(|case| &case.body) {
                collect_schema_stmt(module, stmt, seen, out);
            }
        }
        Stmt::Block(body) => {
            for stmt in body {
                collect_schema_stmt(module, stmt, seen, out);
            }
        }
        Stmt::Break(_) | Stmt::Continue(_) => {}
    }
}

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
            collect_schema_type(module, &binding.item_ty, &mut seen, &mut out);
        }
    }
    for global in kernel_globals(module, kernel, shells)? {
        collect_schema_type(module, &global.ty, &mut seen, &mut out);
    }
    for name in dependencies(module, kernel, shells)? {
        let helper = function(module, &name).ok_or_else(|| {
            generator_diagnostic("a helper disappeared from typed HIR", pipeline.pos.clone())
        })?;
        for param in &helper.params {
            collect_schema_type(module, &param.ty, &mut seen, &mut out);
        }
        collect_schema_type(module, &helper.ret, &mut seen, &mut out);
        if !crate::shell::function_is_shell(shells, &name) {
            for stmt in &helper.body {
                collect_schema_stmt(module, stmt, &mut seen, &mut out);
            }
        }
    }
    for stmt in &kernel.body {
        collect_schema_stmt(module, stmt, &mut seen, &mut out);
    }
    Ok(out)
}

fn dependencies(
    module: &Module,
    kernel: &Function,
    shells: &crate::shell::ShellProgram,
) -> Result<Vec<String>, Diagnostic> {
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
            let mut cycle = stack[start..].to_vec();
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

fn next_line(out: &str) -> u32 {
    out.bytes().filter(|byte| *byte == b'\n').count() as u32 + 1
}

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
        let line = &line[start..];
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
            && type_contains_atomic(module, &binding.item_ty)
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
    );
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
    let mut entry_body = String::new();
    if pipeline.guarded {
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
    BarrierValidator::new(&emitter, kernel).validate(kernel)?;

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

pub(crate) fn referenced_render_schema_names(
    module: &Module,
    pipeline: &RenderPipeline,
    shells: &crate::shell::ShellProgram,
) -> Result<Vec<String>, Diagnostic> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for layout in &pipeline.layouts {
        for binding in &layout.bindings {
            collect_schema_type(module, &binding.item_ty, &mut seen, &mut out);
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
                collect_schema_type(module, &param.ty, &mut seen, &mut out);
            }
            collect_schema_type(module, &helper.ret, &mut seen, &mut out);
            if !crate::shell::function_is_shell(shells, &name) {
                for stmt in &helper.body {
                    collect_schema_stmt(module, stmt, &mut seen, &mut out);
                }
            }
        }
        for stmt in &kernel.body {
            collect_schema_stmt(module, stmt, &mut seen, &mut out);
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
        collect_schema_type(module, &global.ty, &mut seen, &mut out);
    }
    let interface_names = pipeline
        .vertex_buffers
        .iter()
        .map(|buffer| buffer.schema.as_str())
        .chain(std::iter::once(pipeline.varyings_name.as_str()))
        .collect::<BTreeSet<_>>();
    out.retain(|name| !interface_names.contains(name.as_str()));
    Ok(out)
}

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
    );
    let mut fragment_emitter = Emitter::entry(
        module,
        &pipeline.layouts,
        fragment,
        layout_count + 1,
        InvocationKind::Fragment,
        &globals,
        &module_names,
    );
    let mut vertex_body = String::new();
    vertex_emitter.statements(&vertex.body, 1, &mut vertex_body)?;
    let mut fragment_body = String::new();
    fragment_emitter.statements(&fragment.body, 1, &mut fragment_body)?;

    let selected_names = structs
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<BTreeSet<_>>();
    let uses_f16 = schemas
        .iter()
        .filter(|schema| selected_names.contains(schema.name.as_str()))
        .any(|schema| crate::emit::uses_f16(&schema.tree))
        || pipeline
            .layouts
            .iter()
            .flat_map(|layout| &layout.bindings)
            .any(|binding| crate::render::type_uses_f16(module, &binding.item_ty))
        || pipeline
            .vertex_buffers
            .iter()
            .flat_map(|buffer| &buffer.attributes)
            .any(|attribute| attribute.format.starts_with("float16"))
        || pipeline
            .varyings
            .iter()
            .any(|varying| crate::render::type_uses_f16(module, &varying.ty));
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

    let mut vertex_parameters = vertex.params[layout_count..layout_count + vertex_value_count]
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
    out.push_str("@vertex\n");
    out.push_str(&format!(
        "fn {}({}) -> {} {{\n",
        mapping::ident(&pipeline.vertex_entry),
        vertex_parameters.join(", "),
        mapping::ident(&pipeline.varyings_name)
    ));
    out.push_str(&vertex_body);
    out.push_str("}\n\n");

    let input = &fragment.params[layout_count];
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
