//! Typed-HIR schema discovery and validation.

use std::collections::{BTreeSet, HashSet};

use subscript_compiler::hir::{ClassDef, Module};
use subscript_compiler::{Diagnostic, Pos, RuleCode, Type};

use crate::layout::{self, Matrix, Member, Scalar, Struct, TypeTree, Vector};

#[derive(Debug, Clone)]
pub(crate) struct Schema {
    pub(crate) name: String,
    pub(crate) tree: TypeTree,
    pub(crate) pos: Pos,
    pub(crate) field_positions: Vec<Pos>,
}

fn diagnostic(rule: &str, message: impl Into<String>, pos: Pos) -> Diagnostic {
    Diagnostic::new(
        RuleCode::S100,
        format!("{rule}: {} (author)", message.into()),
        pos,
    )
}

pub(crate) fn is_bool_vector(module: &Module, ty: &Type) -> bool {
    matches!(ty, Type::Class(id)
        if module.classes[id.0].pos.file == "typegpu-types.ts"
            && matches!(module.classes[id.0].name.as_str(), "Vec2b" | "Vec3b" | "Vec4b"))
}

fn is_indirect_schema(class: &ClassDef) -> bool {
    class.pos.file == "typegpu-types.ts"
        && matches!(
            class.name.as_str(),
            "DispatchIndirectArgs" | "DrawIndirectArgs" | "DrawIndexedIndirectArgs"
        )
}

fn is_schema_candidate(class: &ClassDef) -> bool {
    !matches!(
        class.pos.file.as_str(),
        "webgpu.ts" | "typegpu-types.ts" | "typegpu.ts"
    ) || is_indirect_schema(class)
}

fn vector_shape(name: &str) -> Option<(Scalar, u8)> {
    Some(match name {
        "Vec2f" => (Scalar::F32, 2),
        "Vec3f" => (Scalar::F32, 3),
        "Vec4f" => (Scalar::F32, 4),
        "Vec2i" => (Scalar::I32, 2),
        "Vec3i" => (Scalar::I32, 3),
        "Vec4i" => (Scalar::I32, 4),
        "Vec2u" => (Scalar::U32, 2),
        "Vec3u" => (Scalar::U32, 3),
        "Vec4u" => (Scalar::U32, 4),
        "Vec2h" => (Scalar::F16, 2),
        "Vec3h" => (Scalar::F16, 3),
        "Vec4h" => (Scalar::F16, 4),
        _ => return None,
    })
}

fn matrix_shape(name: &str) -> Option<(u8, u8)> {
    Some(match name {
        "Mat2x2f" => (2, 2),
        "Mat3x3f" => (3, 3),
        "Mat4x4f" => (4, 4),
        _ => return None,
    })
}

fn class_alignment(class: &ClassDef) -> Option<u32> {
    class.alignment_override.as_ref().map(|value| value.value)
}

pub(crate) fn library_tree(module: &Module, class: &ClassDef) -> Option<TypeTree> {
    if class.pos.file != "typegpu-types.ts" {
        return None;
    }
    if let Some((scalar, lanes)) = vector_shape(&class.name) {
        return Some(TypeTree::Vector(Vector {
            scalar,
            lanes,
            c_alignment: class_alignment(class),
        }));
    }
    if class.name == "AtomicU32" {
        return Some(TypeTree::Atomic(Scalar::U32));
    }
    if class.name == "AtomicI32" {
        return Some(TypeTree::Atomic(Scalar::I32));
    }
    let (columns, rows) = matrix_shape(&class.name)?;
    let column_name = format!("Vec{rows}f");
    let column_alignment = module
        .classes
        .iter()
        .find(|candidate| candidate.pos.file == "typegpu-types.ts" && candidate.name == column_name)
        .and_then(class_alignment);
    Some(TypeTree::Matrix(Matrix {
        columns,
        rows,
        c_alignment: class_alignment(class),
        column_alignment,
    }))
}

struct Walker<'a> {
    module: &'a Module,
    stack: HashSet<usize>,
}

