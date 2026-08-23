//! Assembles the generated Rust facade implementation.

use crate::patterns::{
    adapter_limits, byte_pair, constants, descriptor, descriptor_async, device_events, future_poll,
    handle_array, handles, label, map_async, shader_wgsl, strings, sync, typed_pair, write_texture,
};
use crate::plan::{Chunk, Plan};
use std::collections::BTreeSet;

const MODULE_DOC: &str = "//! GENERATED FILE — DO NOT EDIT.\n//!\n//! `tools/regen.sh` emits this file from the pinned inputs and policy.\n//! The regeneration test compares the committed bytes.\n//!\n//! Async callbacks use AllowProcessEvents and copy borrowed messages.\n//! Backend handle cleanup occurs after each callback returns.\n";

const MODULE_DOC_DEVICE_EVENTS: &str = "//! GENERATED FILE — DO NOT EDIT.\n//!\n//! `tools/regen.sh` emits this file from the pinned inputs and policy.\n//! The regeneration test compares the committed bytes.\n//!\n//! Future and lost callbacks use AllowProcessEvents.\n//! The uncaptured-error callback only records copied data.\n//! Backend handle cleanup occurs after each callback returns.\n";

const FFI_SEPARATOR: &str = "// ---------------------------------------------------------------------\n// webgpu.h FFI subset (emitted from webgpu.yml for the policy subset;\n// no rust-bindgen). webgpu.h names stay private to this module.\n// ---------------------------------------------------------------------\n";

const SUBSCRIPT_TYPEGPU_SEPARATOR: &str = "// ---------------------------------------------------------------------\n// subscript-typegpu.h surface and panic-free export bodies.\n// ---------------------------------------------------------------------\n";

const FUTURE_STRUCT: &str = "/// webgpu.h `WGPUFuture { uint64_t id; }`.\n#[repr(C)]\nstruct WGPUFuture {\n    id: u64,\n}\n";

struct FunctionSignature {
    name: String,
    params: String,
    param_types: String,
    args: String,
    result: String,
}

fn function_signatures(declarations: &str) -> Vec<FunctionSignature> {
    let mut signatures = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for declaration in declarations.split(';') {
        let normalized = declaration
            .lines()
            .skip_while(|line| !line.trim_start().starts_with("fn "))
            .flat_map(str::split_whitespace)
            .collect::<Vec<_>>()
            .join(" ");
        let Some(signature) = normalized.strip_prefix("fn ") else {
            continue;
        };
        let open = signature
            .find('(')
            .expect("generated function has parameters");
        let close = signature
            .rfind(')')
            .expect("generated function closes parameters");
        let name = signature[..open].to_owned();
        if !seen.insert(name.clone()) {
            continue;
        }
        let params = signature[open + 1..close].trim().to_owned();
        let result = signature[close + 1..].trim().to_owned();
        let mut pairs = Vec::new();
        let mut start = 0;
        let mut depth = 0_u32;
        for (index, character) in params.char_indices() {
            match character {
                '(' | '[' | '<' => depth += 1,
                ')' | ']' | '>' => depth = depth.saturating_sub(1),
                ',' if depth == 0 => {
                    let pair = params[start..index].trim();
                    if !pair.is_empty() {
                        pairs.push(pair);
                    }
                    start = index + character.len_utf8();
                }
                _ => {}
            }
        }
        let pair = params[start..].trim();
        if !pair.is_empty() {
            pairs.push(pair);
        }
        let param_types = pairs
            .iter()
            .map(|pair| {
                pair.split_once(':')
                    .unwrap_or_else(|| {
                        panic!("generated parameter has a type: `{pair}` in `{params}`")
                    })
                    .1
                    .trim()
            })
            .collect::<Vec<_>>()
            .join(", ");
        let args = pairs
            .iter()
            .map(|pair| {
                pair.split_once(':')
                    .unwrap_or_else(|| {
                        panic!("generated parameter has a name: `{pair}` in `{params}`")
                    })
                    .0
                    .trim()
            })
            .collect::<Vec<_>>()
            .join(", ");
        signatures.push(FunctionSignature {
            name,
            params,
            param_types,
            args,
            result,
        });
    }
    signatures
}

