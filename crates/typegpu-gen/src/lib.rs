//! Typed schema layout and WGSL support generation.

mod emit;
mod kernel;
pub mod layout;
mod mapping;
mod pipeline;
mod render;
mod schema;

use std::collections::BTreeSet;

use subscript_compiler::{CheckOptions, Diagnostic, Pos, RuleCode, SourceFile};

use crate::layout::{Layout, TypeTree};

/// The generated layouts for one schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedLayout {
    /// The schema name.
    pub name: String,
    /// The schema type tree.
    pub tree: TypeTree,
    /// The C layout.
    pub c: Layout,
    /// The WGSL layout.
    pub wgsl: Layout,
}

/// Generated TypeGPU support for one checked program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Generated {
    /// The in-memory subscript support module.
    pub support_module: String,
    /// Schema names and their WGSL struct text.
    pub wgsl_structs: Vec<(String, String)>,
    /// The complete WGSL module.
    pub wgsl_module: String,
    /// Schema trees and their C and WGSL layouts.
    pub layouts: Vec<GeneratedLayout>,
    /// Pipeline declaration names and their complete WGSL modules.
    pub pipelines: Vec<(String, String)>,
}

#[derive(Debug)]
struct SupportImport {
    names: BTreeSet<String>,
    pos: Pos,
}

fn diagnostic(rule: &str, message: impl Into<String>, pos: Pos) -> Diagnostic {
    Diagnostic::new(
        RuleCode::S100,
        format!("{rule}: {} (author)", message.into()),
        pos,
    )
}

fn is_library_file(name: &str) -> bool {
    matches!(
        name,
        "webgpu.ts"
            | "typegpu-types.ts"
            | "typegpu.ts"
            | "subscript-typegpu.generated.d.ts"
            | "wire-enum-aliases.generated.d.ts"
    )
}

fn discovery_options(files: &[SourceFile]) -> Result<CheckOptions, Vec<Diagnostic>> {
    let mut modules = Vec::new();
    for file in files {
        if file.dts || is_library_file(&file.name) || file.name.ends_with(".typegpu.ts") {
            continue;
        }
        let Some(stem) = file.name.strip_suffix(".ts") else {
            continue;
        };
        modules.push((file.name.as_str(), format!("./{stem}.typegpu")));
    }
    if modules.len() > 1 {
        return Err(vec![diagnostic(
            "SC1",
            "one generator run received more than one program file",
            Pos::new(modules[1].0, 1, 1),
        )]);
    }
    let mut options = CheckOptions::default();
    options.poison_missing_modules = modules.into_iter().map(|(_, module)| module).collect();
    Ok(options)
}

fn support_import(
    module: &subscript_compiler::hir::Module,
) -> Result<Option<SupportImport>, Vec<Diagnostic>> {
    match module.poisoned_imports.as_slice() {
        [] => Ok(None),
        [support] => Ok(Some(SupportImport {
            names: support
                .names
                .iter()
                .map(|(imported, _)| imported.clone())
                .collect(),
            pos: support.pos.clone(),
        })),
        supports => Err(vec![diagnostic(
            "SC1",
            "one generator run found more than one support-module import",
            supports[1].pos.clone(),
        )]),
    }
}

fn schema_name(export: &str) -> Option<&str> {
    if export.ends_with("Resources") {
        return None;
    }
    for suffix in ["_SIZE", "_ALIGN", "_STRIDE", "_WGSL"] {
        if let Some(name) = export.strip_suffix(suffix) {
            return Some(name);
        }
    }
    export
        .split_once("_OFFSET_")
        .or_else(|| export.split_once("_STRIDE_"))
        .map(|(name, _)| name)
}

fn intended_schemas(support: Option<&SupportImport>) -> BTreeSet<String> {
    support
        .into_iter()
        .flat_map(|support| &support.names)
        .filter_map(|name| schema_name(name))
        .filter(|name| name.starts_with(|ch: char| ch.is_ascii_uppercase()))
        .map(str::to_owned)
        .collect()
}

fn generated_export_names(
    schemas: &[schema::Schema],
    pipelines: &[pipeline::Pipeline],
    render_pipelines: &[render::RenderPipeline],
) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for schema in schemas {
        names.extend([
            format!("{}_SIZE", schema.name),
            format!("{}_ALIGN", schema.name),
            format!("{}_STRIDE", schema.name),
            format!("{}_WGSL", schema.name),
        ]);
        emit::layout_export_names(&schema.name, &schema.tree, &mut names);
    }
    for pipeline in pipelines {
        names.extend([
            format!("{}_WGSL", pipeline.declaration),
            format!("{}_ENTRY", pipeline.declaration),
            format!("{}_WORKGROUP_X", pipeline.declaration),
            format!("{}_WORKGROUP_Y", pipeline.declaration),
            format!("{}_WORKGROUP_Z", pipeline.declaration),
        ]);
        for layout in &pipeline.layouts {
            names.insert(format!("{}_LAYOUT{}", pipeline.declaration, layout.group));
            names.insert(format!("{}Resources", layout.name));
            names.insert(format!("create{}Resources", layout.name));
            names.insert(emit::bind_group_factory_name(
                &pipeline.declaration,
                layout.group,
            ));
        }
    }
    for pipeline in render_pipelines {
        names.extend([
            format!("{}_WGSL", pipeline.declaration),
            format!("{}_VERTEX_ENTRY", pipeline.declaration),
            format!("{}_FRAGMENT_ENTRY", pipeline.declaration),
            format!("{}_TARGET_FORMAT", pipeline.declaration),
        ]);
        for layout in &pipeline.layouts {
            names.insert(format!("{}_LAYOUT{}", pipeline.declaration, layout.group));
            names.insert(format!("{}Resources", layout.name));
            names.insert(format!("create{}Resources", layout.name));
            names.insert(emit::bind_group_factory_name(
                &pipeline.declaration,
                layout.group,
            ));
        }
        for buffer in &pipeline.vertex_buffers {
            names.insert(format!(
                "{}_VERTEX_LAYOUT{}",
                pipeline.declaration, buffer.slot
            ));
        }
    }
    names
}

