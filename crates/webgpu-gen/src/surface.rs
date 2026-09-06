//! F23 Rust-only surface slice.

use std::collections::BTreeSet;

use crate::model::{Arg, Function, Member, Struct, Yml};
use crate::policy::{Policy, PolicyError};

/// The exact host-only construct set: the ten surface functions and the twelve types they name.
///
/// The two-way check rejects a `[[host_only]]` row outside this set and a member of this set
/// that no row names (L14).
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

/// Every webgpu.h name the pinned yml defines.
///
/// The set holds the freestanding functions, each object with its methods and its two lifetime
/// calls, each struct with its free-members helper, the enums, and the flag types.
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

/// The webgpu.h names that `[[exclude]]` rows remove.
///
/// The `addref` row covers every subset object at once, so it expands to one name per object.
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

/// Builds a `PolicyError::Invalid` for one construct.
fn invalid(entry: &str, message: impl Into<String>) -> PolicyError {
    PolicyError::Invalid {
        entry: entry.to_owned(),
        message: message.into(),
    }
}

/// The Rust type of one yml type name, for the surface module.
///
/// A scalar name maps directly. A `enum.`, `bitflag.`, `object.`, or `struct.` name resolves
/// against the yml and returns `PolicyError::Unknown` when the yml lacks it. Any other spelling
/// returns an invalid-row error, because the surface module supports no wider ABI.
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

/// The Rust type of one member or argument, with the yml pointer attribute applied.
///
/// A pointer kind other than `immutable` or `mutable` returns an invalid-row error.
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

/// The count-field name that webgpu.h gives an array member.
///
/// The name is the singular member name plus `_count`. A member ending in `ies` becomes `y`,
/// and a member ending in `s` drops it.
fn backend_array_count(member: &str) -> String {
    let singular = member
        .strip_suffix("ies")
        .map(|stem| format!("{stem}y"))
        .or_else(|| member.strip_suffix('s').map(str::to_owned))
        .unwrap_or_else(|| member.to_owned());
    format!("{singular}_count")
}

/// Appends one struct member to the emitted text.
///
/// An `array<T>` member expands into a `usize` count and an element pointer, count first. An
/// array member without a pointer kind returns an invalid-row error.
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
        out.push_str(&format!(
            "    /// The number of elements at `{}`.\n    pub {count}: usize,\n",
            crate::naming::camel(&member.name)
        ));
        out.push_str(&format!(
            "    /// The element array with length `{count}`.\n    pub {}: {pointer} {element},\n",
            crate::naming::camel(&member.name)
        ));
        return Ok(());
    }
    let fallback = format!("The native `{}` value for this structure.", member.name);
    let description = match member.name.as_str() {
        "label" => "The diagnostic label for the surface.",
        "window" => "The native window that receives the surface output.",
        "layer" => "The Metal layer that receives the surface output.",
        "display" => "The native display connection that owns the surface.",
        "surface" => "The Wayland surface that receives the output.",
        "hinstance" => "The Windows module instance that owns the window.",
        "hwnd" => "The Windows window handle that receives the output.",
        "connection" => "The XCB connection that owns the window.",
        "device" => "The device that creates the surface textures.",
        "format" => "The pixel format of the surface textures.",
        "usage" => "The permitted uses of the surface textures.",
        "width" => "The surface texture width in pixels.",
        "height" => "The surface texture height in pixels.",
        "alpha_mode" => "The alpha composition mode for presentation.",
        "present_mode" => "The frame presentation mode.",
        "usages" => "The texture uses that the surface supports.",
        "texture" => "The current surface texture returned by the backend.",
        "status" => "The result of the current surface texture request.",
        _ => &fallback,
    };
    out.push_str(&format!(
        "    /// {description}\n    pub {}: {},\n",
        crate::naming::camel(&member.name),
        pointed_type(yml, &member.ty, member.pointer.as_deref())?
    ));
    Ok(())
}

