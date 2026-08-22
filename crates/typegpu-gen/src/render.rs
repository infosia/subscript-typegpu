//! Render-pipeline declaration and interface discovery.

use std::collections::{BTreeMap, BTreeSet};

use subscript_compiler::hir::{Callee, Expr, ExprKind, Function, Module, Stmt};
use subscript_compiler::{Diagnostic, Pos, RuleCode, Type};

use crate::pipeline::{self, BindingKind, Layout};

#[derive(Debug, Clone)]
pub(crate) struct VertexAttribute {
    pub(crate) format: &'static str,
    pub(crate) location: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct VertexBuffer {
    pub(crate) schema: String,
    pub(crate) slot: u32,
    pub(crate) step_mode: &'static str,
    pub(crate) attributes: Vec<VertexAttribute>,
}

#[derive(Debug, Clone)]
pub(crate) struct Varying {
    pub(crate) name: String,
    pub(crate) ty: Type,
    pub(crate) builtin_position: bool,
    pub(crate) location: Option<u32>,
    pub(crate) flat: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct FragmentOutput {
    pub(crate) name: String,
    pub(crate) fields: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct RenderPipeline {
    pub(crate) declaration: String,
    pub(crate) vertex_entry: String,
    pub(crate) fragment_entry: String,
    pub(crate) layouts: Vec<Layout>,
    pub(crate) vertex_buffers: Vec<VertexBuffer>,
    pub(crate) varyings_name: String,
    pub(crate) varyings: Vec<Varying>,
    pub(crate) fragment_output: Option<FragmentOutput>,
    pub(crate) target_format: String,
    pub(crate) pos: Pos,
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

fn render_shape(name: &str) -> Option<(usize, usize, Option<usize>)> {
    Some(match base_name(name) {
        "renderPipeline" => (0, 0, None),
        "renderPipelineL" => (1, 0, None),
        "renderPipelineInstanced" => (0, 0, Some(1)),
        _ => return None,
    })
}

fn library_named(module: &Module, ty: &Type, name: &str) -> bool {
    matches!(ty, Type::Class(id) if matches!(module.classes[id.0].pos.file.as_str(), "typegpu-types.ts" | "typegpu.ts") && module.classes[id.0].name == name)
}

fn value_class<'a>(
    module: &'a Module,
    ty: &Type,
    rule: &str,
    purpose: &str,
    pos: &Pos,
) -> Result<&'a subscript_compiler::hir::ClassDef, Diagnostic> {
    let Type::Class(id) = ty else {
        return Err(diagnostic(
            rule,
            format!("{purpose} is not a @CStruct class"),
            pos.clone(),
        ));
    };
    let class = &module.classes[id.0];
    if !class.is_value || class.pos.file == "typegpu-types.ts" {
        return Err(diagnostic(
            rule,
            format!("{purpose} `{}` is not a program @CStruct class", class.name),
            pos.clone(),
        ));
    }
    Ok(class)
}

fn vertex_format(module: &Module, ty: &Type) -> Option<&'static str> {
    Some(match ty {
        Type::F32 => "float32",
        Type::U32 => "uint32",
        Type::I32 => "sint32",
        Type::Class(id) if module.classes[id.0].pos.file == "typegpu-types.ts" => {
            match module.classes[id.0].name.as_str() {
                "Vec2f" => "float32x2",
                "Vec3f" => "float32x3",
                "Vec4f" => "float32x4",
                "Vec2u" => "uint32x2",
                "Vec3u" => "uint32x3",
                "Vec4u" => "uint32x4",
                "Vec2i" => "sint32x2",
                "Vec3i" => "sint32x3",
                "Vec4i" => "sint32x4",
                "Vec2h" => "float16x2",
                "Vec4h" => "float16x4",
                _ => return None,
            }
        }
        _ => return None,
    })
}

fn vertex_buffer(
    module: &Module,
    ty: &Type,
    slot: u32,
    first_location: u32,
    step_mode: &'static str,
    pos: &Pos,
) -> Result<VertexBuffer, Diagnostic> {
    let class = value_class(module, ty, "RN4", "vertex input", pos)?;
    let mut attributes = Vec::new();
    for (index, field) in class.fields.iter().enumerate() {
        let format = vertex_format(module, &field.ty).ok_or_else(|| {
            diagnostic(
                "RN5",
                format!(
                    "vertex attribute `{}.{}` has unsupported type `{}`",
                    class.name,
                    field.name,
                    pipeline::type_name(module, &field.ty)
                ),
                field.pos.clone(),
            )
        })?;
        attributes.push(VertexAttribute {
            format,
            location: first_location + index as u32,
        });
    }
    Ok(VertexBuffer {
        schema: class.name.clone(),
        slot,
        step_mode,
        attributes,
    })
}

fn varyings(module: &Module, ty: &Type, pos: &Pos) -> Result<(String, Vec<Varying>), Diagnostic> {
    let class = value_class(module, ty, "RN7", "varyings", pos)?;
    let position = class
        .fields
        .iter()
        .find(|field| field.name == "position" && library_named(module, &field.ty, "Vec4f"));
    if position.is_none() {
        return Err(diagnostic(
            "RN7",
            format!("varyings `{}` has no `position: Vec4f` field", class.name),
            class.pos.clone(),
        ));
    }
    let mut location = 0;
    let mut fields = Vec::new();
    for field in &class.fields {
        let builtin_position = field.name == "position";
        let field_location = (!builtin_position).then_some(location);
        if !builtin_position {
            location += 1;
        }
        let flat = matches!(field.ty, Type::I32 | Type::U32)
            || matches!(&field.ty, Type::Class(id) if module.classes[id.0].pos.file == "typegpu-types.ts" && matches!(module.classes[id.0].name.as_str(), "Vec2i" | "Vec3i" | "Vec4i" | "Vec2u" | "Vec3u" | "Vec4u"));
        fields.push(Varying {
            name: field.name.clone(),
            ty: field.ty.clone(),
            builtin_position,
            location: field_location,
            flat,
        });
    }
    Ok((class.name.clone(), fields))
}

fn descriptor_string(
    module: &Module,
    expr: &Expr,
    field_name: &str,
    required: bool,
    default: &str,
) -> Result<String, Diagnostic> {
    let ExprKind::DescriptorLit { class, fields } = &expr.kind else {
        return Err(diagnostic(
            "RN1",
            "render options must be a descriptor literal",
            expr.pos.clone(),
        ));
    };
    let descriptor = &module.classes[class.0];
    let Some(index) = descriptor
        .fields
        .iter()
        .position(|field| field.name == field_name)
    else {
        return Err(diagnostic(
            "RN1",
            format!("render options have no `{field_name}` member"),
            expr.pos.clone(),
        ));
    };
    match fields.get(index).and_then(Option::as_ref) {
        Some(Expr {
            kind: ExprKind::Str(value),
            ..
        }) => Ok(value.clone()),
        Some(Expr {
            kind: ExprKind::EnumMember { member, .. },
            ..
        }) => Ok(member.clone()),
        Some(Expr {
            kind: ExprKind::Int(value),
            ty: Type::StringAlias(alias),
            pos,
            ..
        }) => {
            let definition = &module.string_aliases[alias.0];
            let index = if let Some(wire_values) = &definition.wire_values {
                wire_values
                    .iter()
                    .position(|wire| i64::from(*wire) == *value)
            } else {
                usize::try_from(*value).ok()
            };
            index
                .and_then(|index| definition.members.get(index))
                .cloned()
                .ok_or_else(|| {
                    diagnostic(
                        "RN1",
                        format!("render option `{field_name}` has an unknown enum value"),
                        pos.clone(),
                    )
                })
        }
        None if !required => Ok(default.to_owned()),
        None => Err(diagnostic(
            "RN1",
            format!("render options omit `{field_name}`"),
            expr.pos.clone(),
        )),
        Some(value) => Err(diagnostic(
            "RN1",
            format!("render option `{field_name}` must be a string literal"),
            value.pos.clone(),
        )),
    }
}

fn called_forbidden_expr(expr: &Expr) -> Option<(&'static str, Pos)> {
    match &expr.kind {
        ExprKind::Call { callee, args } => {
            if matches!(callee, Callee::Func(name) if base_name(name) == "textureSample") {
                return Some((
                    "textureSample is unavailable in the current render surface",
                    expr.pos.clone(),
                ));
            }
            let nested = match callee {
                Callee::Value(value) => called_forbidden_expr(value),
                Callee::Method { recv, .. } => called_forbidden_expr(recv),
                _ => None,
            };
            nested.or_else(|| args.iter().find_map(called_forbidden_expr))
        }
        ExprKind::Unary { operand, .. } | ExprKind::Cast(operand) | ExprKind::Length(operand) => {
            called_forbidden_expr(operand)
        }
        ExprKind::Binary { left, right, .. } => {
            called_forbidden_expr(left).or_else(|| called_forbidden_expr(right))
        }
        ExprKind::Assign { target, value, .. } => {
            called_forbidden_expr(target).or_else(|| called_forbidden_expr(value))
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => {
            args.iter().find_map(called_forbidden_expr)
        }
        ExprKind::DescriptorLit { fields, .. } => {
            fields.iter().flatten().find_map(called_forbidden_expr)
        }
        ExprKind::Field { obj, .. } | ExprKind::JsonResultValue(obj) => called_forbidden_expr(obj),
        ExprKind::Index { obj, index, .. } => {
            called_forbidden_expr(obj).or_else(|| called_forbidden_expr(index))
        }
        ExprKind::Cond { cond, then, els } => called_forbidden_expr(cond)
            .or_else(|| called_forbidden_expr(then))
            .or_else(|| called_forbidden_expr(els)),
        _ => None,
    }
}

fn called_forbidden_stmt(stmt: &Stmt) -> Option<(&'static str, Pos)> {
    match stmt {
        Stmt::Let { init, .. } | Stmt::Expr(init) => called_forbidden_expr(init),
        Stmt::Return { value, .. } => value.as_ref().and_then(called_forbidden_expr),
        Stmt::If {
            cond, then, els, ..
        } => called_forbidden_expr(cond)
            .or_else(|| then.iter().find_map(called_forbidden_stmt))
            .or_else(|| {
                els.as_ref()
                    .and_then(|body| body.iter().find_map(called_forbidden_stmt))
            }),
        Stmt::While { cond, body, .. } => {
            called_forbidden_expr(cond).or_else(|| body.iter().find_map(called_forbidden_stmt))
        }
        Stmt::For {
            init,
            cond,
            step,
            body,
            ..
        } => init
            .as_deref()
            .and_then(called_forbidden_stmt)
            .or_else(|| cond.as_ref().and_then(called_forbidden_expr))
            .or_else(|| step.as_ref().and_then(called_forbidden_expr))
            .or_else(|| body.iter().find_map(called_forbidden_stmt)),
        Stmt::ForOf { subject, body, .. } => {
            called_forbidden_expr(subject).or_else(|| body.iter().find_map(called_forbidden_stmt))
        }
        Stmt::Switch { disc, cases, .. } => called_forbidden_expr(disc).or_else(|| {
            cases
                .iter()
                .flat_map(|case| &case.body)
                .find_map(called_forbidden_stmt)
        }),
        Stmt::Block(body) => body.iter().find_map(called_forbidden_stmt),
        _ => None,
    }
}

fn contains_render_call_expr(expr: &Expr) -> bool {
    if matches!(&expr.kind, ExprKind::Call { callee: Callee::Func(name), .. } if render_shape(name).is_some())
    {
        return true;
    }
    match &expr.kind {
        ExprKind::Unary { operand, .. } | ExprKind::Cast(operand) | ExprKind::Length(operand) => {
            contains_render_call_expr(operand)
        }
        ExprKind::Binary { left, right, .. } => {
            contains_render_call_expr(left) || contains_render_call_expr(right)
        }
        ExprKind::Assign { target, value, .. } => {
            contains_render_call_expr(target) || contains_render_call_expr(value)
        }
        ExprKind::Call { callee, args } => {
            let receiver = match callee {
                Callee::Value(value) => contains_render_call_expr(value),
                Callee::Method { recv, .. } => contains_render_call_expr(recv),
                _ => false,
            };
            receiver || args.iter().any(contains_render_call_expr)
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => {
            args.iter().any(contains_render_call_expr)
        }
        ExprKind::DescriptorLit { fields, .. } => {
            fields.iter().flatten().any(contains_render_call_expr)
        }
        ExprKind::Field { obj, .. } | ExprKind::JsonResultValue(obj) => {
            contains_render_call_expr(obj)
        }
        ExprKind::Index { obj, index, .. } => {
            contains_render_call_expr(obj) || contains_render_call_expr(index)
        }
        ExprKind::Cond { cond, then, els } => {
            contains_render_call_expr(cond)
                || contains_render_call_expr(then)
                || contains_render_call_expr(els)
        }
        _ => false,
    }
}

fn contains_render_call_stmt(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Let { init, .. } | Stmt::Expr(init) => contains_render_call_expr(init),
        Stmt::Return { value, .. } => value.as_ref().is_some_and(contains_render_call_expr),
        Stmt::If {
            cond, then, els, ..
        } => {
            contains_render_call_expr(cond)
                || then.iter().any(contains_render_call_stmt)
                || els
                    .as_ref()
                    .is_some_and(|body| body.iter().any(contains_render_call_stmt))
        }
        Stmt::While { cond, body, .. } => {
            contains_render_call_expr(cond) || body.iter().any(contains_render_call_stmt)
        }
        Stmt::For {
            init,
            cond,
            step,
            body,
            ..
        } => {
            init.as_deref().is_some_and(contains_render_call_stmt)
                || cond.as_ref().is_some_and(contains_render_call_expr)
                || step.as_ref().is_some_and(contains_render_call_expr)
                || body.iter().any(contains_render_call_stmt)
        }
        Stmt::ForOf { subject, body, .. } => {
            contains_render_call_expr(subject) || body.iter().any(contains_render_call_stmt)
        }
        Stmt::Switch { disc, cases, .. } => {
            contains_render_call_expr(disc)
                || cases
                    .iter()
                    .flat_map(|case| &case.body)
                    .any(contains_render_call_stmt)
        }
        Stmt::Block(body) => body.iter().any(contains_render_call_stmt),
        _ => false,
    }
}

fn signature_types<'a>(module: &'a Module, callee: &str) -> Option<(&'a Type, &'a Type)> {
    let declaration = pipeline::function(module, callee)?;
    Some((
        &declaration.params.first()?.ty,
        &declaration.params.get(1)?.ty,
    ))
}

fn compare_signature(
    declaration_ty: &Type,
    kernel: &Function,
    role: &str,
    pos: &Pos,
) -> Result<(), Diagnostic> {
    let Type::Func(signature) = declaration_ty else {
        return Err(Diagnostic::new(
            RuleCode::S100,
            format!("K15: render {role} declaration lost its function type (generator)"),
            pos.clone(),
        ));
    };
    let same = signature.params.len() == kernel.params.len()
        && signature
            .params
            .iter()
            .zip(&kernel.params)
            .all(|(declared, actual)| declared == &actual.ty)
        && signature.ret == kernel.ret;
    if !same {
        return Err(diagnostic(
            "RN2",
            format!("{role} kernel signature differs from the declaration type arguments"),
            kernel.pos.clone(),
        ));
    }
    Ok(())
}

fn validate_rn2(
    module: &Module,
    vertex: &Function,
    fragment: &Function,
    layout_count: usize,
    instance_index: Option<usize>,
) -> Result<(), Diagnostic> {
    let vertex_values = if instance_index.is_some() { 2 } else { 1 };
    if vertex.params.len() != layout_count + vertex_values + 1
        || fragment.params.len() != layout_count + 2
    {
        return Err(diagnostic(
            "RN2",
            "render kernel parameter counts do not match the declaration form",
            vertex.pos.clone(),
        ));
    }
    let vertex_invocation = &vertex.params[layout_count + vertex_values];
    let fragment_invocation = &fragment.params[layout_count + 1];
    if !library_named(module, &vertex_invocation.ty, "VertexInvocation")
        || !library_named(module, &fragment_invocation.ty, "FragmentInvocation")
    {
        return Err(diagnostic(
            "RN2",
            "render kernels require VertexInvocation and FragmentInvocation parameters",
            vertex_invocation.pos.clone(),
        ));
    }
    if vertex.ret != fragment.params[layout_count].ty {
        return Err(diagnostic(
            "RN2",
            "the vertex return type differs from the fragment input type",
            fragment.params[layout_count].pos.clone(),
        ));
    }
    if layout_count == 1 && vertex.params[0].ty != fragment.params[0].ty {
        return Err(diagnostic(
            "RN2",
            "the vertex and fragment layout parameter types differ",
            fragment.params[0].pos.clone(),
        ));
    }
    Ok(())
}

pub(crate) fn discover(module: &Module) -> Result<Vec<RenderPipeline>, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    for function in &module.functions {
        if function.pos.file != "typegpu.ts" && function.body.iter().any(contains_render_call_stmt)
        {
            diagnostics.push(diagnostic(
                "RN1",
                "a render pipeline declaration appears inside a function",
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
        let Some((layout_count, vertex_index, instance_index)) = render_shape(callee) else {
            continue;
        };
        if global.mutable {
            diagnostics.push(diagnostic(
                "RN1",
                "a render pipeline declaration must be const",
                global.pos.clone(),
            ));
            continue;
        }
        let (
            Some(Expr {
                kind: ExprKind::FuncRef(vertex_name),
                ..
            }),
            Some(Expr {
                kind: ExprKind::FuncRef(fragment_name),
                ..
            }),
        ) = (args.first(), args.get(1))
        else {
            diagnostics.push(diagnostic(
                "RN1",
                "render kernels must be named functions",
                global.init.pos.clone(),
            ));
            continue;
        };
        let (Some(vertex), Some(fragment)) = (
            pipeline::function(module, vertex_name),
            pipeline::function(module, fragment_name),
        ) else {
            diagnostics.push(Diagnostic::new(
                RuleCode::S100,
                "K15: a render kernel disappeared from typed HIR (generator)",
                global.init.pos.clone(),
            ));
            continue;
        };
        if let Err(error) = validate_rn2(module, vertex, fragment, layout_count, instance_index) {
            diagnostics.push(error);
            continue;
        }
        let Some((vertex_declared, fragment_declared)) = signature_types(module, callee) else {
            diagnostics.push(Diagnostic::new(
                RuleCode::S100,
                "K15: the render declaration disappeared from typed HIR (generator)",
                global.init.pos.clone(),
            ));
            continue;
        };
        if let Err(error) = compare_signature(vertex_declared, vertex, "vertex", &global.pos) {
            diagnostics.push(error);
            continue;
        }
        if let Err(error) = compare_signature(fragment_declared, fragment, "fragment", &global.pos)
        {
            diagnostics.push(error);
            continue;
        }
        let mut layouts = Vec::new();
        if layout_count == 1 {
            match pipeline::layout(module, &vertex.params[0].ty, 0) {
                Ok(layout) => layouts.push(layout),
                Err(error) => diagnostics.push(error),
            }
        }
        let vertex_param = &vertex.params[layout_count + vertex_index];
        let first = match vertex_buffer(module, &vertex_param.ty, 0, 0, "vertex", &vertex_param.pos)
        {
            Ok(value) => value,
            Err(error) => {
                diagnostics.push(error);
                continue;
            }
        };
        let mut vertex_buffers = vec![first];
        if let Some(instance_index) = instance_index {
            let param = &vertex.params[layout_count + instance_index];
            let first_location = vertex_buffers[0].attributes.len() as u32;
            match vertex_buffer(module, &param.ty, 1, first_location, "instance", &param.pos) {
                Ok(value) => vertex_buffers.push(value),
                Err(error) => {
                    diagnostics.push(error);
                    continue;
                }
            }
        }
        let (varyings_name, varyings) = match varyings(module, &vertex.ret, &vertex.pos) {
            Ok(value) => value,
            Err(error) => {
                diagnostics.push(error);
                continue;
            }
        };
        let fragment_output = if library_named(module, &fragment.ret, "Vec4f") {
            None
        } else if let Type::Class(id) = &fragment.ret {
            let class = &module.classes[id.0];
            if !class.is_value
                || class.fields.is_empty()
                || !class
                    .fields
                    .iter()
                    .all(|field| library_named(module, &field.ty, "Vec4f"))
            {
                diagnostics.push(diagnostic(
                    "RN8",
                    "fragment kernel return is neither Vec4f nor a @CStruct of Vec4f fields",
                    fragment.pos.clone(),
                ));
                continue;
            }
            Some(FragmentOutput {
                name: class.name.clone(),
                fields: class
                    .fields
                    .iter()
                    .map(|field| field.name.clone())
                    .collect(),
            })
        } else {
            diagnostics.push(diagnostic(
                "RN8",
                "fragment kernel return is neither Vec4f nor a @CStruct of Vec4f fields",
                fragment.pos.clone(),
            ));
            continue;
        };
        let forbidden = vertex
            .body
            .iter()
            .find_map(called_forbidden_stmt)
            .or_else(|| fragment.body.iter().find_map(called_forbidden_stmt));
        if let Some((message, pos)) = forbidden {
            diagnostics.push(diagnostic("RN16", message, pos));
            continue;
        }
        let Some(options) = args.get(2) else {
            diagnostics.push(diagnostic(
                "RN1",
                "render declaration omits options",
                global.pos.clone(),
            ));
            continue;
        };
        let target_format = match descriptor_string(module, options, "format", true, "") {
            Ok(value) => value,
            Err(error) => {
                diagnostics.push(error);
                continue;
            }
        };
        for (field, default) in [
            ("topology", "triangle-list"),
            ("cullMode", "none"),
            ("frontFace", "ccw"),
        ] {
            if let Err(error) = descriptor_string(module, options, field, false, default) {
                diagnostics.push(error);
            }
        }
        pipelines.push(RenderPipeline {
            declaration: global.name.clone(),
            vertex_entry: vertex_name.clone(),
            fragment_entry: fragment_name.clone(),
            layouts,
            vertex_buffers,
            varyings_name,
            varyings,
            fragment_output,
            target_format,
            pos: global.pos.clone(),
        });
    }
    if diagnostics.is_empty() {
        Ok(pipelines)
    } else {
        Err(diagnostics)
    }
}

pub(crate) fn schema_names(pipelines: &[RenderPipeline]) -> BTreeSet<String> {
    pipelines
        .iter()
        .flat_map(|pipeline| &pipeline.vertex_buffers)
        .map(|buffer| buffer.schema.clone())
        .collect()
}

fn binding_reads_expr(
    expr: &Expr,
    layout_params: &BTreeMap<String, usize>,
    out: &mut BTreeSet<(usize, String)>,
) {
    if let ExprKind::Field { obj, name } = &expr.kind {
        if let ExprKind::Local(param) = &obj.kind {
            if let Some(group) = layout_params.get(param) {
                out.insert((*group, name.clone()));
            }
        }
    }
    match &expr.kind {
        ExprKind::Unary { operand, .. } | ExprKind::Cast(operand) | ExprKind::Length(operand) => {
            binding_reads_expr(operand, layout_params, out)
        }
        ExprKind::Binary { left, right, .. } => {
            binding_reads_expr(left, layout_params, out);
            binding_reads_expr(right, layout_params, out);
        }
        ExprKind::Assign { target, value, .. } => {
            binding_reads_expr(target, layout_params, out);
            binding_reads_expr(value, layout_params, out);
        }
        ExprKind::Call { callee, args } => {
            match callee {
                Callee::Value(value) => binding_reads_expr(value, layout_params, out),
                Callee::Method { recv, .. } => binding_reads_expr(recv, layout_params, out),
                _ => {}
            }
            for arg in args {
                binding_reads_expr(arg, layout_params, out);
            }
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => {
            for arg in args {
                binding_reads_expr(arg, layout_params, out);
            }
        }
        ExprKind::DescriptorLit { fields, .. } => {
            for value in fields.iter().flatten() {
                binding_reads_expr(value, layout_params, out);
            }
        }
        ExprKind::Field { obj, .. } | ExprKind::JsonResultValue(obj) => {
            binding_reads_expr(obj, layout_params, out)
        }
        ExprKind::Index { obj, index, .. } => {
            binding_reads_expr(obj, layout_params, out);
            binding_reads_expr(index, layout_params, out);
        }
        ExprKind::Cond { cond, then, els } => {
            binding_reads_expr(cond, layout_params, out);
            binding_reads_expr(then, layout_params, out);
            binding_reads_expr(els, layout_params, out);
        }
        _ => {}
    }
}

fn binding_reads_stmt(
    stmt: &Stmt,
    layout_params: &BTreeMap<String, usize>,
    out: &mut BTreeSet<(usize, String)>,
) {
    match stmt {
        Stmt::Let { init, .. } | Stmt::Expr(init) => binding_reads_expr(init, layout_params, out),
        Stmt::Return {
            value: Some(value), ..
        } => binding_reads_expr(value, layout_params, out),
        Stmt::Return { value: None, .. } => {}
        Stmt::If {
            cond, then, els, ..
        } => {
            binding_reads_expr(cond, layout_params, out);
            for item in then {
                binding_reads_stmt(item, layout_params, out);
            }
            if let Some(els) = els {
                for item in els {
                    binding_reads_stmt(item, layout_params, out);
                }
            }
        }
        Stmt::While { cond, body, .. } => {
            binding_reads_expr(cond, layout_params, out);
            for item in body {
                binding_reads_stmt(item, layout_params, out);
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
                binding_reads_stmt(init, layout_params, out);
            }
            if let Some(cond) = cond {
                binding_reads_expr(cond, layout_params, out);
            }
            if let Some(step) = step {
                binding_reads_expr(step, layout_params, out);
            }
            for item in body {
                binding_reads_stmt(item, layout_params, out);
            }
        }
        Stmt::ForOf { subject, body, .. } => {
            binding_reads_expr(subject, layout_params, out);
            for item in body {
                binding_reads_stmt(item, layout_params, out);
            }
        }
        Stmt::Switch { disc, cases, .. } => {
            binding_reads_expr(disc, layout_params, out);
            for item in cases.iter().flat_map(|case| &case.body) {
                binding_reads_stmt(item, layout_params, out);
            }
        }
        Stmt::Block(body) => {
            for item in body {
                binding_reads_stmt(item, layout_params, out);
            }
        }
        _ => {}
    }
}

fn stage_bindings(
    module: &Module,
    pipeline: &RenderPipeline,
    entry: &str,
) -> BTreeSet<(usize, String)> {
    let Some(kernel) = pipeline::function(module, entry) else {
        return BTreeSet::new();
    };
    let layout_params = pipeline
        .layouts
        .iter()
        .enumerate()
        .map(|(group, _)| (kernel.params[group].name.clone(), group))
        .collect::<BTreeMap<_, _>>();
    let mut out = BTreeSet::new();
    for stmt in &kernel.body {
        binding_reads_stmt(stmt, &layout_params, &mut out);
    }
    out
}

pub(crate) fn binding_visibility(
    module: &Module,
    pipeline: &RenderPipeline,
    group: usize,
    name: &str,
) -> (bool, bool) {
    let key = (group, name.to_owned());
    (
        stage_bindings(module, pipeline, &pipeline.vertex_entry).contains(&key),
        stage_bindings(module, pipeline, &pipeline.fragment_entry).contains(&key),
    )
}

fn binding_key(expr: &Expr, layout_params: &BTreeMap<String, usize>) -> Option<(usize, String)> {
    match &expr.kind {
        ExprKind::Field { obj, name } => {
            let ExprKind::Local(param) = &obj.kind else {
                return None;
            };
            Some((*layout_params.get(param)?, name.clone()))
        }
        ExprKind::Index { obj, .. } => binding_key(obj, layout_params),
        _ => None,
    }
}

fn written_binding_expr(
    expr: &Expr,
    layout_params: &BTreeMap<String, usize>,
) -> Option<(usize, String)> {
    match &expr.kind {
        ExprKind::Assign { target, value, .. } => binding_key(target, layout_params)
            .or_else(|| written_binding_expr(target, layout_params))
            .or_else(|| written_binding_expr(value, layout_params)),
        ExprKind::Call {
            callee: Callee::Method { recv, name },
            args,
        } => {
            if name == "set" {
                if let Some(binding) = binding_key(recv, layout_params) {
                    return Some(binding);
                }
            }
            written_binding_expr(recv, layout_params).or_else(|| {
                args.iter()
                    .find_map(|arg| written_binding_expr(arg, layout_params))
            })
        }
        ExprKind::Call { callee, args } => {
            let receiver = match callee {
                Callee::Value(value) => written_binding_expr(value, layout_params),
                _ => None,
            };
            receiver.or_else(|| {
                args.iter()
                    .find_map(|arg| written_binding_expr(arg, layout_params))
            })
        }
        ExprKind::Unary { operand, .. } | ExprKind::Cast(operand) | ExprKind::Length(operand) => {
            written_binding_expr(operand, layout_params)
        }
        ExprKind::Binary { left, right, .. } => written_binding_expr(left, layout_params)
            .or_else(|| written_binding_expr(right, layout_params)),
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => args
            .iter()
            .find_map(|arg| written_binding_expr(arg, layout_params)),
        ExprKind::DescriptorLit { fields, .. } => fields
            .iter()
            .flatten()
            .find_map(|value| written_binding_expr(value, layout_params)),
        ExprKind::Field { obj, .. } | ExprKind::JsonResultValue(obj) => {
            written_binding_expr(obj, layout_params)
        }
        ExprKind::Index { obj, index, .. } => written_binding_expr(obj, layout_params)
            .or_else(|| written_binding_expr(index, layout_params)),
        ExprKind::Cond { cond, then, els } => written_binding_expr(cond, layout_params)
            .or_else(|| written_binding_expr(then, layout_params))
            .or_else(|| written_binding_expr(els, layout_params)),
        _ => None,
    }
}

fn written_binding_stmt(
    stmt: &Stmt,
    layout_params: &BTreeMap<String, usize>,
) -> Option<(usize, String)> {
    match stmt {
        Stmt::Let { init, .. } | Stmt::Expr(init) => written_binding_expr(init, layout_params),
        Stmt::Return { value, .. } => value
            .as_ref()
            .and_then(|value| written_binding_expr(value, layout_params)),
        Stmt::If {
            cond, then, els, ..
        } => written_binding_expr(cond, layout_params)
            .or_else(|| {
                then.iter()
                    .find_map(|item| written_binding_stmt(item, layout_params))
            })
            .or_else(|| {
                els.as_ref().and_then(|body| {
                    body.iter()
                        .find_map(|item| written_binding_stmt(item, layout_params))
                })
            }),
        Stmt::While { cond, body, .. } => written_binding_expr(cond, layout_params).or_else(|| {
            body.iter()
                .find_map(|item| written_binding_stmt(item, layout_params))
        }),
        Stmt::For {
            init,
            cond,
            step,
            body,
            ..
        } => init
            .as_deref()
            .and_then(|item| written_binding_stmt(item, layout_params))
            .or_else(|| {
                cond.as_ref()
                    .and_then(|value| written_binding_expr(value, layout_params))
            })
            .or_else(|| {
                step.as_ref()
                    .and_then(|value| written_binding_expr(value, layout_params))
            })
            .or_else(|| {
                body.iter()
                    .find_map(|item| written_binding_stmt(item, layout_params))
            }),
        Stmt::ForOf { subject, body, .. } => {
            written_binding_expr(subject, layout_params).or_else(|| {
                body.iter()
                    .find_map(|item| written_binding_stmt(item, layout_params))
            })
        }
        Stmt::Switch { disc, cases, .. } => {
            written_binding_expr(disc, layout_params).or_else(|| {
                cases
                    .iter()
                    .flat_map(|case| &case.body)
                    .find_map(|item| written_binding_stmt(item, layout_params))
            })
        }
        Stmt::Block(body) => body
            .iter()
            .find_map(|item| written_binding_stmt(item, layout_params)),
        _ => None,
    }
}

pub(crate) fn reject_vertex_storage_writes(
    module: &Module,
    pipeline: &RenderPipeline,
) -> Result<(), Diagnostic> {
    let Some(vertex) = pipeline::function(module, &pipeline.vertex_entry) else {
        return Ok(());
    };
    let layout_params = pipeline
        .layouts
        .iter()
        .enumerate()
        .map(|(group, _)| (vertex.params[group].name.clone(), group))
        .collect::<BTreeMap<_, _>>();
    let written = vertex
        .body
        .iter()
        .find_map(|stmt| written_binding_stmt(stmt, &layout_params));
    if let Some((group, name)) = written {
        let mutable = pipeline.layouts[group]
            .bindings
            .iter()
            .any(|binding| binding.name == name && binding.kind == BindingKind::MutStorage);
        if mutable {
            return Err(diagnostic(
                "RN16",
                format!("vertex kernel writes storage binding `{name}`"),
                vertex.pos.clone(),
            ));
        }
    }
    Ok(())
}
