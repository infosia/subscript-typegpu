//! F10 input string-view method pattern.

use crate::naming;
use crate::plan::LabelOp;

/// Renders the label setter declaration for `subscript-typegpu.h`.
///
/// The string crosses as one `SubscriptTypegpuStringView` parameter (F10).
pub(crate) fn c_decl(op: &LabelOp) -> String {
    format!(
        "void {}({} {}, SubscriptTypegpuStringView {});",
        op.subscript_typegpu_fn,
        naming::subscript_typegpu_type(&op.receiver),
        naming::camel(&op.receiver),
        op.param,
    )
}

/// Renders the private webgpu.h declaration of the label setter.
///
/// The line joins the declaration text that becomes one function-table field and one shim.
pub(crate) fn rust_extern(op: &LabelOp) -> String {
    format!(
        "    fn {}({}: {}, {}: WGPUStringView);\n",
        op.wgpu_fn,
        naming::camel(&op.receiver),
        naming::wgpu_type(&op.receiver),
        op.param,
    )
}

/// Renders the exported label setter body.
///
/// A null receiver returns without a backend call (L9). The backend borrows the view only for
/// the duration of the call, so the export copies nothing.
pub(crate) fn rust_export(op: &LabelOp) -> String {
    let recv = naming::camel(&op.receiver);
    format!(
        "/// `subscript-typegpu.h`: forwards a borrowed label string view.\n\
         #[no_mangle]\n\
         pub extern \"C\" fn {subscript_typegpu_fn}({recv}: {recv_ty}, {param}: SubscriptTypegpuStringView) {{\n\
         \x20   if {recv}.is_null() {{\n\
         \x20       return;\n\
         \x20   }}\n\
         \x20   // SAFETY: the receiver is non-null and the input view is borrowed\n\
         \x20   // only for this call.\n\
         \x20   unsafe {{ {wgpu_fn}({recv}.cast(), wgpu_string_view({param})) }}\n\
         }}\n",
        subscript_typegpu_fn = op.subscript_typegpu_fn,
        recv_ty = naming::subscript_typegpu_type(&op.receiver),
        param = op.param,
        wgpu_fn = op.wgpu_fn,
    )
}
