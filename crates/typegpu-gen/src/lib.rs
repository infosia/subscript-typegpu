//! Typed schema layout and WGSL support generation.

mod emit;
mod kernel;
pub mod layout;
mod mapping;
mod pipeline;
mod render;
mod schema;
mod shell;

use std::collections::BTreeSet;

use subscript_compiler::hir::{Expr, ExprKind, Module};
use subscript_compiler::{CheckOptions, Diagnostic, Pos, RuleCode, SourceFile};

use crate::layout::{Layout, TypeTree};

pub(crate) fn wgsl_u32_literal(value: impl std::fmt::Display) -> String {
    format!("{value}u")
}

pub(crate) fn wgsl_i32_literal(value: impl std::fmt::Display) -> String {
    format!("{value}i")
}

pub(crate) fn base_name(name: &str) -> &str {
    name.split('<').next().unwrap_or(name)
}

pub(crate) fn descriptor_field<'a>(
    module: &Module,
    expr: &'a Expr,
    field_name: &str,
) -> Option<Option<&'a Expr>> {
    let ExprKind::DescriptorLit { class, fields } = &expr.kind else {
        return None;
    };
    module.classes[class.0]
        .fields
        .iter()
        .position(|field| field.name == field_name)
        .map(|index| fields.get(index).and_then(Option::as_ref))
}

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

/// One generated compute pipeline and its host-simulation facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedComputePipeline {
    /// The module-level pipeline declaration.
    pub declaration: String,
    /// The named kernel function.
    pub kernel: String,
    /// Whether sequential host simulation preserves the kernel's behavior.
    pub host_runnable: bool,
}

/// A named author-WGSL line range in one generated module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedWgslSpan {
    /// The pipeline declaration that owns the module.
    pub pipeline: String,
    /// `shell <name>` or `declarations`.
    pub label: String,
    /// First one-based line in the range.
    pub start_line: u32,
    /// Last one-based line in the range.
    pub end_line: u32,
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
    /// Compute pipeline declarations and their host-simulation facts.
    pub compute_pipelines: Vec<GeneratedComputePipeline>,
    /// Recorded author-WGSL ranges for K31 attribution.
    pub wgsl_spans: Vec<GeneratedWgslSpan>,
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
            | "typegpu-noise.ts"
            | "typegpu-sdf.ts"
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

fn intended_schemas(
    support: Option<&SupportImport>,
    pipeline_declarations: &BTreeSet<String>,
) -> BTreeSet<String> {
    support
        .into_iter()
        .flat_map(|support| &support.names)
        .filter_map(|name| schema_name(name))
        .filter(|name| !pipeline_declarations.contains(*name))
        .map(str::to_owned)
        .collect()
}

