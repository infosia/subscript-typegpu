//! B3 queue texture upload: three chain-free struct pointers plus the
//! public F20 count-first byte pair.

use crate::naming;
use crate::patterns::rust_signature;
use crate::plan::{StructPlan, WriteTextureOp};

pub(crate) fn c_decl(
    op: &WriteTextureOp,
    destination: &StructPlan,
    layout: &StructPlan,
    extent: &StructPlan,
) -> String {
    format!(
        "void {}({} {}, const {}* dst, const {}* layout, const {}* extent, size_t dataCount, const uint8_t* data);",
        op.subscript_typegpu_fn,
        naming::subscript_typegpu_type(&op.receiver),
        naming::camel(&op.receiver),
        destination.subscript_typegpu_struct,
        layout.subscript_typegpu_struct,
        extent.subscript_typegpu_struct,
    )
}

pub(crate) fn rust_extern(
    op: &WriteTextureOp,
    destination: &StructPlan,
    layout: &StructPlan,
    extent: &StructPlan,
) -> String {
    rust_signature(
        &format!("    fn {}", op.wgpu_fn),
        &[
            format!(
                "{}: {}",
                naming::camel(&op.receiver),
                naming::wgpu_type(&op.receiver),
            ),
            format!("destination: *const {}", destination.wgpu_struct),
            "data: *const std::ffi::c_void".into(),
            "data_size: usize".into(),
            format!("data_layout: *const {}", layout.wgpu_struct),
            format!("write_size: *const {}", extent.wgpu_struct),
        ],
        ";",
    ) + "\n"
}

pub(crate) fn rust_export(
    op: &WriteTextureOp,
    destination: &StructPlan,
    layout: &StructPlan,
    extent: &StructPlan,
) -> String {
    let recv = naming::camel(&op.receiver);
    let sig = rust_signature(
        &format!("pub extern \"C\" fn {}", op.subscript_typegpu_fn),
        &[
            format!("{recv}: {}", naming::subscript_typegpu_type(&op.receiver)),
            format!("dst: *const {}", destination.subscript_typegpu_struct),
            format!("layout: *const {}", layout.subscript_typegpu_struct),
            format!("extent: *const {}", extent.subscript_typegpu_struct),
            "dataCount: usize".into(),
            "data: *const u8".into(),
        ],
        " {",
    );
    format!(
        "/// `subscript-typegpu.h`: uploads a texture region with a count-first byte array.\n\
         #[no_mangle]\n\
         {sig}\n\
         \x20   if {recv}.is_null() || dst.is_null() || layout.is_null() || extent.is_null()\n\
         \x20       || (dataCount != 0 && data.is_null()) {{\n\
         \x20       return;\n\
         \x20   }}\n\
         \x20   // SAFETY: public pointer checks above establish live input structs.\n\
         \x20   let dst = convert_{destination_source}(unsafe {{ *dst }});\n\
         \x20   // SAFETY: as above.\n\
         \x20   let layout = convert_{layout_source}(unsafe {{ *layout }});\n\
         \x20   // SAFETY: as above.\n\
         \x20   let extent = convert_{extent_source}(unsafe {{ *extent }});\n\
         \x20   // SAFETY: converted structs outlive the call; a non-empty byte\n\
         \x20   // array has a non-null pointer valid for `dataCount` bytes.\n\
         \x20   unsafe {{\n\
         \x20       {wgpu_fn}(\n\
         \x20           {recv}.cast(),\n\
         \x20           &dst,\n\
         \x20           data.cast(),\n\
         \x20           dataCount,\n\
         \x20           &layout,\n\
         \x20           &extent,\n\
         \x20       );\n\
         \x20   }}\n\
         }}\n",
        destination_source = destination.source,
        layout_source = layout.source,
        extent_source = extent.source,
        wgpu_fn = op.wgpu_fn,
    )
}
