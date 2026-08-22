//! Emits the harness facade symbol table from the resolved plan.

use crate::naming;
use crate::plan::{Chunk, Plan};

pub(crate) struct GeneratedNativeSymbols {
    pub(crate) source: String,
    pub(crate) names: Vec<String>,
}

fn export_names(plan: &Plan) -> Vec<String> {
    let mut names = Vec::new();
    names.extend(
        plan.creates
            .iter()
            .map(|item| item.subscript_typegpu_fn.clone()),
    );
    names.extend(
        plan.anchor_syncs
            .iter()
            .map(|item| item.subscript_typegpu_fn.clone()),
    );
    names.push(format!(
        "subscript_typegpu_{}_release",
        naming::snake(&plan.anchor)
    ));
    for chunk in &plan.chunks {
        match chunk {
            Chunk::Async(item) => {
                names.push(item.subscript_typegpu_fn.clone());
                if item.device_descriptor {
                    names.push(format!("{}_with_descriptor", item.subscript_typegpu_fn));
                }
                if item.first {
                    names.push("subscript_typegpu_future_status".to_owned());
                    names.push("subscript_typegpu_future_drop".to_owned());
                }
                if let Some(take) = &item.take_fn {
                    names.push(take.clone());
                }
            }
            Chunk::Sync(item) => names.push(item.subscript_typegpu_fn.clone()),
            Chunk::Descriptor(item) => names.push(item.subscript_typegpu_fn.clone()),
            Chunk::DescriptorAsync(item) => {
                names.push(item.async_op.subscript_typegpu_fn.clone());
                if let Some(take) = &item.async_op.take_fn {
                    names.push(take.clone());
                }
            }
            Chunk::ShaderWgsl(item) => names.push(item.subscript_typegpu_fn.clone()),
            Chunk::Label(item) => names.push(item.subscript_typegpu_fn.clone()),
            Chunk::BytePair(item) => names.push(item.subscript_typegpu_fn.clone()),
            Chunk::TypedPair(item) => names.push(item.subscript_typegpu_fn.clone()),
            Chunk::Array(item) => names.push(item.subscript_typegpu_fn.clone()),
            Chunk::MapAsync(item) => {
                names.push(item.async_op.subscript_typegpu_fn.clone());
                names.push(item.whole_subscript_typegpu_fn.clone());
            }
            Chunk::WriteTexture(item) => names.push(item.subscript_typegpu_fn.clone()),
            Chunk::DeviceEvents(item) => {
                names.push(item.subscript_typegpu_fn.clone());
                names.push(item.take_fn.clone());
                names.push("subscript_typegpu_device_next_uncaptured_error".to_owned());
                names.push("subscript_typegpu_device_lost_info".to_owned());
            }
            Chunk::Limits(item) => names.push(item.subscript_typegpu_fn.clone()),
            Chunk::AdapterInfo(item) => names.push(item.subscript_typegpu_fn.clone()),
            Chunk::Feature(item) => names.push(item.subscript_typegpu_fn.clone()),
        }
    }
    names.extend(
        plan.objects
            .iter()
            .rev()
            .filter(|object| **object != plan.anchor)
            .map(|object| format!("subscript_typegpu_{}_release", naming::snake(object))),
    );
    names
}

fn rust_signature<'a>(rust: &'a str, name: &str) -> (&'a str, Vec<&'a str>, &'a str) {
    let marker = format!("pub extern \"C\" fn {name}(");
    let start = rust
        .find(&marker)
        .unwrap_or_else(|| panic!("facade Rust lacks export `{name}`"));
    let signature = &rust[start + marker.len() - 1..];
    let open = 0;
    let mut depth = 0usize;
    let mut close = None;
    for (offset, byte) in signature[open..].bytes().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(open + offset);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close.unwrap_or_else(|| panic!("facade export `{name}` has unclosed parameters"));
    let params = &signature[open + 1..close];
    let result_tail = &signature[close + 1..];
    let body = result_tail
        .find('{')
        .unwrap_or_else(|| panic!("facade export `{name}` lacks a body"));
    let result = result_tail[..body].trim();
    let arguments = params
        .split(',')
        .map(str::trim)
        .filter(|param| !param.is_empty())
        .map(|param| {
            param
                .split_once(':')
                .map_or_else(
                    || panic!("facade export `{name}` has malformed `{param}`"),
                    |item| item.0,
                )
                .trim()
                .trim_start_matches("mut ")
        })
        .collect();
    (params, arguments, result)
}

pub(crate) fn render(plan: &Plan, rust: &str) -> GeneratedNativeSymbols {
    let names = export_names(plan);
    let mut source = String::from(
        "// GENERATED FILE — DO NOT EDIT.\n//\n// Facade exports emitted from the resolved generator plan.\n\n#![allow(non_snake_case)]\n\n",
    );
    source
        .push_str("use subscript_typegpu_facade as facade;\nuse subscript_typegpu_facade::*;\n\n");
    for (index, name) in names.iter().enumerate() {
        let (params, arguments, result) = rust_signature(rust, name);
        let result = if result.is_empty() {
            String::new()
        } else {
            format!(" {result}")
        };
        source.push_str(&format!(
            "extern \"C\" fn coverage_{index}({params}){result} {{\n    super::coverage_hit({index});\n    facade::{name}({})\n}}\n\n",
            arguments.join(", ")
        ));
    }
    source.push_str("pub fn facade_export_names() -> &'static [&'static str] {\n    &[\n");
    for name in &names {
        source.push_str(&format!("        \"{name}\",\n"));
    }
    source.push_str("    ]\n}\n\n");
    source.push_str("pub fn facade_symbols() -> Vec<(String, *const u8)> {\n    vec![\n");
    for name in &names {
        source.push_str(&format!(
            "        (\"{name}\".to_owned(), facade::{name} as *const u8),\n"
        ));
    }
    source.push_str("    ]\n}\n\n");
    source.push_str("pub fn facade_counting_symbols() -> Vec<(String, *const u8)> {\n    vec![\n");
    for (index, name) in names.iter().enumerate() {
        source.push_str(&format!(
            "        (\"{name}\".to_owned(), coverage_{index} as *const u8),\n"
        ));
    }
    source.push_str("    ]\n}\n");
    GeneratedNativeSymbols { source, names }
}
