//! Pipeline declarations, layouts, and binding-wrapper discovery.

use std::collections::BTreeSet;

use subscript_compiler::hir::{Callee, Expr, ExprKind, Function, Module, Stmt};
use subscript_compiler::{Diagnostic, Pos, RuleCode, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindingKind {
    Uniform,
    Storage,
    MutStorage,
}

impl BindingKind {
    pub(crate) fn wgsl(self) -> &'static str {
        match self {
            Self::Uniform => "uniform",
            Self::Storage => "storage, read",
            Self::MutStorage => "storage, read_write",
        }
    }

    pub(crate) fn webgpu(self) -> &'static str {
        match self {
            Self::Uniform => "uniform",
            Self::Storage => "read-only-storage",
            Self::MutStorage => "storage",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Binding {
    pub(crate) name: String,
    pub(crate) index: u32,
    pub(crate) kind: BindingKind,
    pub(crate) item_ty: Type,
    pub(crate) pos: Pos,
}

#[derive(Debug, Clone)]
pub(crate) struct Layout {
    pub(crate) name: String,
    pub(crate) group: u32,
    pub(crate) bindings: Vec<Binding>,
}

#[derive(Debug, Clone)]
pub(crate) struct Pipeline {
    pub(crate) declaration: String,
    pub(crate) entry: String,
    pub(crate) workgroup: [u32; 3],
    pub(crate) layouts: Vec<Layout>,
    pub(crate) pos: Pos,
}

fn diagnostic(rule: &str, message: impl Into<String>, pos: Pos) -> Diagnostic {
    Diagnostic::new(
        RuleCode::S100,
        format!("{rule}: {} (author)", message.into()),
        pos,
    )
}

pub(crate) fn class_name<'a>(module: &'a Module, ty: &Type) -> Option<&'a str> {
    let Type::Class(id) = ty else { return None };
    module.classes.get(id.0).map(|class| class.name.as_str())
}

pub(crate) fn type_name(module: &Module, ty: &Type) -> String {
    subscript_compiler::types::display_type(
        ty,
        &|id| module.classes[id.0].name.clone(),
        &|id| module.enums[id.0].name.clone(),
        &|id| module.string_aliases[id.0].name.clone(),
    )
}

pub(crate) fn library_class<'a>(
    module: &'a Module,
    ty: &Type,
) -> Option<&'a subscript_compiler::hir::ClassDef> {
    let Type::Class(id) = ty else { return None };
    module
        .classes
        .get(id.0)
        .filter(|class| class.pos.file == "typegpu.ts")
}

fn wrapper(module: &Module, ty: &Type) -> Option<(BindingKind, Type)> {
    let class = library_class(module, ty)?;
    let kind = if class.name.starts_with("Uniform<") {
        BindingKind::Uniform
    } else if class.name.starts_with("Storage<") {
        BindingKind::Storage
    } else if class.name.starts_with("MutStorage<") {
        BindingKind::MutStorage
    } else {
        return None;
    };
    let Type::Array(item) = &class.fields.iter().find(|field| field.name == "values")?.ty else {
        return None;
    };
    Some((kind, (**item).clone()))
}

fn allowed_binding_item(module: &Module, ty: &Type) -> bool {
    match ty {
        Type::F32 | Type::I32 | Type::U32 => true,
        Type::Class(id) => {
            let class = &module.classes[id.0];
            class.is_value && (class.pos.file != "typegpu.ts") && (class.pos.file != "webgpu.ts")
        }
        _ => false,
    }
}

