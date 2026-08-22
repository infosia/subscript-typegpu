//! F11/F14 error records, pop-error-scope future, and device event drains.

use crate::naming;
use crate::plan::DeviceEventsOp;

pub(crate) fn c_records() -> &'static str {
    "typedef struct SubscriptTypegpuErrorRecord {\n\
     \x20   SubscriptTypegpuErrorType type;\n\
     \x20   SubscriptTypegpuStringView message;\n\
     } SubscriptTypegpuErrorRecord;\n\
     \n\
     typedef struct SubscriptTypegpuLostRecord {\n\
     \x20   SubscriptTypegpuDeviceLostReason reason;\n\
     \x20   SubscriptTypegpuStringView message;\n\
     } SubscriptTypegpuLostRecord;"
}

pub(crate) fn c_decls(op: &DeviceEventsOp, anchor: &str) -> Vec<String> {
    vec![
        format!(
            "SubscriptTypegpuFutureId {}({} {});",
            op.subscript_typegpu_fn,
            naming::subscript_typegpu_type(&op.receiver),
            naming::camel(&op.receiver),
        ),
        format!(
            "bool {}({} {}, SubscriptTypegpuFutureId future, SubscriptTypegpuErrorRecord* out);",
            op.take_fn,
            naming::subscript_typegpu_type(anchor),
            naming::camel(anchor),
        ),
        "bool subscript_typegpu_device_next_uncaptured_error(SubscriptTypegpuDevice device, SubscriptTypegpuErrorRecord* out);".into(),
        "bool subscript_typegpu_device_lost_info(SubscriptTypegpuDevice device, SubscriptTypegpuLostRecord* out);".into(),
    ]
}

pub(crate) fn rust_private_types(op: &DeviceEventsOp) -> String {
    format!(
        "/// webgpu.h request-device descriptor types used to install F14 callbacks.\n\
         type WGPUFeatureName = i32;\n\
         type WGPUDeviceLostCallback = Option<\n\
         \x20   unsafe extern \"C\" fn(\n\
         \x20       device: *const WGPUDevice,\n\
         \x20       reason: i32,\n\
         \x20       message: WGPUStringView,\n\
         \x20       userdata1: *mut c_void,\n\
         \x20       userdata2: *mut c_void,\n\
         \x20   ),\n\
         >;\n\
         #[repr(C)]\n\
         struct WGPUDeviceLostCallbackInfo {{\n\
         \x20   next_in_chain: *mut WGPUChainedStruct,\n\
         \x20   mode: i32,\n\
         \x20   callback: WGPUDeviceLostCallback,\n\
         \x20   userdata1: *mut c_void,\n\
         \x20   userdata2: *mut c_void,\n\
         }}\n\
         type WGPUUncapturedErrorCallback = Option<\n\
         \x20   unsafe extern \"C\" fn(\n\
         \x20       device: *const WGPUDevice,\n\
         \x20       error_type: i32,\n\
         \x20       message: WGPUStringView,\n\
         \x20       userdata1: *mut c_void,\n\
         \x20       userdata2: *mut c_void,\n\
         \x20   ),\n\
         >;\n\
         #[repr(C)]\n\
         struct WGPUUncapturedErrorCallbackInfo {{\n\
         \x20   next_in_chain: *mut WGPUChainedStruct,\n\
         \x20   callback: WGPUUncapturedErrorCallback,\n\
         \x20   userdata1: *mut c_void,\n\
         \x20   userdata2: *mut c_void,\n\
         }}\n\
         #[repr(C)]\n\
         struct WGPUDeviceDescriptor {{\n\
         \x20   next_in_chain: *mut WGPUChainedStruct,\n\
         \x20   label: WGPUStringView,\n\
         \x20   required_feature_count: usize,\n\
         \x20   required_features: *const WGPUFeatureName,\n\
         \x20   required_limits: *const WGPULimits,\n\
         \x20   default_queue: WGPUQueueDescriptor,\n\
         \x20   device_lost_callback_info: WGPUDeviceLostCallbackInfo,\n\
         \x20   uncaptured_error_callback_info: WGPUUncapturedErrorCallbackInfo,\n\
         }}\n\
         /// webgpu.h `{cb}` callback and info.\n\
         type {cb} = Option<\n\
         \x20   unsafe extern \"C\" fn(\n\
         \x20       status: i32,\n\
         \x20       error_type: i32,\n\
         \x20       message: WGPUStringView,\n\
         \x20       userdata1: *mut c_void,\n\
         \x20       userdata2: *mut c_void,\n\
         \x20   ),\n\
         >;\n\
         #[repr(C)]\n\
         struct {info} {{\n\
         \x20   next_in_chain: *mut WGPUChainedStruct,\n\
         \x20   mode: i32,\n\
         \x20   callback: {cb},\n\
         \x20   userdata1: *mut c_void,\n\
         \x20   userdata2: *mut c_void,\n\
         }}\n",
        cb = op.cb_type,
        info = op.cb_info,
    )
}

