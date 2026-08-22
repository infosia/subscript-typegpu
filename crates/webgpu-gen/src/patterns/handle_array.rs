//! Count-first scalar/handle arrays with ordinary scalar/handle prefixes.

use crate::naming;
use crate::plan::{ArrayElement, ArrayOp, MethodArg};

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
        MethodArg::StructPointer { .. } => {
            unreachable!("array prefixes reject struct pointers")
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
        MethodArg::StructPointer { .. } => {
            unreachable!("array prefixes reject struct pointers")
        }
    }
}

fn c_element(element: &ArrayElement) -> String {
    match element {
        ArrayElement::Scalar(scalar) => scalar.c_name().into(),
        ArrayElement::Object(object) => naming::subscript_typegpu_type(object),
    }
}

fn rust_element(element: &ArrayElement, backend: bool) -> String {
    match element {
        ArrayElement::Scalar(scalar) => scalar.rust_name().into(),
        ArrayElement::Object(object) => {
            if backend {
                naming::wgpu_type(object)
            } else {
                naming::subscript_typegpu_type(object)
            }
        }
    }
}

pub(crate) fn c_decl(op: &ArrayOp) -> String {
    let mut params = vec![format!(
        "{} {}",
        naming::subscript_typegpu_type(&op.receiver),
        naming::camel(&op.receiver),
    )];
    params.extend(op.args.iter().map(c_arg));
    params.push(format!("size_t {}", op.public_count));
    params.push(format!("const {}* {}", c_element(&op.element), op.param));
    format!("void {}({});", op.subscript_typegpu_fn, params.join(", "))
}

pub(crate) fn rust_extern(op: &ArrayOp) -> String {
    let mut params = vec![format!(
        "{}: {}",
        naming::camel(&op.receiver),
        naming::wgpu_type(&op.receiver),
    )];
    params.extend(op.args.iter().map(|arg| {
        format!(
            "{}: {}",
            naming::camel(arg_name(arg)),
            rust_arg_type(arg, true)
        )
    }));
    params.push(format!("{}: usize", op.backend_count));
    params.push(format!(
        "{}: *const {}",
        op.param,
        rust_element(&op.element, true)
    ));
    format!("    fn {}({});\n", op.wgpu_fn, params.join(", "))
}

pub(crate) fn rust_export(op: &ArrayOp) -> String {
    let recv = naming::camel(&op.receiver);
    let mut params = vec![format!(
        "{recv}: {}",
        naming::subscript_typegpu_type(&op.receiver)
    )];
    let mut guards = String::new();
    let mut call_args = vec![format!("{recv}.cast()")];
    for arg in &op.args {
        let name = naming::camel(arg_name(arg));
        params.push(format!("{name}: {}", rust_arg_type(arg, false)));
        match arg {
            MethodArg::Object { nullable, .. } => {
                if !nullable {
                    guards.push_str(&format!(
                        "    if {name}.is_null() {{\n        return;\n    }}\n"
                    ));
                }
                call_args.push(format!("{name}.cast()"));
            }
            MethodArg::Scalar(..) | MethodArg::Bitflag(..) | MethodArg::Enum(..) => {
                call_args.push(name);
            }
            MethodArg::StructPointer { .. } => {
                unreachable!("array prefixes reject struct pointers")
            }
        }
    }
    params.push(format!("{}_count: usize", op.param));
    params.push(format!(
        "{}: *const {}",
        op.param,
        rust_element(&op.element, false)
    ));
    call_args.push(format!("{}_count", op.param));
    call_args.push(match op.element {
        ArrayElement::Object(_) => format!("{}.cast()", op.param),
        ArrayElement::Scalar(_) => op.param.clone(),
    });
    format!(
        "/// `subscript-typegpu.h`: forwards a count-first input array.\n\
         #[no_mangle]\n\
         pub extern \"C\" fn {subscript_typegpu_fn}({params}) {{\n\
         \x20   if {recv}.is_null() {{\n\
         \x20       return;\n\
         \x20   }}\n\
         {guards}\
         \x20   if {param}_count != 0 && {param}.is_null() {{\n\
         \x20       return;\n\
         \x20   }}\n\
         \x20   // SAFETY: non-null receiver and the pair promises `count` readable elements.\n\
         \x20   unsafe {{ {wgpu_fn}({call_args}) }}\n\
         }}\n",
        subscript_typegpu_fn = op.subscript_typegpu_fn,
        params = params.join(", "),
        param = op.param,
        wgpu_fn = op.wgpu_fn,
        call_args = call_args.join(", "),
    )
}
