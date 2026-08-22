//! Plain-sync pattern: freestanding create functions with dropped
//! optional descriptors, and ordinary handle/scalar/struct methods.

use crate::naming;
use crate::plan::{CreateOp, MethodArg, Scalar, SyncOp, SyncRet};

/// Private instance descriptor and yawgpu backend-select extension types.
pub(crate) fn rust_instance_backend_types(instance_descriptor: &str) -> String {
    "// Companion header: https://github.com/infosia/yawgpu/blob/main/ffi/webgpu-headers/yawgpu.h\n\
     #[repr(C)]\n\
     struct YawgpuChainedStruct {\n\
         next: *mut YawgpuChainedStruct,\n\
         s_type: i32,\n\
     }\n\n\
     #[repr(C)]\n\
     struct YawgpuInstanceBackendSelect {\n\
         chain: YawgpuChainedStruct,\n\
         backend: u32,\n\
     }\n\n\
     #[repr(C)]\n\
     struct $INSTANCE_DESCRIPTOR$ {\n\
         next_in_chain: *mut YawgpuChainedStruct,\n\
         required_feature_count: usize,\n\
         required_features: *const i32,\n\
         required_limits: *const WGPUInstanceLimits,\n\
     }\n\n\
     const YAWGPU_STYPE_INSTANCE_BACKEND_SELECT: i32 = 0x7000_0001;\n\
     #[allow(dead_code)]\n\
     const YAWGPU_INSTANCE_BACKEND_NOOP: u32 = 0;\n\
     const YAWGPU_INSTANCE_BACKEND_METAL: u32 = 1;\n\
     const YAWGPU_INSTANCE_BACKEND_VULKAN: u32 = 2;\n\
     const YAWGPU_INSTANCE_BACKEND_GLES: u32 = 3;\n"
        .replace("$INSTANCE_DESCRIPTOR$", instance_descriptor)
}

fn arg_name(arg: &MethodArg) -> &str {
    match arg {
        MethodArg::Scalar(name, _) | MethodArg::Bitflag(name, _) | MethodArg::Enum(name, _) => name,
        MethodArg::Object { name, .. } | MethodArg::StructPointer { name, .. } => name,
    }
}

fn c_arg(arg: &MethodArg) -> String {
    let name = naming::camel(arg_name(arg));
    match arg {
        MethodArg::Scalar(_, scalar) => format!("{} {name}", scalar.c_name()),
        MethodArg::Bitflag(_, ty) | MethodArg::Enum(_, ty) => {
            format!("{} {name}", naming::subscript_typegpu_type(ty))
        }
        MethodArg::Object {
            object, nullable, ..
        } => format!(
            "{}{} {name}",
            naming::subscript_typegpu_type(object),
            if *nullable { " _Nullable" } else { "" }
        ),
        MethodArg::StructPointer { shape, .. } => {
            format!("const {}* {name}", naming::subscript_typegpu_type(shape))
        }
    }
}

fn rust_arg_type(arg: &MethodArg, backend: bool) -> String {
    match arg {
        MethodArg::Scalar(_, scalar) => scalar.rust_name().into(),
        MethodArg::Bitflag(_, _) => "u64".into(),
        MethodArg::Enum(_, _) => "i32".into(),
        MethodArg::Object { object, .. } => {
            if backend {
                naming::wgpu_type(object)
            } else {
                naming::subscript_typegpu_type(object)
            }
        }
        MethodArg::StructPointer { shape, .. } => format!(
            "*const {}",
            if backend {
                naming::wgpu_type(shape)
            } else {
                naming::subscript_typegpu_type(shape)
            }
        ),
    }
}

fn null_return(ret: &SyncRet) -> &'static str {
    match ret {
        SyncRet::Void => "        return;\n",
        SyncRet::Handle(_) => "        return std::ptr::null_mut();\n",
        SyncRet::Scalar(Scalar::F32) => "        return 0.0;\n",
        SyncRet::Scalar(_) | SyncRet::Bitflag(_) | SyncRet::Enum(_) => "        return 0;\n",
    }
}

/// C declaration for a create function: `SubscriptTypegpuXxx subscript_typegpu_create_xxx(void);`
pub(crate) fn c_create_decl(op: &CreateOp) -> String {
    format!(
        "{} {}(void);",
        naming::subscript_typegpu_type(&op.returns_object),
        op.subscript_typegpu_fn
    )
}

