//! Future-poll pattern (F6/F7/F8): requests register
//! AllowProcessEvents callbacks, record completion in the runtime, and
//! expose polling, typed takes where applicable, and explicit drop.

use crate::naming;
use crate::patterns::rust_signature;
use crate::plan::AsyncOp;

pub(crate) const FUTURE_ID_COMMENT: &str =
    "/* Facade-owned future identifier (monotonically increasing, never 0). */";

pub(crate) const COMPLETED_COMMENT: &str = "/*\n * 0 = pending, 1 = success, negative = failed (the negated backend\n * status enum value); -100 = unknown future id.\n */";

pub(crate) const TAKE_COMMENT: &str = "/*\n * NULL until the future completed successfully; ownership transfers\n * once — a second take returns NULL.\n */";

/// Private webgpu.h request-adapter options with INIT-compatible fields.
pub(crate) fn rust_request_adapter_options(options: &str) -> String {
    format!(
        "#[repr(C)]\n\
         struct {options} {{\n\
         \x20   next_in_chain: *mut WGPUChainedStruct,\n\
         \x20   feature_level: i32,\n\
         \x20   power_preference: i32,\n\
         \x20   force_fallback_adapter: u32,\n\
         \x20   backend_type: i32,\n\
         \x20   compatible_surface: *mut c_void,\n\
         }}\n"
    )
}

pub(crate) fn c_request_decl(op: &AsyncOp, anchor: &str) -> String {
    let anchor_param = format!(
        "{} {}",
        naming::subscript_typegpu_type(anchor),
        naming::camel(anchor)
    );
    if op.receiver == anchor {
        format!(
            "SubscriptTypegpuFutureId {}({anchor_param});",
            op.subscript_typegpu_fn
        )
    } else {
        format!(
            "SubscriptTypegpuFutureId {}({anchor_param}, {} {});",
            op.subscript_typegpu_fn,
            naming::subscript_typegpu_type(&op.receiver),
            naming::camel(&op.receiver),
        )
    }
}

pub(crate) fn c_request_descriptor_decl(op: &AsyncOp, anchor: &str) -> Option<String> {
    op.device_descriptor.then(|| {
        format!(
            "SubscriptTypegpuFutureId {}_with_descriptor({} {}, {} {}, const SubscriptTypegpuDeviceDescriptor* descriptor);",
            op.subscript_typegpu_fn,
            naming::subscript_typegpu_type(anchor),
            naming::camel(anchor),
            naming::subscript_typegpu_type(&op.receiver),
            naming::camel(&op.receiver),
        )
    })
}

pub(crate) fn c_completed_decl(anchor: &str) -> String {
    format!(
        "int32_t subscript_typegpu_future_status({} {}, SubscriptTypegpuFutureId future);",
        naming::subscript_typegpu_type(anchor),
        naming::camel(anchor),
    )
}

pub(crate) fn c_drop_decl(anchor: &str) -> String {
    format!(
        "void subscript_typegpu_future_drop({} {}, SubscriptTypegpuFutureId future);",
        naming::subscript_typegpu_type(anchor),
        naming::camel(anchor),
    )
}

pub(crate) fn c_take_decl(op: &AsyncOp, anchor: &str) -> Option<String> {
    Some(format!(
        "{} {}({} {}, SubscriptTypegpuFutureId future);",
        naming::subscript_typegpu_type(op.cb.handle_object.as_ref()?),
        op.take_fn.as_ref()?,
        naming::subscript_typegpu_type(anchor),
        naming::camel(anchor),
    ))
}

pub(crate) fn rust_callback_typedef(op: &AsyncOp) -> String {
    let handle = op
        .cb
        .handle_object
        .as_ref()
        .map_or_else(String::new, |object| {
            format!(
                "        {}: {},\n",
                naming::camel(object),
                naming::wgpu_type(object),
            )
        });
    format!(
        "/// webgpu.h `{cb}` callback and userdata.\n\
         type {cb} = Option<\n\
         \x20   // SAFETY: the callback signature matches the pinned webgpu.h declaration.\n\
         \x20   unsafe extern \"C\" fn(\n\
         \x20       status: i32,\n\
         {handle}\
         \x20       message: WGPUStringView,\n\
         \x20       userdata1: *mut c_void,\n\
         \x20       userdata2: *mut c_void,\n\
         \x20   ),\n\
         >;\n",
        cb = op.cb.cb_type,
    )
}

