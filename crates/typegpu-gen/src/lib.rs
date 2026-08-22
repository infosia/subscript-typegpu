//! Typed schema layout and WGSL support generation.

mod emit;
pub mod layout;
mod schema;

use std::collections::BTreeSet;

use subscript_compiler::{Diagnostic, Pos, RuleCode, SourceFile};

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
}

#[derive(Debug)]
struct SupportImport {
    program_index: usize,
    module_name: String,
    names: BTreeSet<String>,
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

/// This import-statement scanner is a typed-HIR discovery deviation with R35 as its kill date.
fn scan_support_import(source: &str, target: &str) -> Result<BTreeSet<String>, String> {
    for statement in source.split_inclusive(';') {
        let mut statement = statement.trim_start();
        loop {
            if let Some(comment) = statement.strip_prefix("//") {
                statement = comment
                    .split_once('\n')
                    .map_or("", |(_, remainder)| remainder)
                    .trim_start();
                continue;
            }
            if let Some(comment) = statement.strip_prefix("/*") {
                statement = comment
                    .split_once("*/")
                    .map_or("", |(_, remainder)| remainder)
                    .trim_start();
                continue;
            }
            break;
        }
        if !statement.starts_with("import") {
            continue;
        }
        let double_quoted = format!("\"{target}\"");
        let single_quoted = format!("'{target}'");
        if !statement.contains(&double_quoted) && !statement.contains(&single_quoted) {
            continue;
        }
        let open = statement
            .find('{')
            .ok_or_else(|| format!("support import `{target}` has no named imports"))?;
        let close = statement[open + 1..]
            .find('}')
            .map(|index| open + 1 + index)
            .ok_or_else(|| format!("support import `{target}` has no closing brace"))?;
        let mut names = BTreeSet::new();
        for item in statement[open + 1..close].split(',') {
            let Some(name) = item.split_whitespace().next() else {
                continue;
            };
            if !name.is_empty() {
                names.insert(name.to_owned());
            }
        }
        return Ok(names);
    }
    Err(format!("cannot read confirmed support import `{target}`"))
}

fn support_import(files: &[SourceFile]) -> Result<Option<SupportImport>, Vec<Diagnostic>> {
    let mut found = Vec::new();
    for (program_index, file) in files.iter().enumerate() {
        if file.dts || is_library_file(&file.name) || file.name.ends_with(".typegpu.ts") {
            continue;
        }
        let Some(stem) = file.name.strip_suffix(".ts") else {
            continue;
        };
        let module_name = format!("./{stem}.typegpu");
        let specifiers = subscript_compiler::parse_import_specifiers(file)?;
        if !specifiers.iter().any(|specifier| specifier == &module_name) {
            continue;
        }
        let names = scan_support_import(&file.source, &module_name)
            .map_err(|message| vec![diagnostic("SC1", message, Pos::new(&file.name, 1, 1))])?;
        found.push(SupportImport {
            program_index,
            module_name,
            names,
        });
    }
    match found.len() {
        0 => Ok(None),
        1 => Ok(found.pop()),
        _ => Err(vec![diagnostic(
            "SC1",
            "one generator run found more than one support-module import",
            Pos::new(&files[found[1].program_index].name, 1, 1),
        )]),
    }
}

fn stub_file(support: &SupportImport) -> SourceFile {
    let mut source = String::new();
    for name in &support.names {
        let ty = if name.ends_with("_WGSL") {
            "string"
        } else {
            "u32"
        };
        source.push_str(&format!("export const {name}: {ty} = "));
        source.push_str(if ty == "string" { "\"\";\n" } else { "0;\n" });
    }
    SourceFile::new(
        format!("{}.ts", support.module_name.trim_start_matches("./")),
        source,
    )
}

fn schema_name(export: &str) -> Option<&str> {
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
        .map(str::to_owned)
        .collect()
}

fn imported_field<'a>(support: &'a SupportImport, schema: &str) -> &'a str {
    let prefix = format!("{schema}_OFFSET_");
    support
        .names
        .iter()
        .find_map(|name| name.strip_prefix(&prefix))
        .unwrap_or("<unknown>")
}

fn illegal_type(message: &str) -> &str {
    message
        .split('`')
        .nth(1)
        .filter(|value| !value.is_empty())
        .unwrap_or("<unknown>")
}

fn translate_illegal_fields(
    support: Option<&SupportImport>,
    intended: &BTreeSet<String>,
    diagnostics: &[Diagnostic],
) -> Option<Vec<Diagnostic>> {
    let support = support?;
    if intended.is_empty()
        || diagnostics
            .iter()
            .any(|item| !item.message.contains("outside the value-class whitelist"))
    {
        return None;
    }
    let mut translated = Vec::new();
    for item in diagnostics {
        let ty = illegal_type(&item.message);
        for name in intended {
            let field = imported_field(support, name);
            translated.push(diagnostic(
                "SC3",
                format!("schema `{name}` field `{field}` has illegal schema type `{ty}`"),
                item.pos.clone(),
            ));
        }
    }
    Some(translated)
}

fn generated_export_names(schemas: &[schema::Schema]) -> BTreeSet<String> {
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
    names
}

/// Checks source files and generates schema support.
///
/// # Errors
///
/// Returns compiler or schema diagnostics with source positions.
pub fn generate(files: &[SourceFile]) -> Result<Generated, Vec<Diagnostic>> {
    let support = support_import(files)?;
    let intended = intended_schemas(support.as_ref());
    let mut checked_files = files.to_vec();
    if let Some(support) = &support {
        checked_files.push(stub_file(support));
    }
    let module = match subscript_compiler::check_program(&checked_files) {
        Ok(module) => module,
        Err(diagnostics) => {
            return Err(
                translate_illegal_fields(support.as_ref(), &intended, &diagnostics)
                    .unwrap_or(diagnostics),
            )
        }
    };
    let schemas = schema::discover(&module, &intended)?;
    if let Some(support) = &support {
        let exports = generated_export_names(&schemas);
        let missing = support
            .names
            .iter()
            .filter(|name| !exports.contains(*name))
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            let file = &files[support.program_index];
            return Err(missing
                .into_iter()
                .map(|name| {
                    diagnostic(
                        "SC1",
                        format!("imported `{name}` is not a generated schema fact"),
                        Pos::new(&file.name, 1, 1),
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
    let support_module = emit::support_module(&schemas, &wgsl_structs);
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_stub_exports_only_imported_names() {
        let files = vec![SourceFile::new(
            "demo.ts",
            "// program header\nimport {\n  A_SIZE,\n  A_WGSL,\n} from\n  \"./demo.typegpu\";\n",
        )];
        let support = support_import(&files)
            .expect("parse support import")
            .expect("support import");
        assert_eq!(
            stub_file(&support).source,
            "export const A_SIZE: u32 = 0;\nexport const A_WGSL: string = \"\";\n"
        );
    }
}
