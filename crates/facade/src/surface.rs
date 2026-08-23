//! Generated from webgpu.yml plus policy.toml. Do not edit.
#![allow(missing_docs, non_snake_case, non_upper_case_globals)]

use std::ffi::c_void;
use std::sync::OnceLock;

use crate::{
    SubscriptTypegpuAdapter as WGPUAdapter, SubscriptTypegpuDevice as WGPUDevice,
    SubscriptTypegpuInstance as WGPUInstance, SubscriptTypegpuTexture as WGPUTexture,
};

pub type WGPUSurface = *mut c_void;
pub type WGPUStatus = u32;
pub type WGPUSurfaceGetCurrentTextureStatus = u32;
pub type WGPUTextureFormat = u32;
pub type WGPUTextureUsage = u64;
pub type WGPUCompositeAlphaMode = u32;
pub type WGPUPresentMode = u32;
pub type WGPUSType = u32;

pub const WGPUStatus_Success: WGPUStatus = 1;
pub const WGPUStatus_Error: WGPUStatus = 2;
pub const WGPUSurfaceGetCurrentTextureStatus_SuccessOptimal: WGPUSurfaceGetCurrentTextureStatus = 1;
pub const WGPUSurfaceGetCurrentTextureStatus_SuccessSuboptimal: WGPUSurfaceGetCurrentTextureStatus = 2;
pub const WGPUSurfaceGetCurrentTextureStatus_Timeout: WGPUSurfaceGetCurrentTextureStatus = 3;
pub const WGPUSurfaceGetCurrentTextureStatus_Outdated: WGPUSurfaceGetCurrentTextureStatus = 4;
pub const WGPUSurfaceGetCurrentTextureStatus_Lost: WGPUSurfaceGetCurrentTextureStatus = 5;
pub const WGPUSurfaceGetCurrentTextureStatus_Error: WGPUSurfaceGetCurrentTextureStatus = 6;
pub const WGPUTextureFormat_BGRA8Unorm: WGPUTextureFormat = 27;
pub const WGPUTextureUsage_RenderAttachment: WGPUTextureUsage = 16;
pub const WGPUCompositeAlphaMode_Auto: WGPUCompositeAlphaMode = 0;
pub const WGPUPresentMode_Fifo: WGPUPresentMode = 1;
pub const WGPUSType_SurfaceSourceMetalLayer: WGPUSType = 4;
pub const WGPUSType_SurfaceSourceWindowsHWND: WGPUSType = 5;
pub const WGPUSType_SurfaceSourceXlibWindow: WGPUSType = 6;
pub const WGPUSType_SurfaceSourceWaylandSurface: WGPUSType = 7;
pub const WGPUSType_SurfaceSourceAndroidNativeWindow: WGPUSType = 8;
pub const WGPUSType_SurfaceSourceXCBWindow: WGPUSType = 9;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUChainedStruct {
    pub next: *const WGPUChainedStruct,
    pub sType: WGPUSType,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUStringView {
    pub data: *const u8,
    pub length: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceDescriptor {
    pub nextInChain: *mut WGPUChainedStruct,
    pub label: WGPUStringView,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceSourceAndroidNativeWindow {
    pub chain: WGPUChainedStruct,
    pub window: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceSourceMetalLayer {
    pub chain: WGPUChainedStruct,
    pub layer: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceSourceWaylandSurface {
    pub chain: WGPUChainedStruct,
    pub display: *mut c_void,
    pub surface: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceSourceWindowsHWND {
    pub chain: WGPUChainedStruct,
    pub hinstance: *mut c_void,
    pub hwnd: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceSourceXCBWindow {
    pub chain: WGPUChainedStruct,
    pub connection: *mut c_void,
    pub window: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceSourceXlibWindow {
    pub chain: WGPUChainedStruct,
    pub display: *mut c_void,
    pub window: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceConfiguration {
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
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceCapabilities {
    pub nextInChain: *mut WGPUChainedStruct,
    pub usages: WGPUTextureUsage,
    pub formatCount: usize,
    pub formats: *const WGPUTextureFormat,
    pub presentModeCount: usize,
    pub presentModes: *const WGPUPresentMode,
    pub alphaModeCount: usize,
    pub alphaModes: *const WGPUCompositeAlphaMode,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUSurfaceTexture {
    pub nextInChain: *mut WGPUChainedStruct,
    pub texture: WGPUTexture,
    pub status: WGPUSurfaceGetCurrentTextureStatus,
}

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

pub struct SurfaceTable {
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
}

static SURFACE_TABLE: OnceLock<SurfaceTable> = OnceLock::new();

pub fn table() -> Result<&'static SurfaceTable, String> {
    if let Some(table) = SURFACE_TABLE.get() {
        return Ok(table);
    }
    let loaded = SurfaceTable {
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
    };
    let _ = SURFACE_TABLE.set(loaded);
    SURFACE_TABLE
        .get()
        .ok_or_else(|| "surface function table initialization failed".to_owned())
}