impl Walker<'_> {
    fn type_tree(
        &mut self,
        ty: &Type,
        field_name: &str,
        pos: &Pos,
    ) -> Result<TypeTree, Diagnostic> {
        match ty {
            Type::F32 => Ok(TypeTree::Scalar(Scalar::F32)),
            Type::I32 => Ok(TypeTree::Scalar(Scalar::I32)),
            Type::U32 => Ok(TypeTree::Scalar(Scalar::U32)),
            Type::F16 => Ok(TypeTree::Scalar(Scalar::F16)),
            Type::Bool => Err(diagnostic(
                "LY8",
                format!("field `{field_name}` has WGSL-incompatible type `boolean`. Use `u32`"),
                pos.clone(),
            )),
            Type::FixedArray(element, length) => Ok(TypeTree::Array(
                Box::new(self.type_tree(element, field_name, pos)?),
                *length,
            )),
            Type::Class(id) if is_bool_vector(self.module, ty) => Err(diagnostic(
                "SC5",
                format!(
                    "field `{field_name}` has non-host-shareable boolean vector type `{}`",
                    self.module.classes[id.0].name
                ),
                pos.clone(),
            )),
            Type::Class(id) if self.module.classes[id.0].is_value => self.class_tree(id.0),
            other => Err(diagnostic(
                "SC3",
                format!(
                    "field `{field_name}` has illegal schema type `{}`",
                    subscript_compiler::types::display_type(
                        other,
                        &|id| self.module.classes[id.0].name.clone(),
                        &|id| self.module.enums[id.0].name.clone(),
                        &|id| self.module.string_aliases[id.0].name.clone(),
                    )
                ),
                pos.clone(),
            )),
        }
    }

    fn class_tree(&mut self, index: usize) -> Result<TypeTree, Diagnostic> {
        let class = &self.module.classes[index];
        if let Some(tree) = library_tree(self.module, class) {
            return Ok(tree);
        }
        if class.fields.is_empty() {
            return Err(diagnostic(
                "LY9",
                format!("schema `{}` has no fields", class.name),
                class.pos.clone(),
            ));
        }
        if !self.stack.insert(index) {
            return Err(diagnostic(
                "SC3",
                format!("schema `{}` contains a value-class cycle", class.name),
                class.pos.clone(),
            ));
        }
        let members = class
            .fields
            .iter()
            .map(|field| {
                Ok(Member {
                    name: field.name.clone(),
                    ty: self.type_tree(&field.ty, &field.name, &field.pos)?,
                })
            })
            .collect::<Result<Vec<_>, Diagnostic>>();
        self.stack.remove(&index);
        Ok(TypeTree::Struct(Struct {
            name: class.name.clone(),
            members: members?,
            c_alignment: class_alignment(class),
        }))
    }
}

fn identity_diagnostic(schema: &Schema) -> Option<Diagnostic> {
    let wgsl = layout::wgsl_layout(&schema.tree);
    let c = layout::c_layout(&schema.tree);
    for (index, (wgsl_member, c_member)) in wgsl.members.iter().zip(&c.members).enumerate() {
        if wgsl_member.offset != c_member.offset {
            return Some(diagnostic(
                "SC9",
                format!(
                    "schema `{}` field `{}` has C offset {} and WGSL offset {}. Add an alignment override to the field class or reorder fields",
                    schema.name, wgsl_member.name, c_member.offset, wgsl_member.offset
                ),
                schema.field_positions[index].clone(),
            ));
        }
    }
    if wgsl.align != c.align {
        return Some(diagnostic(
            "SC9",
            format!(
                "schema `{}` has C alignment {} and WGSL alignment {}. Add an alignment override to the field class or reorder fields",
                schema.name, c.align, wgsl.align
            ),
            schema.pos.clone(),
        ));
    }
    if wgsl.size != c.size {
        return Some(diagnostic(
            "SC9",
            format!(
                "schema `{}` has C size {} and WGSL size {}. Add an alignment override to the field class or reorder fields",
                schema.name, c.size, wgsl.size
            ),
            schema.pos.clone(),
        ));
    }
    None
}

fn uniform_violation(tree: &TypeTree, path: &str) -> Option<String> {
    match tree {
        TypeTree::Array(element, _) => {
            let stride = layout::wgsl_layout(tree).stride.unwrap_or(0);
            if !stride.is_multiple_of(16) {
                return Some(format!(
                    "member `{path}` has array stride {stride}, not a multiple of 16"
                ));
            }
            uniform_violation(element, path)
        }
        TypeTree::Struct(structure) => {
            let layout = layout::wgsl_layout(tree);
            for (member, member_layout) in structure.members.iter().zip(layout.members) {
                let member_path = if path.is_empty() {
                    member.name.clone()
                } else {
                    format!("{path}.{}", member.name)
                };
                if matches!(member.ty, TypeTree::Struct(_))
                    && !member_layout.offset.is_multiple_of(16)
                {
                    return Some(format!(
                        "member `{member_path}` has struct offset {}, not a multiple of 16",
                        member_layout.offset
                    ));
                }
                if let Some(violation) = uniform_violation(&member.ty, &member_path) {
                    return Some(violation);
                }
            }
            None
        }
        TypeTree::Scalar(_) | TypeTree::Vector(_) | TypeTree::Matrix(_) | TypeTree::Atomic(_) => {
            None
        }
    }
}

