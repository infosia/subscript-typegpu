//! Opaque-handle pattern (F4): subset objects become opaque typedefs
//! in `subscript-typegpu.h`, `#[repr(C)]` zero-sized types plus pointer aliases in
//! Rust, and Release-only wrappers (AddRef is policy-excluded).

use crate::naming;

/// `typedef struct SubscriptTypegpuXxxImpl* SubscriptTypegpuXxx;`
pub(crate) fn c_typedef(object: &str) -> String {
    let ty = naming::subscript_typegpu_type(object);
    format!("typedef struct {ty}Impl* {ty};")
}

/// `void subscript_typegpu_xxx_release(SubscriptTypegpuXxx xxx);`
pub(crate) fn c_release_decl(object: &str) -> String {
    let ty = naming::subscript_typegpu_type(object);
    format!(
        "void subscript_typegpu_{}_release({ty} {});",
        object,
        naming::camel(object)
    )
}

/// The `opaque!` macro shared by both opaque blocks.
pub(crate) fn rust_opaque_macro() -> &'static str {
    "/// Declares opaque handle/pointer-only types (never dereferenced\n\
     /// here). The caller supplies the visibility: the WGPU set stays\n\
     /// `pub(crate)` (webgpu.h internals never enter the crate's public\n\
     /// API); only the `SubscriptTypegpu*` set is `pub`.\n\
     macro_rules! opaque {\n\
     \x20   ($vis:vis $($name:ident),* $(,)?) => {\n\
     \x20       $(\n\
     \x20           #[repr(C)]\n\
     \x20           #[doc = \"Opaque handle type (never dereferenced here).\"]\n\
     \x20           $vis struct $name {\n\
     \x20               _private: [u8; 0],\n\
     \x20           }\n\
     \x20       )*\n\
     \x20   };\n\
     }\n"
}

/// The `opaque!(pub(crate) ...)` block for the WGPU side, including
/// pointer-only types (`WGPUChainedStruct` when any emitted struct
/// carries `next_in_chain`, plus dropped descriptor types).
pub(crate) fn rust_wgpu_opaque_block(
    objects: &[String],
    needs_chain: bool,
    pointer_only: &[String],
) -> String {
    let mut out = String::from("opaque!(\n    pub(crate)\n");
    let mut names: Vec<String> = objects
        .iter()
        .map(|o| format!("{}Impl", naming::wgpu_type(o)))
        .collect();
    if needs_chain || !pointer_only.is_empty() {
        names.push("// Pointer-only in this subset (always null / never built):".into());
        if needs_chain {
            names.push("WGPUChainedStruct".into());
        }
        names.extend(pointer_only.iter().cloned());
    }
    for name in names {
        out.push_str(&format!(
            "    {name}{}\n",
            if name.starts_with("//") { "" } else { "," }
        ));
    }
    out.push_str(");\n");
    out
}

/// The `opaque!(pub ...)` block for the SubscriptTypegpu side.
pub(crate) fn rust_subscript_typegpu_opaque_block(objects: &[String]) -> String {
    let mut out = String::from("opaque!(\n    pub\n");
    for object in objects {
        out.push_str(&format!(
            "    {}Impl,\n",
            naming::subscript_typegpu_type(object)
        ));
    }
    out.push_str(");\n");
    out
}

/// `type WGPUXxx = *mut WGPUXxxImpl;` aliases.
pub(crate) fn rust_wgpu_aliases(objects: &[String]) -> String {
    objects
        .iter()
        .map(|o| {
            let ty = naming::wgpu_type(o);
            format!("type {ty} = *mut {ty}Impl;\n")
        })
        .collect()
}

/// `pub type SubscriptTypegpuXxx = *mut SubscriptTypegpuXxxImpl;` aliases with docs.
pub(crate) fn rust_subscript_typegpu_aliases(objects: &[String]) -> String {
    objects
        .iter()
        .map(|o| {
            let ty = naming::subscript_typegpu_type(o);
            format!(
                "/// `subscript-typegpu.h`: opaque {} handle.\npub type {ty} = *mut {ty}Impl;\n",
                o.replace('_', " ")
            )
        })
        .collect()
}

/// `fn wgpuXxxRelease(...)` extern declaration.
pub(crate) fn rust_release_extern(object: &str) -> String {
    let ty = naming::wgpu_type(object);
    format!(
        "    fn wgpu{}Release({}: {ty});\n",
        naming::pascal(object),
        naming::camel(object)
    )
}

/// The exported `subscript_typegpu_xxx_release` wrapper.
pub(crate) fn rust_release_export(object: &str, device_events: bool) -> String {
    let subscript_typegpu_ty = naming::subscript_typegpu_type(object);
    let pascal = naming::pascal(object);
    let snake = naming::snake(object);
    let param = naming::camel(object);
    let cleanup = match (device_events, object) {
        (true, "device") => "    runtime::release_device_events(device as usize);\n    runtime::release_adapter_info_strings(device as usize);\n",
        (true, "adapter") => {
            "    runtime::release_adapter_info_strings(adapter as usize);\n"
        }
        _ => "",
    };
    format!(
        "/// `subscript-typegpu.h`: releases the {word} handle.\n\
         #[no_mangle]\n\
         pub extern \"C\" fn subscript_typegpu_{snake}_release({param}: {subscript_typegpu_ty}) {{\n\
         \x20   if {param}.is_null() {{\n\
         \x20       return;\n\
         \x20   }}\n\
         \x20   // SAFETY: non-null handle owned by the caller.\n\
         \x20   unsafe {{ wgpu{pascal}Release({param}.cast()) }}\n\
         {cleanup}\
         }}\n",
        word = object.replace('_', " ")
    )
}