fn render_webgpu_table(declarations: &str) -> String {
    let signatures = function_signatures(declarations);
    let mut out = String::from(
        "pub(crate) struct WebgpuTable {\n    pub(crate) library: libloading::Library,\n    pub(crate) is_yawgpu: bool,\n",
    );
    for signature in &signatures {
        out.push_str(&format!(
            "    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.\n\
                 {}: unsafe extern \"C\" fn({}){},\n",
            signature.name, signature.param_types, signature.result
        ));
    }
    out.push_str("}\n\nimpl WebgpuTable {\n    pub(crate) fn load(path: &std::path::Path) -> Result<Self, String> {\n        #[cfg(windows)]\n        // SAFETY: The library stays owned by the returned table.\n        // The Windows flag searches the backend directory for dependent libraries.\n        let library = unsafe {\n            libloading::os::windows::Library::load_with_flags(\n                path,\n                libloading::os::windows::LOAD_WITH_ALTERED_SEARCH_PATH,\n            )\n        }\n        .map(libloading::Library::from);\n        #[cfg(not(windows))]\n        // SAFETY: The library stays owned by the returned table.\n        let library = unsafe { libloading::Library::new(path) };\n        let library = library\n            .map_err(|error| format!(\"load {}: {error}\", path.display()))?;\n        fn symbol<T: Copy>(\n            library: &libloading::Library,\n            path: &std::path::Path,\n            name: &'static [u8],\n        ) -> Result<T, String> {\n            // SAFETY: each call uses the pinned webgpu.h signature for this symbol.\n            unsafe { library.get::<T>(name) }\n                .map(|value| *value)\n                .map_err(|error| {\n                    let name = std::str::from_utf8(&name[..name.len() - 1]).unwrap_or(\"<invalid>\");\n                    format!(\"missing symbol {name} in {}: {error}\", path.display())\n                })\n        }\n        Ok(Self {\n");
    out = out.replace(
        "        fn symbol<T: Copy>(",
        "        // SAFETY: the marker is probed but never called.\n\
         \x20       let is_yawgpu = unsafe {\n\
         \x20           library\n\
         \x20               .get::<*const ()>(b\"yawgpuDeviceCreateExternalTexture\\0\")\n\
         \x20               .is_ok()\n\
         \x20       };\n\
         \x20       fn symbol<T: Copy>(",
    );
    for signature in &signatures {
        out.push_str(&format!(
            "            {0}: symbol(&library, path, b\"{0}\\0\")?,\n",
            signature.name
        ));
    }
    out.push_str("            library,\n            is_yawgpu,\n        })\n    }\n}\n\n");
    for signature in &signatures {
        out.push_str(&format!(
            "unsafe fn {name}({params}){result} {{\n    let Some(table) = crate::runtime::table() else {{\n        eprintln!(\"subscript-typegpu: cannot call {name}: set SUBSCRIPT_TYPEGPU_BACKEND_LIB\");\n        std::process::abort();\n    }};\n    // SAFETY: the table stores the pinned signature for this symbol.\n    unsafe {{ (table.{name})({args}) }}\n}}\n\n",
            name = signature.name,
            params = signature.params,
            result = signature.result,
            args = signature.args,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::render_webgpu_table;

    #[test]
    fn function_table_accepts_docs_and_function_pointer_parameters() {
        let declarations = r#"
            /// A declaration with a function-pointer parameter.
            fn wgpuWithCallback(
                callback: Option<unsafe extern "C" fn(i32, *mut std::ffi::c_void)>,
                userdata: *mut std::ffi::c_void,
            );
            /// A documented scalar declaration.
            fn wgpuDocumented(value: u32) -> u32;
        "#;
        let table = render_webgpu_table(declarations);
        assert!(table.contains("wgpuWithCallback: unsafe extern \"C\" fn"));
        assert!(table.contains("wgpuDocumented: unsafe extern \"C\" fn"));
        assert!(table.contains("(table.wgpuWithCallback)(callback, userdata)"));
        assert!(table.contains("pub(crate) is_yawgpu: bool"));
        assert!(table.contains("yawgpuDeviceCreateExternalTexture"));
    }
}

pub(crate) fn render(plan: &Plan, excluded_exports: &BTreeSet<String>) -> String {
    let async_ops: Vec<_> = plan
        .chunks
        .iter()
        .filter_map(|chunk| match chunk {
            Chunk::Async(op) => Some(op),
            Chunk::DescriptorAsync(op) => Some(&op.async_op),
            Chunk::MapAsync(op) => Some(&op.async_op),
            _ => None,
        })
        .collect();
    let shader_ops: Vec<_> = plan
        .chunks
        .iter()
        .filter_map(|chunk| match chunk {
            Chunk::ShaderWgsl(op) => Some(op),
            _ => None,
        })
        .collect();
    let device_events_op = plan.chunks.iter().find_map(|chunk| match chunk {
        Chunk::DeviceEvents(op) => Some(op),
        _ => None,
    });
    let info_ops: Vec<_> = plan
        .chunks
        .iter()
        .filter_map(|chunk| match chunk {
            Chunk::AdapterInfo(op) => Some(op),
            _ => None,
        })
        .collect();
    let has_async = !async_ops.is_empty();
    let has_byte_pointer = plan
        .chunks
        .iter()
        .any(|chunk| matches!(chunk, Chunk::BytePair(_) | Chunk::WriteTexture(_)));
    let needs_opaque_chain =
        shader_ops.is_empty() && (has_async || plan.structs.iter().any(|shape| shape.extensible));
    let mut all_objects = plan.objects.clone();
    for object in &plan.referenced_objects {
        if !all_objects.contains(object) {
            all_objects.push(object.clone());
        }
    }

    let mut out = String::from(if plan.device_events {
        MODULE_DOC_DEVICE_EVENTS
    } else {
        MODULE_DOC
    });
    out.push('\n');
    out.push_str("#![allow(non_snake_case, non_upper_case_globals)]\n\n");
    out.push_str("#![allow(clippy::not_unsafe_ptr_arg_deref, clippy::vec_box)]\n\n");

    let mut std_ffi = Vec::new();
    if plan.needs_string_view {
        std_ffi.push("c_char");
    }
    if has_async || has_byte_pointer {
        std_ffi.push("c_void");
    }
    if has_async {
        std_ffi.push("CStr");
    }
    if !std_ffi.is_empty() {
        out.push_str(&format!("use std::ffi::{{{}}};\n\n", std_ffi.join(", ")));
    }
    if has_async {
        out.push_str("use crate::runtime;\n\n");
    }

    out.push_str(FFI_SEPARATOR);
    out.push('\n');
    out.push_str(handles::rust_opaque_macro());
    out.push('\n');
    let instance_descriptor = plan
        .creates
        .iter()
        .find(|create| create.returns_object == "instance")
        .and_then(|create| create.dropped_arg.as_ref())
        .map(|(_, descriptor)| descriptor.as_str());
    let mut pointer_only = plan.pointer_only.clone();
    if let Some(instance_descriptor) = instance_descriptor {
        pointer_only.retain(|name| name != instance_descriptor);
    }
    let request_adapter_options = async_ops
        .iter()
        .find(|op| op.wgpu_fn == "wgpuInstanceRequestAdapter")
        .and_then(|op| op.dropped_arg.as_ref())
        .map(|(_, options)| options.as_str());
    if let Some(request_adapter_options) = request_adapter_options {
        pointer_only.retain(|name| name != request_adapter_options);
    }
    out.push_str(&handles::rust_wgpu_opaque_block(
        &all_objects,
        needs_opaque_chain,
        &pointer_only,
    ));
    out.push('\n');
    out.push_str(&handles::rust_wgpu_aliases(&all_objects));

    if let Some(instance_descriptor) = instance_descriptor {
        out.push('\n');
        out.push_str(&sync::rust_instance_backend_types(instance_descriptor));
    }
    if let Some(request_adapter_options) = request_adapter_options {
        out.push('\n');
        out.push_str(&future_poll::rust_request_adapter_options(
            request_adapter_options,
        ));
    }

    for op in &shader_ops {
        out.push('\n');
        out.push_str(&shader_wgsl::rust_private_types(op));
    }

    if plan.needs_string_view {
        out.push('\n');
        out.push_str(strings::rust_string_views());
        out.push('\n');
        out.push_str(constants::rust_strlen_const());
    }
    if let Some(op) = device_events_op {
        out.push('\n');
        out.push_str(&device_events::rust_private_types(op));
    }
    if !info_ops.is_empty() {
        out.push('\n');
        out.push_str(adapter_limits::rust_adapter_info_private());
    }
    if has_async {
        out.push('\n');
        out.push_str(FUTURE_STRUCT);
        if let Some((name, value)) = &plan.mode_const {
            out.push('\n');
            out.push_str(&constants::rust_mode_const(name, *value));
        }
        let mut emitted_statuses = std::collections::BTreeSet::new();
        for op in &async_ops {
            if !emitted_statuses.insert(op.cb.status_const.as_str()) {
                continue;
            }
            out.push('\n');
            out.push_str(&future_poll::rust_status_const(op));
        }
    }
    for set in &plan.const_sets {
        out.push('\n');
        out.push_str(&constants::rust_const_set(set));
    }
    for sentinel in &plan.sentinel_consts {
        out.push('\n');
        out.push_str(&descriptor::rust_sentinel_const(sentinel));
    }
    if let Some(op) = info_ops.first() {
        out.push('\n');
        out.push_str(&adapter_limits::rust_info_success_const(op));
    }
    for op in &async_ops {
        out.push('\n');
        out.push_str(&future_poll::rust_callback_typedef(op));
        out.push('\n');
        out.push_str(&future_poll::rust_callback_info(op));
    }
    for shape in &plan.structs {
        out.push('\n');
        out.push_str(&descriptor::rust_structs(shape));
        out.push('\n');
        out.push_str(&descriptor::rust_conversion(shape));
    }

    let mut declarations = String::new();
    for create in &plan.creates {
        if !excluded_exports.contains(&create.subscript_typegpu_fn) {
            declarations.push_str(&sync::rust_create_extern(create));
        }
    }
    for op in &plan.anchor_syncs {
        if !excluded_exports.contains(&op.subscript_typegpu_fn) {
            declarations.push_str(&sync::rust_sync_extern(op));
        }
    }
    declarations.push_str(&handles::rust_release_extern(&plan.anchor));
    for chunk in &plan.chunks {
        match chunk {
            Chunk::Async(op) => declarations.push_str(&future_poll::rust_async_extern(op)),
            Chunk::Sync(op) => {
                if !excluded_exports.contains(&op.subscript_typegpu_fn) {
                    declarations.push_str(&sync::rust_sync_extern(op));
                }
            }
            Chunk::Descriptor(op) => {
                let shape = plan
                    .structs
                    .iter()
                    .find(|shape| shape.source == op.descriptor)
                    .expect("descriptor shape exists");
                declarations.push_str(&descriptor::rust_extern(op, shape));
            }
            Chunk::DescriptorAsync(op) => {
                let shape = plan
                    .structs
                    .iter()
                    .find(|shape| shape.source == op.descriptor)
                    .expect("descriptor shape exists");
                declarations.push_str(&descriptor_async::rust_extern(op, shape));
            }
            Chunk::ShaderWgsl(op) => declarations.push_str(&shader_wgsl::rust_extern(op)),
            Chunk::Label(op) => declarations.push_str(&label::rust_extern(op)),
            Chunk::BytePair(op) => declarations.push_str(&byte_pair::rust_extern(op)),
            Chunk::TypedPair(_) => {}
            Chunk::Array(op) => declarations.push_str(&handle_array::rust_extern(op)),
            Chunk::MapAsync(op) => declarations.push_str(&map_async::rust_extern(op)),
            Chunk::WriteTexture(op) => {
                let find = |name: &str| {
                    plan.structs
                        .iter()
                        .find(|shape| shape.source == name)
                        .expect("write-texture shape exists")
                };
                declarations.push_str(&write_texture::rust_extern(
                    op,
                    find(&op.destination),
                    find(&op.layout),
                    find(&op.extent),
                ));
            }
            Chunk::DeviceEvents(op) => declarations.push_str(&device_events::rust_extern(op)),
            Chunk::Limits(op) => {
                if !excluded_exports.contains(&op.subscript_typegpu_fn) {
                    let shape = plan
                        .structs
                        .iter()
                        .find(|shape| shape.source == op.shape)
                        .expect("limits shape exists");
                    declarations.push_str(&adapter_limits::rust_limits_extern(op, shape));
                }
            }
            Chunk::AdapterInfo(op) => declarations.push_str(&adapter_limits::rust_info_extern(op)),
            Chunk::Feature(op) => {
                if !excluded_exports.contains(&op.subscript_typegpu_fn) {
                    declarations.push_str(&adapter_limits::rust_feature_extern(op));
                }
            }
        }
    }
    if !info_ops.is_empty() {
        declarations.push_str(adapter_limits::rust_info_free_extern());
    }
    for object in plan
        .objects
        .iter()
        .rev()
        .filter(|object| **object != plan.anchor)
    {
        declarations.push_str(&handles::rust_release_extern(object));
    }
    out.push('\n');
    out.push_str(&render_webgpu_table(&declarations));

    out.push('\n');
    out.push_str(SUBSCRIPT_TYPEGPU_SEPARATOR);
    out.push('\n');
    out.push_str(&handles::rust_subscript_typegpu_opaque_block(&plan.objects));
    out.push('\n');
    out.push_str(&handles::rust_subscript_typegpu_aliases(&plan.objects));
    if plan.device_events {
        out.push('\n');
        out.push_str(device_events::rust_public_records());
    }
    if !info_ops.is_empty() {
        out.push('\n');
        out.push_str(adapter_limits::rust_adapter_info_public());
    }
    if plan.device_descriptor {
        out.push('\n');
        out.push_str(adapter_limits::rust_device_descriptor_public());
        out.push('\n');
        out.push_str(adapter_limits::rust_required_limits_probe());
    }
    for op in &shader_ops {
        out.push('\n');
        out.push_str(&shader_wgsl::rust_public_type(op));
    }
    for set in &plan.const_sets {
        out.push('\n');
        out.push_str(&constants::rust_subscript_typegpu_const_set(set));
    }
    if has_async {
        out.push('\n');
        out.push_str("/// `subscript-typegpu.h`: facade-owned future id.\npub type SubscriptTypegpuFutureId = u64;\n");
        for op in &async_ops {
            out.push('\n');
            out.push_str(&future_poll::rust_kind_const(op));
        }
        out.push('\n');
        out.push_str(future_poll::rust_copy_string_view());
        for op in &async_ops {
            out.push('\n');
            out.push_str(&future_poll::rust_callback_fn(op, plan.device_events));
        }
        if let Some(op) = device_events_op {
            out.push('\n');
            out.push_str(&device_events::rust_constants(op));
            out.push('\n');
            out.push_str(&device_events::rust_callbacks(op));
        }
        out.push('\n');
        out.push_str(&future_poll::rust_release_helpers(&async_ops));
    }

    for create in &plan.creates {
        out.push('\n');
        out.push_str(&sync::rust_create_export(create));
    }
    for op in &plan.anchor_syncs {
        out.push('\n');
        out.push_str(&sync::rust_sync_export(op, has_async));
    }
    out.push('\n');
    if has_async {
        out.push_str(&future_poll::rust_anchor_release_export(&plan.anchor));
    } else {
        out.push_str(&handles::rust_release_export(
            &plan.anchor,
            plan.device_events,
        ));
    }
    for chunk in &plan.chunks {
        match chunk {
            Chunk::Async(op) => {
                let mode = plan
                    .mode_const
                    .as_ref()
                    .map(|(name, _)| name.as_str())
                    .expect("async ops imply a mode constant");
                out.push('\n');
                out.push_str(&future_poll::rust_request_export(
                    op,
                    &plan.anchor,
                    mode,
                    plan.device_events,
                ));
                if op.first {
                    out.push('\n');
                    out.push_str(&future_poll::rust_completed_export(&plan.anchor));
                    out.push('\n');
                    out.push_str(&future_poll::rust_drop_export(&plan.anchor));
                }
                if op.take_fn.is_some() {
                    out.push('\n');
                    out.push_str(&future_poll::rust_take_export(op, &plan.anchor));
                }
            }
            Chunk::Sync(op) => {
                out.push('\n');
                out.push_str(&sync::rust_sync_export(op, false));
            }
            Chunk::Descriptor(op) => {
                let shape = plan
                    .structs
                    .iter()
                    .find(|shape| shape.source == op.descriptor)
                    .expect("descriptor shape exists");
                out.push('\n');
                out.push_str(&descriptor::rust_export(op, shape));
            }
            Chunk::DescriptorAsync(op) => {
                let shape = plan
                    .structs
                    .iter()
                    .find(|shape| shape.source == op.descriptor)
                    .expect("descriptor shape exists");
                let mode = plan
                    .mode_const
                    .as_ref()
                    .map(|(name, _)| name.as_str())
                    .expect("async ops imply a mode constant");
                out.push('\n');
                out.push_str(&descriptor_async::rust_export(
                    op,
                    shape,
                    &plan.anchor,
                    mode,
                ));
                if op.async_op.take_fn.is_some() {
                    out.push('\n');
                    out.push_str(&future_poll::rust_take_export(&op.async_op, &plan.anchor));
                }
            }
            Chunk::ShaderWgsl(op) => {
                out.push('\n');
                out.push_str(&shader_wgsl::rust_export(op));
            }
            Chunk::Label(op) => {
                out.push('\n');
                out.push_str(&label::rust_export(op));
            }
            Chunk::BytePair(op) => {
                out.push('\n');
                out.push_str(&byte_pair::rust_export(op));
            }
            Chunk::TypedPair(op) => {
                out.push('\n');
                out.push_str(&typed_pair::rust_export(op));
            }
            Chunk::Array(op) => {
                out.push('\n');
                out.push_str(&handle_array::rust_export(op));
            }
            Chunk::MapAsync(op) => {
                let mode = plan
                    .mode_const
                    .as_ref()
                    .map(|(name, _)| name.as_str())
                    .expect("async ops imply a mode constant");
                out.push('\n');
                out.push_str(&map_async::rust_exports(op, mode));
            }
            Chunk::WriteTexture(op) => {
                let find = |name: &str| {
                    plan.structs
                        .iter()
                        .find(|shape| shape.source == name)
                        .expect("write-texture shape exists")
                };
                out.push('\n');
                out.push_str(&write_texture::rust_export(
                    op,
                    find(&op.destination),
                    find(&op.layout),
                    find(&op.extent),
                ));
            }
            Chunk::DeviceEvents(op) => {
                let mode = plan
                    .mode_const
                    .as_ref()
                    .map(|(name, _)| name.as_str())
                    .expect("device events imply a mode constant");
                out.push('\n');
                out.push_str(&device_events::rust_exports(op, &plan.anchor, mode));
            }
            Chunk::Limits(op) => {
                let shape = plan
                    .structs
                    .iter()
                    .find(|shape| shape.source == op.shape)
                    .expect("limits shape exists");
                out.push('\n');
                out.push_str(&adapter_limits::rust_limits_export(op, shape));
            }
            Chunk::AdapterInfo(op) => {
                out.push('\n');
                out.push_str(&adapter_limits::rust_info_export(op));
            }
            Chunk::Feature(op) => {
                out.push('\n');
                out.push_str(&adapter_limits::rust_feature_export(op));
            }
        }
    }
    for object in plan
        .objects
        .iter()
        .rev()
        .filter(|object| **object != plan.anchor)
    {
        out.push('\n');
        out.push_str(&handles::rust_release_export(object, plan.device_events));
    }
    out
}
