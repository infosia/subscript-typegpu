//! Support-module and WGSL struct emission.

use std::collections::BTreeSet;

use crate::layout::{self, Member, Scalar, TypeTree};
use crate::pipeline::BindingKind;
use crate::pipeline::Pipeline;
use crate::render::RenderPipeline;
use crate::schema::Schema;
use subscript_compiler::{hir::Module, Diagnostic, Pos, RuleCode, Type};

/// Builds the name of the generated bind-group factory of one declaration and group (EG1).
///
/// The declaration name gains an uppercase first letter, so `step` with group 0 gives
/// `createStepBindGroup0`.
pub(crate) fn bind_group_factory_name(declaration: &str, group: u32) -> String {
    let mut chars = declaration.chars();
    let Some(first) = chars.next() else {
        return format!("createBindGroup{group}");
    };
    format!(
        "create{}{}BindGroup{group}",
        first.to_ascii_uppercase(),
        chars.as_str()
    )
}

/// Returns the API-layer handle class that one binding kind takes as its resource (TX4).
fn resource_type(kind: BindingKind) -> &'static str {
    match kind {
        BindingKind::Uniform
        | BindingKind::Storage
        | BindingKind::MutStorage
        | BindingKind::Guard => "GPUBuffer",
        BindingKind::Texture(_, _) | BindingKind::StorageTexture(_, _, _) => "GPUTextureView",
        BindingKind::Sampler => "GPUSampler",
    }
}

/// Returns the `BindingResource` factory that wraps one binding kind's handle (TX4).
fn resource_factory(kind: BindingKind) -> &'static str {
    match kind {
        BindingKind::Uniform
        | BindingKind::Storage
        | BindingKind::MutStorage
        | BindingKind::Guard => "bufferResource",
        BindingKind::Texture(_, _) | BindingKind::StorageTexture(_, _, _) => "textureResource",
        BindingKind::Sampler => "samplerResource",
    }
}

/// Appends the typed resources class of one layout class and its factory (EG1, PI8).
///
/// The three loops walk the same bindings in declaration order, so the class fields, the factory
/// parameters, and the returned members stay in one order. A guard binding is hidden: the runtime
/// supplies its buffer, and it never appears in the author's resource list (PI15).
fn emit_resources_class(out: &mut String, layout: &crate::pipeline::Layout) {
    out.push_str(&format!(
        "@Descriptor\nexport class {}Resources {{\n",
        layout.name
    ));
    for binding in layout
        .bindings
        .iter()
        .filter(|binding| binding.kind != BindingKind::Guard)
    {
        out.push_str(&format!(
            "  {}!: {};\n",
            binding.name,
            resource_type(binding.kind)
        ));
    }
    out.push_str("}\n\n");
    out.push_str(&format!(
        "export function create{}Resources(\n",
        layout.name
    ));
    for binding in layout
        .bindings
        .iter()
        .filter(|binding| binding.kind != BindingKind::Guard)
    {
        out.push_str(&format!(
            "  {}: {},\n",
            binding.name,
            resource_type(binding.kind)
        ));
    }
    out.push_str(&format!("): {}Resources {{\n  return {{\n", layout.name));
    for binding in layout
        .bindings
        .iter()
        .filter(|binding| binding.kind != BindingKind::Guard)
    {
        out.push_str(&format!("    {}: {},\n", binding.name, binding.name));
    }
    out.push_str("  };\n}\n\n");
}

