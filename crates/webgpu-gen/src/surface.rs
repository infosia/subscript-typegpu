//! F23 Rust-only surface slice.

use std::collections::BTreeSet;

use crate::model::Yml;
use crate::policy::{Policy, PolicyError};

const REQUIRED: [&str; 22] = [
    "wgpuInstanceCreateSurface",
    "wgpuSurfaceConfigure",
    "wgpuSurfaceUnconfigure",
    "wgpuSurfaceGetCapabilities",
    "wgpuSurfaceCapabilitiesFreeMembers",
    "wgpuSurfaceGetCurrentTexture",
    "wgpuSurfacePresent",
    "wgpuSurfaceAddRef",
    "wgpuSurfaceRelease",
    "wgpuSurfaceSetLabel",
    "WGPUSurfaceDescriptor",
    "WGPUSurfaceSourceAndroidNativeWindow",
    "WGPUSurfaceSourceMetalLayer",
    "WGPUSurfaceSourceWaylandSurface",
    "WGPUSurfaceSourceWindowsHWND",
    "WGPUSurfaceSourceXCBWindow",
    "WGPUSurfaceSourceXlibWindow",
    "WGPUSurfaceConfiguration",
    "WGPUSurfaceCapabilities",
    "WGPUSurfaceTexture",
    "WGPUStatus",
    "WGPUSurfaceGetCurrentTextureStatus",
];

fn known_constructs(yml: &Yml) -> BTreeSet<String> {
    let mut known = BTreeSet::new();
    for function in &yml.functions {
        known.insert(format!("wgpu{}", crate::naming::pascal(&function.name)));
    }
    for object in &yml.objects {
        known.insert(crate::naming::wgpu_type(&object.name));
        known.insert(crate::naming::wgpu_method(&object.name, "add_ref"));
        known.insert(crate::naming::wgpu_method(&object.name, "release"));
        for method in &object.methods {
            known.insert(crate::naming::wgpu_method(&object.name, &method.name));
        }
    }
    for shape in &yml.structs {
        known.insert(crate::naming::wgpu_type(&shape.name));
        if shape.free_members {
            known.insert(format!(
                "wgpu{}FreeMembers",
                crate::naming::pascal(&shape.name)
            ));
        }
    }
    known.extend(
        yml.enums
            .iter()
            .map(|value| crate::naming::wgpu_type(&value.name)),
    );
    known.extend(
        yml.bitflags
            .iter()
            .map(|value| crate::naming::wgpu_type(&value.name)),
    );
    known
}

fn excluded_constructs(policy: &Policy) -> BTreeSet<String> {
    let mut excluded = BTreeSet::new();
    for row in &policy.exclude {
        if row.construct == "addref" {
            for object in &policy.slice.objects {
                excluded.insert(crate::naming::wgpu_method(object, "add_ref"));
            }
        } else if let Some((object, method)) = row.construct.split_once('.') {
            excluded.insert(crate::naming::wgpu_method(object, method));
        } else {
            excluded.insert(format!("wgpu{}", crate::naming::pascal(&row.construct)));
        }
    }
    excluded
}

fn enum_value(yml: &Yml, name: &str, entry: &str) -> Result<u32, PolicyError> {
    yml.enum_(name)
        .and_then(|value| value.value_of(entry))
        .ok_or_else(|| PolicyError::Unknown {
            entry: format!("enum.{name}.{entry}"),
        })
}

fn flag_value(yml: &Yml, name: &str, entry: &str) -> Result<u64, PolicyError> {
    yml.bitflag(name)
        .and_then(|value| value.value_of(entry))
        .ok_or_else(|| PolicyError::Unknown {
            entry: format!("bitflag.{name}.{entry}"),
        })
}

