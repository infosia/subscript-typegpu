//! F12 chain-free struct and descriptor pattern.

use crate::naming;
use crate::patterns::rust_signature;
use crate::plan::{DescriptorField, DescriptorFieldKind, DescriptorOp, SentinelConst, StructPlan};

/// The yml name of the construct that a field references.
///
/// A struct, object, enum, flag, or array-element field carries such a name. An absent name is
/// a plan defect, so the function returns an `internal:` error.
fn named(field: &DescriptorField) -> Result<&str, crate::policy::PolicyError> {
    field.named_type.as_deref().ok_or_else(|| {
        crate::internal(
            "patterns::descriptor::named",
            "missing named descriptor field type",
        )
    })
}

/// The C type of one scalar or named field of a chain-free struct.
///
/// A flag or enum field takes its facade typedef, so the header keeps the pinned numeric
/// values (F16). A struct-pointer field and an array field return an `internal:` error,
/// because `c_struct` emits both of those shapes itself.
fn c_field_type(field: &DescriptorField) -> Result<String, crate::policy::PolicyError> {
    Ok(match field.kind {
        DescriptorFieldKind::StringView => "SubscriptTypegpuStringView".into(),
        DescriptorFieldKind::Bitflag
        | DescriptorFieldKind::Enum
        | DescriptorFieldKind::Struct
        | DescriptorFieldKind::Object => naming::subscript_typegpu_type(named(field)?),
        DescriptorFieldKind::U32 => "uint32_t".into(),
        DescriptorFieldKind::U64 => "uint64_t".into(),
        DescriptorFieldKind::Usize => "size_t".into(),
        DescriptorFieldKind::U16 => "uint16_t".into(),
        DescriptorFieldKind::F32 => "float".into(),
        DescriptorFieldKind::F64 => "double".into(),
        DescriptorFieldKind::I32 => "int32_t".into(),
        DescriptorFieldKind::Bool => "bool".into(),
        DescriptorFieldKind::StructPointer => {
            return Err(crate::internal(
                "patterns::descriptor::c_field_type",
                "struct pointers are emitted specially",
            ))
        }
        DescriptorFieldKind::EnumArray
        | DescriptorFieldKind::StructArray
        | DescriptorFieldKind::ObjectArray => {
            return Err(crate::internal(
                "patterns::descriptor::c_field_type",
                "arrays expand to two fields",
            ))
        }
    })
}

/// Reports whether the field is one of the three array kinds that expand into a count and a
/// pointer (B1).
fn is_array(field: &DescriptorField) -> bool {
    matches!(
        field.kind,
        DescriptorFieldKind::EnumArray
            | DescriptorFieldKind::StructArray
            | DescriptorFieldKind::ObjectArray
    )
}

/// Renders the chain-free struct typedef for `subscript-typegpu.h`.
///
/// An array member expands into two fields, the `size_t` count first, in the yml member's
/// position (F12, B1). A nullable object member carries the `_Nullable` marker (C1). No chain
/// field enters the public struct.
pub(crate) fn c_struct(shape: &StructPlan) -> Result<String, crate::policy::PolicyError> {
    let mut out = format!("typedef struct {} {{\n", shape.subscript_typegpu_struct);
    for field in &shape.fields {
        let name = naming::camel(&field.name);
        if is_array(field) {
            let count = field.public_count_name.as_deref().ok_or_else(|| {
                crate::internal(
                    "patterns::descriptor::c_struct",
                    "missing public array count name",
                )
            })?;
            out.push_str(&format!("    size_t {count};\n"));
            out.push_str(&format!(
                "    const {}* {name};\n",
                naming::subscript_typegpu_type(named(field)?),
            ));
        } else if field.kind == DescriptorFieldKind::StructPointer {
            out.push_str(&format!(
                "    const {}* {name};\n",
                naming::subscript_typegpu_type(named(field)?)
            ));
        } else if field.nullable {
            out.push_str(&format!("    {} _Nullable {name};\n", c_field_type(field)?));
        } else {
            out.push_str(&format!("    {} {name};\n", c_field_type(field)?));
        }
    }
    out.push_str(&format!("}} {};", shape.subscript_typegpu_struct));
    Ok(out)
}

/// Renders the descriptor-taking create declaration for `subscript-typegpu.h`.
pub(crate) fn c_decl(op: &DescriptorOp, shape: &StructPlan) -> String {
    format!(
        "{} {}({} {}, const {}* descriptor);",
        naming::subscript_typegpu_type(&op.returns_object),
        op.subscript_typegpu_fn,
        naming::subscript_typegpu_type(&op.receiver),
        naming::camel(&op.receiver),
        shape.subscript_typegpu_struct,
    )
}