/// Appends the bind-group factory of one declaration and layout class (EG1, PI8).
///
/// `pipeline_type` is `ComputePipeline` or `RenderPipeline` and names the parameter's type. The
/// resource list follows binding declaration order, which `createBindGroup` reads positionally.
/// The compute form passes the pipeline's guard buffer, which the runtime appends for a guard
/// entry (PI15).
fn emit_bind_group_factory(
    out: &mut String,
    declaration: &str,
    layout: &crate::pipeline::Layout,
    pipeline_type: &str,
) {
    let factory = bind_group_factory_name(declaration, layout.group);
    out.push_str(&format!(
        "export function {factory}(\n  device: GPUDevice,\n  pipeline: {pipeline_type},\n  resources: {}Resources,\n): GPUBindGroup {{\n  using layout = pipeline.bindGroupLayout({});\n  return createBindGroup(device, layout, {declaration}_LAYOUT{}, [\n",
        layout.name, layout.group, layout.group,
    ));
    for binding in layout
        .bindings
        .iter()
        .filter(|binding| binding.kind != BindingKind::Guard)
    {
        out.push_str(&format!(
            "    {}(resources.{}),\n",
            resource_factory(binding.kind),
            binding.name
        ));
    }
    if pipeline_type == "ComputePipeline" {
        out.push_str(&format!(
            "  ], pipeline.guardBuffer({}));\n}}\n",
            layout.group
        ));
    } else {
        out.push_str("  ]);\n}\n");
    }
}

/// Builds one author-facing diagnostic that names `rule`, the single rule it enforces.
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

/// Returns the WGSL spelling of one layout tree (LY12).
fn wgsl_type(tree: &TypeTree) -> String {
    match tree {
        TypeTree::Scalar(scalar) => scalar.wgsl().to_owned(),
        TypeTree::Vector(vector) => {
            format!("vec{}<{}>", vector.lanes, vector.scalar.wgsl())
        }
        TypeTree::Matrix(matrix) => format!("mat{}x{}<f32>", matrix.columns, matrix.rows),
        TypeTree::Atomic(scalar) => format!("atomic<{}>", scalar.wgsl()),
        TypeTree::Array(element, length) => {
            format!(
                "array<{}, {}>",
                wgsl_type(element),
                crate::wgsl_u32_literal(length)
            )
        }
        TypeTree::Struct(structure) => crate::mapping::ident(&structure.name),
    }
}

/// Reports whether the tree holds an `f16` scalar or vector, which puts `enable f16;` on the
/// module.
pub(crate) fn uses_f16(tree: &TypeTree) -> bool {
    match tree {
        TypeTree::Scalar(Scalar::F16) => true,
        TypeTree::Vector(vector) => vector.scalar == Scalar::F16,
        TypeTree::Matrix(_) => false,
        TypeTree::Atomic(_) => false,
        TypeTree::Array(element, _) => uses_f16(element),
        TypeTree::Struct(structure) => structure.members.iter().any(|member| uses_f16(&member.ty)),
        TypeTree::Scalar(_) => false,
    }
}

/// Renders the WGSL `struct` text of one schema, with its fields in declaration order (SC12).
///
/// The text carries no `@align` and no `@size` attribute. A schema whose tree is not a struct
/// gives an empty string.
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

/// Joins the schema structs into the support module's WGSL text.
///
/// The text opens with `enable f16;` when a schema holds an `f16` type. One blank line separates
/// the structs, which keep the order of `structs`.
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

/// Escapes a text for one double-quoted subscript string literal in the support module.
fn escape_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

/// Appends the `X_OFFSET_<path>` and `X_STRIDE_<path>` constants of one struct level (SC11).
///
/// `prefix` is the member path so far, empty at the schema root, and a nested path joins with an
/// underscore. `base_offset` is the path's offset from the schema start, so every emitted offset is
/// absolute. A field name never holds an underscore, so the paths stay unique.
///
/// # Errors
///
/// If an array member layout carries no stride, returns an internal diagnostic.
fn emit_nested_offsets(
    out: &mut String,
    schema_name: &str,
    prefix: &str,
    base_offset: u32,
    members: &[Member],
    layout: &layout::Layout,
    pos: &Pos,
) -> Result<(), Diagnostic> {
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
        // Every array member layout contains its element stride.
        if matches!(member.ty, TypeTree::Array(_, _)) {
            out.push_str(&format!(
                "export const {schema_name}_STRIDE_{path}: u32 = {};\n",
                member_layout.layout.stride.ok_or_else(|| crate::internal(
                    "emit::emit_nested_offsets",
                    "array layout has no stride",
                    pos
                ))?
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
                pos,
            )?;
        }
    }

    Ok(())
}

