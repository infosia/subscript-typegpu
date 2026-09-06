//! Generated from webgpu.yml plus policy.toml. Do not edit.
#![allow(non_snake_case, non_upper_case_globals)]

use std::ffi::{c_char, c_void};
use std::sync::OnceLock;

/// The facade handle for a native adapter.
pub type WGPUAdapter = crate::SubscriptTypegpuAdapter;
/// The facade handle for a native device.
pub type WGPUDevice = crate::SubscriptTypegpuDevice;
/// The facade handle for a native instance.
pub type WGPUInstance = crate::SubscriptTypegpuInstance;
/// An opaque backend handle for a native surface.
pub type WGPUSurface = *mut c_void;
/// The facade handle for a native texture.
pub type WGPUTexture = crate::SubscriptTypegpuTexture;

/// The native composite alpha mode selection encoded for the C ABI.
pub type WGPUCompositeAlphaMode = u32;
/// Selects `auto` for [`WGPUCompositeAlphaMode`].
pub const WGPUCompositeAlphaMode_Auto: WGPUCompositeAlphaMode = 0;
/// Selects `opaque` for [`WGPUCompositeAlphaMode`].
pub const WGPUCompositeAlphaMode_Opaque: WGPUCompositeAlphaMode = 1;
/// Selects `premultiplied` for [`WGPUCompositeAlphaMode`].
pub const WGPUCompositeAlphaMode_Premultiplied: WGPUCompositeAlphaMode = 2;
/// Selects `unpremultiplied` for [`WGPUCompositeAlphaMode`].
pub const WGPUCompositeAlphaMode_Unpremultiplied: WGPUCompositeAlphaMode = 3;
/// Selects `inherit` for [`WGPUCompositeAlphaMode`].
pub const WGPUCompositeAlphaMode_Inherit: WGPUCompositeAlphaMode = 4;

/// The native present mode selection encoded for the C ABI.
pub type WGPUPresentMode = u32;
/// Selects `undefined` for [`WGPUPresentMode`].
pub const WGPUPresentMode_Undefined: WGPUPresentMode = 0;
/// Selects `fifo` for [`WGPUPresentMode`].
pub const WGPUPresentMode_Fifo: WGPUPresentMode = 1;
/// Selects `fifo_relaxed` for [`WGPUPresentMode`].
pub const WGPUPresentMode_FifoRelaxed: WGPUPresentMode = 2;
/// Selects `immediate` for [`WGPUPresentMode`].
pub const WGPUPresentMode_Immediate: WGPUPresentMode = 3;
/// Selects `mailbox` for [`WGPUPresentMode`].
pub const WGPUPresentMode_Mailbox: WGPUPresentMode = 4;

/// The native s type selection encoded for the C ABI.
pub type WGPUSType = u32;
/// Selects `shader_source_SPIRV` for [`WGPUSType`].
pub const WGPUSType_ShaderSourceSPIRV: WGPUSType = 1;
/// Selects `shader_source_WGSL` for [`WGPUSType`].
pub const WGPUSType_ShaderSourceWGSL: WGPUSType = 2;
/// Selects `render_pass_max_draw_count` for [`WGPUSType`].
pub const WGPUSType_RenderPassMaxDrawCount: WGPUSType = 3;
/// Selects `surface_source_metal_layer` for [`WGPUSType`].
pub const WGPUSType_SurfaceSourceMetalLayer: WGPUSType = 4;
/// Selects `surface_source_windows_HWND` for [`WGPUSType`].
pub const WGPUSType_SurfaceSourceWindowsHWND: WGPUSType = 5;
/// Selects `surface_source_xlib_window` for [`WGPUSType`].
pub const WGPUSType_SurfaceSourceXlibWindow: WGPUSType = 6;
/// Selects `surface_source_wayland_surface` for [`WGPUSType`].
pub const WGPUSType_SurfaceSourceWaylandSurface: WGPUSType = 7;
/// Selects `surface_source_android_native_window` for [`WGPUSType`].
pub const WGPUSType_SurfaceSourceAndroidNativeWindow: WGPUSType = 8;
/// Selects `surface_source_XCB_window` for [`WGPUSType`].
pub const WGPUSType_SurfaceSourceXCBWindow: WGPUSType = 9;
/// Selects `surface_color_management` for [`WGPUSType`].
pub const WGPUSType_SurfaceColorManagement: WGPUSType = 10;
/// Selects `request_adapter_WebXR_options` for [`WGPUSType`].
pub const WGPUSType_RequestAdapterWebXROptions: WGPUSType = 11;
/// Selects `texture_component_swizzle_descriptor` for [`WGPUSType`].
pub const WGPUSType_TextureComponentSwizzleDescriptor: WGPUSType = 12;
/// Selects `external_texture_binding_layout` for [`WGPUSType`].
pub const WGPUSType_ExternalTextureBindingLayout: WGPUSType = 13;
/// Selects `external_texture_binding_entry` for [`WGPUSType`].
pub const WGPUSType_ExternalTextureBindingEntry: WGPUSType = 14;
/// Selects `compatibility_mode_limits` for [`WGPUSType`].
pub const WGPUSType_CompatibilityModeLimits: WGPUSType = 15;
/// Selects `texture_binding_view_dimension` for [`WGPUSType`].
pub const WGPUSType_TextureBindingViewDimension: WGPUSType = 16;