/// The Rust type of one struct field. `backend` selects the webgpu.h side.
///
/// A flag is `u64` and a plain enum is `i32` on both sides (F16). A boolean is `u32` on the
/// backend side, because the pinned header spells `WGPUBool` as `uint32_t`. An array field is
/// a `*const` element pointer, and its count travels in a separate field.
fn rust_field_type(
    field: &DescriptorField,
    backend: bool,
) -> Result<String, crate::policy::PolicyError> {
    Ok(match field.kind {
        DescriptorFieldKind::StringView => if backend {
            "WGPUStringView"
        } else {
            "SubscriptTypegpuStringView"
        }
        .into(),
        DescriptorFieldKind::Bitflag | DescriptorFieldKind::U64 => "u64".into(),
        DescriptorFieldKind::Usize => "usize".into(),
        DescriptorFieldKind::Enum => "i32".into(),
        DescriptorFieldKind::U32 => "u32".into(),
        DescriptorFieldKind::U16 => "u16".into(),
        DescriptorFieldKind::F32 => "f32".into(),
        DescriptorFieldKind::F64 => "f64".into(),
        DescriptorFieldKind::I32 => "i32".into(),
        DescriptorFieldKind::Bool => if backend { "u32" } else { "bool" }.into(),
        DescriptorFieldKind::Struct => {
            if backend {
                naming::wgpu_type(named(field)?)
            } else {
                naming::subscript_typegpu_type(named(field)?)
            }
        }
        DescriptorFieldKind::StructPointer => {
            let ty = if backend {
                naming::wgpu_type(named(field)?)
            } else {
                naming::subscript_typegpu_type(named(field)?)
            };
            format!("*const {ty}")
        }
        DescriptorFieldKind::Object => {
            if backend {
                naming::wgpu_type(named(field)?)
            } else {
                naming::subscript_typegpu_type(named(field)?)
            }
        }
        DescriptorFieldKind::EnumArray => "*const i32".into(),
        DescriptorFieldKind::StructArray | DescriptorFieldKind::ObjectArray => {
            let ty = if backend {
                naming::wgpu_type(named(field)?)
            } else {
                naming::subscript_typegpu_type(named(field)?)
            };
            format!("*const {ty}")
        }
    })
}

/// Renders one private sentinel constant for the generated facade.
///
/// The constant stays internal, because its value is at or above 2^53 and cannot cross the
/// boundary (F15). A zero-rule conversion substitutes it for a public zero.
pub(crate) fn rust_sentinel_const(sentinel: &SentinelConst) -> String {
    format!(
        "/// webgpu.yml `{}`; kept internal because it exceeds the exact script integer range.\n\
         const {}: u64 = {};\n",
        sentinel.source, sentinel.rust_name, sentinel.rust_value,
    )
}

/// Renders the private webgpu.h struct and the public boundary struct, in that order.
///
/// The backend struct keeps the yml field order and the chain head. The public struct replaces
/// each array with a count field and a pointer, and drops the chain (F12).
pub(crate) fn rust_structs(shape: &StructPlan) -> Result<String, crate::policy::PolicyError> {
    let derive = if shape.backend_copy {
        "#[derive(Clone, Copy)]\n"
    } else {
        ""
    };
    let mut out = format!(
        "/// webgpu.h `{}`.\n#[repr(C)]\n{derive}struct {} {{\n",
        shape.wgpu_struct, shape.wgpu_struct,
    );
    if shape.extensible {
        out.push_str("    next_in_chain: *mut WGPUChainedStruct,\n");
    }
    for field in &shape.fields {
        if is_array(field) {
            out.push_str(&format!(
                "    {}: usize,\n",
                field
                    .backend_count_name
                    .as_deref()
                    .ok_or_else(|| crate::internal(
                        "patterns::descriptor::rust_structs",
                        "missing backend array count name"
                    ))?,
            ));
        }
        out.push_str(&format!(
            "    {}: {},\n",
            naming::rust_ident(&field.name),
            rust_field_type(field, true)?,
        ));
    }
    out.push_str("}\n\n");
    out.push_str(&format!(
        "/// `subscript-typegpu.h`: chain-free struct.\n#[repr(C)]\n#[derive(Clone, Copy)]\npub struct {} {{\n",
        shape.subscript_typegpu_struct,
    ));
    for field in &shape.fields {
        if is_array(field) {
            out.push_str(&format!(
                "    /// Element count for `{}`.\n    pub {}_count: usize,\n",
                naming::camel(&field.name),
                naming::rust_ident(&field.name),
            ));
        }
        out.push_str(&format!(
            "    /// Struct field `{}`.\n    pub {}: {},\n",
            naming::camel(&field.name),
            naming::rust_ident(&field.name),
            rust_field_type(field, false)?,
        ));
    }
    out.push_str("}\n");
    Ok(out)
}

