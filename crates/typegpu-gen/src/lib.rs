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

/// `load_program_files` and its library loader `read_libraries` report failures through `LibraryLoadError`.
pub use library::{load_program_files, registered_library, support_module_key, LibraryLoadError};

use std::collections::{BTreeMap, BTreeSet};

use subscript_compiler::hir::{Expr, ExprKind, Module, Symbol};
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

/// Returns the source spelling of a declaration symbol without its generic arguments.
///
/// A generic instance symbol spells `name<args>`. The result is `name`.
pub(crate) fn base_name(symbol: &Symbol) -> String {
    let mut source = symbol.source_name();
    if let Some(start) = source.find('<') {
        source.truncate(start);
    }
    source
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
    /// The declaration symbol of the kernel function, as a function reference carries it.
    pub kernel: Symbol,
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

/// Rejects two program classes that share one source name when one of them is a value class (K14).
///
/// The emitted WGSL names a struct by the class's source name, and the support module names the
/// layout constants by it. Two modules can each declare a class of one name. A value class then
/// has no single WGSL struct and no single set of layout constants.
///
/// # Errors
///
/// Returns a K14 diagnostic at the second class that names the shared source name.
fn validate_unique_classes(module: &Module) -> Result<(), Diagnostic> {
    let mut owners =
        std::collections::BTreeMap::<String, &subscript_compiler::hir::ClassDef>::new();
    for class in &module.classes {
        let name = class.name.clone();
        match owners.get(&name) {
            Some(owner) if owner.is_value || class.is_value => {
                return Err(diagnostic(
                    "K14",
                    format!("two program classes share the source name `{name}`"),
                    class.pos.clone(),
                ));
            }
            Some(_) => {}
            None => {
                owners.insert(name, class);
            }
        }
    }
    Ok(())
}

/// Rejects two used classes that share one source name when one of them is a layout class (K14).
///
/// The support module names a layout's resources class and its constructor by the layout's
/// source name, and names a schema's layout constants by the schema's source name. Two modules
/// can each declare a class of one name. A layout class then shares its exports with the other
/// class. `layouts` holds the layout class symbols, and `others` holds the schema and varyings
/// class symbols. A class that neither list names keeps its source name to itself.
///
/// # Errors
///
/// Returns a K14 diagnostic at the second class, in module order, that names the shared source
/// name.
fn validate_unique_layouts(
    module: &Module,
    layouts: &[&Symbol],
    others: &[&Symbol],
) -> Result<(), Diagnostic> {
    let mut owners = std::collections::BTreeMap::<&str, (&Symbol, bool)>::new();
    for class in &module.classes {
        let is_layout = layouts.contains(&&class.symbol);
        if !is_layout && !others.contains(&&class.symbol) {
            continue;
        }
        match owners.get(class.name.as_str()) {
            Some((owner, owner_is_layout))
                if **owner != class.symbol && (*owner_is_layout || is_layout) =>
            {
                return Err(diagnostic(
                    "K14",
                    format!("two program classes share the source name `{}`", class.name),
                    class.pos.clone(),
                ));
            }
            Some(_) => {}
            None => {
                owners.insert(class.name.as_str(), (&class.symbol, is_layout));
            }
        }
    }
    Ok(())
}

/// Builds the check options that let the program import a support module that does not exist yet.
///
/// An explicit entry selects the support specifier (SC1a). Without one, the sole ordinary source selects it.
///
/// # Errors
///
/// If no file is the entry and more than one ordinary source exists, returns an SC1 diagnostic.
fn discovery_options(files: &[SourceFile]) -> Result<CheckOptions, Vec<Diagnostic>> {
    let ordinary: Vec<_> = files
        .iter()
        .filter(|file| !file.dts && file.name.ends_with(".ts"))
        .map(|file| (file, subscript_compiler::module_key(&file.name)))
        .collect();
    let explicit = ordinary.iter().find(|(file, _)| file.entry);
    // Without an explicit entry, its registered siblings and generated support can accompany it.
    let candidate = explicit.or_else(|| {
        ordinary.iter().find(|(_, key)| {
            library::registered(library::directory_prefix(key), key).is_none()
                && !ordinary
                    .iter()
                    .any(|(_, other)| key.strip_suffix(".typegpu") == Some(other))
        })
    });
    let mut modules = Vec::new();
    if let Some((entry, key)) = candidate {
        let prefix = library::directory_prefix(key);
        let support = support_module_key(&entry.name);
        for (file, key) in &ordinary {
            if explicit.is_some() && !file.entry {
                continue;
            }
            if library::registered(prefix, key).is_none() && *key != support {
                modules.push((file.name.as_str(), support_module_key(&file.name)));
            }
        }
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

/// Merges imports of the entry's support module and keeps the import site for each name (SC1a).
fn support_imports(module: &Module, support_key: Option<&String>) -> BTreeMap<String, Pos> {
    let mut names = BTreeMap::new();
    let Some(key) = support_key else {
        return names;
    };
    for import in &module.poisoned_imports {
        if subscript_compiler::resolve_module_specifier(&import.pos.file, &import.module).as_ref()
            == Some(key)
        {
            for (name, _) in &import.names {
                names
                    .entry(name.clone())
                    .or_insert_with(|| import.pos.clone());
            }
        }
    }
    names
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
    support: &BTreeMap<String, Pos>,
    pipeline_declarations: &BTreeSet<String>,
) -> BTreeMap<String, Pos> {
    support
        .iter()
        .filter_map(|(fact, pos)| Some((schema_name(fact)?, pos)))
        .filter(|(name, _)| !pipeline_declarations.contains(*name))
        .map(|(name, pos)| (name.to_owned(), pos.clone()))
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

/// Selects each struct once. Direct struct members follow their parent in compute modules.
/// Render modules declare their directly referenced schemas first.
fn struct_closure(
    references: &[Symbol],
    schemas: &[schema::Schema],
    structs: &[(String, String)],
    render: Option<&render::RenderPipeline>,
) -> Vec<(String, String)> {
    let mut pending: std::collections::VecDeque<_> = references
        .iter()
        .filter_map(|symbol| {
            schemas
                .iter()
                .find(|schema| schema.symbol == *symbol)
                .map(|schema| &schema.tree)
        })
        .collect();
    let mut seen = BTreeSet::new();
    if let Some(pipeline) = render {
        seen.insert(pipeline.varyings_name.as_str());
        seen.extend(
            pipeline
                .vertex_buffers
                .iter()
                .map(|buffer| buffer.schema.as_str()),
        );
    }
    let mut selected = Vec::new();
    while let Some(tree) = pending.pop_front() {
        match tree {
            TypeTree::Array(element, _) if render.is_none() => pending.push_front(element),
            TypeTree::Array(element, _) => pending.push_back(element),
            TypeTree::Struct(structure) if seen.insert(&structure.name) => {
                if let Some(text) = structs.iter().find(|(name, _)| *name == structure.name) {
                    selected.push(text.clone());
                }
                if render.is_some() {
                    pending.extend(structure.members.iter().map(|member| &member.ty));
                } else {
                    for member in structure.members.iter().rev() {
                        pending.push_front(&member.ty);
                    }
                }
            }
            _ => {}
        }
    }
    selected
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
    // Every emitted struct name and layout constant name below is a source name, so the check
    // runs before the first one.
    validate_unique_classes(&module).map_err(|item| vec![item])?;
    let support = support_imports(&module, options.poison_missing_modules.first());
    let shell_program = shell::discover(&module)?;
    let pipeline_definitions = pipeline::discover(&module, &shell_program)?;
    let render_definitions = render::discover(&module)?;
    let kernel_names = pipeline_definitions
        .iter()
        .map(|pipeline| &pipeline.entry)
        .chain(
            render_definitions
                .iter()
                .flat_map(|pipeline| [&pipeline.vertex_entry, &pipeline.fragment_entry]),
        )
        .collect::<BTreeSet<_>>();
    // A shell keeps its subscript body for the host lane and never reaches the walker. A kernel is
    // the opposite, so one function cannot be both (K29).
    if let Some(shell) = shell_program
        .shells
        .iter()
        .find(|shell| kernel_names.contains(&shell.function))
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
    let imported = intended_schemas(&support, &pipeline_declarations);
    let mut intended = pipeline::schema_classes(&module, &pipeline_definitions);
    intended.extend(render::schema_classes(&render_definitions));
    for pipeline in &pipeline_definitions {
        intended.extend(
            kernel::referenced_schema_classes(&module, pipeline, &shell_program)
                .map_err(|item| vec![item])?,
        );
    }
    for pipeline in &render_definitions {
        intended.extend(
            kernel::referenced_render_schema_classes(&module, pipeline, &shell_program)
                .map_err(|item| vec![item])?,
        );
    }
    let schemas = schema::discover(&module, &intended, &imported)?;
    // The support module names each layout's exports by the layout's source name, so the check
    // runs before the first export.
    let layout_symbols = all_layouts
        .iter()
        .map(|layout| &layout.symbol)
        .collect::<Vec<_>>();
    let other_symbols = schemas
        .iter()
        .map(|schema| &schema.symbol)
        .chain(
            render_definitions
                .iter()
                .map(|pipeline| &pipeline.varyings_symbol),
        )
        .collect::<Vec<_>>();
    validate_unique_layouts(&module, &layout_symbols, &other_symbols).map_err(|item| vec![item])?;
    // Every name the emitter writes at module scope. A shell or a raw declaration that repeats one
    // of them is a diagnostic, so the set must be complete before the collision check (K30).
    let mut generated_names = schemas
        .iter()
        .map(|schema| schema.name.clone())
        .collect::<BTreeSet<_>>();
    generated_names.extend(
        pipeline_definitions
            .iter()
            .map(|pipeline| base_name(&pipeline.entry)),
    );
    generated_names.extend(render_definitions.iter().flat_map(|pipeline| {
        [
            base_name(&pipeline.vertex_entry),
            base_name(&pipeline.fragment_entry),
        ]
    }));
    for layouts in pipeline_definitions
        .iter()
        .map(|pipeline| &pipeline.layouts)
        .chain(render_definitions.iter().map(|pipeline| &pipeline.layouts))
    {
        let shared = pipeline::shared_binding_fields(layouts);
        generated_names.extend(
            layouts
                .iter()
                .flat_map(|layout| &layout.bindings)
                .filter(|binding| !shared.contains(binding.name.as_str()))
                .map(|binding| binding.name.clone()),
        );
    }
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
            .any(|schema| schema.symbol == pipeline.varyings_symbol)
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
            let references = kernel::referenced_schema_classes(&module, pipeline, &shell_program)?;
            let selected_structs = struct_closure(&references, &schemas, &wgsl_structs, None);
            let uses_f16 = references.iter().any(|symbol| {
                schemas
                    .iter()
                    .any(|schema| schema.symbol == *symbol && emit::uses_f16(&schema.tree))
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
                kernel::referenced_render_schema_classes(&module, pipeline, &shell_program)?;
            let selected_structs =
                struct_closure(&references, &schemas, &wgsl_structs, Some(pipeline));
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
    let exports = support_export_names(&support_module);
    let missing: Vec<_> = support
        .into_iter()
        .filter(|(name, _)| !exports.contains(name.as_str()))
        .map(|(name, pos)| {
            diagnostic(
                "SC1",
                format!("imported `{name}` is not a schema or pipeline fact"),
                pos,
            )
        })
        .collect();
    if !missing.is_empty() {
        return Err(missing);
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