/// The native status selection encoded for the C ABI.
pub type WGPUStatus = u32;
/// Selects `success` for [`WGPUStatus`].
pub const WGPUStatus_Success: WGPUStatus = 1;
/// Selects `error` for [`WGPUStatus`].
pub const WGPUStatus_Error: WGPUStatus = 2;

/// The native surface get current texture status selection encoded for the C ABI.
pub type WGPUSurfaceGetCurrentTextureStatus = u32;
/// Selects `success_optimal` for [`WGPUSurfaceGetCurrentTextureStatus`].
pub const WGPUSurfaceGetCurrentTextureStatus_SuccessOptimal: WGPUSurfaceGetCurrentTextureStatus = 1;
/// Selects `success_suboptimal` for [`WGPUSurfaceGetCurrentTextureStatus`].
pub const WGPUSurfaceGetCurrentTextureStatus_SuccessSuboptimal: WGPUSurfaceGetCurrentTextureStatus = 2;
/// Selects `timeout` for [`WGPUSurfaceGetCurrentTextureStatus`].
pub const WGPUSurfaceGetCurrentTextureStatus_Timeout: WGPUSurfaceGetCurrentTextureStatus = 3;
/// Selects `outdated` for [`WGPUSurfaceGetCurrentTextureStatus`].
pub const WGPUSurfaceGetCurrentTextureStatus_Outdated: WGPUSurfaceGetCurrentTextureStatus = 4;
/// Selects `lost` for [`WGPUSurfaceGetCurrentTextureStatus`].
pub const WGPUSurfaceGetCurrentTextureStatus_Lost: WGPUSurfaceGetCurrentTextureStatus = 5;
/// Selects `error` for [`WGPUSurfaceGetCurrentTextureStatus`].
pub const WGPUSurfaceGetCurrentTextureStatus_Error: WGPUSurfaceGetCurrentTextureStatus = 6;

