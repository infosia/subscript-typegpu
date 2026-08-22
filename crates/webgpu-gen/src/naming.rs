//! Name derivation from yml snake_case identifiers, following the
//! webgpu.h convention with repository facade prefixes.

/// `request_adapter` -> `RequestAdapter`; a yml double underscore is
/// one literal output underscore (`unorm10__10` -> `Unorm10_10`).
/// A token's interior casing is kept while its first character is title-cased
/// (`openGL` -> `OpenGL`); already-uppercase tokens (`GPU`, `WGSL`, `3D`)
/// remain unchanged.
pub(crate) fn pascal(name: &str) -> String {
    name.split("__")
        .map(pascal_words)
        .collect::<Vec<_>>()
        .join("_")
}

fn pascal_words(name: &str) -> String {
    name.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}

/// `buffer_offset` -> `bufferOffset` (C parameter style).
pub(crate) fn camel(name: &str) -> String {
    let pascal = pascal(name);
    let mut chars = pascal.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_ascii_lowercase().to_string() + chars.as_str(),
    }
}

/// `request_adapter` -> `REQUEST_ADAPTER` (slot-kind tag style).
pub(crate) fn upper_snake(name: &str) -> String {
    name.to_ascii_uppercase()
}

/// Converts a Pascal or camel identifier to snake case.
pub(crate) fn snake(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::new();
    for (index, current) in chars.iter().copied().enumerate() {
        let previous = index
            .checked_sub(1)
            .and_then(|value| chars.get(value))
            .copied();
        let next = chars.get(index + 1).copied();
        let boundary = current.is_ascii_uppercase()
            && previous.is_some_and(|value| value != '_')
            && (previous.is_some_and(|value| value.is_ascii_lowercase() || value.is_ascii_digit())
                || next.is_some_and(|value| value.is_ascii_lowercase()));
        if boundary {
            out.push('_');
        }
        out.push(current.to_ascii_lowercase());
    }
    out
}

/// Escapes Rust keywords used as webgpu.yml field names.
pub(crate) fn rust_ident(name: &str) -> String {
    match name {
        "as" | "break" | "const" | "continue" | "crate" | "else" | "enum" | "extern" | "false"
        | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move"
        | "mut" | "pub" | "ref" | "return" | "self" | "Self" | "static" | "struct" | "super"
        | "trait" | "true" | "type" | "unsafe" | "use" | "where" | "while" | "async" | "await"
        | "dyn" | "abstract" | "become" | "box" | "do" | "final" | "macro" | "override"
        | "priv" | "typeof" | "unsized" | "virtual" | "yield" | "try" => format!("r#{name}"),
        _ => name.to_string(),
    }
}

/// The webgpu.h type name of a yml construct: `WGPU` + Pascal.
pub(crate) fn wgpu_type(name: &str) -> String {
    format!("WGPU{}", pascal(name))
}

/// The facade handle type of a yml object.
pub(crate) fn subscript_typegpu_type(name: &str) -> String {
    format!("SubscriptTypegpu{}", pascal(name))
}

/// The webgpu.h export for `object.method`: `wgpu` + Pascal + Pascal.
pub(crate) fn wgpu_method(object: &str, method: &str) -> String {
    format!("wgpu{}{}", pascal(object), pascal(method))
}

/// The facade export for `object.method`.
pub(crate) fn subscript_typegpu_method(object: &str, method: &str) -> String {
    format!("subscript_typegpu_{object}_{method}")
}

/// The facade constant for a webgpu.h enum member.
pub(crate) fn facade_constant(name: &str) -> String {
    let suffix = name.strip_prefix("WGPU").unwrap_or(name);
    format!("SUBSCRIPT_TYPEGPU_{}", snake(suffix).to_ascii_uppercase())
}

/// The webgpu.h enum member constant: `WGPUEnum_Entry`.
pub(crate) fn wgpu_enum_member(enum_name: &str, entry: &str) -> String {
    format!("WGPU{}_{}", pascal(enum_name), pascal(entry))
}

/// Formats an `i32` enum value in webgpu.h hex style (`0x0000_0002`).
pub(crate) fn hex_enum(value: u32) -> String {
    format!("0x{:04X}_{:04X}", value >> 16, value & 0xFFFF)
}

/// Formats a `u64` flag value in compact hex (`0x200`).
pub(crate) fn hex_flag(value: u64) -> String {
    format!("0x{value:X}")
}