fn support_export_names(source: &str) -> BTreeSet<&str> {
    source
        .lines()
        .filter_map(|line| {
            ["export const ", "export function ", "export class "]
                .into_iter()
                .find_map(|prefix| line.strip_prefix(prefix))
        })
        .map(|tail| {
            tail.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                .next()
                .unwrap_or("")
        })
        .filter(|name| !name.is_empty())
        .collect()
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
    let shell_program = shell::discover(&module)?;
    let pipeline_definitions = pipeline::discover(&module, &shell_program)?;
    let render_definitions = render::discover(&module)?;
    let kernel_names = pipeline_definitions
        .iter()
        .map(|pipeline| pipeline.entry.as_str())
        .chain(render_definitions.iter().flat_map(|pipeline| {
            [
                pipeline.vertex_entry.as_str(),
                pipeline.fragment_entry.as_str(),
            ]
        }))
        .collect::<BTreeSet<_>>();
    if let Some(shell) = shell_program
        .shells
        .iter()
        .find(|shell| kernel_names.contains(shell.function.as_str()))
    {
        return Err(vec![diagnostic(
            "K29",
            format!("WGSL shell `{}` is also a pipeline kernel", shell.name),
            shell.pos.clone(),
        )]);
    }
    let all_layouts = pipeline_definitions
        .iter()
        .flat_map(|pipeline| &pipeline.layouts)
        .chain(
            render_definitions
                .iter()
                .flat_map(|pipeline| &pipeline.layouts),
        )
        .cloned()
        .collect::<Vec<_>>();
    for shell in &shell_program.shells {
        shell::validate_signature(&module, shell, &all_layouts).map_err(|item| vec![item])?;
    }
    let pipeline_declarations = pipeline_definitions
        .iter()
        .map(|pipeline| pipeline.declaration.clone())
        .chain(
            render_definitions
                .iter()
                .map(|pipeline| pipeline.declaration.clone()),
        )
        .collect::<BTreeSet<_>>();
    let mut intended = intended_schemas(support.as_ref(), &pipeline_declarations);
    intended.extend(pipeline::schema_names(&module, &pipeline_definitions));
    intended.extend(render::schema_names(&render_definitions));
    for pipeline in &pipeline_definitions {
        intended.extend(
            kernel::referenced_schema_names(&module, pipeline, &shell_program)
                .map_err(|item| vec![item])?,
        );
    }
    for pipeline in &render_definitions {
        intended.extend(
            kernel::referenced_render_schema_names(&module, pipeline, &shell_program)
                .map_err(|item| vec![item])?,
        );
    }
    let schemas = schema::discover(&module, &intended, support.as_ref().map(|item| &item.pos))?;
    let mut generated_names = schemas
        .iter()
        .map(|schema| schema.name.clone())
        .collect::<BTreeSet<_>>();
    generated_names.extend(
        pipeline_definitions
            .iter()
            .map(|pipeline| pipeline.entry.clone()),
    );
    generated_names.extend(render_definitions.iter().flat_map(|pipeline| {
        [
            pipeline.vertex_entry.clone(),
            pipeline.fragment_entry.clone(),
        ]
    }));
    generated_names.extend(
        pipeline_definitions
            .iter()
            .flat_map(|pipeline| &pipeline.layouts)
            .chain(
                render_definitions
                    .iter()
                    .flat_map(|pipeline| &pipeline.layouts),
            )
            .flat_map(|layout| &layout.bindings)
            .map(|binding| binding.name.clone()),
    );
    for pipeline in &pipeline_definitions {
        generated_names.extend(
            kernel::reached_global_names(&module, pipeline, &shell_program)
                .map_err(|item| vec![item])?,
        );
    }
    for pipeline in &render_definitions {
        generated_names.extend(
            kernel::reached_render_global_names(&module, pipeline, &shell_program)
                .map_err(|item| vec![item])?,
        );
    }
    shell::validate_collisions(&shell_program, &generated_names)?;
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
    let wgsl_structs = schemas
        .iter()
        .map(|schema| (schema.name.clone(), emit::wgsl_struct(schema)))
        .collect::<Vec<_>>();
    let wgsl_module = emit::wgsl_module(&schemas, &wgsl_structs);
    let emitted_compute = pipeline_definitions
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
            let references = kernel::referenced_schema_names(&module, pipeline, &shell_program)?;
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
            let emitted = kernel::emit(
                &module,
                pipeline,
                &selected_structs,
                uses_f16,
                &shell_program,
            )?;
            Ok((pipeline.declaration.clone(), emitted))
        })
        .collect::<Result<Vec<_>, Diagnostic>>()
        .map_err(|diagnostic| vec![diagnostic])?;
    let emitted_render = render_definitions
        .iter()
        .map(|pipeline| {
            let references =
                kernel::referenced_render_schema_names(&module, pipeline, &shell_program)?;
            let selected_structs = references
                .iter()
                .filter_map(|name| {
                    wgsl_structs
                        .iter()
                        .find(|(schema, _)| schema == name)
                        .cloned()
                })
                .collect::<Vec<_>>();
            let emitted = kernel::emit_render(
                &module,
                pipeline,
                &selected_structs,
                &schemas,
                &shell_program,
            )?;
            Ok((pipeline.declaration.clone(), emitted))
        })
        .collect::<Result<Vec<_>, Diagnostic>>()
        .map_err(|diagnostic| vec![diagnostic])?;
    let mut pipelines = emitted_compute
        .iter()
        .map(|(name, emitted)| (name.clone(), emitted.text.clone()))
        .collect::<Vec<_>>();
    let render_texts = emitted_render
        .iter()
        .map(|(name, emitted)| (name.clone(), emitted.text.clone()))
        .collect::<Vec<_>>();
    pipelines.extend(render_texts.iter().cloned());
    let wgsl_spans = emitted_compute
        .iter()
        .chain(&emitted_render)
        .flat_map(|(pipeline, emitted)| {
            emitted.spans.iter().map(|span| GeneratedWgslSpan {
                pipeline: pipeline.clone(),
                label: span.label.clone(),
                start_line: span.start_line,
                end_line: span.end_line,
            })
        })
        .collect();
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
    if let Some(support) = &support {
        let exports = support_export_names(&support_module);
        let missing = support
            .names
            .iter()
            .filter(|name| !exports.contains(name.as_str()))
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
    let layouts = schemas
        .iter()
        .map(|schema| GeneratedLayout {
            name: schema.name.clone(),
            tree: schema.tree.clone(),
            c: layout::c_layout(&schema.tree),
            wgsl: layout::wgsl_layout(&schema.tree),
        })
        .collect();
    let compute_pipelines = pipeline_definitions
        .iter()
        .map(|pipeline| GeneratedComputePipeline {
            declaration: pipeline.declaration.clone(),
            kernel: pipeline.entry.clone(),
            host_runnable: pipeline.host_runnable,
        })
        .collect();
    Ok(Generated {
        support_module,
        wgsl_structs,
        wgsl_module,
        layouts,
        pipelines,
        compute_pipelines,
        wgsl_spans,
    })
}