/// The native texture format selection encoded for the C ABI.
pub type WGPUTextureFormat = u32;
/// Selects `undefined` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_Undefined: WGPUTextureFormat = 0;
/// Selects `R8_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_R8Unorm: WGPUTextureFormat = 1;
/// Selects `R8_snorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_R8Snorm: WGPUTextureFormat = 2;
/// Selects `R8_uint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_R8Uint: WGPUTextureFormat = 3;
/// Selects `R8_sint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_R8Sint: WGPUTextureFormat = 4;
/// Selects `R16_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_R16Unorm: WGPUTextureFormat = 5;
/// Selects `R16_snorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_R16Snorm: WGPUTextureFormat = 6;
/// Selects `R16_uint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_R16Uint: WGPUTextureFormat = 7;
/// Selects `R16_sint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_R16Sint: WGPUTextureFormat = 8;
/// Selects `R16_float` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_R16Float: WGPUTextureFormat = 9;
/// Selects `RG8_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RG8Unorm: WGPUTextureFormat = 10;
/// Selects `RG8_snorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RG8Snorm: WGPUTextureFormat = 11;
/// Selects `RG8_uint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RG8Uint: WGPUTextureFormat = 12;
/// Selects `RG8_sint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RG8Sint: WGPUTextureFormat = 13;
/// Selects `R32_float` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_R32Float: WGPUTextureFormat = 14;
/// Selects `R32_uint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_R32Uint: WGPUTextureFormat = 15;
/// Selects `R32_sint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_R32Sint: WGPUTextureFormat = 16;
/// Selects `RG16_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RG16Unorm: WGPUTextureFormat = 17;
/// Selects `RG16_snorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RG16Snorm: WGPUTextureFormat = 18;
/// Selects `RG16_uint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RG16Uint: WGPUTextureFormat = 19;
/// Selects `RG16_sint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RG16Sint: WGPUTextureFormat = 20;
/// Selects `RG16_float` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RG16Float: WGPUTextureFormat = 21;
/// Selects `RGBA8_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RGBA8Unorm: WGPUTextureFormat = 22;
/// Selects `RGBA8_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RGBA8UnormSrgb: WGPUTextureFormat = 23;
/// Selects `RGBA8_snorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RGBA8Snorm: WGPUTextureFormat = 24;
/// Selects `RGBA8_uint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RGBA8Uint: WGPUTextureFormat = 25;
/// Selects `RGBA8_sint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RGBA8Sint: WGPUTextureFormat = 26;
/// Selects `BGRA8_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_BGRA8Unorm: WGPUTextureFormat = 27;
/// Selects `BGRA8_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_BGRA8UnormSrgb: WGPUTextureFormat = 28;
/// Selects `RGB10_A2_uint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RGB10A2Uint: WGPUTextureFormat = 29;
/// Selects `RGB10_A2_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RGB10A2Unorm: WGPUTextureFormat = 30;
/// Selects `RG11_B10_ufloat` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RG11B10Ufloat: WGPUTextureFormat = 31;
/// Selects `RGB9_E5_ufloat` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RGB9E5Ufloat: WGPUTextureFormat = 32;
/// Selects `RG32_float` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RG32Float: WGPUTextureFormat = 33;
/// Selects `RG32_uint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RG32Uint: WGPUTextureFormat = 34;
/// Selects `RG32_sint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RG32Sint: WGPUTextureFormat = 35;
/// Selects `RGBA16_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RGBA16Unorm: WGPUTextureFormat = 36;
/// Selects `RGBA16_snorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RGBA16Snorm: WGPUTextureFormat = 37;
/// Selects `RGBA16_uint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RGBA16Uint: WGPUTextureFormat = 38;
/// Selects `RGBA16_sint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RGBA16Sint: WGPUTextureFormat = 39;
/// Selects `RGBA16_float` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RGBA16Float: WGPUTextureFormat = 40;
/// Selects `RGBA32_float` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RGBA32Float: WGPUTextureFormat = 41;
/// Selects `RGBA32_uint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RGBA32Uint: WGPUTextureFormat = 42;
/// Selects `RGBA32_sint` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_RGBA32Sint: WGPUTextureFormat = 43;
/// Selects `stencil8` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_Stencil8: WGPUTextureFormat = 44;
/// Selects `depth16_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_Depth16Unorm: WGPUTextureFormat = 45;
/// Selects `depth24_plus` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_Depth24Plus: WGPUTextureFormat = 46;
/// Selects `depth24_plus_stencil8` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_Depth24PlusStencil8: WGPUTextureFormat = 47;
/// Selects `depth32_float` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_Depth32Float: WGPUTextureFormat = 48;
/// Selects `depth32_float_stencil8` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_Depth32FloatStencil8: WGPUTextureFormat = 49;
/// Selects `BC1_RGBA_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_BC1RGBAUnorm: WGPUTextureFormat = 50;
/// Selects `BC1_RGBA_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_BC1RGBAUnormSrgb: WGPUTextureFormat = 51;
/// Selects `BC2_RGBA_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_BC2RGBAUnorm: WGPUTextureFormat = 52;
/// Selects `BC2_RGBA_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_BC2RGBAUnormSrgb: WGPUTextureFormat = 53;
/// Selects `BC3_RGBA_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_BC3RGBAUnorm: WGPUTextureFormat = 54;
/// Selects `BC3_RGBA_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_BC3RGBAUnormSrgb: WGPUTextureFormat = 55;
/// Selects `BC4_R_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_BC4RUnorm: WGPUTextureFormat = 56;
/// Selects `BC4_R_snorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_BC4RSnorm: WGPUTextureFormat = 57;
/// Selects `BC5_RG_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_BC5RGUnorm: WGPUTextureFormat = 58;
/// Selects `BC5_RG_snorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_BC5RGSnorm: WGPUTextureFormat = 59;
/// Selects `BC6H_RGB_ufloat` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_BC6HRGBUfloat: WGPUTextureFormat = 60;
/// Selects `BC6H_RGB_float` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_BC6HRGBFloat: WGPUTextureFormat = 61;
/// Selects `BC7_RGBA_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_BC7RGBAUnorm: WGPUTextureFormat = 62;
/// Selects `BC7_RGBA_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_BC7RGBAUnormSrgb: WGPUTextureFormat = 63;
/// Selects `ETC2_RGB8_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ETC2RGB8Unorm: WGPUTextureFormat = 64;
/// Selects `ETC2_RGB8_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ETC2RGB8UnormSrgb: WGPUTextureFormat = 65;
/// Selects `ETC2_RGB8A1_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ETC2RGB8A1Unorm: WGPUTextureFormat = 66;
/// Selects `ETC2_RGB8A1_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ETC2RGB8A1UnormSrgb: WGPUTextureFormat = 67;
/// Selects `ETC2_RGBA8_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ETC2RGBA8Unorm: WGPUTextureFormat = 68;
/// Selects `ETC2_RGBA8_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ETC2RGBA8UnormSrgb: WGPUTextureFormat = 69;
/// Selects `EAC_R11_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_EACR11Unorm: WGPUTextureFormat = 70;
/// Selects `EAC_R11_snorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_EACR11Snorm: WGPUTextureFormat = 71;
/// Selects `EAC_RG11_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_EACRG11Unorm: WGPUTextureFormat = 72;
/// Selects `EAC_RG11_snorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_EACRG11Snorm: WGPUTextureFormat = 73;
/// Selects `ASTC_4x4_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC4x4Unorm: WGPUTextureFormat = 74;
/// Selects `ASTC_4x4_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC4x4UnormSrgb: WGPUTextureFormat = 75;
/// Selects `ASTC_5x4_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC5x4Unorm: WGPUTextureFormat = 76;
/// Selects `ASTC_5x4_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC5x4UnormSrgb: WGPUTextureFormat = 77;
/// Selects `ASTC_5x5_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC5x5Unorm: WGPUTextureFormat = 78;
/// Selects `ASTC_5x5_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC5x5UnormSrgb: WGPUTextureFormat = 79;
/// Selects `ASTC_6x5_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC6x5Unorm: WGPUTextureFormat = 80;
/// Selects `ASTC_6x5_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC6x5UnormSrgb: WGPUTextureFormat = 81;
/// Selects `ASTC_6x6_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC6x6Unorm: WGPUTextureFormat = 82;
/// Selects `ASTC_6x6_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC6x6UnormSrgb: WGPUTextureFormat = 83;
/// Selects `ASTC_8x5_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC8x5Unorm: WGPUTextureFormat = 84;
/// Selects `ASTC_8x5_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC8x5UnormSrgb: WGPUTextureFormat = 85;
/// Selects `ASTC_8x6_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC8x6Unorm: WGPUTextureFormat = 86;
/// Selects `ASTC_8x6_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC8x6UnormSrgb: WGPUTextureFormat = 87;
/// Selects `ASTC_8x8_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC8x8Unorm: WGPUTextureFormat = 88;
/// Selects `ASTC_8x8_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC8x8UnormSrgb: WGPUTextureFormat = 89;
/// Selects `ASTC_10x5_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC10x5Unorm: WGPUTextureFormat = 90;
/// Selects `ASTC_10x5_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC10x5UnormSrgb: WGPUTextureFormat = 91;
/// Selects `ASTC_10x6_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC10x6Unorm: WGPUTextureFormat = 92;
/// Selects `ASTC_10x6_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC10x6UnormSrgb: WGPUTextureFormat = 93;
/// Selects `ASTC_10x8_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC10x8Unorm: WGPUTextureFormat = 94;
/// Selects `ASTC_10x8_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC10x8UnormSrgb: WGPUTextureFormat = 95;
/// Selects `ASTC_10x10_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC10x10Unorm: WGPUTextureFormat = 96;
/// Selects `ASTC_10x10_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC10x10UnormSrgb: WGPUTextureFormat = 97;
/// Selects `ASTC_12x10_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC12x10Unorm: WGPUTextureFormat = 98;
/// Selects `ASTC_12x10_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC12x10UnormSrgb: WGPUTextureFormat = 99;
/// Selects `ASTC_12x12_unorm` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC12x12Unorm: WGPUTextureFormat = 100;
/// Selects `ASTC_12x12_unorm_srgb` for [`WGPUTextureFormat`].
pub const WGPUTextureFormat_ASTC12x12UnormSrgb: WGPUTextureFormat = 101;