pub(crate) fn rust_callback_info(op: &AsyncOp) -> String {
    format!(
        "/// webgpu.h `{info}` (passed by value).\n\
         #[repr(C)]\n\
         struct {info} {{\n\
         \x20   next_in_chain: *mut WGPUChainedStruct,\n\
         \x20   mode: i32,\n\
         \x20   callback: {cb},\n\
         \x20   userdata1: *mut c_void,\n\
         \x20   userdata2: *mut c_void,\n\
         }}\n",
        info = op.cb.cb_info,
        cb = op.cb.cb_type,
    )
}

pub(crate) fn rust_async_extern(op: &AsyncOp) -> String {
    let mut params = vec![format!(
        "{}: {}",
        naming::camel(&op.receiver),
        naming::wgpu_type(&op.receiver),
    )];
    if let Some((name, ty)) = &op.dropped_arg {
        params.push(format!("{}: *const {ty}", naming::camel(name)));
    }
    params.push(format!("callback_info: {}", op.cb.cb_info));
    rust_signature(
        &format!("    fn {}", op.wgpu_fn),
        &params,
        " -> WGPUFuture;",
    ) + "\n"
}

pub(crate) fn rust_callback_fn(op: &AsyncOp, device_events: bool) -> String {
    let (handle_param, handle_value) = op.cb.handle_object.as_ref().map_or_else(
        || (String::new(), "0".to_string()),
        |object| {
            let name = naming::camel(object);
            (
                format!("    {name}: {},\n", naming::wgpu_type(object)),
                format!("{name} as usize"),
            )
        },
    );
    let request_device = device_events && op.wgpu_fn == "wgpuAdapterRequestDevice";
    let userdata2 = if request_device {
        "userdata2"
    } else {
        "_userdata2"
    };
    let event_action = if request_device {
        format!(
            "        let event_id = userdata2 as usize;\n\
             \x20       if status == {status_const} && {handle_value} != 0 {{\n\
             \x20           runtime::associate_device_events(event_id, {handle_value});\n\
             \x20       }} else {{\n\
             \x20           runtime::discard_device_event_slot(event_id);\n\
             \x20       }}\n",
            status_const = op.cb.status_const,
        )
    } else {
        String::new()
    };
    format!(
        "// SAFETY: the callback signature matches the pinned webgpu.h declaration.\n\
         unsafe extern \"C\" fn {rust_fn}(\n\
         \x20   status: i32,\n\
         {handle_param}\
         \x20   message: WGPUStringView,\n\
         \x20   userdata1: *mut c_void,\n\
         \x20   {userdata2}: *mut c_void,\n\
         ) {{\n\
         \x20   // SAFETY: callback pointers and views remain valid for this callback.\n\
         \x20   runtime::callback_guard(|| unsafe {{\n\
         \x20       let message = copy_string_view(message);\n\
         {event_action}\
         \x20       runtime::complete_from_callback(\n\
         \x20           userdata1,\n\
         \x20           {kind},\n\
         \x20           status == {status_const},\n\
         \x20           status,\n\
         \x20           {handle_value},\n\
         \x20           message,\n\
         \x20       );\n\
         \x20   }});\n\
         }}\n",
        rust_fn = op.cb.rust_fn,
        kind = op.kind_const,
        status_const = op.cb.status_const,
    )
}

pub(crate) fn rust_copy_string_view() -> &'static str {
    "/// Copies a callback-scope string view before the callback returns.\n\
     ///\n\
     /// # Safety\n\
     ///\n\
     /// `view` is valid for the callback duration.\n\
     unsafe fn copy_string_view(view: WGPUStringView) -> String {\n\
     \x20   if view.data.is_null() {\n\
     \x20       return String::new();\n\
     \x20   }\n\
     \x20   let bytes: &[u8] = if view.length == WGPU_STRLEN {\n\
     \x20       CStr::from_ptr(view.data).to_bytes()\n\
     \x20   } else {\n\
     \x20       std::slice::from_raw_parts(view.data.cast::<u8>(), view.length)\n\
     \x20   };\n\
     \x20   String::from_utf8_lossy(bytes).into_owned()\n\
     }\n"
}

