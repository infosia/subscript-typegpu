//! Support-module and WGSL struct emission.

use std::collections::BTreeSet;

use crate::layout::{self, Member, Scalar, TypeTree};
use crate::pipeline::Pipeline;
use crate::schema::Schema;
use subscript_compiler::{hir::Module, Type};

fn wgsl_type(tree: &TypeTree) -> String {
    match tree {
        TypeTree::Scalar(scalar) => scalar.wgsl().to_owned(),
        TypeTree::Vector(vector) => {
            format!("vec{}<{}>", vector.lanes, vector.scalar.wgsl())
        }
        TypeTree::Matrix(matrix) => format!("mat{}x{}<f32>", matrix.columns, matrix.rows),
        TypeTree::Array(element, length) => {
            format!("array<{}, {length}>", wgsl_type(element))
        }
        TypeTree::Struct(structure) => crate::mapping::ident(&structure.name),
    }
}

pub(crate) fn uses_f16(tree: &TypeTree) -> bool {
    match tree {
        TypeTree::Scalar(Scalar::F16) => true,
        TypeTree::Vector(vector) => vector.scalar == Scalar::F16,
        TypeTree::Matrix(_) => false,
        TypeTree::Array(element, _) => uses_f16(element),
        TypeTree::Struct(structure) => structure.members.iter().any(|member| uses_f16(&member.ty)),
        TypeTree::Scalar(_) => false,
    }
}

pub(crate) fn wgsl_struct(schema: &Schema) -> String {
    let TypeTree::Struct(structure) = &schema.tree else {
        return String::new();
    };
    let mut out = String::new();
    out.push_str(&format!(
        "struct {} {{\n",
        crate::mapping::ident(&structure.name)
    ));
    for member in &structure.members {
        out.push_str(&format!(
            "  {}: {},\n",
            crate::mapping::ident(&member.name),
            wgsl_type(&member.ty)
        ));
    }
    out.push_str("}\n");
    out
}

pub(crate) fn wgsl_module(schemas: &[Schema], structs: &[(String, String)]) -> String {
    let mut out = String::new();
    if schemas.iter().any(|schema| uses_f16(&schema.tree)) {
        out.push_str("enable f16;\n\n");
    }
    for (index, (_, structure)) in structs.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(structure);
    }
    out
}

fn escape_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn emit_nested_offsets(
    out: &mut String,
    schema_name: &str,
    prefix: &str,
    base_offset: u32,
    members: &[Member],
    layout: &layout::Layout,
) {
    for (member, member_layout) in members.iter().zip(&layout.members) {
        let path = if prefix.is_empty() {
            member.name.clone()
        } else {
            format!("{prefix}_{}", member.name)
        };
        let offset = base_offset + member_layout.offset;
        out.push_str(&format!(
            "export const {schema_name}_OFFSET_{path}: u32 = {offset};\n"
        ));
        if matches!(member.ty, TypeTree::Array(_, _)) {
            out.push_str(&format!(
                "export const {schema_name}_STRIDE_{path}: u32 = {};\n",
                member_layout
                    .layout
                    .stride
                    .expect("array layout has a stride")
            ));
        }
        if let TypeTree::Struct(nested) = &member.ty {
            emit_nested_offsets(
                out,
                schema_name,
                &path,
                offset,
                &nested.members,
                &member_layout.layout,
            );
        }
    }
}

pub(crate) fn layout_export_names(
    schema_name: &str,
    tree: &TypeTree,
    names: &mut BTreeSet<String>,
) {
    fn visit(schema_name: &str, prefix: &str, tree: &TypeTree, names: &mut BTreeSet<String>) {
        let TypeTree::Struct(structure) = tree else {
            return;
        };
        for member in &structure.members {
            let path = if prefix.is_empty() {
                member.name.clone()
            } else {
                format!("{prefix}_{}", member.name)
            };
            names.insert(format!("{schema_name}_OFFSET_{path}"));
            match &member.ty {
                TypeTree::Struct(_) => visit(schema_name, &path, &member.ty, names),
                TypeTree::Array(_, _) => {
                    names.insert(format!("{schema_name}_STRIDE_{path}"));
                }
                _ => {}
            }
        }
    }
    visit(schema_name, "", tree, names);
}