fn uniform_schema_names(module: &Module) -> BTreeSet<String> {
    module
        .classes
        .iter()
        .filter(|class| {
            class.pos.file == "typegpu.ts" && !class.is_value && class.name.starts_with("Uniform<")
        })
        .filter_map(|class| class.fields.iter().find(|field| field.name == "values"))
        .filter_map(|field| match &field.ty {
            Type::Class(id) => Some(module.classes[id.0].name.clone()),
            Type::Array(element) => match &**element {
                Type::Class(id) => Some(module.classes[id.0].name.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

fn collect_reachable(module: &Module, index: usize, reachable: &mut BTreeSet<usize>) {
    if !reachable.insert(index) {
        return;
    }
    for field in &module.classes[index].fields {
        collect_type_reachable(module, &field.ty, reachable);
    }
}

fn collect_type_reachable(module: &Module, ty: &Type, reachable: &mut BTreeSet<usize>) {
    match ty {
        Type::FixedArray(element, _) => collect_type_reachable(module, element, reachable),
        Type::Class(id)
            if module.classes[id.0].is_value
                && library_tree(module, &module.classes[id.0]).is_none()
                && !is_bool_vector(module, ty) =>
        {
            collect_reachable(module, id.0, reachable);
        }
        _ => {}
    }
}

pub(crate) fn discover(
    module: &Module,
    intended: &BTreeSet<String>,
    import_pos: Option<&Pos>,
) -> Result<Vec<Schema>, Vec<Diagnostic>> {
    let uniform_names = uniform_schema_names(module);
    let mut schemas = Vec::new();
    let mut diagnostics = Vec::new();
    let mut reachable = BTreeSet::new();
    for name in intended {
        let Some((index, class)) = module
            .classes
            .iter()
            .enumerate()
            .find(|(_, class)| class.name == *name && is_schema_candidate(class))
        else {
            diagnostics.push(diagnostic(
                "SC1",
                format!("`{name}` is not a schema"),
                import_pos.cloned().unwrap_or_else(|| Pos::new("", 1, 1)),
            ));
            continue;
        };
        if !class.is_value || class.is_boundary {
            diagnostics.push(diagnostic(
                "SC1",
                format!("`{name}` is not a schema"),
                class.pos.clone(),
            ));
            continue;
        }
        collect_reachable(module, index, &mut reachable);
    }
    for (index, class) in module.classes.iter().enumerate() {
        if !reachable.contains(&index) {
            continue;
        }
        for field in &class.fields {
            if field.name.contains('_') {
                diagnostics.push(diagnostic(
                    "SC11",
                    format!(
                        "schema `{}` field `{}` contains `_`, which makes layout constant names ambiguous",
                        class.name, field.name
                    ),
                    field.pos.clone(),
                ));
            }
        }
        let mut walker = Walker {
            module,
            stack: HashSet::new(),
        };
        match walker.class_tree(index) {
            Ok(tree) => {
                let schema = Schema {
                    name: class.name.clone(),
                    tree,
                    pos: class.pos.clone(),
                    field_positions: class.fields.iter().map(|field| field.pos.clone()).collect(),
                };
                if let Some(error) = identity_diagnostic(&schema) {
                    diagnostics.push(error);
                } else if uniform_names.contains(&schema.name) {
                    if let Some(message) = uniform_violation(&schema.tree, "") {
                        diagnostics.push(diagnostic(
                            "SC10",
                            format!(
                                "uniform schema `{}` {message}. Add `@CStruct({{ align: 16 }})` to the member class or wrap the array element",
                                schema.name
                            ),
                            schema.pos.clone(),
                        ));
                    }
                }
                schemas.push(schema);
            }
            Err(error) => diagnostics.push(error),
        }
    }
    if diagnostics.is_empty() {
        Ok(schemas)
    } else {
        Err(diagnostics)
    }
}
