//! Generated from webgpu.yml plus policy.toml. Do not edit.
#![allow(missing_docs, non_snake_case, non_upper_case_globals)]

use std::ffi::{c_char, c_void};
use std::sync::OnceLock;

pub type WGPUAdapter = crate::SubscriptTypegpuAdapter;
pub type WGPUDevice = crate::SubscriptTypegpuDevice;
pub type WGPUInstance = crate::SubscriptTypegpuInstance;
pub type WGPUSurface = *mut c_void;
pub type WGPUTexture = crate::SubscriptTypegpuTexture;

pub type WGPUCompositeAlphaMode = u32;
pub const WGPUCompositeAlphaMode_Auto: WGPUCompositeAlphaMode = 0;
pub const WGPUCompositeAlphaMode_Opaque: WGPUCompositeAlphaMode = 1;
pub const WGPUCompositeAlphaMode_Premultiplied: WGPUCompositeAlphaMode = 2;
pub const WGPUCompositeAlphaMode_Unpremultiplied: WGPUCompositeAlphaMode = 3;
pub const WGPUCompositeAlphaMode_Inherit: WGPUCompositeAlphaMode = 4;

pub type WGPUPresentMode = u32;
pub const WGPUPresentMode_Undefined: WGPUPresentMode = 0;
pub const WGPUPresentMode_Fifo: WGPUPresentMode = 1;
pub const WGPUPresentMode_FifoRelaxed: WGPUPresentMode = 2;
pub const WGPUPresentMode_Immediate: WGPUPresentMode = 3;
pub const WGPUPresentMode_Mailbox: WGPUPresentMode = 4;

pub type WGPUSType = u32;
pub const WGPUSType_ShaderSourceSPIRV: WGPUSType = 1;
pub const WGPUSType_ShaderSourceWGSL: WGPUSType = 2;
pub const WGPUSType_RenderPassMaxDrawCount: WGPUSType = 3;
pub const WGPUSType_SurfaceSourceMetalLayer: WGPUSType = 4;
pub const WGPUSType_SurfaceSourceWindowsHWND: WGPUSType = 5;
pub const WGPUSType_SurfaceSourceXlibWindow: WGPUSType = 6;
pub const WGPUSType_SurfaceSourceWaylandSurface: WGPUSType = 7;
pub const WGPUSType_SurfaceSourceAndroidNativeWindow: WGPUSType = 8;
pub const WGPUSType_SurfaceSourceXCBWindow: WGPUSType = 9;
pub const WGPUSType_SurfaceColorManagement: WGPUSType = 10;
pub const WGPUSType_RequestAdapterWebXROptions: WGPUSType = 11;
pub const WGPUSType_TextureComponentSwizzleDescriptor: WGPUSType = 12;
pub const WGPUSType_ExternalTextureBindingLayout: WGPUSType = 13;
pub const WGPUSType_ExternalTextureBindingEntry: WGPUSType = 14;
pub const WGPUSType_CompatibilityModeLimits: WGPUSType = 15;
pub const WGPUSType_TextureBindingViewDimension: WGPUSType = 16;

pub type WGPUStatus = u32;
pub const WGPUStatus_Success: WGPUStatus = 1;
pub const WGPUStatus_Error: WGPUStatus = 2;

pub type WGPUSurfaceGetCurrentTextureStatus = u32;
pub const WGPUSurfaceGetCurrentTextureStatus_SuccessOptimal: WGPUSurfaceGetCurrentTextureStatus = 1;
pub const WGPUSurfaceGetCurrentTextureStatus_SuccessSuboptimal: WGPUSurfaceGetCurrentTextureStatus = 2;
pub const WGPUSurfaceGetCurrentTextureStatus_Timeout: WGPUSurfaceGetCurrentTextureStatus = 3;
pub const WGPUSurfaceGetCurrentTextureStatus_Outdated: WGPUSurfaceGetCurrentTextureStatus = 4;
pub const WGPUSurfaceGetCurrentTextureStatus_Lost: WGPUSurfaceGetCurrentTextureStatus = 5;
pub const WGPUSurfaceGetCurrentTextureStatus_Error: WGPUSurfaceGetCurrentTextureStatus = 6;