pub(crate) fn layout(module: &Module, ty: &Type, group: u32) -> Result<Layout, Diagnostic> {
    let Type::Class(id) = ty else {
        return Err(diagnostic(
            "PI3",
            "pipeline layout is not a class",
            Pos::new("", 1, 1),
        ));
    };
    let class = &module.classes[id.0];
    if class.is_value || class.is_descriptor || class.pos.file == "typegpu.ts" {
        return Err(diagnostic(
            "PI3",
            format!("`{}` is not a plain layout class", class.name),
            class.pos.clone(),
        ));
    }
    if class.ctor.is_some() || !class.methods.is_empty() || class.index_signature.is_some() {
        return Err(diagnostic(
            "PI3",
            format!("layout class `{}` contains a non-field member", class.name),
            class.pos.clone(),
        ));
    }
    let mut bindings = Vec::new();
    for (index, field) in class.fields.iter().enumerate() {
        let Some((kind, item_ty)) = wrapper(module, &field.ty) else {
            return Err(diagnostic(
                "PI13",
                format!(
                    "layout field `{}.{}` is not a Uniform, Storage, or MutStorage binding wrapper",
                    class.name, field.name
                ),
                field.pos.clone(),
            ));
        };
        if !allowed_binding_item(module, &item_ty) {
            return Err(diagnostic(
                "PI13",
                format!(
                    "layout field `{}.{}` has a binding item type outside PI13",
                    class.name, field.name
                ),
                field.pos.clone(),
            ));
        }
        bindings.push(Binding {
            name: field.name.clone(),
            index: index as u32,
            kind,
            item_ty,
            pos: field.pos.clone(),
        });
    }
    Ok(Layout {
        name: class.name.clone(),
        group,
        bindings,
    })
}

fn literal_u32(expr: &Expr) -> Option<u32> {
    let ExprKind::Int(value) = expr.kind else {
        return None;
    };
    u32::try_from(value).ok()
}

fn workgroup(module: &Module, expr: &Expr) -> Result<[u32; 3], Diagnostic> {
    let ExprKind::DescriptorLit { class, fields } = &expr.kind else {
        return Err(diagnostic(
            "PI13",
            "pipeline workgroup options must be a descriptor literal",
            expr.pos.clone(),
        ));
    };
    let descriptor = &module.classes[class.0];
    let Some(index) = descriptor
        .fields
        .iter()
        .position(|field| field.name == "workgroupSize")
    else {
        return Err(diagnostic(
            "PI13",
            "pipeline options omit workgroupSize",
            expr.pos.clone(),
        ));
    };
    let Some(Some(value)) = fields.get(index) else {
        return Err(diagnostic(
            "PI13",
            "pipeline workgroup size is not literal",
            expr.pos.clone(),
        ));
    };
    let ExprKind::ArrayLit(values) = &value.kind else {
        return Err(diagnostic(
            "PI13",
            "pipeline workgroup size is not literal",
            value.pos.clone(),
        ));
    };
    if values.len() != 3 {
        return Err(diagnostic(
            "PI13",
            "pipeline workgroup size requires three literals",
            value.pos.clone(),
        ));
    }
    let Some(x) = literal_u32(&values[0]) else {
        return Err(diagnostic(
            "PI13",
            "pipeline workgroup size is not literal",
            values[0].pos.clone(),
        ));
    };
    let Some(y) = literal_u32(&values[1]) else {
        return Err(diagnostic(
            "PI13",
            "pipeline workgroup size is not literal",
            values[1].pos.clone(),
        ));
    };
    let Some(z) = literal_u32(&values[2]) else {
        return Err(diagnostic(
            "PI13",
            "pipeline workgroup size is not literal",
            values[2].pos.clone(),
        ));
    };
    if x == 0 || y == 0 || z == 0 {
        return Err(diagnostic(
            "PI13",
            "pipeline workgroup dimensions must be nonzero",
            value.pos.clone(),
        ));
    }
    Ok([x, y, z])
}

pub(crate) fn function<'a>(module: &'a Module, name: &str) -> Option<&'a Function> {
    module
        .functions
        .iter()
        .find(|function| function.name == name)
}

fn compute_arity(name: &str) -> Option<usize> {
    let base = name.split('<').next().unwrap_or(name);
    Some(match base {
        "computePipeline" => 1,
        "computePipeline2" => 2,
        "computePipeline3" => 3,
        "computePipeline4" => 4,
        _ => return None,
    })
}

