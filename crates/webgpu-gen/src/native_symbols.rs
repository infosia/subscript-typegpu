//! Emits the harness facade symbol table from the resolved plan.

use crate::naming;
use crate::plan::{Chunk, Plan};

/// The emitted harness symbol table plus the export names it declares.
pub(crate) struct GeneratedNativeSymbols {
    /// The `crates/harness/src/native_symbols.generated.rs` text.
    pub(crate) source: String,
    /// The exports in table order, after the F22 exclusions.
    pub(crate) names: Vec<String>,
}

/// The facade export names in declaration order, before the F22 exclusions.
///
/// The order follows the facade emission order. The creates come first, then the anchor sync
/// methods and the anchor release. The chunks follow in policy order, and the other releases
/// come last in reverse object order. The first async chunk contributes the shared status and
/// drop exports (F6).
pub(crate) fn export_names(plan: &Plan) -> Vec<String> {
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

fn rust_signature<'a>(
    rust: &'a str,
    name: &str,
) -> Result<(&'a str, Vec<&'a str>, &'a str), crate::policy::PolicyError> {
    let marker = format!("pub extern \"C\" fn {name}(");
    let start = rust.find(&marker).ok_or_else(|| {
        crate::internal(
            "native_symbols::rust_signature",
            format!("facade Rust lacks export `{name}`"),
        )
    })?;
    let signature = rust.get(start + marker.len() - 1..).ok_or_else(|| {
        crate::internal(
            "native_symbols::rust_signature",
            "invalid rust range start + marker.len() - 1..",
        )
    })?;
    let open = 0;
    let mut depth = 0usize;
    let mut close = None;
    for (offset, byte) in signature
        .get(open..)
        .ok_or_else(|| {
            crate::internal(
                "native_symbols::rust_signature",
                "invalid signature range open..",
            )
        })?
        .bytes()
        .enumerate()
    {
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
    let close = close.ok_or_else(|| {
        crate::internal(
            "native_symbols::rust_signature",
            format!("facade export `{name}` has unclosed parameters"),
        )
    })?;
    let params = signature.get(open + 1..close).ok_or_else(|| {
        crate::internal(
            "native_symbols::rust_signature",
            "invalid signature range open + 1..close",
        )
    })?;
    let result_tail = signature.get(close + 1..).ok_or_else(|| {
        crate::internal(
            "native_symbols::rust_signature",
            "invalid signature range close + 1..",
        )
    })?;
    let body = result_tail.find('{').ok_or_else(|| {
        crate::internal(
            "native_symbols::rust_signature",
            format!("facade export `{name}` lacks a body"),
        )
    })?;
    let result = result_tail
        .get(..body)
        .ok_or_else(|| {
            crate::internal(
                "native_symbols::rust_signature",
                "invalid result_tail range ..body",
            )
        })?
        .trim();
    let arguments = params
        .split(',')
        .map(str::trim)
        .filter(|param| !param.is_empty())
        .map(|param| {
            let (argument, _) = param.split_once(':').ok_or_else(|| {
                crate::internal(
                    "native_symbols::rust_signature",
                    format!("facade export `{name}` has malformed `{param}`"),
                )
            })?;
            Ok(argument.trim().trim_start_matches("mut "))
        })
        .collect::<Result<Vec<_>, crate::policy::PolicyError>>()?;
    Ok((params, arguments, result))
}

/// Renders the harness symbol table from the plan and the emitted facade source.
///
/// `rust` is the `generated.rs` text, read only for each export's parameter list and result.
/// `excluded_exports` drops the F22 rows. Each retained name also gains a shim that records the
/// call, so the harness counts the exports a program reaches.
pub(crate) fn render(
    plan: &Plan,
    rust: &str,
    excluded_exports: &std::collections::BTreeSet<String>,
) -> Result<GeneratedNativeSymbols, crate::policy::PolicyError> {
    let names = export_names(plan)
        .into_iter()
        .filter(|name| !excluded_exports.contains(name))
        .collect::<Vec<_>>();
    let mut source = String::from(
        "// GENERATED FILE — DO NOT EDIT.\n//\n// Facade exports emitted from the resolved generator plan.\n\n#![allow(non_snake_case)]\n\n",
    );
    source
        .push_str("use subscript_typegpu_facade as facade;\nuse subscript_typegpu_facade::*;\n\n");
    for (index, name) in names.iter().enumerate() {
        let (params, arguments, result) = rust_signature(rust, name)?;
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
    Ok(GeneratedNativeSymbols { source, names })
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable,
    clippy::todo,
    clippy::unimplemented,
    clippy::indexing_slicing
)]
mod tests {
    #[test]
    fn malformed_exports_return_internal_errors() {
        for source in [
            "",
            "pub extern \"C\" fn broken(",
            "pub extern \"C\" fn broken()",
            "pub extern \"C\" fn broken(value) {}",
        ] {
            let error = super::rust_signature(source, "broken").unwrap_err();
            let message = error.to_string();
            assert!(message.starts_with("internal: native_symbols::rust_signature:"));
            assert!(message.contains("broken"));
        }
    }
}