pub(crate) fn rust_request_export(
    op: &AsyncOp,
    anchor: &str,
    mode_const: &str,
    device_events: bool,
) -> String {
    let recv = naming::camel(&op.receiver);
    let recv_ty = naming::subscript_typegpu_type(&op.receiver);
    let instance = naming::camel(anchor);
    let mut params = if op.receiver == anchor {
        vec![format!("{recv}: {recv_ty}")]
    } else {
        vec![
            format!("{instance}: {}", naming::subscript_typegpu_type(anchor)),
            format!("{recv}: {recv_ty}"),
        ]
    };
    if op.device_descriptor {
        params.push("descriptor: *const SubscriptTypegpuDeviceDescriptor".into());
    }
    let dropped = op
        .dropped_arg
        .as_ref()
        .map(|(name, _)| naming::camel(name))
        .unwrap_or_else(|| "descriptor".to_string());
    let request_device = device_events && op.wgpu_fn == "wgpuAdapterRequestDevice";
    let request_adapter = op.wgpu_fn == "wgpuInstanceRequestAdapter";
    let export_name = if op.device_descriptor {
        format!("{}_with_descriptor", op.subscript_typegpu_fn)
    } else {
        op.subscript_typegpu_fn.clone()
    };
    let compatibility_export = if op.device_descriptor {
        format!(
            "/// `subscript-typegpu.h`: backward-compatible request with the default descriptor.\n\
             #[no_mangle]\n\
             pub extern \"C\" fn {base}(\n\
             \x20   {instance}: {instance_ty},\n\
             \x20   {recv}: {recv_ty},\n\
             ) -> SubscriptTypegpuFutureId {{\n\
             \x20   {with_descriptor}({instance}, {recv}, std::ptr::null())\n\
             }}\n\n",
            base = op.subscript_typegpu_fn,
            instance_ty = naming::subscript_typegpu_type(anchor),
            with_descriptor = export_name,
        )
    } else {
        String::new()
    };
    let event_setup = if request_device {
        format!(
            "    let event_id = runtime::new_device_event_slot();\n\
             \x20   runtime::attach_device_event_to_future(id, event_id);\n\
             \x20   let empty_view = WGPUStringView {{\n\
             \x20       data: std::ptr::null(),\n\
             \x20       length: 0,\n\
             \x20   }};\n\
             \x20   let public_descriptor = if descriptor.is_null() {{\n\
             \x20       None\n\
             \x20   }} else {{\n\
             \x20       // SAFETY: a non-null descriptor is readable for this call.\n\
             \x20       Some(unsafe {{ *descriptor }})\n\
             \x20   }};\n\
             \x20   let required_limits = public_descriptor.as_ref().and_then(|source| {{\n\
             \x20       if source.required_limits.is_null() {{\n\
             \x20           None\n\
             \x20       }} else {{\n\
             \x20           // SAFETY: the nested limits pointer is readable for this call.\n\
             \x20           Some(convert_limits(unsafe {{ *source.required_limits }}))\n\
             \x20       }}\n\
             \x20   }});\n\
             \x20   let default_queue = public_descriptor.as_ref().map_or(\n\
             \x20       WGPUQueueDescriptor {{\n\
             \x20           next_in_chain: std::ptr::null_mut(),\n\
             \x20           label: empty_view,\n\
             \x20       }},\n\
             \x20       |source| convert_queue_descriptor(source.default_queue),\n\
             \x20   );\n\
             \x20   let descriptor = WGPUDeviceDescriptor {{\n\
             \x20       next_in_chain: std::ptr::null_mut(),\n\
             \x20       label: public_descriptor.as_ref().map_or(\n\
             \x20           empty_view, |source| wgpu_string_view(source.label),\n\
             \x20       ),\n\
             \x20       required_feature_count: public_descriptor.as_ref().map_or(\n\
             \x20           0, |source| source.required_features_count,\n\
             \x20       ),\n\
             \x20       required_features: public_descriptor.as_ref().map_or(\n\
             \x20           std::ptr::null(), |source| source.required_features,\n\
             \x20       ),\n\
             \x20       required_limits: required_limits.as_ref().map_or(\n\
             \x20           std::ptr::null(), |limits| limits as *const _,\n\
             \x20       ),\n\
             \x20       default_queue,\n\
             \x20       device_lost_callback_info: WGPUDeviceLostCallbackInfo {{\n\
             \x20           next_in_chain: std::ptr::null_mut(),\n\
             \x20           mode: {mode_const},\n\
             \x20           callback: Some(device_lost_callback),\n\
             \x20           userdata1: event_id as *mut c_void,\n\
             \x20           userdata2: std::ptr::null_mut(),\n\
             \x20       }},\n\
             \x20       uncaptured_error_callback_info: WGPUUncapturedErrorCallbackInfo {{\n\
             \x20           next_in_chain: std::ptr::null_mut(),\n\
             \x20           callback: Some(uncaptured_error_callback),\n\
             \x20           userdata1: event_id as *mut c_void,\n\
             \x20           userdata2: std::ptr::null_mut(),\n\
             \x20       }},\n\
             \x20   }};\n"
        )
    } else {
        String::new()
    };
    let userdata2 = if request_device {
        "event_id as *mut c_void"
    } else {
        "std::ptr::null_mut()"
    };
    let adapter_setup = if request_adapter {
        concat!(
            "    let requested_backend = std::env::var_os(\"SUBSCRIPT_TYPEGPU_BACKEND\");\n",
            "    let backend_type = match requested_backend.as_deref().and_then(std::ffi::OsStr::to_str) {\n",
            "        None if requested_backend.is_none() => None,\n",
            "        Some(\"metal\") => Some(WGPUBackendType_Metal),\n",
            "        Some(\"vulkan\") => Some(WGPUBackendType_Vulkan),\n",
            "        Some(\"gles\") => Some(WGPUBackendType_OpenGLES),\n",
            "        Some(\"d3d11\") => Some(WGPUBackendType_D3D11),\n",
            "        Some(\"d3d12\") => Some(WGPUBackendType_D3D12),\n",
            "        _ => return 0,\n",
            "    };\n",
            "    let options = backend_type.map(|backend_type| WGPURequestAdapterOptions {\n",
            "        next_in_chain: std::ptr::null_mut(),\n",
            "        feature_level: 0,\n",
            "        power_preference: 0,\n",
            "        force_fallback_adapter: 0,\n",
            "        backend_type,\n",
            "        compatible_surface: std::ptr::null_mut(),\n",
            "    });\n",
            "    let options = options.as_ref().map_or(std::ptr::null(), |value| value);\n",
        )
    } else {
        ""
    };
    let call_args = match (&op.dropped_arg, request_device) {
        (Some(_), true) => format!("{recv}.cast(), &descriptor, info"),
        (Some(_), false) if request_adapter => format!("{recv}.cast(), options, info"),
        (Some(_), false) => format!("{recv}.cast(), std::ptr::null(), info"),
        (None, _) => format!("{recv}.cast(), info"),
    };
    let sig = rust_signature(
        &format!("pub extern \"C\" fn {export_name}"),
        &params,
        " -> SubscriptTypegpuFutureId {",
    );
    format!(
        "{compatibility_export}/// `subscript-typegpu.h`: begins the `{wgpu}` request; poll after pumping.\n\
         #[no_mangle]\n\
         {sig}\n\
         \x20   if {recv}.is_null() {{\n\
         \x20       return 0;\n\
         \x20   }}\n\
         {adapter_setup}\
         \x20   let (id, userdata1) = runtime::new_pending_slot({instance} as usize, {kind});\n\
         {event_setup}\
         \x20   let info = {info} {{\n\
         \x20       next_in_chain: std::ptr::null_mut(),\n\
         \x20       mode: {mode_const},\n\
         \x20       callback: Some({cb_fn}),\n\
         \x20       userdata1,\n\
         \x20       userdata2: {userdata2},\n\
         \x20   }};\n\
         \x20   // SAFETY: non-null receiver; NULL {dropped} is allowed and\n\
         \x20   // callback userdata remains live until completion or release.\n\
         \x20   let _ = unsafe {{ {wgpu}({call_args}) }};\n\
         \x20   id\n\
         }}\n",
        wgpu = op.wgpu_fn,
        instance = instance,
        kind = op.kind_const,
        info = op.cb.cb_info,
        cb_fn = op.cb.rust_fn,
        compatibility_export = compatibility_export,
    )
}

