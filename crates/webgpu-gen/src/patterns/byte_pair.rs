//! F20 count-first `u8` pairs keep count and pointer in generated ABI order.
//! Raw pointers never cross subscript-typegpu.h.

use crate::naming;
use crate::patterns::rust_signature;
use crate::plan::{ByteArg, BytePairOp};

fn public_pair(op: &BytePairOp) -> (&'static str, &'static str, &'static str) {
    if op.mutable {
        ("outCount", "uint8_t*", "out")
    } else {
        ("dataCount", "const uint8_t*", "data")
    }
}

pub(crate) fn c_decl(op: &BytePairOp) -> String {
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
                naming::camel(name),
            )),
            ByteArg::Scalar(name, scalar) => {
                params.push(format!("{} {}", scalar.c_name(), naming::camel(name)))
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

pub(crate) fn rust_extern(op: &BytePairOp) -> String {
    let mut params = vec![format!(
        "{}: {}",
        naming::camel(&op.receiver),
        naming::wgpu_type(&op.receiver),
    )];
    for arg in &op.args {
        match arg {
            ByteArg::Object(name, object) => params.push(format!(
                "{}: {}",
                naming::camel(name),
                naming::wgpu_type(object),
            )),
            ByteArg::Scalar(name, scalar) => {
                params.push(format!("{}: {}", naming::camel(name), scalar.rust_name()))
            }
        }
    }
    params.push(format!(
        "data: {} c_void",
        if op.mutable { "*mut" } else { "*const" },
    ));
    params.push("size: usize".to_string());
    rust_signature(
        &format!("    fn {}", op.wgpu_fn),
        &params,
        if op.returns_status { " -> i32;" } else { ";" },
    ) + "\n"
}

pub(crate) fn rust_export(op: &BytePairOp) -> String {
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
                let name = naming::camel(name);
                params.push(format!(
                    "{name}: {}",
                    naming::subscript_typegpu_type(object)
                ));
                checks.push(format!("{name}.is_null()"));
                call_args.push(format!("{name}.cast()"));
            }
            ByteArg::Scalar(name, scalar) => {
                let name = naming::camel(name);
                params.push(format!("{name}: {}", scalar.rust_name()));
                call_args.push(name);
            }
        }
    }
    let (count, _, name) = public_pair(op);
    params.push(format!("{count}: usize"));
    params.push(format!(
        "{name}: {} u8",
        if op.mutable { "*mut" } else { "*const" },
    ));
    let ret_sig = if op.returns_status { " -> i32 {" } else { " {" };
    let sig = rust_signature(
        &format!("pub extern \"C\" fn {}", op.subscript_typegpu_fn),
        &params,
        ret_sig,
    );
    let null_return = if op.returns_status {
        format!("return {};", op.error_status)
    } else {
        "return;".to_string()
    };

    call_args.push(format!("{name}.cast()"));
    call_args.push(count.to_string());
    let call = rust_signature(&format!("        {}", op.wgpu_fn), &call_args, "");
    format!(
        "/// `subscript-typegpu.h`: forwards a count-first byte array (F20).\n\
         #[no_mangle]\n\
         {sig}\n\
         \x20   if {checks} || ({count} != 0 && {name}.is_null()) {{\n\
         \x20       {null_return}\n\
         \x20   }}\n\
         \x20   // SAFETY: handles are non-null; a non-empty array has a non-null\n\
         \x20   // pointer valid for `{count}` bytes for this call.\n\
         \x20   unsafe {{\n\
         {call}\n\
         \x20   }}\n\
         }}\n",
        checks = checks.join(" || "),
    )
}
