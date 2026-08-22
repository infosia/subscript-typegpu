//! Pipeline declarations, layouts, and binding-wrapper discovery.

use std::collections::BTreeSet;

use subscript_compiler::hir::{Callee, Expr, ExprKind, Function, Module, Stmt};
use subscript_compiler::{Diagnostic, Pos, RuleCode, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindingKind {
    Uniform,
    Storage,
    MutStorage,
    Texture(TextureSampleType),
    StorageTexture(StorageTextureFormat),
    Sampler,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextureSampleType {
    Float,
}

impl TextureSampleType {
    pub(crate) fn wgsl(self) -> &'static str {
        match self {
            Self::Float => "f32",
        }
    }

    pub(crate) fn webgpu(self) -> &'static str {
        match self {
            Self::Float => "float",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StorageTextureFormat {
    Rgba8unorm,
    Rgba16float,
    R32float,
    Rgba32float,
}

impl StorageTextureFormat {
    pub(crate) fn wgsl(self) -> &'static str {
        match self {
            Self::Rgba8unorm => "rgba8unorm",
            Self::Rgba16float => "rgba16float",
            Self::R32float => "r32float",
            Self::Rgba32float => "rgba32float",
        }
    }

    pub(crate) fn webgpu(self) -> &'static str {
        self.wgsl()
    }
}

impl BindingKind {
    pub(crate) fn wgsl(self) -> &'static str {
        match self {
            Self::Uniform => "uniform",
            Self::Storage => "storage, read",
            Self::MutStorage => "storage, read_write",
            Self::Texture(_) | Self::StorageTexture(_) | Self::Sampler => "",
        }
    }

    pub(crate) fn webgpu(self) -> &'static str {
        match self {
            Self::Uniform => "uniform",
            Self::Storage => "read-only-storage",
            Self::MutStorage => "storage",
            Self::Texture(_) => "texture",
            Self::StorageTexture(_) => "storageTexture",
            Self::Sampler => "sampler",
        }
    }

    pub(crate) fn is_buffer(self) -> bool {
        matches!(self, Self::Uniform | Self::Storage | Self::MutStorage)
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

fn generator_diagnostic(message: impl Into<String>, pos: Pos) -> Diagnostic {
    Diagnostic::new(
        RuleCode::S100,
        format!("K15: {} (generator)", message.into()),
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

fn wrapper(
    module: &Module,
    ty: &Type,
    pos: &Pos,
) -> Result<Option<(BindingKind, Type)>, Diagnostic> {
    if class_name(module, ty).is_some_and(|name| name == "ComparisonSampler") {
        return Err(diagnostic(
            "TX1",
            "comparison samplers are not supported in this revision",
            pos.clone(),
        ));
    }
    let Some(class) = library_class(module, ty) else {
        return Ok(None);
    };
    let kind = if class.name.starts_with("Uniform<") {
        BindingKind::Uniform
    } else if class.name.starts_with("Storage<") {
        BindingKind::Storage
    } else if class.name.starts_with("MutStorage<") {
        BindingKind::MutStorage
    } else if class.name.starts_with("Texture2d<") {
        let Some(values) = class.fields.iter().find(|field| field.name == "values") else {
            return Err(generator_diagnostic(
                "library Texture2d lost its typed marker field",
                pos.clone(),
            ));
        };
        let Type::Array(item) = &values.ty else {
            return Err(generator_diagnostic(
                "library Texture2d typed marker is not an array",
                pos.clone(),
            ));
        };
        if item.as_ref() != &Type::F32 {
            return Err(diagnostic(
                "TX1",
                "Texture2d sample type must be f32",
                pos.clone(),
            ));
        }
        return Ok(Some((
            BindingKind::Texture(TextureSampleType::Float),
            Type::F32,
        )));
    } else if class.name.starts_with("StorageTexture2d<") {
        let Some(formats) = class.fields.iter().find(|field| field.name == "formats") else {
            return Err(generator_diagnostic(
                "library StorageTexture2d lost its format marker field",
                pos.clone(),
            ));
        };
        let Type::Array(item) = &formats.ty else {
            return Err(generator_diagnostic(
                "library StorageTexture2d format marker is not an array",
                pos.clone(),
            ));
        };
        let marker = library_class(module, item).and_then(|marker| match marker.name.as_str() {
            "Rgba8unorm" => Some(StorageTextureFormat::Rgba8unorm),
            "Rgba16float" => Some(StorageTextureFormat::Rgba16float),
            "R32float" => Some(StorageTextureFormat::R32float),
            "Rgba32float" => Some(StorageTextureFormat::Rgba32float),
            _ => None,
        });
        let Some(format) = marker else {
            return Err(diagnostic(
                "TX1",
                "StorageTexture2d format must be a float-channel library marker",
                pos.clone(),
            ));
        };
        return Ok(Some((
            BindingKind::StorageTexture(format),
            (**item).clone(),
        )));
    } else if class.name == "Sampler" {
        return Ok(Some((BindingKind::Sampler, ty.clone())));
    } else {
        return Ok(None);
    };
    let Some(values) = class.fields.iter().find(|field| field.name == "values") else {
        return Ok(None);
    };
    let Type::Array(item) = &values.ty else {
        return Ok(None);
    };
    Ok(Some((kind, (**item).clone())))
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
    if class.fields.is_empty() {
        return Err(diagnostic(
            "TX2",
            format!("layout class `{}` is empty", class.name),
            class.pos.clone(),
        ));
    }
    let mut bindings = Vec::new();
    for (index, field) in class.fields.iter().enumerate() {
        let Some((kind, item_ty)) = wrapper(module, &field.ty, &field.pos)? else {
            return Err(diagnostic(
                "PI3",
                format!(
                    "layout field `{}.{}` is not a Uniform, Storage, or MutStorage binding wrapper",
                    class.name, field.name
                ),
                field.pos.clone(),
            ));
        };
        if kind.is_buffer() && !allowed_binding_item(module, &item_ty) {
            return Err(diagnostic(
                "PI5",
                format!(
                    "layout field `{}.{}` has a binding item type outside PI5",
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
            "PI1",
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
            "PI1",
            "pipeline options omit workgroupSize",
            expr.pos.clone(),
        ));
    };
    let Some(Some(value)) = fields.get(index) else {
        return Err(diagnostic(
            "PI1",
            "pipeline workgroup size is not literal",
            expr.pos.clone(),
        ));
    };
    let ExprKind::ArrayLit(values) = &value.kind else {
        return Err(diagnostic(
            "PI1",
            "pipeline workgroup size is not literal",
            value.pos.clone(),
        ));
    };
    if values.len() != 3 {
        return Err(diagnostic(
            "PI1",
            "pipeline workgroup size requires three literals",
            value.pos.clone(),
        ));
    }
    let Some(x) = literal_u32(&values[0]) else {
        return Err(diagnostic(
            "PI1",
            "pipeline workgroup size is not literal",
            values[0].pos.clone(),
        ));
    };
    let Some(y) = literal_u32(&values[1]) else {
        return Err(diagnostic(
            "PI1",
            "pipeline workgroup size is not literal",
            values[1].pos.clone(),
        ));
    };
    let Some(z) = literal_u32(&values[2]) else {
        return Err(diagnostic(
            "PI1",
            "pipeline workgroup size is not literal",
            values[2].pos.clone(),
        ));
    };
    if x == 0 || y == 0 || z == 0 {
        return Err(diagnostic(
            "PI1",
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

fn compute_arity(module: &Module, name: &str) -> Option<usize> {
    let base = name.split('<').next().unwrap_or(name);
    let declaration = function(module, name)?;
    if declaration.params.first()?.pos.file != "typegpu.ts" {
        return None;
    }
    Some(match base {
        "computePipeline" => 1,
        "computePipeline2" => 2,
        "computePipeline3" => 3,
        "computePipeline4" => 4,
        _ => return None,
    })
}

fn call_in_expr(module: &Module, expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Call { callee: Callee::Func(name), .. } if compute_arity(module, name).is_some() => true,
        ExprKind::Unary { operand, .. } | ExprKind::Cast(operand) | ExprKind::Length(operand) => call_in_expr(module, operand),
        ExprKind::Binary { left, right, .. } => call_in_expr(module, left) || call_in_expr(module, right),
        ExprKind::Assign { target, value, .. } => call_in_expr(module, target) || call_in_expr(module, value),
        ExprKind::Call { callee, args } => {
            matches!(callee, Callee::Value(value) if call_in_expr(module, value))
                || matches!(callee, Callee::Method { recv, .. } if call_in_expr(module, recv))
                || args.iter().any(|arg| call_in_expr(module, arg))
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => args.iter().any(|arg| call_in_expr(module, arg)),
        ExprKind::DescriptorLit { fields, .. } => fields.iter().flatten().any(|value| call_in_expr(module, value)),
        ExprKind::Field { obj, .. } | ExprKind::JsonResultValue(obj) => call_in_expr(module, obj),
        ExprKind::Index { obj, index, .. } => call_in_expr(module, obj) || call_in_expr(module, index),
        ExprKind::Template(parts) => parts.iter().any(|part| matches!(part, subscript_compiler::hir::TplPart::Expr(value) if call_in_expr(module, value))),
        ExprKind::Lambda { body, .. } => body.iter().any(|stmt| stmt_has_compute(module, stmt)),
        ExprKind::Cond { cond, then, els } => call_in_expr(module, cond) || call_in_expr(module, then) || call_in_expr(module, els),
        _ => false,
    }
}

fn stmt_has_compute(module: &Module, stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Let { init, .. } | Stmt::Expr(init) => call_in_expr(module, init),
        Stmt::Return { value, .. } => value
            .as_ref()
            .is_some_and(|value| call_in_expr(module, value)),
        Stmt::If {
            cond, then, els, ..
        } => {
            call_in_expr(module, cond)
                || then.iter().any(|stmt| stmt_has_compute(module, stmt))
                || els
                    .as_ref()
                    .is_some_and(|items| items.iter().any(|stmt| stmt_has_compute(module, stmt)))
        }
        Stmt::While { cond, body, .. } => {
            call_in_expr(module, cond) || body.iter().any(|stmt| stmt_has_compute(module, stmt))
        }
        Stmt::For {
            init,
            cond,
            step,
            body,
            ..
        } => {
            init.as_deref()
                .is_some_and(|stmt| stmt_has_compute(module, stmt))
                || cond
                    .as_ref()
                    .is_some_and(|value| call_in_expr(module, value))
                || step
                    .as_ref()
                    .is_some_and(|value| call_in_expr(module, value))
                || body.iter().any(|stmt| stmt_has_compute(module, stmt))
        }
        Stmt::ForOf { subject, body, .. } => {
            call_in_expr(module, subject) || body.iter().any(|stmt| stmt_has_compute(module, stmt))
        }
        Stmt::Switch { disc, cases, .. } => {
            call_in_expr(module, disc)
                || cases
                    .iter()
                    .flat_map(|case| &case.body)
                    .any(|stmt| stmt_has_compute(module, stmt))
        }
        Stmt::Block(body) => body.iter().any(|stmt| stmt_has_compute(module, stmt)),
        Stmt::Break(_) | Stmt::Continue(_) => false,
        _ => false,
    }
}

pub(crate) fn discover(module: &Module) -> Result<Vec<Pipeline>, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    for function in &module.functions {
        if function.pos.file != "typegpu.ts"
            && function
                .body
                .iter()
                .any(|stmt| stmt_has_compute(module, stmt))
        {
            diagnostics.push(diagnostic(
                "PI1",
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
        let Some(arity) = compute_arity(module, callee) else {
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
            diagnostics.push(generator_diagnostic(
                format!("kernel `{entry}` disappeared from typed HIR"),
                global.init.pos.clone(),
            ));
            continue;
        };
        if kernel.params.len() != arity + 1 {
            diagnostics.push(generator_diagnostic(
                format!("kernel `{entry}` has an impossible parameter count"),
                kernel.pos.clone(),
            ));
            continue;
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
            diagnostics.push(generator_diagnostic(
                format!("kernel `{entry}` lost its ComputeInvocation parameter"),
                kernel.params[arity].pos.clone(),
            ));
            continue;
        }
        let Some(options) = args.get(1) else {
            diagnostics.push(diagnostic(
                "PI1",
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