pub(crate) fn render(yml: &Yml, policy: &Policy) -> Result<String, PolicyError> {
    if yml.object("surface").is_none() && policy.host_only.is_empty() {
        return Ok("//! No host-only surface slice in this fixture.\n".to_owned());
    }
    let known = known_constructs(yml);
    let excluded = excluded_constructs(policy);
    let required = REQUIRED.into_iter().collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    for row in &policy.host_only {
        if !seen.insert(row.construct.as_str()) {
            return Err(PolicyError::Duplicate {
                entry: row.construct.clone(),
            });
        }
        if !known.contains(&row.construct) {
            return Err(PolicyError::Unknown {
                entry: row.construct.clone(),
            });
        }
        if row.reason.trim().is_empty() {
            return Err(PolicyError::Invalid {
                entry: row.construct.clone(),
                message: "host-only construct requires a reason".to_owned(),
            });
        }
        if excluded.contains(&row.construct) {
            return Err(PolicyError::Invalid {
                entry: row.construct.clone(),
                message: "construct is both host_only and exclude".to_owned(),
            });
        }
        if !required.contains(row.construct.as_str()) {
            return Err(PolicyError::Dead {
                entry: row.construct.clone(),
            });
        }
    }
    if let Some(missing) = required.difference(&seen).next() {
        return Err(PolicyError::Unpoliced {
            construct: (*missing).to_owned(),
        });
    }

    let bgra8 = enum_value(yml, "texture_format", "BGRA8_unorm")?;
    let render_attachment = flag_value(yml, "texture_usage", "render_attachment")?;
    let alpha_auto = enum_value(yml, "composite_alpha_mode", "auto")?;
    let present_fifo = enum_value(yml, "present_mode", "fifo")?;
    let status_success = enum_value(yml, "status", "success")?;
    let status_error = enum_value(yml, "status", "error")?;
    let surface_success = enum_value(yml, "surface_get_current_texture_status", "success_optimal")?;
    let surface_suboptimal = enum_value(
        yml,
        "surface_get_current_texture_status",
        "success_suboptimal",
    )?;
    let surface_timeout = enum_value(yml, "surface_get_current_texture_status", "timeout")?;
    let surface_outdated = enum_value(yml, "surface_get_current_texture_status", "outdated")?;
    let surface_lost = enum_value(yml, "surface_get_current_texture_status", "lost")?;
    let surface_error = enum_value(yml, "surface_get_current_texture_status", "error")?;
    let stype_metal = enum_value(yml, "s_type", "surface_source_metal_layer")?;
    let stype_windows = enum_value(yml, "s_type", "surface_source_windows_HWND")?;
    let stype_xlib = enum_value(yml, "s_type", "surface_source_xlib_window")?;
    let stype_wayland = enum_value(yml, "s_type", "surface_source_wayland_surface")?;
    let stype_android = enum_value(yml, "s_type", "surface_source_android_native_window")?;
    let stype_xcb = enum_value(yml, "s_type", "surface_source_XCB_window")?;

    Ok(format!(
        r#"//! Generated from webgpu.yml plus policy.toml. Do not edit.
#![allow(missing_docs, non_snake_case, non_upper_case_globals)]

use std::ffi::c_void;
use std::sync::OnceLock;

use crate::{{
    SubscriptTypegpuAdapter as WGPUAdapter, SubscriptTypegpuDevice as WGPUDevice,
    SubscriptTypegpuInstance as WGPUInstance, SubscriptTypegpuTexture as WGPUTexture,
}};

pub type WGPUSurface = *mut c_void;
pub type WGPUStatus = u32;
pub type WGPUSurfaceGetCurrentTextureStatus = u32;
pub type WGPUTextureFormat = u32;
pub type WGPUTextureUsage = u64;
pub type WGPUCompositeAlphaMode = u32;
pub type WGPUPresentMode = u32;
pub type WGPUSType = u32;

pub const WGPUStatus_Success: WGPUStatus = {status_success};
pub const WGPUStatus_Error: WGPUStatus = {status_error};
pub const WGPUSurfaceGetCurrentTextureStatus_SuccessOptimal: WGPUSurfaceGetCurrentTextureStatus = {surface_success};
pub const WGPUSurfaceGetCurrentTextureStatus_SuccessSuboptimal: WGPUSurfaceGetCurrentTextureStatus = {surface_suboptimal};
pub const WGPUSurfaceGetCurrentTextureStatus_Timeout: WGPUSurfaceGetCurrentTextureStatus = {surface_timeout};
pub const WGPUSurfaceGetCurrentTextureStatus_Outdated: WGPUSurfaceGetCurrentTextureStatus = {surface_outdated};
pub const WGPUSurfaceGetCurrentTextureStatus_Lost: WGPUSurfaceGetCurrentTextureStatus = {surface_lost};
pub const WGPUSurfaceGetCurrentTextureStatus_Error: WGPUSurfaceGetCurrentTextureStatus = {surface_error};
pub const WGPUTextureFormat_BGRA8Unorm: WGPUTextureFormat = {bgra8};
pub const WGPUTextureUsage_RenderAttachment: WGPUTextureUsage = {render_attachment};
pub const WGPUCompositeAlphaMode_Auto: WGPUCompositeAlphaMode = {alpha_auto};
pub const WGPUPresentMode_Fifo: WGPUPresentMode = {present_fifo};
pub const WGPUSType_SurfaceSourceMetalLayer: WGPUSType = {stype_metal};
pub const WGPUSType_SurfaceSourceWindowsHWND: WGPUSType = {stype_windows};
pub const WGPUSType_SurfaceSourceXlibWindow: WGPUSType = {stype_xlib};
pub const WGPUSType_SurfaceSourceWaylandSurface: WGPUSType = {stype_wayland};
pub const WGPUSType_SurfaceSourceAndroidNativeWindow: WGPUSType = {stype_android};
pub const WGPUSType_SurfaceSourceXCBWindow: WGPUSType = {stype_xcb};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUChainedStruct {{
    pub next: *const WGPUChainedStruct,
    pub sType: WGPUSType,
}}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUStringView {{
    pub data: *const u8,
    pub length: usize,
}}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceDescriptor {{
    pub nextInChain: *mut WGPUChainedStruct,
    pub label: WGPUStringView,
}}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceSourceAndroidNativeWindow {{
    pub chain: WGPUChainedStruct,
    pub window: *mut c_void,
}}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceSourceMetalLayer {{
    pub chain: WGPUChainedStruct,
    pub layer: *mut c_void,
}}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceSourceWaylandSurface {{
    pub chain: WGPUChainedStruct,
    pub display: *mut c_void,
    pub surface: *mut c_void,
}}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceSourceWindowsHWND {{
    pub chain: WGPUChainedStruct,
    pub hinstance: *mut c_void,
    pub hwnd: *mut c_void,
}}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceSourceXCBWindow {{
    pub chain: WGPUChainedStruct,
    pub connection: *mut c_void,
    pub window: u32,
}}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceSourceXlibWindow {{
    pub chain: WGPUChainedStruct,
    pub display: *mut c_void,
    pub window: u64,
}}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceConfiguration {{
    pub nextInChain: *mut WGPUChainedStruct,
    pub device: WGPUDevice,
    pub format: WGPUTextureFormat,
    pub usage: WGPUTextureUsage,
    pub width: u32,
    pub height: u32,
    pub viewFormatCount: usize,
    pub viewFormats: *const WGPUTextureFormat,
    pub alphaMode: WGPUCompositeAlphaMode,
    pub presentMode: WGPUPresentMode,
}}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceCapabilities {{
    pub nextInChain: *mut WGPUChainedStruct,
    pub usages: WGPUTextureUsage,
    pub formatCount: usize,
    pub formats: *const WGPUTextureFormat,
    pub presentModeCount: usize,
    pub presentModes: *const WGPUPresentMode,
    pub alphaModeCount: usize,
    pub alphaModes: *const WGPUCompositeAlphaMode,
}}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceTexture {{
    pub nextInChain: *mut WGPUChainedStruct,
    pub texture: WGPUTexture,
    pub status: WGPUSurfaceGetCurrentTextureStatus,
}}

// SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
pub type WGPUProcInstanceCreateSurface = unsafe extern "C" fn(WGPUInstance, *const WGPUSurfaceDescriptor) -> WGPUSurface;
// SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
pub type WGPUProcSurfaceConfigure = unsafe extern "C" fn(WGPUSurface, *const WGPUSurfaceConfiguration);
// SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
pub type WGPUProcSurfaceUnconfigure = unsafe extern "C" fn(WGPUSurface);
// SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
pub type WGPUProcSurfaceGetCapabilities = unsafe extern "C" fn(WGPUSurface, WGPUAdapter, *mut WGPUSurfaceCapabilities) -> WGPUStatus;
// SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
pub type WGPUProcSurfaceCapabilitiesFreeMembers = unsafe extern "C" fn(WGPUSurfaceCapabilities);
// SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
pub type WGPUProcSurfaceGetCurrentTexture = unsafe extern "C" fn(WGPUSurface, *mut WGPUSurfaceTexture);
// SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
pub type WGPUProcSurfacePresent = unsafe extern "C" fn(WGPUSurface) -> WGPUStatus;
// SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
pub type WGPUProcSurfaceAddRef = unsafe extern "C" fn(WGPUSurface);
// SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
pub type WGPUProcSurfaceRelease = unsafe extern "C" fn(WGPUSurface);
// SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
pub type WGPUProcSurfaceSetLabel = unsafe extern "C" fn(WGPUSurface, WGPUStringView);

pub struct SurfaceTable {{
    pub wgpuInstanceCreateSurface: WGPUProcInstanceCreateSurface,
    pub wgpuSurfaceConfigure: WGPUProcSurfaceConfigure,
    pub wgpuSurfaceUnconfigure: WGPUProcSurfaceUnconfigure,
    pub wgpuSurfaceGetCapabilities: WGPUProcSurfaceGetCapabilities,
    pub wgpuSurfaceCapabilitiesFreeMembers: WGPUProcSurfaceCapabilitiesFreeMembers,
    pub wgpuSurfaceGetCurrentTexture: WGPUProcSurfaceGetCurrentTexture,
    pub wgpuSurfacePresent: WGPUProcSurfacePresent,
    pub wgpuSurfaceAddRef: WGPUProcSurfaceAddRef,
    pub wgpuSurfaceRelease: WGPUProcSurfaceRelease,
    pub wgpuSurfaceSetLabel: WGPUProcSurfaceSetLabel,
}}

static SURFACE_TABLE: OnceLock<SurfaceTable> = OnceLock::new();

pub fn table() -> Result<&'static SurfaceTable, String> {{
    if let Some(table) = SURFACE_TABLE.get() {{
        return Ok(table);
    }}
    let loaded = SurfaceTable {{
        wgpuInstanceCreateSurface: crate::runtime::surface_symbol(b"wgpuInstanceCreateSurface\0")?,
        wgpuSurfaceConfigure: crate::runtime::surface_symbol(b"wgpuSurfaceConfigure\0")?,
        wgpuSurfaceUnconfigure: crate::runtime::surface_symbol(b"wgpuSurfaceUnconfigure\0")?,
        wgpuSurfaceGetCapabilities: crate::runtime::surface_symbol(b"wgpuSurfaceGetCapabilities\0")?,
        wgpuSurfaceCapabilitiesFreeMembers: crate::runtime::surface_symbol(b"wgpuSurfaceCapabilitiesFreeMembers\0")?,
        wgpuSurfaceGetCurrentTexture: crate::runtime::surface_symbol(b"wgpuSurfaceGetCurrentTexture\0")?,
        wgpuSurfacePresent: crate::runtime::surface_symbol(b"wgpuSurfacePresent\0")?,
        wgpuSurfaceAddRef: crate::runtime::surface_symbol(b"wgpuSurfaceAddRef\0")?,
        wgpuSurfaceRelease: crate::runtime::surface_symbol(b"wgpuSurfaceRelease\0")?,
        wgpuSurfaceSetLabel: crate::runtime::surface_symbol(b"wgpuSurfaceSetLabel\0")?,
    }};
    let _ = SURFACE_TABLE.set(loaded);
    SURFACE_TABLE
        .get()
        .ok_or_else(|| "surface function table initialization failed".to_owned())
}}
"#,
    ))
}
