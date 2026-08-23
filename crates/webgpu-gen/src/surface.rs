//! F23 Rust-only surface slice.

use std::collections::BTreeSet;

use crate::model::{Arg, Function, Member, Struct, Yml};
use crate::policy::{Policy, PolicyError};

const REQUIRED: [&str; 22] = [
    "wgpuInstanceCreateSurface",
    "wgpuSurfaceConfigure",
    "wgpuSurfaceUnconfigure",
    "wgpuSurfaceGetCapabilities",
    "wgpuSurfaceCapabilitiesFreeMembers",
    "wgpuSurfaceGetCurrentTexture",
    "wgpuSurfacePresent",
    "wgpuSurfaceAddRef",
    "wgpuSurfaceRelease",
    "wgpuSurfaceSetLabel",
    "WGPUSurfaceDescriptor",
    "WGPUSurfaceSourceAndroidNativeWindow",
    "WGPUSurfaceSourceMetalLayer",
    "WGPUSurfaceSourceWaylandSurface",
    "WGPUSurfaceSourceWindowsHWND",
    "WGPUSurfaceSourceXCBWindow",
    "WGPUSurfaceSourceXlibWindow",
    "WGPUSurfaceConfiguration",
    "WGPUSurfaceCapabilities",
    "WGPUSurfaceTexture",
    "WGPUStatus",
    "WGPUSurfaceGetCurrentTextureStatus",
];

fn known_constructs(yml: &Yml) -> BTreeSet<String> {
    let mut known = BTreeSet::new();
    for function in &yml.functions {
        known.insert(format!("wgpu{}", crate::naming::pascal(&function.name)));
    }
    for object in &yml.objects {
        known.insert(crate::naming::wgpu_type(&object.name));
        known.insert(crate::naming::wgpu_method(&object.name, "add_ref"));
        known.insert(crate::naming::wgpu_method(&object.name, "release"));
        for method in &object.methods {
            known.insert(crate::naming::wgpu_method(&object.name, &method.name));
        }
    }
    for shape in &yml.structs {
        known.insert(crate::naming::wgpu_type(&shape.name));
        if shape.free_members {
            known.insert(format!(
                "wgpu{}FreeMembers",
                crate::naming::pascal(&shape.name)
            ));
        }
    }
    known.extend(
        yml.enums
            .iter()
            .map(|value| crate::naming::wgpu_type(&value.name)),
    );
    known.extend(
        yml.bitflags
            .iter()
            .map(|value| crate::naming::wgpu_type(&value.name)),
    );
    known
}

fn excluded_constructs(policy: &Policy) -> BTreeSet<String> {
    let mut excluded = BTreeSet::new();
    for row in &policy.exclude {
        if row.construct == "addref" {
            for object in &policy.slice.objects {
                excluded.insert(crate::naming::wgpu_method(object, "add_ref"));
            }
        } else if let Some((object, method)) = row.construct.split_once('.') {
            excluded.insert(crate::naming::wgpu_method(object, method));
        } else {
            excluded.insert(format!("wgpu{}", crate::naming::pascal(&row.construct)));
        }
    }
    excluded
}

fn invalid(entry: &str, message: impl Into<String>) -> PolicyError {
    PolicyError::Invalid {
        entry: entry.to_owned(),
        message: message.into(),
    }
}

fn rust_type(yml: &Yml, source: &str) -> Result<String, PolicyError> {
    let scalar = match source {
        "uint16" => Some("u16"),
        "uint32" => Some("u32"),
        "uint64" => Some("u64"),
        "int16" => Some("i16"),
        "int32" => Some("i32"),
        "int64" => Some("i64"),
        "size_t" => Some("usize"),
        "float32" => Some("f32"),
        "float64" => Some("f64"),
        "bool" => Some("bool"),
        "c_void" => Some("c_void"),
        "string_with_default_empty" | "string_view" => Some("WGPUStringView"),
        _ => None,
    };
    if let Some(scalar) = scalar {
        return Ok(scalar.to_owned());
    }
    for prefix in ["enum.", "bitflag.", "object.", "struct."] {
        if let Some(name) = source.strip_prefix(prefix) {
            let exists = match prefix {
                "enum." => yml.enum_(name).is_some(),
                "bitflag." => yml.bitflag(name).is_some(),
                "object." => yml.object(name).is_some(),
                "struct." => yml.struct_(name).is_some(),
                _ => false,
            };
            if !exists {
                return Err(PolicyError::Unknown {
                    entry: source.to_owned(),
                });
            }
            return Ok(crate::naming::wgpu_type(name));
        }
    }
    Err(invalid(source, "unsupported host-only ABI type"))
}

