//! F10 string-view boundary support shared by descriptor fields and
//! label parameters.

/// Public C string-view declaration.
pub(crate) fn c_string_view() -> &'static str {
    "typedef struct SubscriptTypegpuStringView {\n    const char* data;\n    size_t length;\n} SubscriptTypegpuStringView;"
}

/// Public facade and private backend Rust string-view declarations.
pub(crate) fn rust_string_views() -> &'static str {
    "/// `subscript-typegpu.h`: borrowed UTF-8 string view.\n\
     #[repr(C)]\n\
     #[derive(Clone, Copy)]\n\
     pub struct SubscriptTypegpuStringView {\n\
     \x20   /// Pointer to UTF-8 bytes.\n\
     \x20   pub data: *const c_char,\n\
     \x20   /// Byte length, or `usize::MAX` for null-terminated input.\n\
     \x20   pub length: usize,\n\
     }\n\
     \n\
     /// webgpu.h `WGPUStringView`.\n\
     #[repr(C)]\n\
     #[derive(Clone, Copy)]\n\
     struct WGPUStringView {\n\
     \x20   data: *const c_char,\n\
     \x20   length: usize,\n\
     }\n\
     \n\
     fn wgpu_string_view(view: SubscriptTypegpuStringView) -> WGPUStringView {\n\
     \x20   WGPUStringView {\n\
     \x20       data: view.data,\n\
     \x20       length: view.length,\n\
     \x20   }\n\
     }\n"
}