/// The expression that converts one public field into its backend field.
///
/// A zero-rule field routes through its own substitution function (F15). A struct pointer and
/// a struct array return an `internal:` error, because `rust_conversion` binds those two
/// shapes to locals before it builds the struct literal.
fn conversion_value(
    field: &DescriptorField,
    source: &str,
    shape_source: &str,
) -> Result<String, crate::policy::PolicyError> {
    let access = format!("{source}.{}", naming::rust_ident(&field.name));
    if field.zero_constant.is_some() {
        return Ok(format!(
            "convert_{shape_source}_{}_zero_rule({access})",
            field.name
        ));
    }
    Ok(match field.kind {
        DescriptorFieldKind::StringView => format!("wgpu_string_view({access})"),
        DescriptorFieldKind::Bool => format!("u32::from({access})"),
        DescriptorFieldKind::Struct if field.nested_owns_storage => {
            format!("{}.value", naming::rust_ident(&field.name))
        }
        DescriptorFieldKind::Struct => format!("convert_{}({access})", named(field)?),
        DescriptorFieldKind::Object => format!("{access}.cast()"),
        DescriptorFieldKind::ObjectArray => format!("{access}.cast()"),
        DescriptorFieldKind::StructPointer | DescriptorFieldKind::StructArray => {
            return Err(crate::internal(
                "patterns::descriptor::conversion_value",
                "storage fields are converted separately",
            ))
        }
        _ => access,
    })
}