fn pointed_type(yml: &Yml, source: &str, pointer: Option<&str>) -> Result<String, PolicyError> {
    let base = rust_type(yml, source)?;
    match pointer {
        None => Ok(base),
        Some("immutable") => Ok(format!("*const {base}")),
        Some("mutable") => Ok(format!("*mut {base}")),
        Some(other) => Err(invalid(
            source,
            format!("unsupported pointer kind `{other}`"),
        )),
    }
}

fn arg_type(yml: &Yml, arg: &Arg) -> Result<String, PolicyError> {
    pointed_type(yml, &arg.ty, arg.pointer.as_deref())
}

fn backend_array_count(member: &str) -> String {
    let singular = member
        .strip_suffix("ies")
        .map(|stem| format!("{stem}y"))
        .or_else(|| member.strip_suffix('s').map(str::to_owned))
        .unwrap_or_else(|| member.to_owned());
    format!("{singular}_count")
}

fn render_member(out: &mut String, yml: &Yml, member: &Member) -> Result<(), PolicyError> {
    if let Some(element) = member
        .ty
        .strip_prefix("array<")
        .and_then(|value| value.strip_suffix('>'))
    {
        let count = crate::naming::camel(&backend_array_count(&member.name));
        let element = rust_type(yml, element)?;
        let pointer = match member.pointer.as_deref() {
            Some("immutable") => "*const",
            Some("mutable") => "*mut",
            other => {
                return Err(invalid(
                    &member.name,
                    format!("array requires a pointer kind, found {other:?}"),
                ));
            }
        };
        out.push_str(&format!("    pub {count}: usize,\n"));
        out.push_str(&format!(
            "    pub {}: {pointer} {element},\n",
            crate::naming::camel(&member.name)
        ));
        return Ok(());
    }
    out.push_str(&format!(
        "    pub {}: {},\n",
        crate::naming::camel(&member.name),
        pointed_type(yml, &member.ty, member.pointer.as_deref())?
    ));
    Ok(())
}

fn render_struct(out: &mut String, yml: &Yml, shape: &Struct) -> Result<(), PolicyError> {
    out.push_str("#[repr(C)]\n#[derive(Clone, Copy)]\n");
    out.push_str(&format!(
        "pub struct {} {{\n",
        crate::naming::wgpu_type(&shape.name)
    ));
    match shape.kind.as_str() {
        "extensible" => out.push_str("    pub nextInChain: *mut WGPUChainedStruct,\n"),
        "extension" => out.push_str("    pub chain: WGPUChainedStruct,\n"),
        "standalone" => {}
        other => {
            return Err(invalid(
                &shape.name,
                format!("unsupported struct kind `{other}`"),
            ));
        }
    }
    for member in &shape.members {
        render_member(out, yml, member)?;
    }
    out.push_str("}\n\n");
    Ok(())
}

fn collect_type(
    source: &str,
    objects: &mut BTreeSet<String>,
    enums: &mut BTreeSet<String>,
    flags: &mut BTreeSet<String>,
) {
    let source = source
        .strip_prefix("array<")
        .and_then(|value| value.strip_suffix('>'))
        .unwrap_or(source);
    if let Some(name) = source.strip_prefix("object.") {
        objects.insert(name.to_owned());
    } else if let Some(name) = source.strip_prefix("enum.") {
        enums.insert(name.to_owned());
    } else if let Some(name) = source.strip_prefix("bitflag.") {
        flags.insert(name.to_owned());
    }
}

struct HostFunction<'a> {
    name: String,
    receiver: Option<&'a str>,
    function: Option<&'a Function>,
    free_members: Option<&'a Struct>,
}