/// The native texture usage bit mask for the C ABI.
pub type WGPUTextureUsage = u64;
/// The `none` mask for [`WGPUTextureUsage`].
pub const WGPUTextureUsage_None: WGPUTextureUsage = 0;
/// The `copy_src` mask for [`WGPUTextureUsage`].
pub const WGPUTextureUsage_CopySrc: WGPUTextureUsage = 1;
/// The `copy_dst` mask for [`WGPUTextureUsage`].
pub const WGPUTextureUsage_CopyDst: WGPUTextureUsage = 2;
/// The `texture_binding` mask for [`WGPUTextureUsage`].
pub const WGPUTextureUsage_TextureBinding: WGPUTextureUsage = 4;
/// The `storage_binding` mask for [`WGPUTextureUsage`].
pub const WGPUTextureUsage_StorageBinding: WGPUTextureUsage = 8;
/// The `render_attachment` mask for [`WGPUTextureUsage`].
pub const WGPUTextureUsage_RenderAttachment: WGPUTextureUsage = 16;
/// The `transient_attachment` mask for [`WGPUTextureUsage`].
pub const WGPUTextureUsage_TransientAttachment: WGPUTextureUsage = 32;

#[repr(C)]
#[derive(Clone, Copy)]
/// The common header for a native descriptor extension.
pub struct WGPUChainedStruct {
    /// The next extension, or null at the end of the chain.
    pub next: *mut WGPUChainedStruct,
    /// The type tag that identifies the extension structure.
    pub sType: WGPUSType,
}