/// C declaration for a sync method.
pub(crate) fn c_sync_decl(op: &SyncOp) -> String {
    let ret = match &op.ret {
        SyncRet::Void => "void".to_string(),
        SyncRet::Handle(target) => naming::subscript_typegpu_type(target),
        SyncRet::Scalar(s) => s.c_name().to_string(),
        SyncRet::Bitflag(name) | SyncRet::Enum(name) => naming::subscript_typegpu_type(name),
    };
    let mut params = vec![format!(
        "{} {}",
        naming::subscript_typegpu_type(&op.receiver),
        naming::camel(&op.receiver)
    )];
    params.extend(op.args.iter().map(c_arg));
    format!("{ret} {}({});", op.subscript_typegpu_fn, params.join(", "))
}

/// Extern declaration for a create function.
pub(crate) fn rust_create_extern(op: &CreateOp) -> String {
    let ret = naming::wgpu_type(&op.returns_object);
    match &op.dropped_arg {
        Some((name, ty)) => format!(
            "    fn {}({}: *const {ty}) -> {ret};\n",
            op.wgpu_fn,
            naming::camel(name)
        ),
        None => format!("    fn {}() -> {ret};\n", op.wgpu_fn),
    }
}

/// Extern declaration for a sync method.
pub(crate) fn rust_sync_extern(op: &SyncOp) -> String {
    let mut params = vec![format!(
        "{}: {}",
        naming::camel(&op.receiver),
        naming::wgpu_type(&op.receiver)
    )];
    params.extend(op.args.iter().map(|arg| {
        format!(
            "{}: {}",
            naming::camel(arg_name(arg)),
            rust_arg_type(arg, true)
        )
    }));
    let ret = match &op.ret {
        SyncRet::Void => String::new(),
        SyncRet::Handle(target) => format!(" -> {}", naming::wgpu_type(target)),
        SyncRet::Scalar(s) => format!(" -> {}", s.rust_name()),
        SyncRet::Bitflag(_) => " -> u64".to_string(),
        SyncRet::Enum(_) => " -> i32".to_string(),
    };
    format!("    fn {}({}){ret};\n", op.wgpu_fn, params.join(", "))
}