pub(crate) fn rust_extern(op: &DeviceEventsOp) -> String {
    format!(
        "    fn {}(device: WGPUDevice, callback_info: {}) -> WGPUFuture;\n",
        op.wgpu_fn, op.cb_info,
    )
}

pub(crate) fn rust_public_records() -> &'static str {
    "/// `subscript-typegpu.h`: facade-filled error type and message.\n\
     #[repr(C)]\n\
     #[derive(Clone, Copy)]\n\
     pub struct SubscriptTypegpuErrorRecord {\n\
     \x20   /// Pinned `SubscriptTypegpuErrorType` value.\n\
     \x20   pub r#type: SubscriptTypegpuErrorType,\n\
     \x20   /// Facade-owned UTF-8 bytes, valid until the next fill on the parent.\n\
     \x20   pub message: SubscriptTypegpuStringView,\n\
     }\n\
     \n\
     /// `subscript-typegpu.h`: facade-filled device-lost reason and message.\n\
     #[repr(C)]\n\
     #[derive(Clone, Copy)]\n\
     pub struct SubscriptTypegpuLostRecord {\n\
     \x20   /// Pinned `SubscriptTypegpuDeviceLostReason` value.\n\
     \x20   pub reason: SubscriptTypegpuDeviceLostReason,\n\
     \x20   /// Facade-owned UTF-8 bytes, valid until the next fill on the device.\n\
     \x20   pub message: SubscriptTypegpuStringView,\n\
     }\n"
}

pub(crate) fn rust_constants(op: &DeviceEventsOp) -> String {
    format!(
        "/// webgpu.yml pop-error-scope success value.\n\
         const {}: i32 = {};\n\
         /// Runtime slot-kind tag for pop-error-scope futures.\n\
         const {}: u32 = {};\n",
        op.status_const,
        naming::hex_enum(op.status_value),
        op.kind_const,
        op.kind_value,
    )
}

pub(crate) fn rust_callbacks(op: &DeviceEventsOp) -> String {
    format!(
        "unsafe extern \"C\" fn device_lost_callback(\n\
         \x20   _device: *const WGPUDevice,\n\
         \x20   reason: i32,\n\
         \x20   message: WGPUStringView,\n\
         \x20   userdata1: *mut c_void,\n\
         \x20   _userdata2: *mut c_void,\n\
         ) {{\n\
         \x20   runtime::callback_guard(|| unsafe {{\n\
         \x20       runtime::record_device_lost(\n\
         \x20           userdata1 as usize,\n\
         \x20           reason,\n\
         \x20           copy_string_view(message),\n\
         \x20       );\n\
         \x20   }});\n\
         }}\n\
         \n\
         unsafe extern \"C\" fn uncaptured_error_callback(\n\
         \x20   _device: *const WGPUDevice,\n\
         \x20   error_type: i32,\n\
         \x20   message: WGPUStringView,\n\
         \x20   userdata1: *mut c_void,\n\
         \x20   _userdata2: *mut c_void,\n\
         ) {{\n\
         \x20   runtime::callback_guard(|| unsafe {{\n\
         \x20       runtime::enqueue_uncaptured_error(\n\
         \x20           userdata1 as usize,\n\
         \x20           error_type,\n\
         \x20           copy_string_view(message),\n\
         \x20       );\n\
         \x20   }});\n\
         }}\n\
         \n\
         unsafe extern \"C\" fn {callback}(\n\
         \x20   status: i32,\n\
         \x20   error_type: i32,\n\
         \x20   message: WGPUStringView,\n\
         \x20   userdata1: *mut c_void,\n\
         \x20   _userdata2: *mut c_void,\n\
         ) {{\n\
         \x20   runtime::callback_guard(|| unsafe {{\n\
         \x20       runtime::complete_record_from_callback(\n\
         \x20           userdata1,\n\
         \x20           {kind},\n\
         \x20           status == {success},\n\
         \x20           status,\n\
         \x20           error_type,\n\
         \x20           copy_string_view(message),\n\
         \x20       );\n\
         \x20   }});\n\
         }}\n",
        callback = op.cb_fn,
        kind = op.kind_const,
        success = op.status_const,
    )
}

