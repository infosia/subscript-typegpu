//! F11/F13 adapter-info and limits fills plus scalar feature probes.

use crate::naming;
use crate::plan::{AdapterInfoOp, FeatureOp, LimitsOp, StructPlan};

pub(crate) fn c_adapter_info() -> &'static str {
    "typedef struct SubscriptTypegpuAdapterInfo {\n\
     \x20   SubscriptTypegpuStringView vendor;\n\
     \x20   SubscriptTypegpuStringView architecture;\n\
     \x20   SubscriptTypegpuStringView device;\n\
     \x20   SubscriptTypegpuStringView description;\n\
     \x20   SubscriptTypegpuBackendType backendType;\n\
     \x20   SubscriptTypegpuAdapterType adapterType;\n\
     \x20   uint32_t vendorID;\n\
     \x20   uint32_t deviceID;\n\
     } SubscriptTypegpuAdapterInfo;"
}

pub(crate) fn c_device_descriptor() -> &'static str {
    "typedef struct SubscriptTypegpuDeviceDescriptor {\n\
     \x20   SubscriptTypegpuStringView label;\n\
     \x20   size_t requiredFeaturesCount;\n\
     \x20   const SubscriptTypegpuFeatureName* requiredFeatures;\n\
     \x20   const SubscriptTypegpuLimits* requiredLimits;\n\
     \x20   SubscriptTypegpuQueueDescriptor defaultQueue;\n\
     } SubscriptTypegpuDeviceDescriptor;"
}

pub(crate) fn c_limits_decl(op: &LimitsOp, shape: &StructPlan) -> String {
    let mut params = Vec::new();
    if let Some(receiver) = &op.receiver {
        params.push(format!(
            "{} {}",
            naming::subscript_typegpu_type(receiver),
            naming::camel(receiver)
        ));
    }
    params.push(format!("{}* out", shape.subscript_typegpu_struct));
    format!(
        "int32_t {}({});",
        op.subscript_typegpu_fn,
        params.join(", ")
    )
}

pub(crate) fn c_info_decl(op: &AdapterInfoOp) -> String {
    format!(
        "bool {}({} {}, SubscriptTypegpuAdapterInfo* out);",
        op.subscript_typegpu_fn,
        naming::subscript_typegpu_type(&op.receiver),
        naming::camel(&op.receiver),
    )
}

pub(crate) fn c_feature_decl(op: &FeatureOp) -> String {
    let mut params = Vec::new();
    if let Some(receiver) = &op.receiver {
        params.push(format!(
            "{} {}",
            naming::subscript_typegpu_type(receiver),
            naming::camel(receiver)
        ));
    }
    params.push(format!(
        "{} feature",
        naming::subscript_typegpu_type(&op.enum_name)
    ));
    format!("bool {}({});", op.subscript_typegpu_fn, params.join(", "))
}

pub(crate) fn rust_adapter_info_private() -> &'static str {
    "/// webgpu.h `WGPUAdapterInfo`; the two subgroup fields remain private.\n\
     #[repr(C)]\n\
     struct WGPUAdapterInfo {\n\
     \x20   next_in_chain: *mut WGPUChainedStruct,\n\
     \x20   vendor: WGPUStringView,\n\
     \x20   architecture: WGPUStringView,\n\
     \x20   device: WGPUStringView,\n\
     \x20   description: WGPUStringView,\n\
     \x20   backend_type: i32,\n\
     \x20   adapter_type: i32,\n\
     \x20   vendor_id: u32,\n\
     \x20   device_id: u32,\n\
     \x20   subgroup_min_size: u32,\n\
     \x20   subgroup_max_size: u32,\n\
     }\n"
}