fn host_function<'a>(yml: &'a Yml, name: &str) -> Result<HostFunction<'a>, PolicyError> {
    for function in &yml.functions {
        if format!("wgpu{}", crate::naming::pascal(&function.name)) == name {
            return Ok(HostFunction {
                name: name.to_owned(),
                receiver: None,
                function: Some(function),
                free_members: None,
            });
        }
    }
    for object in &yml.objects {
        if crate::naming::wgpu_method(&object.name, "add_ref") == name
            || crate::naming::wgpu_method(&object.name, "release") == name
        {
            return Ok(HostFunction {
                name: name.to_owned(),
                receiver: Some(&object.name),
                function: None,
                free_members: None,
            });
        }
        if let Some(function) = object
            .methods
            .iter()
            .find(|method| crate::naming::wgpu_method(&object.name, &method.name) == name)
        {
            return Ok(HostFunction {
                name: name.to_owned(),
                receiver: Some(&object.name),
                function: Some(function),
                free_members: None,
            });
        }
    }
    for shape in &yml.structs {
        if shape.free_members
            && format!("wgpu{}FreeMembers", crate::naming::pascal(&shape.name)) == name
        {
            return Ok(HostFunction {
                name: name.to_owned(),
                receiver: None,
                function: None,
                free_members: Some(shape),
            });
        }
    }
    Err(PolicyError::Unknown {
        entry: name.to_owned(),
    })
}

fn function_types(
    yml: &Yml,
    function: &HostFunction<'_>,
) -> Result<(Vec<String>, Option<String>), PolicyError> {
    let mut params = Vec::new();
    if let Some(receiver) = function.receiver {
        params.push(rust_type(yml, &format!("object.{receiver}"))?);
    }
    if let Some(shape) = function.free_members {
        params.push(rust_type(yml, &format!("struct.{}", shape.name))?);
    }
    if let Some(source) = function.function {
        for arg in &source.args {
            params.push(arg_type(yml, arg)?);
        }
        let result = source
            .returns
            .as_ref()
            .map(|returns| rust_type(yml, &returns.ty))
            .transpose()?;
        return Ok((params, result));
    }
    Ok((params, None))
}

fn validate_policy(yml: &Yml, policy: &Policy) -> Result<(), PolicyError> {
    let known = known_constructs(yml);
    let excluded = excluded_constructs(policy);
    let required = REQUIRED.into_iter().collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    for row in &policy.host_only {
        if !seen.insert(row.construct.as_str()) {
            return Err(PolicyError::Duplicate {
                entry: row.construct.clone(),
            });
        }
        if !known.contains(&row.construct) {
            return Err(PolicyError::Unknown {
                entry: row.construct.clone(),
            });
        }
        if row.reason.trim().is_empty() {
            return Err(invalid(
                &row.construct,
                "host-only construct requires a reason",
            ));
        }
        if excluded.contains(&row.construct) {
            return Err(invalid(
                &row.construct,
                "construct is both host_only and exclude",
            ));
        }
        if !required.contains(row.construct.as_str()) {
            return Err(PolicyError::Dead {
                entry: row.construct.clone(),
            });
        }
    }
    if let Some(missing) = required.difference(&seen).next() {
        return Err(PolicyError::Unpoliced {
            construct: (*missing).to_owned(),
        });
    }
    Ok(())
}

