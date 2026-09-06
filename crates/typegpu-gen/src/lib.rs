//! Typed schema layout and WGSL support generation.

#![deny(missing_docs)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable,
    clippy::todo,
    clippy::unimplemented,
    clippy::indexing_slicing
)]

mod emit;
mod kernel;
pub mod layout;
mod library;
mod mapping;
mod pipeline;
mod render;
mod schema;
mod shell;
mod ui_atlas;

/// The loader for the library sources that a program reaches, and its error type.
pub use library::{load_library_files, LibraryLoadError};
/// Atlas module generation for the ui library.
pub use ui_atlas::generate_ui_atlas;

use std::collections::BTreeSet;

use subscript_compiler::hir::{Expr, ExprKind, Module};
use subscript_compiler::{CheckOptions, Diagnostic, Pos, RuleCode, SourceFile};

use crate::layout::{Layout, TypeTree};

/// Formats a value as a WGSL `u32` literal with the `u` suffix that K14 requires.
pub(crate) fn wgsl_u32_literal(value: impl std::fmt::Display) -> String {
    format!("{value}u")
}

/// Formats a value as a WGSL `i32` literal with the `i` suffix that K14 requires.
///
/// The `i32` minimum becomes `(-2147483647i - 1i)`. WGSL reads `-2147483648i` as the negation of
/// a literal above the maximum, and Tint refuses that form.
pub(crate) fn wgsl_i32_literal(value: i64) -> String {
    if value == i64::from(i32::MIN) {
        "(-2147483647i - 1i)".to_owned()
    } else {
        format!("{value}i")
    }
}

/// Returns a type or function name without its generic arguments.
pub(crate) fn base_name(name: &str) -> &str {
    name.split('<').next().unwrap_or(name)
}

/// Reads one field of a descriptor literal by name.
///
/// The outer `None` reports that the expression is not a descriptor literal. The inner `None`
/// reports that the literal leaves the field unset.
pub(crate) fn descriptor_field<'a>(
    module: &Module,
    expr: &'a Expr,
    field_name: &str,
) -> Result<Option<Option<&'a Expr>>, Diagnostic> {
    let ExprKind::DescriptorLit { class, fields } = &expr.kind else {
        return Ok(None);
    };
    Ok(
        crate::class(module, class.0, "descriptor_field", &expr.pos)?
            .fields
            .iter()
            .position(|field| field.name == field_name)
            .map(|index| fields.get(index).and_then(Option::as_ref)),
    )
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

/// The program's import of its own support module, which does not exist during discovery (SC1a).
#[derive(Debug)]
struct SupportImport {
    /// The names the program imports.
    names: BTreeSet<String>,
    /// The import declaration's position, which carries every diagnostic about a missing name.
    pos: Pos,
}

/// Builds the diagnostic for a broken generator invariant.
///
/// `site` names the function that found the break. The `internal:` prefix marks the generator as
/// the source, never the author. A reader who sees one has found a generator defect.
pub(crate) fn internal(site: &str, what: impl std::fmt::Display, pos: &Pos) -> Diagnostic {
    Diagnostic::new(
        RuleCode::S100,
        format!("internal: {site}: {what}"),
        pos.clone(),
    )
}

/// Resolves one class id against the checked module.
///
/// # Errors
///
/// If the module holds no class at `index`, returns an internal diagnostic that names `site`. The
/// checker assigns every class id, so an absent entry is a generator defect.
pub(crate) fn class<'a>(
    module: &'a Module,
    index: usize,
    site: &str,
    pos: &Pos,
) -> Result<&'a subscript_compiler::hir::ClassDef, Diagnostic> {
    // The checker assigns each class id to an entry in this module.
    module
        .classes
        .get(index)
        .ok_or_else(|| internal(site, format!("missing class {index}"), pos))
}

/// Builds one author-facing diagnostic.
///
/// `rule` is the single rule id the diagnostic enforces, and the message names the author as the
/// source (SC14, PI13, K17).
fn diagnostic(rule: &str, message: impl Into<String>, pos: Pos) -> Diagnostic {
    Diagnostic::new(
        RuleCode::S100,
        format!("{rule}: {} (author)", message.into()),
        pos,
    )
}

/// Reports whether `name` is a registered library module (LB1).
fn is_library_file(name: &str) -> bool {
    library::LIBRARY_ORDER.contains(&name)
}