/// Checks source files and generates schema support.
///
/// # Errors
///
/// Returns compiler or schema diagnostics with source positions.
pub fn generate(files: &[SourceFile]) -> Result<Generated, Vec<Diagnostic>> {
    let options = discovery_options(files)?;
    let module = subscript_compiler::check_program_with(files, &options)?;
    let support = support_import(&module)?;
    let mut intended = intended_schemas(support.as_ref());
    let pipeline_definitions = pipeline::discover(&module)?;
    let render_definitions = render::discover(&module)?;
    intended.extend(pipeline::schema_names(&module, &pipeline_definitions));
    intended.extend(render::schema_names(&render_definitions));
    for pipeline in &pipeline_definitions {
        intended
            .extend(kernel::referenced_schema_names(&module, pipeline).map_err(|item| vec![item])?);
    }
    for pipeline in &render_definitions {
        intended.extend(
            kernel::referenced_render_schema_names(&module, pipeline).map_err(|item| vec![item])?,
        );
    }
    let schemas = schema::discover(&module, &intended, support.as_ref().map(|item| &item.pos))?;
    if let Some(pipeline) = render_definitions.iter().find(|pipeline| {
        schemas
            .iter()
            .any(|schema| schema.name == pipeline.varyings_name)
    }) {
        return Err(vec![diagnostic(
            "RN7",
            format!(
                "varyings `{}` is also a schema or binding item",
                pipeline.varyings_name
            ),
            pipeline.pos.clone(),
        )]);
    }
    if let Some(support) = &support {
        let exports = generated_export_names(&schemas, &pipeline_definitions, &render_definitions);
        let missing = support
            .names
            .iter()
            .filter(|name| !exports.contains(*name))
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(missing
                .into_iter()
                .map(|name| {
                    diagnostic(
                        "SC1",
                        format!("imported `{name}` is not a schema or pipeline fact"),
                        support.pos.clone(),
                    )
                })
                .collect());
        }
    }
    let wgsl_structs = schemas
        .iter()
        .map(|schema| (schema.name.clone(), emit::wgsl_struct(schema)))
        .collect::<Vec<_>>();
    let wgsl_module = emit::wgsl_module(&schemas, &wgsl_structs);
    let mut pipelines = pipeline_definitions
        .iter()
        .map(|pipeline| {
            fn append_tree(tree: &TypeTree, names: &mut Vec<String>, seen: &mut BTreeSet<String>) {
                let TypeTree::Struct(structure) = tree else {
                    return;
                };
                if seen.insert(structure.name.clone()) {
                    names.push(structure.name.clone());
                }
                for member in &structure.members {
                    append_tree(&member.ty, names, seen);
                }
            }
            let references = kernel::referenced_schema_names(&module, pipeline)?;
            let mut names = Vec::new();
            let mut seen = BTreeSet::new();
            for name in references {
                if let Some(schema) = schemas.iter().find(|schema| schema.name == name) {
                    append_tree(&schema.tree, &mut names, &mut seen);
                }
            }
            let selected_structs = names
                .iter()
                .filter_map(|name| {
                    wgsl_structs
                        .iter()
                        .find(|(schema, _)| schema == name)
                        .cloned()
                })
                .collect::<Vec<_>>();
            let uses_f16 = names.iter().any(|name| {
                schemas
                    .iter()
                    .find(|schema| &schema.name == name)
                    .is_some_and(|schema| emit::uses_f16(&schema.tree))
            });
            let text = kernel::emit(&module, pipeline, &selected_structs, uses_f16)?;
            Ok((pipeline.declaration.clone(), text))
        })
        .collect::<Result<Vec<_>, Diagnostic>>()
        .map_err(|diagnostic| vec![diagnostic])?;
    let render_texts = render_definitions
        .iter()
        .map(|pipeline| {
            let references = kernel::referenced_render_schema_names(&module, pipeline)?;
            let selected_structs = references
                .iter()
                .filter_map(|name| {
                    wgsl_structs
                        .iter()
                        .find(|(schema, _)| schema == name)
                        .cloned()
                })
                .collect::<Vec<_>>();
            let text = kernel::emit_render(&module, pipeline, &selected_structs, &schemas)?;
            Ok((pipeline.declaration.clone(), text))
        })
        .collect::<Result<Vec<_>, Diagnostic>>()
        .map_err(|diagnostic| vec![diagnostic])?;
    pipelines.extend(render_texts.iter().cloned());
    let support_module = emit::support_module(
        &module,
        &schemas,
        &wgsl_structs,
        &pipeline_definitions,
        &pipelines,
        &render_definitions,
        &render_texts,
    )
    .map_err(|diagnostic| vec![diagnostic])?;
    let layouts = schemas
        .iter()
        .map(|schema| GeneratedLayout {
            name: schema.name.clone(),
            tree: schema.tree.clone(),
            c: layout::c_layout(&schema.tree),
            wgsl: layout::wgsl_layout(&schema.tree),
        })
        .collect();
    Ok(Generated {
        support_module,
        wgsl_structs,
        wgsl_module,
        layouts,
        pipelines,
    })
}