pub(crate) fn rust_exports(op: &DeviceEventsOp, anchor: &str, mode_const: &str) -> String {
    format!(
        "/// `subscript-typegpu.h`: pops the current error scope into an F6 future.\n\
         #[no_mangle]\n\
         pub extern \"C\" fn {begin}(device: SubscriptTypegpuDevice) -> SubscriptTypegpuFutureId {{\n\
         \x20   if device.is_null() {{\n\
         \x20       return 0;\n\
         \x20   }}\n\
         \x20   let instance = runtime::instance_for_handle(device as usize);\n\
         \x20   if instance == 0 {{\n\
         \x20       return 0;\n\
         \x20   }}\n\
         \x20   let (id, userdata1) = runtime::new_pending_slot(instance, {kind});\n\
         \x20   let info = {info} {{\n\
         \x20       next_in_chain: std::ptr::null_mut(),\n\
         \x20       mode: {mode},\n\
         \x20       callback: Some({callback}),\n\
         \x20       userdata1,\n\
         \x20       userdata2: std::ptr::null_mut(),\n\
         \x20   }};\n\
         \x20   // SAFETY: the device is non-null and callback userdata remains live.\n\
         \x20   let _ = unsafe {{ {wgpu}(device.cast(), info) }};\n\
         \x20   id\n\
         }}\n\
         \n\
         /// `subscript-typegpu.h`: consumes a successful pop future and fills `out`.\n\
         #[no_mangle]\n\
         pub extern \"C\" fn {take}(\n\
         \x20   {anchor_param}: {anchor_ty},\n\
         \x20   future: SubscriptTypegpuFutureId,\n\
         \x20   out: *mut SubscriptTypegpuErrorRecord,\n\
         ) -> bool {{\n\
         \x20   if out.is_null() {{\n\
         \x20       return false;\n\
         \x20   }}\n\
         \x20   let Some(record) = runtime::take_record(\n\
         \x20       {anchor_param} as usize, future, {kind},\n\
         \x20   ) else {{\n\
         \x20       return false;\n\
         \x20   }};\n\
         \x20   unsafe {{\n\
         \x20       out.write(SubscriptTypegpuErrorRecord {{\n\
         \x20           r#type: record.value,\n\
         \x20           message: record_string_view(record),\n\
         \x20       }});\n\
         \x20   }}\n\
         \x20   true\n\
         }}\n\
         \n\
         fn record_string_view(record: runtime::RecordFill) -> SubscriptTypegpuStringView {{\n\
         \x20   SubscriptTypegpuStringView {{\n\
         \x20       data: record.data as *const c_char,\n\
         \x20       length: record.length,\n\
         \x20   }}\n\
         }}\n\
         \n\
         /// `subscript-typegpu.h`: drains the next uncaptured error in FIFO order.\n\
         #[no_mangle]\n\
         pub extern \"C\" fn subscript_typegpu_device_next_uncaptured_error(\n\
         \x20   device: SubscriptTypegpuDevice,\n\
         \x20   out: *mut SubscriptTypegpuErrorRecord,\n\
         ) -> bool {{\n\
         \x20   if device.is_null() || out.is_null() {{\n\
         \x20       return false;\n\
         \x20   }}\n\
         \x20   let Some(record) = runtime::next_uncaptured_error(device as usize) else {{\n\
         \x20       return false;\n\
         \x20   }};\n\
         \x20   unsafe {{\n\
         \x20       out.write(SubscriptTypegpuErrorRecord {{\n\
         \x20           r#type: record.value,\n\
         \x20           message: record_string_view(record),\n\
         \x20       }});\n\
         \x20   }}\n\
         \x20   true\n\
         }}\n\
         \n\
         /// `subscript-typegpu.h`: fills the recorded device-lost information when present.\n\
         #[no_mangle]\n\
         pub extern \"C\" fn subscript_typegpu_device_lost_info(\n\
         \x20   device: SubscriptTypegpuDevice,\n\
         \x20   out: *mut SubscriptTypegpuLostRecord,\n\
         ) -> bool {{\n\
         \x20   if device.is_null() || out.is_null() {{\n\
         \x20       return false;\n\
         \x20   }}\n\
         \x20   let Some(record) = runtime::device_lost_info(device as usize) else {{\n\
         \x20       return false;\n\
         \x20   }};\n\
         \x20   unsafe {{\n\
         \x20       out.write(SubscriptTypegpuLostRecord {{\n\
         \x20           reason: record.value,\n\
         \x20           message: record_string_view(record),\n\
         \x20       }});\n\
         \x20   }}\n\
         \x20   true\n\
         }}\n\
         \n\
         /// Facade-test injection for the F11 string-byte lifetime rule.\n\
         #[doc(hidden)]\n\
         pub fn subscript_typegpu_internal_enqueue_uncaptured_error_for_test(\n\
         \x20   device: SubscriptTypegpuDevice,\n\
         \x20   error_type: i32,\n\
         \x20   message: &str,\n\
         ) -> bool {{\n\
         \x20   runtime::enqueue_uncaptured_for_device(\n\
         \x20       device as usize, error_type, message.to_owned(),\n\
         \x20   )\n\
         }}\n",
        begin = op.subscript_typegpu_fn,
        kind = op.kind_const,
        info = op.cb_info,
        mode = mode_const,
        callback = op.cb_fn,
        wgpu = op.wgpu_fn,
        take = op.take_fn,
        anchor_param = naming::camel(anchor),
        anchor_ty = naming::subscript_typegpu_type(anchor),
    )
}