/// Renders the public-to-backend conversion function for one struct.
///
/// A shape that owns storage returns a holder struct that keeps every converted array, box, and
/// nested value alive beside the backend value. The caller passes `&converted.value` to the
/// backend and drops the holder after the call. A zero-rule field also gains its own
/// substitution function and a `#[doc(hidden)]` probe the suite reads.
pub(crate) fn rust_conversion(shape: &StructPlan) -> Result<String, crate::policy::PolicyError> {
    let mut out = String::new();
    // A zero-rule field needs a substitution function of its own, plus the probe that lets a
    // suite program read the substituted value without a backend (F15).
    for field in &shape.fields {
        if let Some(constant) = &field.zero_constant {
            let converted_access = if shape.owns_storage {
                format!("converted.value.{}", naming::rust_ident(&field.name))
            } else {
                format!("converted.{}", naming::rust_ident(&field.name))
            };
            out.push_str(&format!(
                "fn convert_{source}_{field}_zero_rule(value: u64) -> u64 {{\n    if value == 0 {{ {constant} }} else {{ value }}\n}}\n\n#[doc(hidden)]\npub fn subscript_typegpu_internal_{source}_{field}_for_test(value: u64) -> u64 {{\n    // SAFETY: generated SubscriptTypegpu descriptor fields all admit an all-zero value.\n    let mut source: {subscript_typegpu} = unsafe {{ std::mem::zeroed() }};\n    source.{rust_field} = value;\n    let converted = convert_{source}(source);\n    {converted_access}\n}}\n\n",
                source = shape.source,
                field = field.name,
                subscript_typegpu = shape.subscript_typegpu_struct,
                rust_field = naming::rust_ident(&field.name),
            ));
        }
    }
    // The holder struct keeps every box, vector, and nested value alive beside the backend
    // value. Without it, the pointers inside the backend value dangle at the call site.
    if shape.owns_storage {
        out.push_str(&format!(
            "struct Converted{} {{\n    value: {},\n",
            naming::pascal(&shape.source),
            shape.wgpu_struct,
        ));
        for field in &shape.fields {
            match field.kind {
                DescriptorFieldKind::Struct if field.nested_owns_storage => {
                    out.push_str(&format!(
                        "    _{}: Converted{},\n",
                        field.name,
                        naming::pascal(named(field)?),
                    ));
                }
                DescriptorFieldKind::StructPointer => {
                    let ty = if field.nested_owns_storage {
                        format!("Converted{}", naming::pascal(named(field)?))
                    } else {
                        naming::wgpu_type(named(field)?)
                    };
                    out.push_str(&format!("    _{}: Option<Box<{ty}>>,\n", field.name));
                }
                DescriptorFieldKind::StructArray => {
                    if field.nested_owns_storage {
                        out.push_str(&format!(
                            "    _{}_converted: Vec<Box<Converted{}>>,\n",
                            field.name,
                            naming::pascal(named(field)?),
                        ));
                    }
                    out.push_str(&format!(
                        "    _{}: Vec<{}>,\n",
                        field.name,
                        naming::wgpu_type(named(field)?),
                    ));
                }
                _ => {}
            }
        }
        out.push_str("}\n\n");
    }
    let return_type = if shape.owns_storage {
        format!("Converted{}", naming::pascal(&shape.source))
    } else {
        shape.wgpu_struct.clone()
    };
    out.push_str(&format!(
        "#[allow(dead_code)]\nfn convert_{}(source: {}) -> {} {{\n",
        shape.source, shape.subscript_typegpu_struct, return_type,
    ));
    // Storage fields become locals before the struct literal, so each pointer the literal
    // stores outlives the literal itself.
    for field in &shape.fields {
        let name = naming::rust_ident(&field.name);
        match field.kind {
            DescriptorFieldKind::Struct if field.nested_owns_storage => {
                out.push_str(&format!(
                    "    let {name} = convert_{element}(source.{name});\n",
                    element = named(field)?,
                ));
            }
            DescriptorFieldKind::StructPointer => {
                out.push_str(&format!(
                    "    let {name} = if source.{name}.is_null() {{\n        None\n    }} else {{\n        // SAFETY: a non-null struct pointer is readable for this call.\n        Some(Box::new(convert_{element}(unsafe {{ *source.{name} }})))\n    }};\n",
                    element = named(field)?,
                ));
                let pointer = if field.nested_owns_storage {
                    "&value.value as *const _"
                } else {
                    "value.as_ref() as *const _"
                };
                out.push_str(&format!(
                    "    let {name}_ptr = {name}.as_ref().map_or(std::ptr::null(), |value| {pointer});\n"
                ));
            }
            DescriptorFieldKind::StructArray => {
                if field.nested_owns_storage {
                    out.push_str(&format!(
                        "    let {name}_converted: Vec<Box<Converted{pascal}>> = if source.{name}.is_null() {{\n        Vec::new()\n    }} else {{\n        // SAFETY: the boundary pair promises `count` readable elements.\n        unsafe {{ std::slice::from_raw_parts(source.{name}, source.{name}_count) }}\n            .iter()\n            .copied()\n            .map(|item| Box::new(convert_{element}(item)))\n            .collect()\n    }};\n    let {name}: Vec<{wgpu}> = {name}_converted.iter().map(|item| item.value).collect();\n",
                        pascal = naming::pascal(named(field)?),
                        element = named(field)?,
                        wgpu = naming::wgpu_type(named(field)?),
                    ));
                } else {
                    out.push_str(&format!(
                        "    let {name}: Vec<{wgpu}> = if source.{name}.is_null() {{\n        Vec::new()\n    }} else {{\n        // SAFETY: the boundary pair promises `count` readable elements.\n        unsafe {{ std::slice::from_raw_parts(source.{name}, source.{name}_count) }}\n            .iter()\n            .copied()\n            .map(convert_{element})\n            .collect()\n    }};\n",
                        wgpu = naming::wgpu_type(named(field)?),
                        element = named(field)?,
                    ));
                }
                out.push_str(&format!(
                    "    let {name}_ptr = if source.{name}.is_null() {{\n        std::ptr::null()\n    }} else {{\n        {name}.as_ptr()\n    }};\n"
                ));
            }
            _ => {}
        }
    }
    // The literal keeps the yml field order: the chain head first, then each array count
    // directly before its pointer (F12, B1).
    let mut fields = String::new();
    if shape.extensible {
        fields.push_str("        next_in_chain: std::ptr::null_mut(),\n");
    }
    for field in &shape.fields {
        if is_array(field) {
            fields.push_str(&format!(
                "        {}: source.{}_count,\n",
                field
                    .backend_count_name
                    .as_deref()
                    .ok_or_else(|| crate::internal(
                        "patterns::descriptor::rust_conversion",
                        "missing backend array count name"
                    ))?,
                naming::rust_ident(&field.name),
            ));
        }
        let value = if matches!(
            field.kind,
            DescriptorFieldKind::StructArray | DescriptorFieldKind::StructPointer
        ) {
            format!("{}_ptr", naming::rust_ident(&field.name))
        } else {
            conversion_value(field, "source", &shape.source)?
        };
        fields.push_str(&format!(
            "        {}: {},\n",
            naming::rust_ident(&field.name),
            value,
        ));
    }
    if shape.owns_storage {
        out.push_str(&format!(
            "    let value = {} {{\n{fields}    }};\n    Converted{} {{\n        value,\n",
            shape.wgpu_struct,
            naming::pascal(&shape.source),
        ));
        for field in &shape.fields {
            match field.kind {
                DescriptorFieldKind::Struct if field.nested_owns_storage => {
                    out.push_str(&format!("        _{0}: {0},\n", field.name));
                }
                DescriptorFieldKind::StructPointer => {
                    out.push_str(&format!("        _{0}: {0},\n", field.name));
                }
                DescriptorFieldKind::StructArray => {
                    if field.nested_owns_storage {
                        out.push_str(&format!(
                            "        _{0}_converted: {0}_converted,\n",
                            field.name
                        ));
                    }
                    out.push_str(&format!("        _{0}: {0},\n", field.name));
                }
                _ => {}
            }
        }
        out.push_str("    }\n}\n");
    } else {
        out.push_str(&format!(
            "    {} {{\n{fields}    }}\n}}\n",
            shape.wgpu_struct,
        ));
    }
    Ok(out)
}