fn call_in_expr(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Call { callee: Callee::Func(name), .. } if compute_arity(name).is_some() => true,
        ExprKind::Unary { operand, .. } | ExprKind::Cast(operand) | ExprKind::Length(operand) => call_in_expr(operand),
        ExprKind::Binary { left, right, .. } => call_in_expr(left) || call_in_expr(right),
        ExprKind::Assign { target, value, .. } => call_in_expr(target) || call_in_expr(value),
        ExprKind::Call { callee, args } => {
            matches!(callee, Callee::Value(value) if call_in_expr(value))
                || matches!(callee, Callee::Method { recv, .. } if call_in_expr(recv))
                || args.iter().any(call_in_expr)
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => args.iter().any(call_in_expr),
        ExprKind::DescriptorLit { fields, .. } => fields.iter().flatten().any(call_in_expr),
        ExprKind::Field { obj, .. } | ExprKind::JsonResultValue(obj) => call_in_expr(obj),
        ExprKind::Index { obj, index, .. } => call_in_expr(obj) || call_in_expr(index),
        ExprKind::Template(parts) => parts.iter().any(|part| matches!(part, subscript_compiler::hir::TplPart::Expr(value) if call_in_expr(value))),
        ExprKind::Lambda { body, .. } => body.iter().any(stmt_has_compute),
        ExprKind::Cond { cond, then, els } => call_in_expr(cond) || call_in_expr(then) || call_in_expr(els),
        _ => false,
    }
}

fn stmt_has_compute(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Let { init, .. } | Stmt::Expr(init) => call_in_expr(init),
        Stmt::Return { value, .. } => value.as_ref().is_some_and(call_in_expr),
        Stmt::If {
            cond, then, els, ..
        } => {
            call_in_expr(cond)
                || then.iter().any(stmt_has_compute)
                || els
                    .as_ref()
                    .is_some_and(|items| items.iter().any(stmt_has_compute))
        }
        Stmt::While { cond, body, .. } => call_in_expr(cond) || body.iter().any(stmt_has_compute),
        Stmt::For {
            init,
            cond,
            step,
            body,
            ..
        } => {
            init.as_deref().is_some_and(stmt_has_compute)
                || cond.as_ref().is_some_and(call_in_expr)
                || step.as_ref().is_some_and(call_in_expr)
                || body.iter().any(stmt_has_compute)
        }
        Stmt::ForOf { subject, body, .. } => {
            call_in_expr(subject) || body.iter().any(stmt_has_compute)
        }
        Stmt::Switch { disc, cases, .. } => {
            call_in_expr(disc)
                || cases
                    .iter()
                    .flat_map(|case| &case.body)
                    .any(stmt_has_compute)
        }
        Stmt::Block(body) => body.iter().any(stmt_has_compute),
        Stmt::Break(_) | Stmt::Continue(_) => false,
        _ => false,
    }
}

pub(crate) fn discover(module: &Module) -> Result<Vec<Pipeline>, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    for function in &module.functions {
        if function.pos.file != "typegpu.ts" && function.body.iter().any(stmt_has_compute) {
            diagnostics.push(diagnostic(
                "PI13",
                "a pipeline declaration appears inside a function",
                function.pos.clone(),
            ));
        }
    }
    let mut pipelines = Vec::new();
    for global in &module.globals {
        let ExprKind::Call {
            callee: Callee::Func(callee),
            args,
        } = &global.init.kind
        else {
            continue;
        };
        let Some(arity) = compute_arity(callee) else {
            continue;
        };
        if global.mutable {
            diagnostics.push(diagnostic(
                "PI1",
                "a pipeline declaration must be const",
                global.pos.clone(),
            ));
            continue;
        }
        let Some(Expr {
            kind: ExprKind::FuncRef(entry),
            ..
        }) = args.first()
        else {
            diagnostics.push(diagnostic(
                "K1",
                "pipeline kernel is not a named function",
                global.init.pos.clone(),
            ));
            continue;
        };
        let Some(kernel) = function(module, entry) else {
            diagnostics.push(Diagnostic::new(
                RuleCode::S100,
                format!("K15: kernel `{entry}` disappeared from typed HIR (generator)"),
                global.init.pos.clone(),
            ));
            continue;
        };
        let declared_layouts = function(module, callee)
            .and_then(|declaration| declaration.params.first())
            .and_then(|parameter| match &parameter.ty {
                Type::Func(signature) => Some(&signature.params),
                _ => None,
            });
        if kernel.params.len() != arity + 1 {
            diagnostics.push(Diagnostic::new(
                RuleCode::S100,
                format!("K15: kernel `{entry}` has an impossible parameter count (generator)"),
                kernel.pos.clone(),
            ));
            continue;
        }
        if let Some(declared_layouts) = declared_layouts {
            let mut mismatch = false;
            for (index, (declared, actual)) in declared_layouts
                .iter()
                .take(arity)
                .zip(&kernel.params[..arity])
                .enumerate()
            {
                if declared != &actual.ty {
                    diagnostics.push(diagnostic(
                        "PI13",
                        format!(
                            "pipeline layout type argument {} is `{}` but kernel parameter is `{}`",
                            index + 1,
                            type_name(module, declared),
                            type_name(module, &actual.ty),
                        ),
                        global.init.pos.clone(),
                    ));
                    mismatch = true;
                }
            }
            if mismatch {
                continue;
            }
        }
        let mut layouts = Vec::new();
        for (group, param) in kernel.params[..arity].iter().enumerate() {
            match layout(module, &param.ty, group as u32) {
                Ok(layout) => layouts.push(layout),
                Err(error) => diagnostics.push(error),
            }
        }
        let invocation_ok = class_name(module, &kernel.params[arity].ty)
            .is_some_and(|name| name == "ComputeInvocation")
            && library_class(module, &kernel.params[arity].ty).is_some();
        if !invocation_ok {
            diagnostics.push(Diagnostic::new(
                RuleCode::S100,
                format!("K15: kernel `{entry}` lost its ComputeInvocation parameter (generator)"),
                kernel.params[arity].pos.clone(),
            ));
            continue;
        }
        let Some(options) = args.get(1) else {
            diagnostics.push(diagnostic(
                "PI13",
                "pipeline declaration omits options",
                global.init.pos.clone(),
            ));
            continue;
        };
        match workgroup(module, options) {
            Ok(workgroup) if layouts.len() == arity => pipelines.push(Pipeline {
                declaration: global.name.clone(),
                entry: entry.clone(),
                workgroup,
                layouts,
                pos: global.pos.clone(),
            }),
            Ok(_) => {}
            Err(error) => diagnostics.push(error),
        }
    }
    if diagnostics.is_empty() {
        Ok(pipelines)
    } else {
        Err(diagnostics)
    }
}