/// Returns the WGSL size of one binding item type, which becomes the entry's `minBindingSize`.
///
/// The size comes from the layout engine, never from the backend (PI8).
///
/// # Errors
///
/// If the engine cannot size the type, returns a PI5 diagnostic. The wrapper item set is closed,
/// so the case reaches an author who names a type outside it.
fn binding_size(
    module: &Module,
    schemas: &[Schema],
    ty: &Type,
    pos: &Pos,
) -> Result<u32, Diagnostic> {
    let tree = match ty {
        Type::F32 => Some(TypeTree::Scalar(Scalar::F32)),
        Type::I32 => Some(TypeTree::Scalar(Scalar::I32)),
        Type::U32 => Some(TypeTree::Scalar(Scalar::U32)),
        Type::Class(id) => {
            let class = crate::class(module, id.0, "emit::binding_size", pos)?;
            schemas
                .iter()
                .find(|schema| schema.name == class.name)
                .map(|schema| schema.tree.clone())
                .or_else(|| crate::schema::library_tree(module, class))
        }
        _ => None,
    };
    tree.map(|tree| layout::wgsl_layout(&tree).size)
        .ok_or_else(|| {
            diagnostic(
                "PI5",
                "binding item type has no WGSL layout size",
                pos.clone(),
            )
        })
}

/// Appends one `BindGroupLayoutEntrySpec` line to the layout spec (PI8, TX5).
///
/// `visibility` is the stage expression the caller computed: a compute declaration passes one
/// constant, a render declaration passes the stages that reach the binding (RN9).
///
/// # Errors
///
/// If the layout engine cannot size a buffer item, returns the PI5 diagnostic of `binding_size`.
fn emit_binding_entry(
    out: &mut String,
    module: &Module,
    schemas: &[Schema],
    binding: &crate::pipeline::Binding,
    visibility: &str,
) -> Result<(), Diagnostic> {
    // A texture, a storage texture, and a sampler carry no buffer size. The kind decides which
    // members the entry needs, and the runtime maps them to the API layer's descriptors (TX5).
    let tail = match binding.kind {
        BindingKind::Uniform | BindingKind::Storage | BindingKind::MutStorage => format!(
            "minBindingSize: {}",
            binding_size(module, schemas, &binding.item_ty, &binding.pos)?,
        ),
        BindingKind::Guard => "minBindingSize: 16".to_owned(),
        BindingKind::Texture(sample, dimension) => {
            let dimension = if dimension == crate::pipeline::TextureViewDimension::TwoD {
                String::new()
            } else {
                format!(", viewDimension: \"{}\"", dimension.webgpu())
            };
            format!(
                "minBindingSize: 0, sampleType: \"{}\"{dimension}",
                sample.webgpu(),
            )
        }
        BindingKind::StorageTexture(format, access, dimension) => {
            let dimension = if dimension == crate::pipeline::TextureViewDimension::TwoD {
                String::new()
            } else {
                format!(", viewDimension: \"{}\"", dimension.webgpu())
            };
            format!(
                "minBindingSize: 0, format: \"{}\", access: \"{}\"{dimension}",
                format.webgpu(),
                access.webgpu(),
            )
        }
        BindingKind::Sampler => "minBindingSize: 0, samplerType: \"filtering\"".to_owned(),
    };
    out.push_str(&format!(
        "  {{ binding: {}, visibility: {visibility}, kind: \"{}\", {tail} }},\n",
        binding.index,
        binding.kind.webgpu(),
    ));
    Ok(())
}

