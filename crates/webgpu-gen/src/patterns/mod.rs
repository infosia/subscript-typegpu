//! A new pattern is a new module, not a rewrite. Each module renders
//! the C declarations and Rust facade definitions for its pattern.

pub(crate) mod adapter_limits;
pub(crate) mod byte_pair;
pub(crate) mod constants;
pub(crate) mod descriptor;
pub(crate) mod descriptor_async;
pub(crate) mod device_events;
pub(crate) mod future_poll;
pub(crate) mod handle_array;
pub(crate) mod handles;
pub(crate) mod label;
pub(crate) mod map_async;
pub(crate) mod shader_wgsl;
pub(crate) mod strings;
pub(crate) mod sync;
pub(crate) mod typed_pair;
pub(crate) mod write_texture;

/// Renders a Rust function signature, wrapping parameters one per
/// line when the single-line form exceeds the width budget.
pub(crate) fn rust_signature(prefix: &str, params: &[String], suffix: &str) -> String {
    let single = format!("{prefix}({}){suffix}", params.join(", "));
    if single.len() <= 88 {
        return single;
    }
    let mut out = format!("{prefix}(\n");
    let indent = " ".repeat(prefix.len() - prefix.trim_start().len() + 4);
    for p in params {
        out.push_str(&format!("{indent}{p},\n"));
    }
    let close_indent = " ".repeat(prefix.len() - prefix.trim_start().len());
    out.push_str(&format!("{close_indent}){suffix}"));
    out
}