/// Appends one `#[repr(C)]` struct to the emitted text.
///
/// An extensible struct leads with the chain head, and an extension struct leads with the chain
/// itself. A standalone struct leads with its first member. Any other kind is an invalid row.
fn render_struct(out: &mut String, yml: &Yml, shape: &Struct) -> Result<(), PolicyError> {
    out.push_str("#[repr(C)]\n#[derive(Clone, Copy)]\n");
    out.push_str(&format!(
        "/// The native {} data with the pinned C field layout.\npub struct {} {{\n",
        shape.name.replace('_', " "),
        crate::naming::wgpu_type(&shape.name)
    ));
    match shape.kind.as_str() {
        "extensible" => out.push_str("    /// The first extension, or null when no extension exists.\n    pub nextInChain: *mut WGPUChainedStruct,\n"),
        "extension" => out.push_str("    /// The extension header that identifies this structure and the next extension.\n    pub chain: WGPUChainedStruct,\n"),
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

/// Records the object, enum, and flag names that one yml type references.
///
/// An `array<T>` type contributes its element type. A scalar type contributes nothing. The
/// three sets decide which aliases and constant sets the module declares.
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

/// One host-only function, resolved against the pinned yml.
struct HostFunction<'a> {
    /// The webgpu.h symbol name.
    name: String,
    /// The receiving object of a method, absent for a freestanding function.
    receiver: Option<&'a str>,
    /// The yml declaration. AddRef and Release stay implicit in the yml, so both leave it empty.
    function: Option<&'a Function>,
    /// The struct of a free-members helper, which takes that struct by value.
    free_members: Option<&'a Struct>,
}

/// Resolves one `wgpu*` name to its yml source.
///
/// The search covers the freestanding functions, each object's AddRef and Release, each object
/// method, and each free-members helper, in that order. A name that matches none of them
/// returns `PolicyError::Unknown`.
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

/// The Rust parameter types and the return type of one host-only function.
///
/// A method leads with its receiver handle. A free-members helper takes its struct by value and
/// returns nothing. AddRef and Release take the receiver alone.
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

/// Validates the `[[host_only]]` rows two ways (F23).
///
/// A repeated row is `Duplicate`. A row the yml does not define is `Unknown`. An empty reason
/// and a row that `[[exclude]]` also names are both `Invalid`. A row outside `REQUIRED` is
/// `Dead`, and a member of `REQUIRED` that no row names is `Unpoliced`.
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

/// Renders the Rust-only `crates/facade/src/surface.rs` text from the `[[host_only]]` rows.
///
/// The output declares the ten surface functions and their structs, and resolves them on first
/// use (L14). A yml without a `surface` object and a policy without host-only rows produce a
/// placeholder module, so a fixture needs no surface data.
///
/// # Errors
///
/// Returns the policy error class that the host-only two-way validation reports (F23).
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
    // Every chained surface struct carries an `s_type` value, so that enum enters the module
    // even when no member names it.
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
        "//! Generated from webgpu.yml plus policy.toml. Do not edit.\n#![allow(non_snake_case, non_upper_case_globals)]\n\nuse std::ffi::{c_char, c_void};\nuse std::sync::OnceLock;\n\n",
    );
    for object in &objects {
        let name = crate::naming::wgpu_type(object);
        if object == "surface" {
            out.push_str(&format!("/// An opaque backend handle for a native surface.\npub type {name} = *mut c_void;\n"));
        } else {
            out.push_str(&format!(
                "/// The facade handle for a native {object}.\npub type {name} = crate::{};\n",
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
        out.push_str(&format!(
            "/// The native {} selection encoded for the C ABI.\npub type {ty} = u32;\n",
            name.replace('_', " ")
        ));
        for (index, entry) in value.entries.iter().enumerate() {
            let Some(entry) = entry else { continue };
            out.push_str(&format!(
                "/// Selects `{}` for [`{ty}`].\npub const {}: {ty} = {index};\n",
                entry.name,
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
        out.push_str(&format!(
            "/// The native {} bit mask for the C ABI.\npub type {ty} = u64;\n",
            name.replace('_', " ")
        ));
        for entry in &value.entries {
            let number = value
                .value_of(&entry.name)
                .ok_or_else(|| PolicyError::Unknown {
                    entry: format!("bitflag.{name}.{}", entry.name),
                })?;
            out.push_str(&format!(
                "/// The `{}` mask for [`{ty}`].\npub const {}: {ty} = {number};\n",
                entry.name,
                crate::naming::wgpu_enum_member(name, &entry.name)
            ));
        }
        out.push('\n');
    }

    out.push_str("#[repr(C)]\n#[derive(Clone, Copy)]\n/// The common header for a native descriptor extension.\npub struct WGPUChainedStruct {\n    /// The next extension, or null at the end of the chain.\n    pub next: *mut WGPUChainedStruct,\n    /// The type tag that identifies the extension structure.\n    pub sType: WGPUSType,\n}\n\n");
    if needs_string_view {
        out.push_str("#[repr(C)]\n#[derive(Clone, Copy)]\n/// A borrowed UTF-8 string for the native C ABI.\npub struct WGPUStringView {\n    /// The string bytes, or null for an absent string.\n    pub data: *const c_char,\n    /// The byte count, or `usize::MAX` for a null-terminated string.\n    pub length: usize,\n}\n\n");
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
            "/// The native `{}` entry point with its pinned argument and result ABI.\npub type {proc_name} = unsafe extern \"C\" fn({}){};\n",
            function.name,
            params.join(", "),
            result.map_or_else(String::new, |result| format!(" -> {result}"))
        ));
    }
    out.push_str("\n/// The surface entry points resolved from the backend library.\npub struct SurfaceTable {\n");
    for function in &functions {
        let proc_name = format!(
            "WGPUProc{}",
            function.name.strip_prefix("wgpu").unwrap_or(&function.name)
        );
        out.push_str(&format!(
            "    /// The backend address for `{}`.\n    pub {}: {proc_name},\n",
            function.name, function.name
        ));
    }
    out.push_str("}\n\nstatic SURFACE_TABLE: OnceLock<SurfaceTable> = OnceLock::new();\n\n/// Returns the cached surface entry points, or resolves them from the configured backend library.\n/// Returns an error if the backend or a required symbol cannot load, or the table cannot initialize.\npub fn table() -> Result<&'static SurfaceTable, String> {\n    if let Some(table) = SURFACE_TABLE.get() {\n        return Ok(table);\n    }\n    let loaded = SurfaceTable {\n");
    for function in &functions {
        out.push_str(&format!(
            "        {}: {{\n            // SAFETY: the type comes from this symbol's pinned webgpu.yml declaration.\n            unsafe {{ crate::runtime::surface_symbol(b\"{}\\0\") }}?\n        }},\n",
            function.name, function.name
        ));
    }
    out.push_str("    };\n    let _ = SURFACE_TABLE.set(loaded);\n    SURFACE_TABLE\n        .get()\n        .ok_or_else(|| \"surface function table initialization failed\".to_owned())\n}\n");
    Ok(out)
}