pub type WGPUTextureFormat = u32;
pub const WGPUTextureFormat_Undefined: WGPUTextureFormat = 0;
pub const WGPUTextureFormat_R8Unorm: WGPUTextureFormat = 1;
pub const WGPUTextureFormat_R8Snorm: WGPUTextureFormat = 2;
pub const WGPUTextureFormat_R8Uint: WGPUTextureFormat = 3;
pub const WGPUTextureFormat_R8Sint: WGPUTextureFormat = 4;
pub const WGPUTextureFormat_R16Unorm: WGPUTextureFormat = 5;
pub const WGPUTextureFormat_R16Snorm: WGPUTextureFormat = 6;
pub const WGPUTextureFormat_R16Uint: WGPUTextureFormat = 7;
pub const WGPUTextureFormat_R16Sint: WGPUTextureFormat = 8;
pub const WGPUTextureFormat_R16Float: WGPUTextureFormat = 9;
pub const WGPUTextureFormat_RG8Unorm: WGPUTextureFormat = 10;
pub const WGPUTextureFormat_RG8Snorm: WGPUTextureFormat = 11;
pub const WGPUTextureFormat_RG8Uint: WGPUTextureFormat = 12;
pub const WGPUTextureFormat_RG8Sint: WGPUTextureFormat = 13;
pub const WGPUTextureFormat_R32Float: WGPUTextureFormat = 14;
pub const WGPUTextureFormat_R32Uint: WGPUTextureFormat = 15;
pub const WGPUTextureFormat_R32Sint: WGPUTextureFormat = 16;
pub const WGPUTextureFormat_RG16Unorm: WGPUTextureFormat = 17;
pub const WGPUTextureFormat_RG16Snorm: WGPUTextureFormat = 18;
pub const WGPUTextureFormat_RG16Uint: WGPUTextureFormat = 19;
pub const WGPUTextureFormat_RG16Sint: WGPUTextureFormat = 20;
pub const WGPUTextureFormat_RG16Float: WGPUTextureFormat = 21;
pub const WGPUTextureFormat_RGBA8Unorm: WGPUTextureFormat = 22;
pub const WGPUTextureFormat_RGBA8UnormSrgb: WGPUTextureFormat = 23;
pub const WGPUTextureFormat_RGBA8Snorm: WGPUTextureFormat = 24;
pub const WGPUTextureFormat_RGBA8Uint: WGPUTextureFormat = 25;
pub const WGPUTextureFormat_RGBA8Sint: WGPUTextureFormat = 26;
pub const WGPUTextureFormat_BGRA8Unorm: WGPUTextureFormat = 27;
pub const WGPUTextureFormat_BGRA8UnormSrgb: WGPUTextureFormat = 28;
pub const WGPUTextureFormat_RGB10A2Uint: WGPUTextureFormat = 29;
pub const WGPUTextureFormat_RGB10A2Unorm: WGPUTextureFormat = 30;
pub const WGPUTextureFormat_RG11B10Ufloat: WGPUTextureFormat = 31;
pub const WGPUTextureFormat_RGB9E5Ufloat: WGPUTextureFormat = 32;
pub const WGPUTextureFormat_RG32Float: WGPUTextureFormat = 33;
pub const WGPUTextureFormat_RG32Uint: WGPUTextureFormat = 34;
pub const WGPUTextureFormat_RG32Sint: WGPUTextureFormat = 35;
pub const WGPUTextureFormat_RGBA16Unorm: WGPUTextureFormat = 36;
pub const WGPUTextureFormat_RGBA16Snorm: WGPUTextureFormat = 37;
pub const WGPUTextureFormat_RGBA16Uint: WGPUTextureFormat = 38;
pub const WGPUTextureFormat_RGBA16Sint: WGPUTextureFormat = 39;
pub const WGPUTextureFormat_RGBA16Float: WGPUTextureFormat = 40;
pub const WGPUTextureFormat_RGBA32Float: WGPUTextureFormat = 41;
pub const WGPUTextureFormat_RGBA32Uint: WGPUTextureFormat = 42;
pub const WGPUTextureFormat_RGBA32Sint: WGPUTextureFormat = 43;
pub const WGPUTextureFormat_Stencil8: WGPUTextureFormat = 44;
pub const WGPUTextureFormat_Depth16Unorm: WGPUTextureFormat = 45;
pub const WGPUTextureFormat_Depth24Plus: WGPUTextureFormat = 46;
pub const WGPUTextureFormat_Depth24PlusStencil8: WGPUTextureFormat = 47;
pub const WGPUTextureFormat_Depth32Float: WGPUTextureFormat = 48;
pub const WGPUTextureFormat_Depth32FloatStencil8: WGPUTextureFormat = 49;
pub const WGPUTextureFormat_BC1RGBAUnorm: WGPUTextureFormat = 50;
pub const WGPUTextureFormat_BC1RGBAUnormSrgb: WGPUTextureFormat = 51;
pub const WGPUTextureFormat_BC2RGBAUnorm: WGPUTextureFormat = 52;
pub const WGPUTextureFormat_BC2RGBAUnormSrgb: WGPUTextureFormat = 53;
pub const WGPUTextureFormat_BC3RGBAUnorm: WGPUTextureFormat = 54;
pub const WGPUTextureFormat_BC3RGBAUnormSrgb: WGPUTextureFormat = 55;
pub const WGPUTextureFormat_BC4RUnorm: WGPUTextureFormat = 56;
pub const WGPUTextureFormat_BC4RSnorm: WGPUTextureFormat = 57;
pub const WGPUTextureFormat_BC5RGUnorm: WGPUTextureFormat = 58;
pub const WGPUTextureFormat_BC5RGSnorm: WGPUTextureFormat = 59;
pub const WGPUTextureFormat_BC6HRGBUfloat: WGPUTextureFormat = 60;
pub const WGPUTextureFormat_BC6HRGBFloat: WGPUTextureFormat = 61;
pub const WGPUTextureFormat_BC7RGBAUnorm: WGPUTextureFormat = 62;
pub const WGPUTextureFormat_BC7RGBAUnormSrgb: WGPUTextureFormat = 63;
pub const WGPUTextureFormat_ETC2RGB8Unorm: WGPUTextureFormat = 64;
pub const WGPUTextureFormat_ETC2RGB8UnormSrgb: WGPUTextureFormat = 65;
pub const WGPUTextureFormat_ETC2RGB8A1Unorm: WGPUTextureFormat = 66;
pub const WGPUTextureFormat_ETC2RGB8A1UnormSrgb: WGPUTextureFormat = 67;
pub const WGPUTextureFormat_ETC2RGBA8Unorm: WGPUTextureFormat = 68;
pub const WGPUTextureFormat_ETC2RGBA8UnormSrgb: WGPUTextureFormat = 69;
pub const WGPUTextureFormat_EACR11Unorm: WGPUTextureFormat = 70;
pub const WGPUTextureFormat_EACR11Snorm: WGPUTextureFormat = 71;
pub const WGPUTextureFormat_EACRG11Unorm: WGPUTextureFormat = 72;
pub const WGPUTextureFormat_EACRG11Snorm: WGPUTextureFormat = 73;
pub const WGPUTextureFormat_ASTC4x4Unorm: WGPUTextureFormat = 74;
pub const WGPUTextureFormat_ASTC4x4UnormSrgb: WGPUTextureFormat = 75;
pub const WGPUTextureFormat_ASTC5x4Unorm: WGPUTextureFormat = 76;
pub const WGPUTextureFormat_ASTC5x4UnormSrgb: WGPUTextureFormat = 77;
pub const WGPUTextureFormat_ASTC5x5Unorm: WGPUTextureFormat = 78;
pub const WGPUTextureFormat_ASTC5x5UnormSrgb: WGPUTextureFormat = 79;
pub const WGPUTextureFormat_ASTC6x5Unorm: WGPUTextureFormat = 80;
pub const WGPUTextureFormat_ASTC6x5UnormSrgb: WGPUTextureFormat = 81;
pub const WGPUTextureFormat_ASTC6x6Unorm: WGPUTextureFormat = 82;
pub const WGPUTextureFormat_ASTC6x6UnormSrgb: WGPUTextureFormat = 83;
pub const WGPUTextureFormat_ASTC8x5Unorm: WGPUTextureFormat = 84;
pub const WGPUTextureFormat_ASTC8x5UnormSrgb: WGPUTextureFormat = 85;
pub const WGPUTextureFormat_ASTC8x6Unorm: WGPUTextureFormat = 86;
pub const WGPUTextureFormat_ASTC8x6UnormSrgb: WGPUTextureFormat = 87;
pub const WGPUTextureFormat_ASTC8x8Unorm: WGPUTextureFormat = 88;
pub const WGPUTextureFormat_ASTC8x8UnormSrgb: WGPUTextureFormat = 89;
pub const WGPUTextureFormat_ASTC10x5Unorm: WGPUTextureFormat = 90;
pub const WGPUTextureFormat_ASTC10x5UnormSrgb: WGPUTextureFormat = 91;
pub const WGPUTextureFormat_ASTC10x6Unorm: WGPUTextureFormat = 92;
pub const WGPUTextureFormat_ASTC10x6UnormSrgb: WGPUTextureFormat = 93;
pub const WGPUTextureFormat_ASTC10x8Unorm: WGPUTextureFormat = 94;
pub const WGPUTextureFormat_ASTC10x8UnormSrgb: WGPUTextureFormat = 95;
pub const WGPUTextureFormat_ASTC10x10Unorm: WGPUTextureFormat = 96;
pub const WGPUTextureFormat_ASTC10x10UnormSrgb: WGPUTextureFormat = 97;
pub const WGPUTextureFormat_ASTC12x10Unorm: WGPUTextureFormat = 98;
pub const WGPUTextureFormat_ASTC12x10UnormSrgb: WGPUTextureFormat = 99;
pub const WGPUTextureFormat_ASTC12x12Unorm: WGPUTextureFormat = 100;
pub const WGPUTextureFormat_ASTC12x12UnormSrgb: WGPUTextureFormat = 101;