/// Renders the private webgpu.h declaration of the descriptor-taking create.
pub(crate) fn rust_extern(op: &DescriptorOp, shape: &StructPlan) -> String {
    format!(
        "    fn {}({}: {}, descriptor: *const {}) -> {};\n",
        op.wgpu_fn,
        naming::camel(&op.receiver),
        naming::wgpu_type(&op.receiver),
        shape.wgpu_struct,
        naming::wgpu_type(&op.returns_object),
    )
}

/// Renders the `unsafe` backend create call.
///
/// `descriptor` is the argument text, which is a reference to converted storage or a null
/// pointer.
fn call(op: &DescriptorOp, recv: &str, descriptor: &str) -> String {
    format!(
        "unsafe {{ {}({recv}.cast(), {descriptor}).cast() }}",
        op.wgpu_fn,
    )
}

/// Renders the exported descriptor-taking create body.
///
/// A null receiver returns a null handle (L9). A null descriptor returns a null handle, unless
/// the yml marks the descriptor optional, and then the body calls the backend with NULL. The
/// created handle inherits the receiver's owning instance (L11).
pub(crate) fn rust_export(op: &DescriptorOp, shape: &StructPlan) -> String {
    let recv = naming::camel(&op.receiver);
    let sig = rust_signature(
        &format!("pub extern \"C\" fn {}", op.subscript_typegpu_fn),
        &[
            format!("{recv}: {}", naming::subscript_typegpu_type(&op.receiver)),
            format!("descriptor: *const {}", shape.subscript_typegpu_struct),
        ],
        &format!(
            " -> {} {{",
            naming::subscript_typegpu_type(&op.returns_object)
        ),
    );
    let null_descriptor = if op.optional {
        format!(
            "    if descriptor.is_null() {{\n        // SAFETY: webgpu.yml marks this descriptor optional.\n        let created = {};\n        runtime::inherit_handle_instance({recv} as usize, created as usize);\n        return created;\n    }}\n",
            call(op, &recv, "std::ptr::null()"),
        )
    } else {
        "    if descriptor.is_null() {\n        return std::ptr::null_mut();\n    }\n".into()
    };
    let descriptor_ref = if shape.owns_storage {
        "&descriptor.value"
    } else {
        "&descriptor"
    };
    format!(
        "/// `subscript-typegpu.h`: creates an object from a chain-free descriptor.\n\
         #[no_mangle]\n\
         {sig}\n\
         \x20   if {recv}.is_null() {{\n\
         \x20       return std::ptr::null_mut();\n\
         \x20   }}\n\
         {null_descriptor}\
         \x20   // SAFETY: the caller supplies a live descriptor for this call.\n\
         \x20   let source = unsafe {{ *descriptor }};\n\
         \x20   let descriptor = convert_{source}(source);\n\
         \x20   // SAFETY: the receiver is non-null and the converted descriptor\n\
         \x20   // outlives the backend call.\n\
         \x20   let created = {call};\n\
         \x20   runtime::inherit_handle_instance({recv} as usize, created as usize);\n\
         \x20   created\n\
         }}\n",
        source = shape.source,
        call = call(op, &recv, descriptor_ref),
    )
}
