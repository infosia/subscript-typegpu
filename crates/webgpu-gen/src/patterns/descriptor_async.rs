//! Descriptor-carrying F6 future-poll request pattern.

use crate::naming;
use crate::patterns::rust_signature;
use crate::plan::{DescriptorAsyncOp, StructPlan};

pub(crate) fn c_request_decl(op: &DescriptorAsyncOp, shape: &StructPlan, anchor: &str) -> String {
    let async_op = &op.async_op;
    format!(
        "SubscriptTypegpuFutureId {}({} {}, {} {}, const {}* descriptor);",
        async_op.subscript_typegpu_fn,
        naming::subscript_typegpu_type(anchor),
        naming::camel(anchor),
        naming::subscript_typegpu_type(&async_op.receiver),
        naming::camel(&async_op.receiver),
        shape.subscript_typegpu_struct,
    )
}

pub(crate) fn rust_extern(op: &DescriptorAsyncOp, shape: &StructPlan) -> String {
    let async_op = &op.async_op;
    format!(
        "    fn {}({}: {}, descriptor: *const {}, callback_info: {}) -> WGPUFuture;\n",
        async_op.wgpu_fn,
        naming::camel(&async_op.receiver),
        naming::wgpu_type(&async_op.receiver),
        shape.wgpu_struct,
        async_op.cb.cb_info,
    )
}

pub(crate) fn rust_export(
    op: &DescriptorAsyncOp,
    shape: &StructPlan,
    anchor: &str,
    mode_const: &str,
) -> String {
    let async_op = &op.async_op;
    let recv = naming::camel(&async_op.receiver);
    let instance = naming::camel(anchor);
    let sig = rust_signature(
        &format!("pub extern \"C\" fn {}", async_op.subscript_typegpu_fn),
        &[
            format!("{instance}: {}", naming::subscript_typegpu_type(anchor)),
            format!(
                "{recv}: {}",
                naming::subscript_typegpu_type(&async_op.receiver)
            ),
            format!("descriptor: *const {}", shape.subscript_typegpu_struct),
        ],
        " -> SubscriptTypegpuFutureId {",
    );
    let descriptor_ref = if shape.owns_storage {
        "&descriptor.value"
    } else {
        "&descriptor"
    };
    format!(
        "/// `subscript-typegpu.h`: begins descriptor-backed `{wgpu}`; poll after pumping.\n#[no_mangle]\n{sig}\n    if {instance}.is_null() || {recv}.is_null() || descriptor.is_null() {{\n        return 0;\n    }}\n    // SAFETY: the caller supplies a live descriptor for this call.\n    let source = unsafe {{ *descriptor }};\n    let descriptor = convert_{source}(source);\n    let (id, userdata1) = runtime::new_pending_slot({instance} as usize, {kind});\n    let info = {info} {{\n        next_in_chain: std::ptr::null_mut(),\n        mode: {mode_const},\n        callback: Some({callback}),\n        userdata1,\n        userdata2: std::ptr::null_mut(),\n    }};\n    // SAFETY: handles and descriptor are non-null, converted storage lives\n    // through the backend request call, and callback userdata stays live.\n    let _ = unsafe {{ {wgpu}({recv}.cast(), {descriptor_ref}, info) }};\n    id\n}}\n",
        wgpu = async_op.wgpu_fn,
        source = shape.source,
        kind = async_op.kind_const,
        info = async_op.cb.cb_info,
        callback = async_op.cb.rust_fn,
    )
}