pub(crate) fn render(yml: &Yml, policy: &Policy) -> Result<String, PolicyError> {
    if yml.object("surface").is_none() && policy.host_only.is_empty() {
        return Ok("//! No host-only surface slice in this fixture.\n".to_owned());
    }
    validate_policy(yml, policy)?;

    let selected_structs = policy
        .host_only
        .iter()
        .filter_map(|row| {
            yml.structs
                .iter()
                .find(|shape| crate::naming::wgpu_type(&shape.name) == row.construct)
        })
        .collect::<Vec<_>>();
    let functions = policy
        .host_only
        .iter()
        .filter(|row| row.construct.starts_with("wgpu"))
        .map(|row| host_function(yml, &row.construct))
        .collect::<Result<Vec<_>, _>>()?;

    let mut objects = BTreeSet::new();
    let mut enums = BTreeSet::from(["s_type".to_owned()]);
    let mut flags = BTreeSet::new();
    let mut needs_string_view = false;
    for shape in &selected_structs {
        for member in &shape.members {
            collect_type(&member.ty, &mut objects, &mut enums, &mut flags);
            needs_string_view |= matches!(
                member.ty.as_str(),
                "string_with_default_empty" | "string_view"
            );
        }
    }
    for function in &functions {
        if let Some(receiver) = function.receiver {
            objects.insert(receiver.to_owned());
        }
        if let Some(source) = function.function {
            if let Some(returns) = &source.returns {
                collect_type(&returns.ty, &mut objects, &mut enums, &mut flags);
                needs_string_view |= matches!(
                    returns.ty.as_str(),
                    "string_with_default_empty" | "string_view"
                );
            }
            for arg in &source.args {
                collect_type(&arg.ty, &mut objects, &mut enums, &mut flags);
                needs_string_view |=
                    matches!(arg.ty.as_str(), "string_with_default_empty" | "string_view");
            }
        }
    }

    let mut out = String::from(
        "//! Generated from webgpu.yml plus policy.toml. Do not edit.\n#![allow(missing_docs, non_snake_case, non_upper_case_globals)]\n\nuse std::ffi::{c_char, c_void};\nuse std::sync::OnceLock;\n\n",
    );
    for object in &objects {
        let name = crate::naming::wgpu_type(object);
        if object == "surface" {
            out.push_str(&format!("pub type {name} = *mut c_void;\n"));
        } else {
            out.push_str(&format!(
                "pub type {name} = crate::{};\n",
                crate::naming::subscript_typegpu_type(object)
            ));
        }
    }
    out.push('\n');

    for name in &enums {
        let value = yml.enum_(name).ok_or_else(|| PolicyError::Unknown {
            entry: format!("enum.{name}"),
        })?;
        let ty = crate::naming::wgpu_type(name);
        out.push_str(&format!("pub type {ty} = u32;\n"));
        for (index, entry) in value.entries.iter().enumerate() {
            let Some(entry) = entry else { continue };
            out.push_str(&format!(
                "pub const {}: {ty} = {index};\n",
                crate::naming::wgpu_enum_member(name, &entry.name)
            ));
        }
        out.push('\n');
    }
    for name in &flags {
        let value = yml.bitflag(name).ok_or_else(|| PolicyError::Unknown {
            entry: format!("bitflag.{name}"),
        })?;
        let ty = crate::naming::wgpu_type(name);
        out.push_str(&format!("pub type {ty} = u64;\n"));
        for entry in &value.entries {
            let number = value
                .value_of(&entry.name)
                .ok_or_else(|| PolicyError::Unknown {
                    entry: format!("bitflag.{name}.{}", entry.name),
                })?;
            out.push_str(&format!(
                "pub const {}: {ty} = {number};\n",
                crate::naming::wgpu_enum_member(name, &entry.name)
            ));
        }
        out.push('\n');
    }

    out.push_str("#[repr(C)]\n#[derive(Clone, Copy)]\npub struct WGPUChainedStruct {\n    pub next: *mut WGPUChainedStruct,\n    pub sType: WGPUSType,\n}\n\n");
    if needs_string_view {
        out.push_str("#[repr(C)]\n#[derive(Clone, Copy)]\npub struct WGPUStringView {\n    pub data: *const c_char,\n    pub length: usize,\n}\n\n");
    }
    for shape in &selected_structs {
        render_struct(&mut out, yml, shape)?;
    }

    for function in &functions {
        let (params, result) = function_types(yml, function)?;
        let proc_name = format!(
            "WGPUProc{}",
            function.name.strip_prefix("wgpu").unwrap_or(&function.name)
        );
        out.push_str(&format!(
            "pub type {proc_name} = unsafe extern \"C\" fn({}){};\n",
            params.join(", "),
            result.map_or_else(String::new, |result| format!(" -> {result}"))
        ));
    }
    out.push_str("\npub struct SurfaceTable {\n");
    for function in &functions {
        let proc_name = format!(
            "WGPUProc{}",
            function.name.strip_prefix("wgpu").unwrap_or(&function.name)
        );
        out.push_str(&format!("    pub {}: {proc_name},\n", function.name));
    }
    out.push_str("}\n\nstatic SURFACE_TABLE: OnceLock<SurfaceTable> = OnceLock::new();\n\npub fn table() -> Result<&'static SurfaceTable, String> {\n    if let Some(table) = SURFACE_TABLE.get() {\n        return Ok(table);\n    }\n    let loaded = SurfaceTable {\n");
    for function in &functions {
        out.push_str(&format!(
            "        {}: {{\n            // SAFETY: the type comes from this symbol's pinned webgpu.yml declaration.\n            unsafe {{ crate::runtime::surface_symbol(b\"{}\\0\") }}?\n        }},\n",
            function.name, function.name
        ));
    }
    out.push_str("    };\n    let _ = SURFACE_TABLE.set(loaded);\n    SURFACE_TABLE\n        .get()\n        .ok_or_else(|| \"surface function table initialization failed\".to_owned())\n}\n");
    Ok(out)
}