/// The exported create wrapper (NULL descriptor when dropped).
pub(crate) fn rust_create_export(op: &CreateOp) -> String {
    let subscript_typegpu_ret = naming::subscript_typegpu_type(&op.returns_object);
    let doc = op
        .doc
        .as_deref()
        .unwrap_or("creates the handle with no descriptor.");
    let (safety, call) = if op.returns_object == "instance" {
        let instance_descriptor = &op
            .dropped_arg
            .as_ref()
            .expect("instance creation has its validated descriptor")
            .1;
        (
            concat!(
                "    let requested_backend = std::env::var_os(\"SUBSCRIPT_TYPEGPU_BACKEND\");\n",
                "    let backend = match requested_backend.as_deref().and_then(std::ffi::OsStr::to_str) {\n",
                "        None if requested_backend.is_none() => None,\n",
                "        Some(\"metal\") => Some((\"metal\", YAWGPU_INSTANCE_BACKEND_METAL)),\n",
                "        Some(\"vulkan\") => Some((\"vulkan\", YAWGPU_INSTANCE_BACKEND_VULKAN)),\n",
                "        Some(\"gles\") => Some((\"gles\", YAWGPU_INSTANCE_BACKEND_GLES)),\n",
                "        _ => {\n",
                "            let value = requested_backend.as_deref().map_or_else(\n",
                "                || \"<non-UTF-8>\".into(),\n",
                "                |value| value.to_string_lossy(),\n",
                "            );\n",
                "            eprintln!(\"subscript-typegpu: unknown SUBSCRIPT_TYPEGPU_BACKEND value `{value}`; expected metal, vulkan, or gles\");\n",
                "            return std::ptr::null_mut();\n",
                "        }\n",
                "    };\n",
                "    if !runtime::initialize_table() {\n",
                "        return std::ptr::null_mut();\n",
                "    }\n",
                "    let mut select = backend.map(|(_, backend)| YawgpuInstanceBackendSelect {\n",
                "        chain: YawgpuChainedStruct {\n",
                "            next: std::ptr::null_mut(),\n",
                "            s_type: YAWGPU_STYPE_INSTANCE_BACKEND_SELECT,\n",
                "        },\n",
                "        backend,\n",
                "    });\n",
                "    let descriptor = select.as_mut().map(|select| $INSTANCE_DESCRIPTOR$ {\n",
                "        next_in_chain: &mut select.chain,\n",
                "        required_feature_count: 0,\n",
                "        required_features: std::ptr::null(),\n",
                "        required_limits: std::ptr::null(),\n",
                "    });\n",
                "    let descriptor = descriptor.as_ref().map_or(std::ptr::null(), |value| value);\n",
                "    // SAFETY: the optional descriptor and chain live through the backend call.\n",
            )
                .replace("$INSTANCE_DESCRIPTOR$", instance_descriptor),
            format!(
                "    let instance: SubscriptTypegpuInstance = unsafe {{ {}(descriptor).cast() }};\n    if instance.is_null() {{\n        if let Some((request, _)) = backend {{\n            let path = std::env::var_os(\"SUBSCRIPT_TYPEGPU_BACKEND_LIB\")\n                .map(std::path::PathBuf::from)\n                .map_or_else(|| \"<unset>\".into(), |path| path.display().to_string());\n            eprintln!(\"subscript-typegpu: backend request `{{request}}` returned a null instance from {{path}}\");\n        }}\n        return std::ptr::null_mut();\n    }}\n    runtime::register_instance(instance as usize);\n    instance\n",
                op.wgpu_fn
            ),
        )
    } else {
        match &op.dropped_arg {
            Some(_) => (
                "    // SAFETY: NULL descriptor is explicitly allowed by webgpu.h.\n".to_owned(),
                format!("    unsafe {{ {}(std::ptr::null()).cast() }}\n", op.wgpu_fn),
            ),
            None => (
                "    // SAFETY: no-argument creation.\n".to_owned(),
                format!("    unsafe {{ {}().cast() }}\n", op.wgpu_fn),
            ),
        }
    };
    format!(
        "/// `subscript-typegpu.h`: {doc}\n\
         #[no_mangle]\n\
         pub extern \"C\" fn {}() -> {subscript_typegpu_ret} {{\n{safety}{call}}}\n",
        op.subscript_typegpu_fn
    )
}