#[repr(C)]
#[derive(Clone, Copy)]
/// A borrowed UTF-8 string for the native C ABI.
pub struct WGPUStringView {
    /// The string bytes, or null for an absent string.
    pub data: *const c_char,
    /// The byte count, or `usize::MAX` for a null-terminated string.
    pub length: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
/// The native surface descriptor data with the pinned C field layout.
pub struct WGPUSurfaceDescriptor {
    /// The first extension, or null when no extension exists.
    pub nextInChain: *mut WGPUChainedStruct,
    /// The diagnostic label for the surface.
    pub label: WGPUStringView,
}

#[repr(C)]
#[derive(Clone, Copy)]
/// The native surface source android native window data with the pinned C field layout.
pub struct WGPUSurfaceSourceAndroidNativeWindow {
    /// The extension header that identifies this structure and the next extension.
    pub chain: WGPUChainedStruct,
    /// The native window that receives the surface output.
    pub window: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
/// The native surface source metal layer data with the pinned C field layout.
pub struct WGPUSurfaceSourceMetalLayer {
    /// The extension header that identifies this structure and the next extension.
    pub chain: WGPUChainedStruct,
    /// The Metal layer that receives the surface output.
    pub layer: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
/// The native surface source wayland surface data with the pinned C field layout.
pub struct WGPUSurfaceSourceWaylandSurface {
    /// The extension header that identifies this structure and the next extension.
    pub chain: WGPUChainedStruct,
    /// The native display connection that owns the surface.
    pub display: *mut c_void,
    /// The Wayland surface that receives the output.
    pub surface: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
/// The native surface source windows HWND data with the pinned C field layout.
pub struct WGPUSurfaceSourceWindowsHWND {
    /// The extension header that identifies this structure and the next extension.
    pub chain: WGPUChainedStruct,
    /// The Windows module instance that owns the window.
    pub hinstance: *mut c_void,
    /// The Windows window handle that receives the output.
    pub hwnd: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
/// The native surface source XCB window data with the pinned C field layout.
pub struct WGPUSurfaceSourceXCBWindow {
    /// The extension header that identifies this structure and the next extension.
    pub chain: WGPUChainedStruct,
    /// The XCB connection that owns the window.
    pub connection: *mut c_void,
    /// The native window that receives the surface output.
    pub window: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
/// The native surface source xlib window data with the pinned C field layout.
pub struct WGPUSurfaceSourceXlibWindow {
    /// The extension header that identifies this structure and the next extension.
    pub chain: WGPUChainedStruct,
    /// The native display connection that owns the surface.
    pub display: *mut c_void,
    /// The native window that receives the surface output.
    pub window: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
/// The native surface configuration data with the pinned C field layout.
pub struct WGPUSurfaceConfiguration {
    /// The first extension, or null when no extension exists.
    pub nextInChain: *mut WGPUChainedStruct,
    /// The device that creates the surface textures.
    pub device: WGPUDevice,
    /// The pixel format of the surface textures.
    pub format: WGPUTextureFormat,
    /// The permitted uses of the surface textures.
    pub usage: WGPUTextureUsage,
    /// The surface texture width in pixels.
    pub width: u32,
    /// The surface texture height in pixels.
    pub height: u32,
    /// The number of elements at `viewFormats`.
    pub viewFormatCount: usize,
    /// The element array with length `viewFormatCount`.
    pub viewFormats: *const WGPUTextureFormat,
    /// The alpha composition mode for presentation.
    pub alphaMode: WGPUCompositeAlphaMode,
    /// The frame presentation mode.
    pub presentMode: WGPUPresentMode,
}

#[repr(C)]
#[derive(Clone, Copy)]
/// The native surface capabilities data with the pinned C field layout.
pub struct WGPUSurfaceCapabilities {
    /// The first extension, or null when no extension exists.
    pub nextInChain: *mut WGPUChainedStruct,
    /// The texture uses that the surface supports.
    pub usages: WGPUTextureUsage,
    /// The number of elements at `formats`.
    pub formatCount: usize,
    /// The element array with length `formatCount`.
    pub formats: *const WGPUTextureFormat,
    /// The number of elements at `presentModes`.
    pub presentModeCount: usize,
    /// The element array with length `presentModeCount`.
    pub presentModes: *const WGPUPresentMode,
    /// The number of elements at `alphaModes`.
    pub alphaModeCount: usize,
    /// The element array with length `alphaModeCount`.
    pub alphaModes: *const WGPUCompositeAlphaMode,
}

#[repr(C)]
#[derive(Clone, Copy)]
/// The native surface texture data with the pinned C field layout.
pub struct WGPUSurfaceTexture {
    /// The first extension, or null when no extension exists.
    pub nextInChain: *mut WGPUChainedStruct,
    /// The current surface texture returned by the backend.
    pub texture: WGPUTexture,
    /// The result of the current surface texture request.
    pub status: WGPUSurfaceGetCurrentTextureStatus,
}

/// The native `wgpuInstanceCreateSurface` entry point with its pinned argument and result ABI.
pub type WGPUProcInstanceCreateSurface = unsafe extern "C" fn(WGPUInstance, *const WGPUSurfaceDescriptor) -> WGPUSurface;
/// The native `wgpuSurfaceConfigure` entry point with its pinned argument and result ABI.
pub type WGPUProcSurfaceConfigure = unsafe extern "C" fn(WGPUSurface, *const WGPUSurfaceConfiguration);
/// The native `wgpuSurfaceUnconfigure` entry point with its pinned argument and result ABI.
pub type WGPUProcSurfaceUnconfigure = unsafe extern "C" fn(WGPUSurface);
/// The native `wgpuSurfaceGetCapabilities` entry point with its pinned argument and result ABI.
pub type WGPUProcSurfaceGetCapabilities = unsafe extern "C" fn(WGPUSurface, WGPUAdapter, *mut WGPUSurfaceCapabilities) -> WGPUStatus;
/// The native `wgpuSurfaceCapabilitiesFreeMembers` entry point with its pinned argument and result ABI.
pub type WGPUProcSurfaceCapabilitiesFreeMembers = unsafe extern "C" fn(WGPUSurfaceCapabilities);
/// The native `wgpuSurfaceGetCurrentTexture` entry point with its pinned argument and result ABI.
pub type WGPUProcSurfaceGetCurrentTexture = unsafe extern "C" fn(WGPUSurface, *mut WGPUSurfaceTexture);
/// The native `wgpuSurfacePresent` entry point with its pinned argument and result ABI.
pub type WGPUProcSurfacePresent = unsafe extern "C" fn(WGPUSurface) -> WGPUStatus;
/// The native `wgpuSurfaceAddRef` entry point with its pinned argument and result ABI.
pub type WGPUProcSurfaceAddRef = unsafe extern "C" fn(WGPUSurface);
/// The native `wgpuSurfaceRelease` entry point with its pinned argument and result ABI.
pub type WGPUProcSurfaceRelease = unsafe extern "C" fn(WGPUSurface);
/// The native `wgpuSurfaceSetLabel` entry point with its pinned argument and result ABI.
pub type WGPUProcSurfaceSetLabel = unsafe extern "C" fn(WGPUSurface, WGPUStringView);

/// The surface entry points resolved from the backend library.
pub struct SurfaceTable {
    /// The backend address for `wgpuInstanceCreateSurface`.
    pub wgpuInstanceCreateSurface: WGPUProcInstanceCreateSurface,
    /// The backend address for `wgpuSurfaceConfigure`.
    pub wgpuSurfaceConfigure: WGPUProcSurfaceConfigure,
    /// The backend address for `wgpuSurfaceUnconfigure`.
    pub wgpuSurfaceUnconfigure: WGPUProcSurfaceUnconfigure,
    /// The backend address for `wgpuSurfaceGetCapabilities`.
    pub wgpuSurfaceGetCapabilities: WGPUProcSurfaceGetCapabilities,
    /// The backend address for `wgpuSurfaceCapabilitiesFreeMembers`.
    pub wgpuSurfaceCapabilitiesFreeMembers: WGPUProcSurfaceCapabilitiesFreeMembers,
    /// The backend address for `wgpuSurfaceGetCurrentTexture`.
    pub wgpuSurfaceGetCurrentTexture: WGPUProcSurfaceGetCurrentTexture,
    /// The backend address for `wgpuSurfacePresent`.
    pub wgpuSurfacePresent: WGPUProcSurfacePresent,
    /// The backend address for `wgpuSurfaceAddRef`.
    pub wgpuSurfaceAddRef: WGPUProcSurfaceAddRef,
    /// The backend address for `wgpuSurfaceRelease`.
    pub wgpuSurfaceRelease: WGPUProcSurfaceRelease,
    /// The backend address for `wgpuSurfaceSetLabel`.
    pub wgpuSurfaceSetLabel: WGPUProcSurfaceSetLabel,
}

static SURFACE_TABLE: OnceLock<SurfaceTable> = OnceLock::new();

/// Returns the cached surface entry points, or resolves them from the configured backend library.
/// Returns an error if the backend or a required symbol cannot load, or the table cannot initialize.
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
