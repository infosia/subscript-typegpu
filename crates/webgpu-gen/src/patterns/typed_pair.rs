//! S3 float siblings derived from F20 byte-pair methods.
//! Public counts name float elements. Backend sizes name bytes.

use crate::naming;
use crate::patterns::rust_signature;
use crate::plan::{ByteArg, TypedPairOp};

/// The public count name, the element pointer type, and the pointer name of one float pair.
///
/// A mutable pair spells `outCount` and `out`, an immutable one `dataCount` and `data`. The
/// element type is `float`, so the count counts elements and never bytes (S3).
fn public_pair(op: &TypedPairOp) -> (&'static str, &'static str, &'static str) {
    if op.mutable {
        ("outCount", "float*", "out")
    } else {
        ("dataCount", "const float*", "data")
    }
}

/// The public spelling of one argument name, with a `Bytes` suffix on the offset parameter.
///
/// The float count and the byte offset sit in one parameter list. The suffix keeps the two
/// units apart (S3).
fn public_arg_name(op: &TypedPairOp, name: &str) -> String {
    let name = naming::camel(name);
    if name == naming::camel(&op.offset_param) {
        format!("{name}Bytes")
    } else {
        name
    }
}

/// Renders the `subscript-typegpu.h` comment that names the unit of each pair parameter.
///
/// The offset counts bytes. The count counts float elements (S3).
pub(crate) fn c_comment(op: &TypedPairOp) -> String {
    let (count, _, _) = public_pair(op);
    format!(
        "/* {} counts bytes; {count} counts float elements. */",
        public_arg_name(op, &op.offset_param)
    )
}

/// Renders the float sibling declaration for `subscript-typegpu.h`.
///
/// The trailing pair carries a `float` pointer. The byte-offset parameter of the source method
/// gains a `Bytes` suffix, so the two units never share a spelling.
pub(crate) fn c_decl(op: &TypedPairOp) -> String {
    let mut params = vec![format!(
        "{} {}",
        naming::subscript_typegpu_type(&op.receiver),
        naming::camel(&op.receiver),
    )];
    for arg in &op.args {
        match arg {
            ByteArg::Object(name, object) => params.push(format!(
                "{} {}",
                naming::subscript_typegpu_type(object),
                public_arg_name(op, name),
            )),
            ByteArg::Scalar(name, scalar) => {
                params.push(format!("{} {}", scalar.c_name(), public_arg_name(op, name)))
            }
        }
    }
    let (count, pointer, name) = public_pair(op);
    params.push(format!("size_t {count}"));
    params.push(format!("{pointer} {name}"));
    format!(
        "{} {}({});",
        if op.returns_status { "int32_t" } else { "void" },
        op.subscript_typegpu_fn,
        params.join(", "),
    )
}

/// Renders the exported float sibling body.
///
/// The body multiplies the element count by the size of `f32` and calls the same webgpu.h
/// function as the byte form (S3). An overflow of that product returns the error status
/// without a backend call, as a null argument does (L9).
pub(crate) fn rust_export(op: &TypedPairOp) -> String {
    let recv = naming::camel(&op.receiver);
    let mut params = vec![format!(
        "{recv}: {}",
        naming::subscript_typegpu_type(&op.receiver)
    )];
    let mut checks = vec![format!("{recv}.is_null()")];
    let mut call_args = vec![format!("{recv}.cast()")];
    for arg in &op.args {
        match arg {
            ByteArg::Object(name, object) => {
                let name = public_arg_name(op, name);
                params.push(format!(
                    "{name}: {}",
                    naming::subscript_typegpu_type(object)
                ));
                checks.push(format!("{name}.is_null()"));
                call_args.push(format!("{name}.cast()"));
            }
            ByteArg::Scalar(name, scalar) => {
                let name = public_arg_name(op, name);
                params.push(format!("{name}: {}", scalar.rust_name()));
                call_args.push(name);
            }
        }
    }
    let (count, _, name) = public_pair(op);
    params.push(format!("{count}: usize"));
    params.push(format!(
        "{name}: {} f32",
        if op.mutable { "*mut" } else { "*const" },
    ));
    let ret_sig = if op.returns_status { " -> i32 {" } else { " {" };
    let sig = rust_signature(
        &format!("pub extern \"C\" fn {}", op.subscript_typegpu_fn),
        &params,
        ret_sig,
    );
    let invalid_return = if op.returns_status {
        format!("return {};", op.error_status)
    } else {
        "return;".to_owned()
    };
    call_args.push(format!("{name}.cast()"));
    call_args.push("byteCount".to_owned());
    let call = rust_signature(&format!("        {}", op.wgpu_fn), &call_args, "");
    format!(
        "/// `subscript-typegpu.h`: `{offset}` counts bytes; `{count}` counts f32 elements.\n\
         #[no_mangle]\n\
         {sig}\n\
         \x20   if {checks} || ({count} != 0 && {name}.is_null()) {{\n\
         \x20       {invalid_return}\n\
         \x20   }}\n\
         \x20   let Some(byteCount) = {count}.checked_mul(std::mem::size_of::<f32>()) else {{\n\
         \x20       {invalid_return}\n\
         \x20   }};\n\
         \x20   // SAFETY: handles are non-null; a non-empty array has a non-null\n\
         \x20   // pointer valid for `{count}` f32 elements, or `byteCount` bytes.\n\
         \x20   unsafe {{\n\
         {call}\n\
         \x20   }}\n\
         }}\n",
        offset = public_arg_name(op, &op.offset_param),
        checks = checks.join(" || "),
    )
}