pub(crate) fn schema_names(module: &Module, pipelines: &[Pipeline]) -> BTreeSet<String> {
    pipelines
        .iter()
        .flat_map(|pipeline| &pipeline.layouts)
        .flat_map(|layout| &layout.bindings)
        .filter_map(|binding| class_name(module, &binding.item_ty))
        .filter(|name| {
            module.classes.iter().any(|class| {
                class.name == **name && class.is_value && class.pos.file != "typegpu-types.ts"
            })
        })
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use subscript_compiler::SourceFile;

    use super::*;

    fn read(root: &std::path::Path, name: &str) -> String {
        std::fs::read_to_string(root.join("lib").join(name)).expect("read test library")
    }

    #[test]
    fn pi2_compares_declared_type_arguments_with_kernel_layouts() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("repository root")
            .to_path_buf();
        let files = vec![
            SourceFile::ambient(
                "subscript-typegpu.generated.d.ts",
                read(&root, "subscript-typegpu.generated.d.ts"),
            ),
            SourceFile::ambient(
                "wire-enum-aliases.generated.d.ts",
                read(&root, "wire-enum-aliases.generated.d.ts"),
            ),
            SourceFile::new("webgpu.ts", read(&root, "webgpu.ts")),
            SourceFile::new("typegpu-types.ts", read(&root, "typegpu-types.ts")),
            SourceFile::new("typegpu.ts", read(&root, "typegpu.ts")),
            SourceFile::new(
                "pi2-test.ts",
                r#"
import { ComputeInvocation, computePipeline, ComputePipelineSpec } from "./typegpu";
class Declared {}
class KernelLayout {}
function kernel(res: Declared, ctx: ComputeInvocation): void {}
export const pipeline: ComputePipelineSpec = computePipeline<Declared>(kernel, { workgroupSize: [1, 1, 1] });
"#,
            ),
        ];
        let mut module = subscript_compiler::check_program(&files).expect("valid PI2 seed HIR");
        let replacement = module
            .classes
            .iter()
            .position(|class| class.name == "KernelLayout")
            .expect("replacement class");
        let kernel = module
            .functions
            .iter_mut()
            .find(|function| function.name == "kernel")
            .expect("kernel");
        kernel.params[0].ty = Type::Class(subscript_compiler::ClassId(replacement));
        let diagnostics = discover(&module).expect_err("PI2 mismatch must fail");
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.starts_with("PI13:"));
        assert!(diagnostics[0].message.contains("`Declared`"));
        assert!(diagnostics[0].message.contains("`KernelLayout`"));
    }
}
