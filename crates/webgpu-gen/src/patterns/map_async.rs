//! A3/F15 mapAsync range and whole-resource variants.

use crate::naming;
use crate::patterns::rust_signature;
use crate::plan::MapAsyncOp;

pub(crate) fn c_decls(op: &MapAsyncOp) -> Vec<String> {
    let recv_ty = naming::subscript_typegpu_type(&op.async_op.receiver);
    let recv = naming::camel(&op.async_op.receiver);
    vec![
        format!(
            "SubscriptTypegpuFutureId {}({recv_ty} {recv}, SubscriptTypegpuMapMode mode, size_t offset, size_t size);",
            op.async_op.subscript_typegpu_fn,
        ),
        format!(
            "SubscriptTypegpuFutureId {}({recv_ty} {recv}, SubscriptTypegpuMapMode mode);",
            op.whole_subscript_typegpu_fn,
        ),
    ]
}

pub(crate) fn rust_extern(op: &MapAsyncOp) -> String {
    rust_signature(
        &format!("    fn {}", op.async_op.wgpu_fn),
        &[
            format!(
                "{}: {}",
                naming::camel(&op.async_op.receiver),
                naming::wgpu_type(&op.async_op.receiver),
            ),
            "mode: u64".to_string(),
            "offset: usize".to_string(),
            "size: usize".to_string(),
            format!("callback_info: {}", op.async_op.cb.cb_info),
        ],
        " -> WGPUFuture;",
    ) + "\n"
}

fn rust_one_export(op: &MapAsyncOp, whole: bool, mode_const: &str) -> String {
    let async_op = &op.async_op;
    let recv = naming::camel(&async_op.receiver);
    let mut params = vec![
        format!(
            "{recv}: {}",
            naming::subscript_typegpu_type(&async_op.receiver)
        ),
        "mode: u64".to_string(),
    ];
    let (name, offset, size) = if whole {
        (op.whole_subscript_typegpu_fn.as_str(), "0", "usize::MAX")
    } else {
        params.push("offset: usize".to_string());
        params.push("size: usize".to_string());
        (async_op.subscript_typegpu_fn.as_str(), "offset", "size")
    };
    let sig = rust_signature(
        &format!("pub extern \"C\" fn {name}"),
        &params,
        " -> SubscriptTypegpuFutureId {",
    );
    format!(
        "/// `subscript-typegpu.h`: begins a buffer map request; poll after pumping.\n\
         #[no_mangle]\n\
         {sig}\n\
         \x20   if {recv}.is_null() {{\n\
         \x20       return 0;\n\
         \x20   }}\n\
         \x20   let instance = runtime::instance_for_handle({recv} as usize);\n\
         \x20   if instance == 0 {{\n\
         \x20       return 0;\n\
         \x20   }}\n\
         \x20   let (id, userdata1) = runtime::new_pending_slot(instance, {kind});\n\
         \x20   let info = {info} {{\n\
         \x20       next_in_chain: std::ptr::null_mut(),\n\
         \x20       mode: {mode_const},\n\
         \x20       callback: Some({callback}),\n\
         \x20       userdata1,\n\
         \x20       userdata2: std::ptr::null_mut(),\n\
         \x20   }};\n\
         \x20   // SAFETY: the receiver is non-null; callback userdata stays live\n\
         \x20   // until completion or instance release.\n\
         \x20   let _ = unsafe {{ {wgpu_fn}({recv}.cast(), mode, {offset}, {size}, info) }};\n\
         \x20   id\n\
         }}\n",
        kind = async_op.kind_const,
        info = async_op.cb.cb_info,
        callback = async_op.cb.rust_fn,
        wgpu_fn = async_op.wgpu_fn,
    )
}

pub(crate) fn rust_exports(op: &MapAsyncOp, mode_const: &str) -> String {
    format!(
        "{}\n{}",
        rust_one_export(op, false, mode_const),
        rust_one_export(op, true, mode_const),
    )
}