pub(crate) fn rust_adapter_info_public() -> &'static str {
    "/// `subscript-typegpu.h`: facade-filled adapter information.\n\
     #[repr(C)]\n\
     #[derive(Clone, Copy)]\n\
     pub struct SubscriptTypegpuAdapterInfo {\n\
     \x20   /// Facade-owned vendor string.\n\
     \x20   pub vendor: SubscriptTypegpuStringView,\n\
     \x20   /// Facade-owned architecture string.\n\
     \x20   pub architecture: SubscriptTypegpuStringView,\n\
     \x20   /// Facade-owned device string.\n\
     \x20   pub device: SubscriptTypegpuStringView,\n\
     \x20   /// Facade-owned description string.\n\
     \x20   pub description: SubscriptTypegpuStringView,\n\
     \x20   /// Pinned backend-type enum value.\n\
     \x20   pub backend_type: SubscriptTypegpuBackendType,\n\
     \x20   /// Pinned adapter-type enum value.\n\
     \x20   pub adapter_type: SubscriptTypegpuAdapterType,\n\
     \x20   /// PCI vendor identifier when reported.\n\
     \x20   pub vendor_id: u32,\n\
     \x20   /// PCI device identifier when reported.\n\
     \x20   pub device_id: u32,\n\
     }\n"
}

pub(crate) fn rust_device_descriptor_public() -> &'static str {
    "/// `subscript-typegpu.h`: request-device descriptor without callback fields.\n\
     #[repr(C)]\n\
     #[derive(Clone, Copy)]\n\
     pub struct SubscriptTypegpuDeviceDescriptor {\n\
     \x20   /// Debug label.\n\
     \x20   pub label: SubscriptTypegpuStringView,\n\
     \x20   /// Number of required feature enum values.\n\
     \x20   pub required_features_count: usize,\n\
     \x20   /// Required feature enum values.\n\
     \x20   pub required_features: *const SubscriptTypegpuFeatureName,\n\
     \x20   /// Optional required-limits record.\n\
     \x20   pub required_limits: *const SubscriptTypegpuLimits,\n\
     \x20   /// Default queue descriptor.\n\
     \x20   pub default_queue: SubscriptTypegpuQueueDescriptor,\n\
     }\n"
}

pub(crate) fn rust_required_limits_probe() -> &'static str {
    "/// Facade-test probe for the H2 required-limits sentinel rules.\n\
     #[doc(hidden)]\n\
     pub fn subscript_typegpu_internal_required_limits_for_test(\n\
     \x20   max_bind_groups: u32,\n\
     \x20   max_uniform_buffer_binding_size: u64,\n\
     \x20   max_storage_buffer_binding_size: u64,\n\
     \x20   max_buffer_size: u64,\n\
     ) -> (u32, u64, u64, u64) {\n\
     \x20   // SAFETY: every facade limits field admits zero.\n\
     \x20   let mut source: SubscriptTypegpuLimits = unsafe { std::mem::zeroed() };\n\
     \x20   source.max_bind_groups = max_bind_groups;\n\
     \x20   source.max_uniform_buffer_binding_size = max_uniform_buffer_binding_size;\n\
     \x20   source.max_storage_buffer_binding_size = max_storage_buffer_binding_size;\n\
     \x20   source.max_buffer_size = max_buffer_size;\n\
     \x20   let converted = convert_limits(source);\n\
     \x20   (\n\
     \x20       converted.max_bind_groups,\n\
     \x20       converted.max_uniform_buffer_binding_size,\n\
     \x20       converted.max_storage_buffer_binding_size,\n\
     \x20       converted.max_buffer_size,\n\
     \x20   )\n\
     }\n"
}

pub(crate) fn rust_limits_extern(op: &LimitsOp, shape: &StructPlan) -> String {
    let mut params = Vec::new();
    if let Some(receiver) = &op.receiver {
        params.push(format!(
            "{}: {}",
            naming::camel(receiver),
            naming::wgpu_type(receiver)
        ));
    }
    params.push(format!("out: *mut {}", shape.wgpu_struct));
    format!("    fn {}({}) -> i32;\n", op.wgpu_fn, params.join(", "))
}

