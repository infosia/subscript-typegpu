//! Plain-sync pattern: freestanding create functions with dropped
//! optional descriptors, and ordinary handle/scalar/struct methods.

use crate::naming;
use crate::plan::{CreateOp, MethodArg, Scalar, SyncOp, SyncRet};

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
        (
            "    if !runtime::initialize_table() {\n        return std::ptr::null_mut();\n    }\n    // SAFETY: webgpu.h accepts a null instance descriptor.\n",
            format!(
                "    let instance = unsafe {{ {}(std::ptr::null()).cast() }};\n    runtime::register_instance(instance as usize);\n    instance\n",
                op.wgpu_fn
            ),
        )
    } else {
        match &op.dropped_arg {
            Some(_) => (
                "    // SAFETY: NULL descriptor is explicitly allowed by webgpu.h.\n",
                format!("    unsafe {{ {}(std::ptr::null()).cast() }}\n", op.wgpu_fn),
            ),
            None => (
                "    // SAFETY: no-argument creation.\n",
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