/// Builds the check options that let the program import a support module that does not exist yet.
///
/// The program's own `.ts` file is the one file that is neither ambient, nor a library module, nor
/// a support module. Its `./<stem>.typegpu` specifier becomes a poisoned module (SC1a).
///
/// # Errors
///
/// If `files` holds more than one program, returns an SC1 diagnostic. One run generates for one
/// program.
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
    if let Some((file, _)) = modules.get(1) {
        return Err(vec![diagnostic(
            "SC1",
            "one generator run received more than one program file",
            Pos::new(*file, 1, 1),
        )]);
    }
    let mut options = CheckOptions::default();
    options.poison_missing_modules = modules.into_iter().map(|(_, module)| module).collect();
    Ok(options)
}

/// Reads the program's support-module import from the discovery HIR (SC1a).
///
/// The result is `None` when the program imports no support module, which a program with no schema
/// and no pipeline does.
///
/// # Errors
///
/// If the module carries a second poisoned import, returns an SC1 diagnostic.
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
        [_, second, ..] => Err(vec![diagnostic(
            "SC1",
            "one generator run found more than one support-module import",
            second.pos.clone(),
        )]),
    }
}

/// Recovers the schema name from one imported constant name (SC11).
///
/// `X_SIZE` and `X_OFFSET_field` both name `X`. A field name never carries an underscore, so the
/// split is unambiguous. The result is `None` when the name is not a schema constant.
fn schema_name(export: &str) -> Option<&str> {
    // A resources class belongs to a layout class, not to a schema (PI8).
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

/// Returns the schema names the program's import names.
///
/// A pipeline declaration produces constants with the same suffixes as a schema (PI8), so the
/// declaration names drop out. The result seeds schema discovery, which adds the reachable classes.
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

/// Returns the names the generated support module exports.
///
/// The generator writes one export per line, so a line scan is exact. The caller compares the set
/// with the program's imported names and reports each name that no schema or pipeline produces.
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
    // One check runs, over a program that imports a module the generator has yet to write. The
    // poisoned import carries the imported names, and nothing here lowers this HIR (SC1a).
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
    // A shell keeps its subscript body for the host lane and never reaches the walker. A kernel is
    // the opposite, so one function cannot be both (K29).
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
    // A class is a schema when a schema use reaches it (SC1). The uses are the program's import,
    // the binding items, the vertex and instance schemas, and the kernel call graph.
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
    // Every name the emitter writes at module scope. A shell or a raw declaration that repeats one
    // of them is a diagnostic, so the set must be complete before the collision check (K30).
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
    // A varyings class carries `@builtin` and `@location` attributes and never gets a layout, so
    // one class cannot serve as both a varyings class and a schema (RN7).
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
            /// Appends `tree`'s struct name and its nested struct names to `names`, outermost
            /// first, with no repeat. `seen` carries the names already appended.
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
            // Only the structs this module references reach its text, in first-use order (K14). A
            // referenced struct pulls in the structs its members name.
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
    // The check runs on the finished support module. An imported name that no schema and no
    // pipeline produces reaches here, and the author sees one diagnostic per name (SC1a).
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

/// `Iterator::any` over a predicate that returns a diagnostic.
pub(crate) trait TryAny: Iterator + Sized {
    /// Returns `true` at the first item the predicate accepts, and stops there.
    ///
    /// # Errors
    ///
    /// Returns the first diagnostic the predicate returns. The remaining items stay unvisited.
    fn try_any(
        mut self,
        mut predicate: impl FnMut(Self::Item) -> Result<bool, Diagnostic>,
    ) -> Result<bool, Diagnostic> {
        for item in &mut self {
            if predicate(item)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
impl<I: Iterator> TryAny for I {}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented,
        clippy::indexing_slicing
    )]

    use super::*;
    use subscript_compiler::types::ClassId;
    use subscript_compiler::Type;

    #[test]
    fn missing_hir_class_returns_internal_diagnostics() {
        let module = subscript_compiler::check_program(&[SourceFile::new("empty.ts", "")])
            .expect("empty module");
        let pos = Pos::new("broken.ts", 7, 11);
        let ty = Type::Class(ClassId(module.classes.len()));
        let errors = [
            class(&module, module.classes.len(), "test lookup", &pos).unwrap_err(),
            kernel::wgsl_type(&module, &ty, &pos).unwrap_err(),
            pipeline::type_name(&module, &ty, &pos).unwrap_err(),
            pipeline::layout(&module, &ty, 0, &pos).unwrap_err(),
            render::type_uses_f16(&module, &ty, &pos).unwrap_err(),
            schema::is_bool_vector(&module, &ty, &pos).unwrap_err(),
        ];
        for error in errors {
            assert_eq!(error.code, RuleCode::S100);
            assert!(error.message.starts_with("internal:"));
            assert!(!error.message.contains("(author)"));
            assert!(!error.message.contains("(generator)"));
            assert_eq!(error.pos, pos);
        }
    }
}
