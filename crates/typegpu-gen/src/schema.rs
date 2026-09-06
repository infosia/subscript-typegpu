//! Typed-HIR schema discovery and validation.

use std::collections::{BTreeSet, HashSet};

use subscript_compiler::hir::{ClassDef, Module};
use subscript_compiler::{Diagnostic, Pos, RuleCode, Type};

use crate::layout::{self, Matrix, Member, Scalar, Struct, TypeTree, Vector};

/// One schema class and the layout tree that the layout engine measures.
#[derive(Debug, Clone)]
pub(crate) struct Schema {
    /// The class name, as the author declared it.
    pub(crate) name: String,
    /// The layout tree, with the members in declaration order (SC2).
    pub(crate) tree: TypeTree,
    /// The class declaration position.
    pub(crate) pos: Pos,
    /// The field declaration positions, in declaration order.
    pub(crate) field_positions: Vec<Pos>,
}

fn diagnostic(rule: &str, message: impl Into<String>, pos: Pos) -> Diagnostic {
    Diagnostic::new(
        RuleCode::S100,
        format!("{rule}: {} (author)", message.into()),
        pos,
    )
}

/// Reports whether the type is a `Vec2b`, `Vec3b`, or `Vec4b` of `typegpu-types.ts` (K26).
///
/// WGSL gives `bool` no host-shareable layout, so a bool vector is never a schema field.
pub(crate) fn is_bool_vector(module: &Module, ty: &Type, pos: &Pos) -> Result<bool, Diagnostic> {
    Ok(matches!(ty, Type::Class(id)
        if crate::class(module, id.0, "schema::is_bool_vector", pos)?.pos.file == "typegpu-types.ts"
            && matches!(crate::class(module, id.0, "schema::is_bool_vector", pos)?.name.as_str(), "Vec2b" | "Vec3b" | "Vec4b")))
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

/// Builds the layout tree of a vector, matrix, or atomic class of `typegpu-types.ts` (SC5).
///
/// A class that another file declares gives `None`, so a name alone never makes a library type.
/// A matrix reads its column alignment from the `Vec<rows>f` class of the same file.
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
            Type::Class(id) if is_bool_vector(self.module, ty, pos)? => Err(diagnostic(
                "SC5",
                format!(
                    "field `{field_name}` has non-host-shareable boolean vector type `{}`",
                    crate::class(self.module, id.0, "schema::type_tree", pos)?.name
                ),
                pos.clone(),
            )),
            Type::Class(id)
                if crate::class(self.module, id.0, "schema::type_tree", pos)?.is_value =>
            {
                self.class_tree(id.0, pos)
            }
            other => Err(diagnostic(
                "SC3",
                format!(
                    "field `{field_name}` has illegal schema type `{}`",
                    crate::pipeline::type_name(self.module, other, pos)?
                ),
                pos.clone(),
            )),
        }
    }

    fn class_tree(&mut self, index: usize, pos: &Pos) -> Result<TypeTree, Diagnostic> {
        let class = crate::class(self.module, index, "schema::class_tree", pos)?;
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

fn identity_diagnostic(schema: &Schema) -> Result<Option<Diagnostic>, Diagnostic> {
    let wgsl = layout::wgsl_layout(&schema.tree);
    let c = layout::c_layout(&schema.tree);
    // Field positions and layout members follow the same schema field order.
    for (index, (wgsl_member, c_member)) in wgsl.members.iter().zip(&c.members).enumerate() {
        if wgsl_member.offset != c_member.offset {
            return Ok(Some(diagnostic(
                "SC9",
                format!(
                    "schema `{}` field `{}` has C offset {} and WGSL offset {}. Add an alignment override to the field class or reorder fields",
                    schema.name, wgsl_member.name, c_member.offset, wgsl_member.offset
                ),
                schema.field_positions.get(index).ok_or_else(|| crate::internal("schema::identity_diagnostic", "missing field position", &schema.pos))?.clone(),
            )));
        }
    }
    if wgsl.align != c.align {
        return Ok(Some(diagnostic(
            "SC9",
            format!(
                "schema `{}` has C alignment {} and WGSL alignment {}. Add an alignment override to the field class or reorder fields",
                schema.name, c.align, wgsl.align
            ),
            schema.pos.clone(),
        )));
    }
    if wgsl.size != c.size {
        return Ok(Some(diagnostic(
            "SC9",
            format!(
                "schema `{}` has C size {} and WGSL size {}. Add an alignment override to the field class or reorder fields",
                schema.name, c.size, wgsl.size
            ),
            schema.pos.clone(),
        )));
    }

    Ok(None)
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

fn uniform_schema_names(module: &Module) -> Result<BTreeSet<String>, Diagnostic> {
    let mut names = BTreeSet::new();
    for class in &module.classes {
        if class.pos.file != "typegpu.ts" || class.is_value || !class.name.starts_with("Uniform<") {
            continue;
        }
        let Some(field) = class.fields.iter().find(|field| field.name == "values") else {
            continue;
        };
        let ty = match &field.ty {
            Type::Array(element) => element.as_ref(),
            ty => ty,
        };
        if let Type::Class(id) = ty {
            names.insert(
                crate::class(module, id.0, "schema::uniform_schema_names", &field.pos)?
                    .name
                    .clone(),
            );
        }
    }
    Ok(names)
}

fn collect_reachable(
    module: &Module,
    index: usize,
    reachable: &mut BTreeSet<usize>,
    pos: &Pos,
) -> Result<(), Diagnostic> {
    if !reachable.insert(index) {
        return Ok(());
    }
    for field in &crate::class(module, index, "schema::collect_reachable", pos)?.fields {
        collect_type_reachable(module, &field.ty, reachable, &field.pos)?;
    }

    Ok(())
}

fn collect_type_reachable(
    module: &Module,
    ty: &Type,
    reachable: &mut BTreeSet<usize>,
    pos: &Pos,
) -> Result<(), Diagnostic> {
    match ty {
        Type::FixedArray(element, _) => collect_type_reachable(module, element, reachable, pos)?,
        Type::Class(id)
            if crate::class(module, id.0, "schema::collect_type_reachable", pos)?.is_value
                && library_tree(
                    module,
                    crate::class(module, id.0, "schema::collect_type_reachable", pos)?,
                )
                .is_none()
                && !is_bool_vector(module, ty, pos)? =>
        {
            collect_reachable(module, id.0, reachable, pos)?;
        }
        _ => {}
    }

    Ok(())
}

/// Collects every schema class that the intended names reach, with its layout tree.
///
/// `intended` names the schemas that the imports, the bindings, and the kernels require.
/// `import_pos` positions a diagnostic about a name that no class matches. The result follows
/// the module's class declaration order, not the order of `intended`.
///
/// # Errors
///
/// Returns every violation of the reachable classes. SC1 names a class that is not a schema, and
/// SC3 or SC5 names an illegal field type. SC9 names a layout mismatch, SC10 a uniform
/// violation, and SC11 a field name that holds `_`.
pub(crate) fn discover(
    module: &Module,
    intended: &BTreeSet<String>,
    import_pos: Option<&Pos>,
) -> Result<Vec<Schema>, Vec<Diagnostic>> {
    let uniform_names = uniform_schema_names(module).map_err(|error| vec![error])?;
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
        collect_reachable(module, index, &mut reachable, &class.pos)
            .map_err(|error| vec![error])?;
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
        match walker.class_tree(index, &class.pos) {
            Ok(tree) => {
                let schema = Schema {
                    name: class.name.clone(),
                    tree,
                    pos: class.pos.clone(),
                    field_positions: class.fields.iter().map(|field| field.pos.clone()).collect(),
                };
                if let Some(error) = identity_diagnostic(&schema).map_err(|error| vec![error])? {
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