/// The exported sync wrapper (NULL-tolerant receiver).
pub(crate) fn rust_sync_export(op: &SyncOp, drain_callbacks: bool) -> String {
    let recv = naming::camel(&op.receiver);
    let recv_ty = naming::subscript_typegpu_type(&op.receiver);
    let mut guards = String::new();
    let mut conversions = String::new();
    let mut arg_values = Vec::new();
    for arg in &op.args {
        let name = naming::camel(arg_name(arg));
        match arg {
            MethodArg::Scalar(..) | MethodArg::Bitflag(..) | MethodArg::Enum(..) => {
                arg_values.push(name);
            }
            MethodArg::Object { nullable, .. } => {
                if !nullable {
                    guards.push_str(&format!(
                        "    if {name}.is_null() {{\n{}    }}\n",
                        null_return(&op.ret)
                    ));
                }
                arg_values.push(format!("{name}.cast()"));
            }
            MethodArg::StructPointer {
                shape,
                nullable,
                owns_storage,
                ..
            } => {
                if *nullable {
                    conversions.push_str(&format!(
                        "    let converted_{name} = if {name}.is_null() {{\n        None\n    }} else {{\n        // SAFETY: a non-null input pointer is readable for this call.\n        Some(convert_{shape}(unsafe {{ *{name} }}))\n    }};\n"
                    ));
                    let value = if *owns_storage {
                        "&value.value"
                    } else {
                        "value"
                    };
                    conversions.push_str(&format!(
                        "    let {name}_ptr = converted_{name}.as_ref().map_or(std::ptr::null(), |value| {value} as *const _);\n"
                    ));
                    arg_values.push(format!("{name}_ptr"));
                } else {
                    guards.push_str(&format!(
                        "    if {name}.is_null() {{\n{}    }}\n",
                        null_return(&op.ret)
                    ));
                    conversions.push_str(&format!(
                        "    // SAFETY: the non-null input pointer is readable for this call.\n    let converted_{name} = convert_{shape}(unsafe {{ *{name} }});\n"
                    ));
                    arg_values.push(if *owns_storage {
                        format!("&converted_{name}.value")
                    } else {
                        format!("&converted_{name}")
                    });
                }
            }
        }
    }
    let call_args = if arg_values.is_empty() {
        format!("{recv}.cast()")
    } else {
        format!("{recv}.cast(), {}", arg_values.join(", "))
    };
    let (ret_sig, call) = match &op.ret {
        SyncRet::Void => (
            String::new(),
            format!(
                "    unsafe {{ {}({call_args}) }}\n{}",
                op.wgpu_fn,
                if drain_callbacks {
                    "    release_deferred_handles();\n"
                } else {
                    ""
                }
            ),
        ),
        SyncRet::Handle(target) => (
            format!(" -> {}", naming::subscript_typegpu_type(target)),
            format!("    unsafe {{ {}({call_args}).cast() }}\n", op.wgpu_fn),
        ),
        SyncRet::Scalar(s) => (
            format!(" -> {}", s.rust_name()),
            format!("    unsafe {{ {}({call_args}) }}\n", op.wgpu_fn),
        ),
        SyncRet::Bitflag(_) => (
            " -> u64".to_string(),
            format!("    unsafe {{ {}({call_args}) }}\n", op.wgpu_fn),
        ),
        SyncRet::Enum(_) => (
            " -> i32".to_string(),
            format!("    unsafe {{ {}({call_args}) }}\n", op.wgpu_fn),
        ),
    };
    let mut params = vec![format!("{recv}: {recv_ty}")];
    params.extend(op.args.iter().map(|arg| {
        format!(
            "{}: {}",
            naming::camel(arg_name(arg)),
            rust_arg_type(arg, false)
        )
    }));
    format!(
        "/// `subscript-typegpu.h`: forwards to `{wgpu}`.\n\
         #[no_mangle]\n\
         pub extern \"C\" fn {subscript_typegpu}({params}){ret_sig} {{\n\
         \x20   if {recv}.is_null() {{\n{null_ret}\x20   }}\n\
         {guards}\
         {conversions}\
         \x20   // SAFETY: non-null handle owned by the caller.\n\
         {call}}}\n",
        wgpu = op.wgpu_fn,
        subscript_typegpu = op.subscript_typegpu_fn,
        params = params.join(", "),
        null_ret = null_return(&op.ret),
    )
}

#[cfg(test)]
mod tests {
    use super::{rust_create_export, rust_instance_backend_types};
    use crate::plan::CreateOp;

    #[test]
    fn instance_backend_types_pin_the_yawgpu_extension() {
        let source = rust_instance_backend_types("PinnedInstanceDescriptor");
        for expected in [
            "yawgpu/blob/main/ffi/webgpu-headers/yawgpu.h",
            "struct PinnedInstanceDescriptor",
            "YAWGPU_STYPE_INSTANCE_BACKEND_SELECT: i32 = 0x7000_0001",
            "YAWGPU_INSTANCE_BACKEND_NOOP: u32 = 0",
            "YAWGPU_INSTANCE_BACKEND_METAL: u32 = 1",
            "YAWGPU_INSTANCE_BACKEND_VULKAN: u32 = 2",
            "YAWGPU_INSTANCE_BACKEND_GLES: u32 = 3",
        ] {
            assert!(source.contains(expected), "missing `{expected}`");
        }
    }

    #[test]
    fn instance_create_reads_and_reports_the_backend_request() {
        let source = rust_create_export(&CreateOp {
            wgpu_fn: "wgpuCreateInstance".to_owned(),
            subscript_typegpu_fn: "subscript_typegpu_create_instance".to_owned(),
            returns_object: "instance".to_owned(),
            dropped_arg: Some((
                "descriptor".to_owned(),
                "PinnedInstanceDescriptor".to_owned(),
            )),
            doc: None,
        });
        for expected in [
            "var_os(\"SUBSCRIPT_TYPEGPU_BACKEND\")",
            "Some(\"metal\")",
            "Some(\"vulkan\")",
            "Some(\"gles\")",
            "PinnedInstanceDescriptor",
            "unknown SUBSCRIPT_TYPEGPU_BACKEND value",
            "backend request `{request}` returned a null instance from {path}",
        ] {
            assert!(
                source.contains(expected),
                "missing `{expected}` in:\n{source}"
            );
        }
        assert!(!source.contains("set_var"));
    }
}