/// Renders the subscript support module of one program (SC13).
///
/// The module carries the layout constants of every schema (SC11). It also carries the pipeline
/// facts, the bind-group layout specs (PI8, RN10), and the resource factories (EG1).
///
/// # Errors
///
/// Returns a PI5 diagnostic when the layout engine cannot size a binding item, and an RN9
/// diagnostic when neither render kernel reaches a binding.
pub(crate) fn support_module(
    module: &Module,
    schemas: &[Schema],
    wgsl: &[(String, String)],
    pipelines: &[Pipeline],
    pipeline_texts: &[(String, String)],
    render_pipelines: &[RenderPipeline],
    render_texts: &[(String, String)],
) -> Result<String, Diagnostic> {
    let mut out = String::from("// Generated by subscript-typegpu-gen.\n\n");
    let has_layouts = pipelines
        .iter()
        .any(|pipeline| !pipeline.layouts.is_empty())
        || render_pipelines
            .iter()
            .any(|pipeline| !pipeline.layouts.is_empty());
    // The arms differ only in the imported names. Every name the text below writes must appear in
    // the arm this program takes, and a program with no declaration imports nothing.
    match (pipelines.is_empty(), render_pipelines.is_empty(), has_layouts) {
        (false, false, true) => out.push_str("import { BindGroupLayoutSpec, COMPUTE_VISIBILITY, ComputePipeline, FRAGMENT_VISIBILITY, RenderPipeline, VERTEX_VISIBILITY, VertexBufferLayoutSpec, bufferResource, createBindGroup, samplerResource, textureResource } from \"./typegpu\";\nimport { GPUBuffer, GPUDevice, GPUBindGroup, GPUSampler, GPUTextureView } from \"./webgpu\";\n\n"),
        (false, true, true) => out.push_str("import { BindGroupLayoutSpec, COMPUTE_VISIBILITY, ComputePipeline, bufferResource, createBindGroup, samplerResource, textureResource } from \"./typegpu\";\nimport { GPUBuffer, GPUDevice, GPUBindGroup, GPUSampler, GPUTextureView } from \"./webgpu\";\n\n"),
        (true, false, true) => out.push_str("import { BindGroupLayoutSpec, FRAGMENT_VISIBILITY, RenderPipeline, VERTEX_VISIBILITY, VertexBufferLayoutSpec, bufferResource, createBindGroup, samplerResource, textureResource } from \"./typegpu\";\nimport { GPUBuffer, GPUDevice, GPUBindGroup, GPUSampler, GPUTextureView } from \"./webgpu\";\n\n"),
        (false, false, false) => out.push_str("import { BindGroupLayoutSpec, COMPUTE_VISIBILITY, FRAGMENT_VISIBILITY, VERTEX_VISIBILITY, VertexBufferLayoutSpec } from \"./typegpu\";\n\n"),
        (false, true, false) => out.push_str("import { BindGroupLayoutSpec, COMPUTE_VISIBILITY } from \"./typegpu\";\n\n"),
        (true, false, false) => out.push_str("import { BindGroupLayoutSpec, FRAGMENT_VISIBILITY, VERTEX_VISIBILITY, VertexBufferLayoutSpec } from \"./typegpu\";\n\n"),
        (true, true, _) => {}
    }

    // Two declarations can share one layout class, and the resources class carries the layout
    // class name. The set keeps one declaration of it in the module.
    let mut emitted_resources = BTreeSet::new();
    for layout in pipelines
        .iter()
        .flat_map(|pipeline| &pipeline.layouts)
        .chain(
            render_pipelines
                .iter()
                .flat_map(|pipeline| &pipeline.layouts),
        )
    {
        if emitted_resources.insert(layout.name.clone()) {
            emit_resources_class(&mut out, layout);
        }
    }
    for schema in schemas {
        // The C layout is the host type's layout, and SC9 already proved it equal to the WGSL
        // layout. A program that sizes a buffer from these constants writes the right bytes (SC9).
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
        emit_nested_offsets(
            &mut out,
            &schema.name,
            "",
            0,
            &structure.members,
            &layout,
            &schema.pos,
        )?;
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
             export const {name}_WORKGROUP_Z: u32 = {z};\n\
             export const {name}_HOST_RUNNABLE: boolean = {host_runnable};\n",
            name = pipeline.declaration,
            wgsl = escape_string(text),
            entry = crate::mapping::ident(&pipeline.entry),
            x = pipeline.workgroup[0],
            y = pipeline.workgroup[1],
            z = pipeline.workgroup[2],
            host_runnable = pipeline.host_runnable,
        ));
        for layout in &pipeline.layouts {
            out.push_str(&format!(
                "\nexport const {}_LAYOUT{}: BindGroupLayoutSpec = {{ entries: [\n",
                pipeline.declaration, layout.group
            ));
            for binding in &layout.bindings {
                emit_binding_entry(&mut out, module, schemas, binding, "COMPUTE_VISIBILITY")?;
            }
            out.push_str("] };\n");
            emit_bind_group_factory(&mut out, &pipeline.declaration, layout, "ComputePipeline");
        }
        out.push('\n');
    }
    for pipeline in render_pipelines {
        let text = render_texts
            .iter()
            .find(|(name, _)| name == &pipeline.declaration)
            .map_or("", |(_, text)| text.as_str());
        out.push_str(&format!(
            "export const {name}_WGSL: string = \"{wgsl}\";\n\
             export const {name}_VERTEX_ENTRY: string = \"{vertex}\";\n\
             export const {name}_FRAGMENT_ENTRY: string = \"{fragment}\";\n\
             export const {name}_TARGET_FORMAT: GPUTextureFormat = \"{format}\";\n",
            name = pipeline.declaration,
            wgsl = escape_string(text),
            vertex = crate::mapping::ident(&pipeline.vertex_entry),
            fragment = crate::mapping::ident(&pipeline.fragment_entry),
            format = pipeline.target_format,
        ));
        if let Some(index_format) = &pipeline.index_format {
            out.push_str(&format!(
                "export const {name}_INDEX_FORMAT: GPUIndexFormat = \"{index_format}\";\n",
                name = pipeline.declaration,
            ));
        }
        for layout in &pipeline.layouts {
            out.push_str(&format!(
                "\nexport const {}_LAYOUT{}: BindGroupLayoutSpec = {{ entries: [\n",
                pipeline.declaration, layout.group
            ));
            for binding in &layout.bindings {
                let (vertex, fragment) = crate::render::binding_visibility(
                    module,
                    pipeline,
                    layout.group as usize,
                    &binding.name,
                )?;
                let visibility = match (vertex, fragment) {
                    (true, true) => "VERTEX_VISIBILITY + FRAGMENT_VISIBILITY",
                    (true, false) => "VERTEX_VISIBILITY",
                    (false, true) => "FRAGMENT_VISIBILITY",
                    (false, false) => {
                        return Err(diagnostic(
                            "RN9",
                            format!(
                                "binding `{}` is not reached by either render kernel",
                                binding.name
                            ),
                            binding.pos.clone(),
                        ));
                    }
                };
                emit_binding_entry(&mut out, module, schemas, binding, visibility)?;
            }
            out.push_str("] };\n");
            emit_bind_group_factory(&mut out, &pipeline.declaration, layout, "RenderPipeline");
        }
        for buffer in &pipeline.vertex_buffers {
            let schema = schemas
                .iter()
                .find(|schema| schema.name == buffer.schema)
                .ok_or_else(|| {
                    generator_diagnostic(
                        format!("vertex schema `{}` has no generated layout", buffer.schema),
                        pipeline.pos.clone(),
                    )
                })?;
            // The vertex buffer layout is the schema's layout: the stride is `X_STRIDE` and each
            // attribute sits at its member offset (RN4).
            let layout = layout::c_layout(&schema.tree);
            out.push_str(&format!(
                "\nexport const {}_VERTEX_LAYOUT{}: VertexBufferLayoutSpec = {{\n  arrayStride: {},\n  stepMode: \"{}\",\n  attributes: [\n",
                pipeline.declaration,
                buffer.slot,
                layout::round_up(layout.size, layout.align),
                buffer.step_mode,
            ));
            for (attribute, member) in buffer.attributes.iter().zip(&layout.members) {
                out.push_str(&format!(
                    "    {{ format: \"{}\", offset: {}, shaderLocation: {} }},\n",
                    attribute.format, member.offset, attribute.location,
                ));
            }
            out.push_str("  ],\n};\n");
        }
        out.push('\n');
    }
    Ok(out)
}
