//! F12 chain-free struct and descriptor pattern.

use crate::naming;
use crate::patterns::rust_signature;
use crate::plan::{DescriptorField, DescriptorFieldKind, DescriptorOp, SentinelConst, StructPlan};

fn named(field: &DescriptorField) -> &str {
    field
        .named_type
        .as_deref()
        .expect("named descriptor field has a type")
}

fn c_field_type(field: &DescriptorField) -> String {
    match field.kind {
        DescriptorFieldKind::StringView => "SubscriptTypegpuStringView".into(),
        DescriptorFieldKind::Bitflag
        | DescriptorFieldKind::Enum
        | DescriptorFieldKind::Struct
        | DescriptorFieldKind::Object => naming::subscript_typegpu_type(named(field)),
        DescriptorFieldKind::U32 => "uint32_t".into(),
        DescriptorFieldKind::U64 => "uint64_t".into(),
        DescriptorFieldKind::Usize => "size_t".into(),
        DescriptorFieldKind::U16 => "uint16_t".into(),
        DescriptorFieldKind::F32 => "float".into(),
        DescriptorFieldKind::F64 => "double".into(),
        DescriptorFieldKind::I32 => "int32_t".into(),
        DescriptorFieldKind::Bool => "bool".into(),
        DescriptorFieldKind::StructPointer => unreachable!("struct pointers are emitted specially"),
        DescriptorFieldKind::EnumArray
        | DescriptorFieldKind::StructArray
        | DescriptorFieldKind::ObjectArray => unreachable!("arrays expand to two fields"),
    }
}

fn is_array(field: &DescriptorField) -> bool {
    matches!(
        field.kind,
        DescriptorFieldKind::EnumArray
            | DescriptorFieldKind::StructArray
            | DescriptorFieldKind::ObjectArray
    )
}

pub(crate) fn c_struct(shape: &StructPlan) -> String {
    let mut out = format!("typedef struct {} {{\n", shape.subscript_typegpu_struct);
    for field in &shape.fields {
        let name = naming::camel(&field.name);
        if is_array(field) {
            let count = field
                .public_count_name
                .as_deref()
                .expect("array has a public count name");
            out.push_str(&format!("    size_t {count};\n"));
            out.push_str(&format!(
                "    const {}* {name};\n",
                naming::subscript_typegpu_type(named(field)),
            ));
        } else if field.kind == DescriptorFieldKind::StructPointer {
            out.push_str(&format!(
                "    const {}* {name};\n",
                naming::subscript_typegpu_type(named(field))
            ));
        } else if field.nullable {
            out.push_str(&format!("    {} _Nullable {name};\n", c_field_type(field)));
        } else {
            out.push_str(&format!("    {} {name};\n", c_field_type(field)));
        }
    }
    out.push_str(&format!("}} {};", shape.subscript_typegpu_struct));
    out
}

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

fn rust_field_type(field: &DescriptorField, backend: bool) -> String {
    match field.kind {
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
                naming::wgpu_type(named(field))
            } else {
                naming::subscript_typegpu_type(named(field))
            }
        }
        DescriptorFieldKind::StructPointer => {
            let ty = if backend {
                naming::wgpu_type(named(field))
            } else {
                naming::subscript_typegpu_type(named(field))
            };
            format!("*const {ty}")
        }
        DescriptorFieldKind::Object => {
            if backend {
                naming::wgpu_type(named(field))
            } else {
                naming::subscript_typegpu_type(named(field))
            }
        }
        DescriptorFieldKind::EnumArray => "*const i32".into(),
        DescriptorFieldKind::StructArray | DescriptorFieldKind::ObjectArray => {
            let ty = if backend {
                naming::wgpu_type(named(field))
            } else {
                naming::subscript_typegpu_type(named(field))
            };
            format!("*const {ty}")
        }
    }
}

pub(crate) fn rust_sentinel_const(sentinel: &SentinelConst) -> String {
    format!(
        "/// webgpu.yml `{}`; kept internal because it exceeds the exact script integer range.\n\
         const {}: u64 = {};\n",
        sentinel.source, sentinel.rust_name, sentinel.rust_value,
    )
}

pub(crate) fn rust_structs(shape: &StructPlan) -> String {
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
                    .expect("array has a backend count name"),
            ));
        }
        out.push_str(&format!(
            "    {}: {},\n",
            naming::rust_ident(&field.name),
            rust_field_type(field, true),
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
            rust_field_type(field, false),
        ));
    }
    out.push_str("}\n");
    out
}