pub(crate) fn rust_completed_export(anchor: &str) -> String {
    format!(
        "/// `subscript-typegpu.h`: 0 pending / 1 success / negative failure / -100 unknown.\n\
         #[no_mangle]\n\
         pub extern \"C\" fn subscript_typegpu_future_status({param}: {ty}, future: SubscriptTypegpuFutureId) -> i32 {{\n\
         \x20   runtime::future_status({param} as usize, future)\n\
         }}\n",
        param = naming::camel(anchor),
        ty = naming::subscript_typegpu_type(anchor),
    )
}

pub(crate) fn rust_drop_export(anchor: &str) -> String {
    format!(
        "/// `subscript-typegpu.h`: drops a future slot; pending slots become doomed.\n\
         #[no_mangle]\n\
         pub extern \"C\" fn subscript_typegpu_future_drop({param}: {ty}, future: SubscriptTypegpuFutureId) {{\n\
         \x20   if let Some(handle) = runtime::drop_future({param} as usize, future) {{\n\
         \x20       release_owned_handle(handle);\n\
         \x20   }}\n\
         }}\n",
        param = naming::camel(anchor),
        ty = naming::subscript_typegpu_type(anchor),
    )
}

pub(crate) fn rust_take_export(op: &AsyncOp, anchor: &str) -> String {
    let object = op
        .cb
        .handle_object
        .as_ref()
        .expect("take export requires a handle callback");
    let take_fn = op.take_fn.as_ref().expect("handle callback has a take");
    let handle_ty = naming::subscript_typegpu_type(object);
    let sig = rust_signature(
        &format!("pub extern \"C\" fn {take_fn}"),
        &[
            format!(
                "{}: {}",
                naming::camel(anchor),
                naming::subscript_typegpu_type(anchor)
            ),
            "future: SubscriptTypegpuFutureId".to_string(),
        ],
        &format!(" -> {handle_ty} {{"),
    );
    format!(
        "/// `subscript-typegpu.h`: takes the {word} once and frees its slot.\n\
         #[no_mangle]\n\
         {sig}\n\
         \x20   runtime::take_handle({instance} as usize, future, {kind}) as {handle_ty}\n\
         }}\n",
        word = object.replace('_', " "),
        instance = naming::camel(anchor),
        kind = op.kind_const,
    )
}

