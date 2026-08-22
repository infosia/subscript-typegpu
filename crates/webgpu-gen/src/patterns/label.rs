//! F10 input string-view method pattern.

use crate::naming;
use crate::plan::LabelOp;

pub(crate) fn c_decl(op: &LabelOp) -> String {
    format!(
        "void {}({} {}, SubscriptTypegpuStringView {});",
        op.subscript_typegpu_fn,
        naming::subscript_typegpu_type(&op.receiver),
        naming::camel(&op.receiver),
        op.param,
    )
}

pub(crate) fn rust_extern(op: &LabelOp) -> String {
    format!(
        "    fn {}({}: {}, {}: WGPUStringView);\n",
        op.wgpu_fn,
        naming::camel(&op.receiver),
        naming::wgpu_type(&op.receiver),
        op.param,
    )
}

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