pub(crate) fn rust_info_extern(op: &AdapterInfoOp) -> String {
    format!(
        "    fn {}({}: {}, out: *mut WGPUAdapterInfo) -> i32;\n",
        op.wgpu_fn,
        naming::camel(&op.receiver),
        naming::wgpu_type(&op.receiver),
    )
}

pub(crate) fn rust_info_free_extern() -> &'static str {
    "    fn wgpuAdapterInfoFreeMembers(info: WGPUAdapterInfo);\n"
}

pub(crate) fn rust_feature_extern(op: &FeatureOp) -> String {
    let mut params = Vec::new();
    if let Some(receiver) = &op.receiver {
        params.push(format!(
            "{}: {}",
            naming::camel(receiver),
            naming::wgpu_type(receiver)
        ));
    }
    params.push("feature: i32".into());
    format!("    fn {}({}) -> u32;\n", op.wgpu_fn, params.join(", "))
}

pub(crate) fn rust_limits_export(op: &LimitsOp, shape: &StructPlan) -> String {
    let mut params = Vec::new();
    let mut guards = String::new();
    let mut call_args = Vec::new();
    if let Some(receiver) = &op.receiver {
        let name = naming::camel(receiver);
        params.push(format!(
            "{name}: {}",
            naming::subscript_typegpu_type(receiver)
        ));
        guards.push_str(&format!(
            "    if {name}.is_null() {{\n        return 0;\n    }}\n"
        ));
        call_args.push(format!("{name}.cast()"));
    } else {
        guards.push_str("    if runtime::table().is_none() {\n        return 0;\n    }\n");
    }
    params.push(format!("out: *mut {}", shape.subscript_typegpu_struct));
    guards.push_str("    if out.is_null() {\n        return 0;\n    }\n");
    call_args.push("&mut backend".into());
    let fields: String = shape
        .fields
        .iter()
        .map(|field| {
            let name = naming::rust_ident(&field.name);
            format!("            {name}: backend.{name},\n")
        })
        .collect();
    format!(
        "/// `subscript-typegpu.h`: fills backend-reported limits and returns status verbatim.\n\
         #[no_mangle]\n\
         pub extern \"C\" fn {subscript_typegpu}({params}) -> i32 {{\n\
         {guards}\
         \x20   // SAFETY: this scalar-only out struct admits all-zero initialization.\n\
         \x20   let mut backend: {wgpu_shape} = unsafe {{ std::mem::zeroed() }};\n\
         \x20   // SAFETY: the optional receiver is non-null and `backend` is writable.\n\
         \x20   let status = unsafe {{ {wgpu_fn}({call_args}) }};\n\
         \x20   // SAFETY: `out` was checked and every field is copied verbatim.\n\
         \x20   unsafe {{\n\
         \x20       out.write({subscript_typegpu_shape} {{\n\
         {fields}\
         \x20       }});\n\
         \x20   }}\n\
         \x20   status\n\
         }}\n",
        subscript_typegpu = op.subscript_typegpu_fn,
        params = params.join(", "),
        wgpu_shape = shape.wgpu_struct,
        wgpu_fn = op.wgpu_fn,
        call_args = call_args.join(", "),
        subscript_typegpu_shape = shape.subscript_typegpu_struct,
    )
}

pub(crate) fn rust_info_success_const(op: &AdapterInfoOp) -> String {
    format!(
        "/// webgpu.yml adapter-info success status.\nconst {}: i32 = {};\n",
        op.success_const,
        naming::hex_enum(op.success_value),
    )
}