pub(crate) fn rust_kind_const(op: &AsyncOp) -> String {
    format!(
        "/// Runtime slot-kind tag for `{cb}` futures.\nconst {kind}: u32 = {value};\n",
        cb = op.cb.rust_fn.trim_end_matches("_callback"),
        kind = op.kind_const,
        value = op.kind_value,
    )
}

pub(crate) fn rust_status_const(op: &AsyncOp) -> String {
    format!(
        "/// webgpu.yml enum value (`success`).\nconst {name}: i32 = {value};\n",
        name = op.cb.status_const,
        value = naming::hex_enum(op.cb.status_value),
    )
}

pub(crate) fn rust_release_helpers(ops: &[&AsyncOp]) -> String {
    let arms: String = ops
        .iter()
        .filter_map(|op| {
            op.cb.handle_object.as_ref().map(|object| {
                let cleanup = if object == "device" {
                    "            runtime::release_device_events(handle.value);\n"
                } else {
                    ""
                };
                format!(
                    "        {kind} => {{\n\
                     \x20           // SAFETY: the owned handle matches this slot kind and is released once.\n\
                     \x20           unsafe {{ wgpu{pascal}Release(handle.value as {ty}) }};\n\
                     {cleanup}\
                     \x20           runtime::note_owned_handle_release();\n\
                     \x20       }}\n",
                    kind = op.kind_const,
                    pascal = naming::pascal(object),
                    ty = naming::wgpu_type(object),
                )
            })
        })
        .collect();
    format!(
        "fn release_owned_handle(handle: runtime::OwnedHandle) {{\n\
         \x20   match handle.kind {{\n\
         {arms}\
         \x20       _ => {{}}\n\
         \x20   }}\n\
         }}\n\
         \n\
         fn release_deferred_handles() {{\n\
         \x20   for handle in runtime::drain_deferred_handles() {{\n\
         \x20       release_owned_handle(handle);\n\
         \x20   }}\n\
         }}\n"
    )
}

pub(crate) fn rust_anchor_release_export(anchor: &str) -> String {
    let pascal = naming::pascal(anchor);
    let snake = naming::snake(anchor);
    let param = naming::camel(anchor);
    let ty = naming::subscript_typegpu_type(anchor);
    format!(
        "/// `subscript-typegpu.h`: releases the {anchor} and every remaining future slot.\n\
         #[no_mangle]\n\
         pub extern \"C\" fn subscript_typegpu_{snake}_release({param}: {ty}) {{\n\
         \x20   if {param}.is_null() {{\n\
         \x20       return;\n\
         \x20   }}\n\
         \x20   // SAFETY: non-null handle owned by the caller.\n\
         \x20   unsafe {{ wgpu{pascal}Release({param}.cast()) }}\n\
         \x20   for handle in runtime::release_all_slots({param} as usize) {{\n\
         \x20       release_owned_handle(handle);\n\
         \x20   }}\n\
         }}\n"
    )
}