pub type WGPUTextureUsage = u64;
pub const WGPUTextureUsage_None: WGPUTextureUsage = 0;
pub const WGPUTextureUsage_CopySrc: WGPUTextureUsage = 1;
pub const WGPUTextureUsage_CopyDst: WGPUTextureUsage = 2;
pub const WGPUTextureUsage_TextureBinding: WGPUTextureUsage = 4;
pub const WGPUTextureUsage_StorageBinding: WGPUTextureUsage = 8;
pub const WGPUTextureUsage_RenderAttachment: WGPUTextureUsage = 16;
pub const WGPUTextureUsage_TransientAttachment: WGPUTextureUsage = 32;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUChainedStruct {
    pub next: *mut WGPUChainedStruct,
    pub sType: WGPUSType,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WGPUStringView {
    pub data: *const c_char,
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

pub type WGPUProcInstanceCreateSurface = unsafe extern "C" fn(WGPUInstance, *const WGPUSurfaceDescriptor) -> WGPUSurface;
pub type WGPUProcSurfaceConfigure = unsafe extern "C" fn(WGPUSurface, *const WGPUSurfaceConfiguration);
pub type WGPUProcSurfaceUnconfigure = unsafe extern "C" fn(WGPUSurface);
pub type WGPUProcSurfaceGetCapabilities = unsafe extern "C" fn(WGPUSurface, WGPUAdapter, *mut WGPUSurfaceCapabilities) -> WGPUStatus;
pub type WGPUProcSurfaceCapabilitiesFreeMembers = unsafe extern "C" fn(WGPUSurfaceCapabilities);
pub type WGPUProcSurfaceGetCurrentTexture = unsafe extern "C" fn(WGPUSurface, *mut WGPUSurfaceTexture);
pub type WGPUProcSurfacePresent = unsafe extern "C" fn(WGPUSurface) -> WGPUStatus;
pub type WGPUProcSurfaceAddRef = unsafe extern "C" fn(WGPUSurface);
pub type WGPUProcSurfaceRelease = unsafe extern "C" fn(WGPUSurface);
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
        wgpuInstanceCreateSurface: {
            // SAFETY: the type comes from this symbol's pinned webgpu.yml declaration.
            unsafe { crate::runtime::surface_symbol(b"wgpuInstanceCreateSurface\0") }?
        },
        wgpuSurfaceConfigure: {
            // SAFETY: the type comes from this symbol's pinned webgpu.yml declaration.
            unsafe { crate::runtime::surface_symbol(b"wgpuSurfaceConfigure\0") }?
        },
        wgpuSurfaceUnconfigure: {
            // SAFETY: the type comes from this symbol's pinned webgpu.yml declaration.
            unsafe { crate::runtime::surface_symbol(b"wgpuSurfaceUnconfigure\0") }?
        },
        wgpuSurfaceGetCapabilities: {
            // SAFETY: the type comes from this symbol's pinned webgpu.yml declaration.
            unsafe { crate::runtime::surface_symbol(b"wgpuSurfaceGetCapabilities\0") }?
        },
        wgpuSurfaceCapabilitiesFreeMembers: {
            // SAFETY: the type comes from this symbol's pinned webgpu.yml declaration.
            unsafe { crate::runtime::surface_symbol(b"wgpuSurfaceCapabilitiesFreeMembers\0") }?
        },
        wgpuSurfaceGetCurrentTexture: {
            // SAFETY: the type comes from this symbol's pinned webgpu.yml declaration.
            unsafe { crate::runtime::surface_symbol(b"wgpuSurfaceGetCurrentTexture\0") }?
        },
        wgpuSurfacePresent: {
            // SAFETY: the type comes from this symbol's pinned webgpu.yml declaration.
            unsafe { crate::runtime::surface_symbol(b"wgpuSurfacePresent\0") }?
        },
        wgpuSurfaceAddRef: {
            // SAFETY: the type comes from this symbol's pinned webgpu.yml declaration.
            unsafe { crate::runtime::surface_symbol(b"wgpuSurfaceAddRef\0") }?
        },
        wgpuSurfaceRelease: {
            // SAFETY: the type comes from this symbol's pinned webgpu.yml declaration.
            unsafe { crate::runtime::surface_symbol(b"wgpuSurfaceRelease\0") }?
        },
        wgpuSurfaceSetLabel: {
            // SAFETY: the type comes from this symbol's pinned webgpu.yml declaration.
            unsafe { crate::runtime::surface_symbol(b"wgpuSurfaceSetLabel\0") }?
        },
    };
    let _ = SURFACE_TABLE.set(loaded);
    SURFACE_TABLE
        .get()
        .ok_or_else(|| "surface function table initialization failed".to_owned())
}
