//! Constant-emission pattern: enum/bitflag values from webgpu.yml
//! (never hand-typed) surfaced as private Rust constants for the
//! generated code and suite evidence.

use crate::naming;
use crate::plan::{ConstKind, ConstSet};

/// Renders the public flag typedef/constants or plain enum.
pub(crate) fn c_const_set(set: &ConstSet) -> String {
    let ty = naming::subscript_typegpu_type(&set.name);
    match set.kind {
        ConstKind::Bitflag => {
            let mut out = format!("typedef uint64_t {ty};\n");
            for (name, _, value) in &set.rows {
                let c_value = value.replace('_', "");
                out.push_str(&format!(
                    "static const {ty} {} = {c_value};\n",
                    naming::facade_constant(name),
                ));
            }
            out.trim_end().to_string()
        }
        ConstKind::Enum => {
            let mut out = format!("typedef enum {ty} {{\n");
            for (name, _, value) in &set.rows {
                let c_value = value.replace('_', "");
                out.push_str(&format!(
                    "    {} = {c_value},\n",
                    naming::facade_constant(name),
                ));
            }
            out.push_str(&format!("}} {ty};"));
            out
        }
    }
}

/// Renders one policy-listed constant set. These sets are emitted for
/// the slice's benefit (e.g. `WGPUBufferUsage` bits) and may be unused
/// by the generated bodies, hence the per-constant `allow(dead_code)`.
pub(crate) fn rust_const_set(set: &ConstSet) -> String {
    let mut out = String::new();
    for (name, ty, value) in &set.rows {
        out.push_str(&format!(
            "/// webgpu.yml `{source}` value.\n#[allow(dead_code)]\nconst {name}: {ty} = {value};\n",
            source = set.source,
        ));
    }
    out
}

/// Public Rust aliases/constants corresponding to the generated header.
pub(crate) fn rust_subscript_typegpu_const_set(set: &ConstSet) -> String {
    let ty = naming::subscript_typegpu_type(&set.name);
    let rust_ty = match set.kind {
        ConstKind::Bitflag => "u64",
        ConstKind::Enum => "i32",
    };
    let mut out = format!(
        "/// `subscript-typegpu.h`: `{source}` scalar set.\npub type {ty} = {rust_ty};\n",
        source = set.source,
    );
    for (name, _, value) in &set.rows {
        out.push_str(&format!(
            "/// `subscript-typegpu.h`: `{source}` value.\npub const {public}: {ty} = {value};\n",
            source = set.source,
            public = naming::facade_constant(name),
        ));
    }
    out
}

/// The `WGPU_STRLEN` constant (value scheme from the yml `strlen`
/// constant: `usize_max` marks a null-terminated view).
pub(crate) fn rust_strlen_const() -> &'static str {
    "/// webgpu.yml constant `strlen` (`usize_max`): marks a\n\
     /// null-terminated `WGPUStringView`.\n\
     const WGPU_STRLEN: usize = usize::MAX;\n"
}

/// The callback-mode constant every registration uses (F6).
pub(crate) fn rust_mode_const(name: &str, value: u32) -> String {
    format!(
        "/// webgpu.yml enum `callback_mode`, entry `allow_process_events`\n\
         /// \u{2014} the only mode the facade registers (CLAUDE.md invariant 3).\n\
         const {name}: i32 = {};\n",
        crate::naming::hex_enum(value)
    )
}