fn binding_size(module: &Module, schemas: &[Schema], ty: &Type) -> u32 {
    let tree = match ty {
        Type::F32 => Some(TypeTree::Scalar(Scalar::F32)),
        Type::I32 => Some(TypeTree::Scalar(Scalar::I32)),
        Type::U32 => Some(TypeTree::Scalar(Scalar::U32)),
        Type::Class(id) => {
            let class = &module.classes[id.0];
            schemas
                .iter()
                .find(|schema| schema.name == class.name)
                .map(|schema| schema.tree.clone())
                .or_else(|| crate::schema::library_tree(module, class))
        }
        _ => None,
    };
    tree.map_or(0, |tree| layout::wgsl_layout(&tree).size)
}

pub(crate) fn support_module(
    module: &Module,
    schemas: &[Schema],
    wgsl: &[(String, String)],
    pipelines: &[Pipeline],
    pipeline_texts: &[(String, String)],
) -> String {
    let mut out = String::from("// Generated by subscript-typegpu-gen.\n\n");
    if !pipelines.is_empty() {
        out.push_str("import { BindGroupLayoutSpec, COMPUTE_VISIBILITY } from \"./typegpu\";\n\n");
    }
    for schema in schemas {
        let layout = layout::c_layout(&schema.tree);
        out.push_str(&format!(
            "export const {name}_SIZE: u32 = {size};\n\
             export const {name}_ALIGN: u32 = {align};\n\
             export const {name}_STRIDE: u32 = {stride};\n",
            name = schema.name,
            size = layout.size,
            align = layout.align,
            stride = layout::round_up(layout.size, layout.align),
        ));
        let TypeTree::Struct(structure) = &schema.tree else {
            continue;
        };
        emit_nested_offsets(&mut out, &schema.name, "", 0, &structure.members, &layout);
        let text = wgsl
            .iter()
            .find(|(name, _)| name == &schema.name)
            .map_or("", |(_, text)| text.as_str());
        out.push_str(&format!(
            "export const {}_WGSL: string = \"{}\";\n\n",
            schema.name,
            escape_string(text)
        ));
    }
    for pipeline in pipelines {
        let text = pipeline_texts
            .iter()
            .find(|(name, _)| name == &pipeline.declaration)
            .map_or("", |(_, text)| text.as_str());
        out.push_str(&format!(
            "export const {name}_WGSL: string = \"{wgsl}\";\n\
             export const {name}_ENTRY: string = \"{entry}\";\n\
             export const {name}_WORKGROUP_X: u32 = {x};\n\
             export const {name}_WORKGROUP_Y: u32 = {y};\n\
             export const {name}_WORKGROUP_Z: u32 = {z};\n",
            name = pipeline.declaration,
            wgsl = escape_string(text),
            entry = crate::mapping::ident(&pipeline.entry),
            x = pipeline.workgroup[0],
            y = pipeline.workgroup[1],
            z = pipeline.workgroup[2],
        ));
        for layout in &pipeline.layouts {
            out.push_str(&format!(
                "\nexport const {}_LAYOUT{}: BindGroupLayoutSpec = {{ entries: [\n",
                pipeline.declaration, layout.group
            ));
            for binding in &layout.bindings {
                out.push_str(&format!(
                    "  {{ binding: {}, visibility: COMPUTE_VISIBILITY, kind: \"{}\", minBindingSize: {} }},\n",
                    binding.index,
                    binding.kind.webgpu(),
                    binding_size(module, schemas, &binding.item_ty),
                ));
            }
            out.push_str("] };\n");
        }
        out.push('\n');
    }
    out
}