pub(crate) fn rust_info_export(op: &AdapterInfoOp) -> String {
    let receiver = naming::camel(&op.receiver);
    let receiver_ty = naming::subscript_typegpu_type(&op.receiver);
    format!(
        "/// `subscript-typegpu.h`: fills copied adapter information and frees backend members.\n\
         #[no_mangle]\n\
         pub extern \"C\" fn {subscript_typegpu}(\n\
         \x20   {receiver}: {receiver_ty},\n\
         \x20   out: *mut SubscriptTypegpuAdapterInfo,\n\
         ) -> bool {{\n\
         \x20   if {receiver}.is_null() || out.is_null() {{\n\
         \x20       return false;\n\
         \x20   }}\n\
         \x20   // SAFETY: WGPUAdapterInfo's initial state is its all-zero initializer.\n\
         \x20   let mut info: WGPUAdapterInfo = unsafe {{ std::mem::zeroed() }};\n\
         \x20   // SAFETY: receiver is non-null and `info` is writable.\n\
         \x20   let status = unsafe {{ {wgpu}({receiver}.cast(), &mut info) }};\n\
         \x20   let strings = runtime::store_adapter_info_strings(\n\
         \x20       {receiver} as usize,\n\
         \x20       [\n\
         \x20           unsafe {{ copy_string_view(info.vendor) }},\n\
         \x20           unsafe {{ copy_string_view(info.architecture) }},\n\
         \x20           unsafe {{ copy_string_view(info.device) }},\n\
         \x20           unsafe {{ copy_string_view(info.description) }},\n\
         \x20       ],\n\
         \x20   );\n\
         \x20   let result = SubscriptTypegpuAdapterInfo {{\n\
         \x20       vendor: SubscriptTypegpuStringView {{ data: strings[0].data as *const c_char, length: strings[0].length }},\n\
         \x20       architecture: SubscriptTypegpuStringView {{ data: strings[1].data as *const c_char, length: strings[1].length }},\n\
         \x20       device: SubscriptTypegpuStringView {{ data: strings[2].data as *const c_char, length: strings[2].length }},\n\
         \x20       description: SubscriptTypegpuStringView {{ data: strings[3].data as *const c_char, length: strings[3].length }},\n\
         \x20       backend_type: info.backend_type,\n\
         \x20       adapter_type: info.adapter_type,\n\
         \x20       vendor_id: info.vendor_id,\n\
         \x20       device_id: info.device_id,\n\
         \x20   }};\n\
         \x20   // SAFETY: ownership of every backend output string is returned once.\n\
         \x20   unsafe {{ wgpuAdapterInfoFreeMembers(info) }};\n\
         \x20   // SAFETY: `out` was checked above.\n\
         \x20   unsafe {{ out.write(result) }};\n\
         \x20   status == {success}\n\
         }}\n",
        subscript_typegpu = op.subscript_typegpu_fn,
        wgpu = op.wgpu_fn,
        success = op.success_const,
    )
}

pub(crate) fn rust_feature_export(op: &FeatureOp) -> String {
    let mut params = Vec::new();
    let mut guards = String::new();
    let mut call_args = Vec::new();
    if let Some(receiver) = &op.receiver {
        let name = naming::camel(receiver);
        params.push(format!(
            "{name}: {}",
            naming::subscript_typegpu_type(receiver)
        ));
        guards.push_str(&format!(
            "    if {name}.is_null() {{\n        return false;\n    }}\n"
        ));
        call_args.push(format!("{name}.cast()"));
    } else {
        guards.push_str("    if runtime::table().is_none() {\n        return false;\n    }\n");
    }
    params.push("feature: i32".into());
    call_args.push("feature".into());
    format!(
        "/// `subscript-typegpu.h`: reports whether one pinned feature enum is present.\n\
         #[no_mangle]\n\
         pub extern \"C\" fn {subscript_typegpu}({params}) -> bool {{\n\
         {guards}\
         \x20   // SAFETY: optional receiver is non-null and the enum is passed verbatim.\n\
         \x20   unsafe {{ {wgpu}({args}) != 0 }}\n\
         }}\n",
        subscript_typegpu = op.subscript_typegpu_fn,
        params = params.join(", "),
        wgpu = op.wgpu_fn,
        args = call_args.join(", "),
    )
}