fn conversion_value(field: &DescriptorField, source: &str, shape_source: &str) -> String {
    let access = format!("{source}.{}", naming::rust_ident(&field.name));
    if field.zero_constant.is_some() {
        return format!("convert_{shape_source}_{}_zero_rule({access})", field.name);
    }
    match field.kind {
        DescriptorFieldKind::StringView => format!("wgpu_string_view({access})"),
        DescriptorFieldKind::Bool => format!("u32::from({access})"),
        DescriptorFieldKind::Struct if field.nested_owns_storage => {
            format!("{}.value", naming::rust_ident(&field.name))
        }
        DescriptorFieldKind::Struct => format!("convert_{}({access})", named(field)),
        DescriptorFieldKind::Object => format!("{access}.cast()"),
        DescriptorFieldKind::ObjectArray => format!("{access}.cast()"),
        DescriptorFieldKind::StructPointer | DescriptorFieldKind::StructArray => {
            unreachable!("storage fields are converted separately")
        }
        _ => access,
    }
}

pub(crate) fn rust_conversion(shape: &StructPlan) -> String {
    let mut out = String::new();
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
                        naming::pascal(named(field)),
                    ));
                }
                DescriptorFieldKind::StructPointer => {
                    let ty = if field.nested_owns_storage {
                        format!("Converted{}", naming::pascal(named(field)))
                    } else {
                        naming::wgpu_type(named(field))
                    };
                    out.push_str(&format!("    _{}: Option<Box<{ty}>>,\n", field.name));
                }
                DescriptorFieldKind::StructArray => {
                    if field.nested_owns_storage {
                        out.push_str(&format!(
                            "    _{}_converted: Vec<Box<Converted{}>>,\n",
                            field.name,
                            naming::pascal(named(field)),
                        ));
                    }
                    out.push_str(&format!(
                        "    _{}: Vec<{}>,\n",
                        field.name,
                        naming::wgpu_type(named(field)),
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
    for field in &shape.fields {
        let name = naming::rust_ident(&field.name);
        match field.kind {
            DescriptorFieldKind::Struct if field.nested_owns_storage => {
                out.push_str(&format!(
                    "    let {name} = convert_{element}(source.{name});\n",
                    element = named(field),
                ));
            }
            DescriptorFieldKind::StructPointer => {
                out.push_str(&format!(
                    "    let {name} = if source.{name}.is_null() {{\n        None\n    }} else {{\n        // SAFETY: a non-null struct pointer is readable for this call.\n        Some(Box::new(convert_{element}(unsafe {{ *source.{name} }})))\n    }};\n",
                    element = named(field),
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
                        pascal = naming::pascal(named(field)),
                        element = named(field),
                        wgpu = naming::wgpu_type(named(field)),
                    ));
                } else {
                    out.push_str(&format!(
                        "    let {name}: Vec<{wgpu}> = if source.{name}.is_null() {{\n        Vec::new()\n    }} else {{\n        // SAFETY: the boundary pair promises `count` readable elements.\n        unsafe {{ std::slice::from_raw_parts(source.{name}, source.{name}_count) }}\n            .iter()\n            .copied()\n            .map(convert_{element})\n            .collect()\n    }};\n",
                        wgpu = naming::wgpu_type(named(field)),
                        element = named(field),
                    ));
                }
                out.push_str(&format!(
                    "    let {name}_ptr = if source.{name}.is_null() {{\n        std::ptr::null()\n    }} else {{\n        {name}.as_ptr()\n    }};\n"
                ));
            }
            _ => {}
        }
    }
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
                    .expect("array has a backend count name"),
                naming::rust_ident(&field.name),
            ));
        }
        let value = if matches!(
            field.kind,
            DescriptorFieldKind::StructArray | DescriptorFieldKind::StructPointer
        ) {
            format!("{}_ptr", naming::rust_ident(&field.name))
        } else {
            conversion_value(field, "source", &shape.source)
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
    out
}

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

fn call(op: &DescriptorOp, recv: &str, descriptor: &str) -> String {
    format!(
        "unsafe {{ {}({recv}.cast(), {descriptor}).cast() }}",
        op.wgpu_fn,
    )
}

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
