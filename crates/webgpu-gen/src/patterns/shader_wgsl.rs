//! F12 WGSL chain flattening: public `{ label, code }`, private
//! `WGPUShaderSourceWGSL` extension chain.

use crate::naming;
use crate::patterns::rust_signature;
use crate::plan::ShaderWgslOp;

pub(crate) fn c_struct(op: &ShaderWgslOp) -> String {
    format!(
        "typedef struct {ty} {{\n    SubscriptTypegpuStringView label;\n    SubscriptTypegpuStringView code;\n}} {ty};",
        ty = op.descriptor_subscript_typegpu,
    )
}

pub(crate) fn c_decl(op: &ShaderWgslOp) -> String {
    format!(
        "{} {}({} {}, const {}* descriptor);",
        naming::subscript_typegpu_type(&op.returns_object),
        op.subscript_typegpu_fn,
        naming::subscript_typegpu_type(&op.receiver),
        naming::camel(&op.receiver),
        op.descriptor_subscript_typegpu,
    )
}

pub(crate) fn rust_private_types(op: &ShaderWgslOp) -> String {
    format!(
        "/// webgpu.h `WGPUChainedStruct`; concrete for WGSL source construction.\n#[repr(C)]\n#[derive(Clone, Copy)]\nstruct WGPUChainedStruct {{\n    next: *mut WGPUChainedStruct,\n    s_type: i32,\n}}\n\n/// webgpu.h `{extension}`.\n#[repr(C)]\nstruct {extension} {{\n    chain: WGPUChainedStruct,\n    code: WGPUStringView,\n}}\n\n/// webgpu.h `{descriptor}`.\n#[repr(C)]\nstruct {descriptor} {{\n    next_in_chain: *mut WGPUChainedStruct,\n    label: WGPUStringView,\n}}\n\n/// webgpu.yml `s_type.shader_source_WGSL`.\nconst {s_type}: i32 = {value};\n",
        extension = op.extension_wgpu,
        descriptor = op.descriptor_wgpu,
        s_type = op.s_type_const,
        value = naming::hex_enum(op.s_type_value),
    )
}

pub(crate) fn rust_public_type(op: &ShaderWgslOp) -> String {
    format!(
        "/// `subscript-typegpu.h`: WGSL shader module descriptor with its source chain flattened.\n#[repr(C)]\n#[derive(Clone, Copy)]\npub struct {descriptor} {{\n    /// Shader module label.\n    pub label: SubscriptTypegpuStringView,\n    /// WGSL source text.\n    pub code: SubscriptTypegpuStringView,\n}}\n",
        descriptor = op.descriptor_subscript_typegpu,
    )
}

pub(crate) fn rust_extern(op: &ShaderWgslOp) -> String {
    format!(
        "    fn {}({}: {}, descriptor: *const {}) -> {};\n",
        op.wgpu_fn,
        naming::camel(&op.receiver),
        naming::wgpu_type(&op.receiver),
        op.descriptor_wgpu,
        naming::wgpu_type(&op.returns_object),
    )
}

pub(crate) fn rust_export(op: &ShaderWgslOp) -> String {
    let recv = naming::camel(&op.receiver);
    let sig = rust_signature(
        &format!("pub extern \"C\" fn {}", op.subscript_typegpu_fn),
        &[
            format!("{recv}: {}", naming::subscript_typegpu_type(&op.receiver)),
            format!("descriptor: *const {}", op.descriptor_subscript_typegpu),
        ],
        &format!(
            " -> {} {{",
            naming::subscript_typegpu_type(&op.returns_object)
        ),
    );
    format!(
        "/// `subscript-typegpu.h`: creates a WGSL shader module through a private source chain.\n#[no_mangle]\n{sig}\n    if {recv}.is_null() || descriptor.is_null() {{\n        return std::ptr::null_mut();\n    }}\n    // SAFETY: the caller supplies a live descriptor for this call.\n    let source = unsafe {{ *descriptor }};\n    let wgsl = {extension} {{\n        chain: WGPUChainedStruct {{\n            next: std::ptr::null_mut(),\n            s_type: {s_type},\n        }},\n        code: wgpu_string_view(source.code),\n    }};\n    let descriptor = {descriptor_wgpu} {{\n        next_in_chain: (&wgsl.chain as *const WGPUChainedStruct).cast_mut(),\n        label: wgpu_string_view(source.label),\n    }};\n    // SAFETY: receiver and descriptor are non-null; the WGSL chain lives\n    // through the backend call.\n    let created = unsafe {{ {wgpu}({recv}.cast(), &descriptor).cast() }};\n    runtime::inherit_handle_instance({recv} as usize, created as usize);\n    created\n}}\n",
        extension = op.extension_wgpu,
        s_type = op.s_type_const,
        descriptor_wgpu = op.descriptor_wgpu,
        wgpu = op.wgpu_fn,
    )
}
