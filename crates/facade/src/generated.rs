//! GENERATED FILE — DO NOT EDIT.
//!
//! `tools/regen.sh` emits this file from the pinned inputs and policy.
//! The regeneration test compares the committed bytes.
//!
//! Future and lost callbacks use AllowProcessEvents.
//! The uncaptured-error callback only records copied data.
//! Backend handle cleanup occurs after each callback returns.

#![allow(non_snake_case, non_upper_case_globals)]

#![allow(clippy::not_unsafe_ptr_arg_deref, clippy::vec_box)]

use std::ffi::{c_char, c_void, CStr};

use crate::runtime;

// ---------------------------------------------------------------------
// webgpu.h FFI subset (emitted from webgpu.yml for the policy subset;
// no rust-bindgen). webgpu.h names stay private to this module.
// ---------------------------------------------------------------------

/// Declares opaque handle/pointer-only types (never dereferenced
/// here). The caller supplies the visibility: the WGPU set stays
/// `pub(crate)` (webgpu.h internals never enter the crate's public
/// API); only the `SubscriptTypegpu*` set is `pub`.
macro_rules! opaque {
    ($vis:vis $($name:ident),* $(,)?) => {
        $(
            #[repr(C)]
            #[doc = "Opaque handle type (never dereferenced here)."]
            $vis struct $name {
                _private: [u8; 0],
            }
        )*
    };
}

opaque!(
    pub(crate)
    WGPUInstanceImpl,
    WGPUAdapterImpl,
    WGPUDeviceImpl,
    WGPUQueueImpl,
    WGPUBufferImpl,
    WGPUTextureImpl,
    WGPUTextureViewImpl,
    WGPUSamplerImpl,
    WGPUBindGroupLayoutImpl,
    WGPUBindGroupImpl,
    WGPUPipelineLayoutImpl,
    WGPUShaderModuleImpl,
    WGPUComputePipelineImpl,
    WGPURenderPipelineImpl,
    WGPUCommandEncoderImpl,
    WGPUComputePassEncoderImpl,
    WGPURenderPassEncoderImpl,
    WGPUCommandBufferImpl,
    WGPURenderBundleEncoderImpl,
    WGPURenderBundleImpl,
    WGPUQuerySetImpl,
    // Pointer-only in this subset (always null / never built):
    WGPURequestAdapterOptions,
);

type WGPUInstance = *mut WGPUInstanceImpl;
type WGPUAdapter = *mut WGPUAdapterImpl;
type WGPUDevice = *mut WGPUDeviceImpl;
type WGPUQueue = *mut WGPUQueueImpl;
type WGPUBuffer = *mut WGPUBufferImpl;
type WGPUTexture = *mut WGPUTextureImpl;
type WGPUTextureView = *mut WGPUTextureViewImpl;
type WGPUSampler = *mut WGPUSamplerImpl;
type WGPUBindGroupLayout = *mut WGPUBindGroupLayoutImpl;
type WGPUBindGroup = *mut WGPUBindGroupImpl;
type WGPUPipelineLayout = *mut WGPUPipelineLayoutImpl;
type WGPUShaderModule = *mut WGPUShaderModuleImpl;
type WGPUComputePipeline = *mut WGPUComputePipelineImpl;
type WGPURenderPipeline = *mut WGPURenderPipelineImpl;
type WGPUCommandEncoder = *mut WGPUCommandEncoderImpl;
type WGPUComputePassEncoder = *mut WGPUComputePassEncoderImpl;
type WGPURenderPassEncoder = *mut WGPURenderPassEncoderImpl;
type WGPUCommandBuffer = *mut WGPUCommandBufferImpl;
type WGPURenderBundleEncoder = *mut WGPURenderBundleEncoderImpl;
type WGPURenderBundle = *mut WGPURenderBundleImpl;
type WGPUQuerySet = *mut WGPUQuerySetImpl;

// Companion header: https://github.com/infosia/yawgpu/blob/main/ffi/webgpu-headers/yawgpu.h
#[repr(C)]
struct YawgpuChainedStruct {
next: *mut YawgpuChainedStruct,
s_type: i32,
}

#[repr(C)]
struct YawgpuInstanceBackendSelect {
chain: YawgpuChainedStruct,
backend: u32,
}

#[repr(C)]
struct WGPUInstanceDescriptor {
next_in_chain: *mut YawgpuChainedStruct,
required_feature_count: usize,
required_features: *const i32,
required_limits: *const WGPUInstanceLimits,
}

const YAWGPU_STYPE_INSTANCE_BACKEND_SELECT: i32 = 0x7000_0001;
#[allow(dead_code)]
const YAWGPU_INSTANCE_BACKEND_NOOP: u32 = 0;
const YAWGPU_INSTANCE_BACKEND_METAL: u32 = 1;
const YAWGPU_INSTANCE_BACKEND_VULKAN: u32 = 2;
const YAWGPU_INSTANCE_BACKEND_GLES: u32 = 3;

/// webgpu.h `WGPUChainedStruct`; concrete for WGSL source construction.
#[repr(C)]
#[derive(Clone, Copy)]
struct WGPUChainedStruct {
    next: *mut WGPUChainedStruct,
    s_type: i32,
}

/// webgpu.h `WGPUShaderSourceWGSL`.
#[repr(C)]
struct WGPUShaderSourceWGSL {
    chain: WGPUChainedStruct,
    code: WGPUStringView,
}

/// webgpu.h `WGPUShaderModuleDescriptor`.
#[repr(C)]
struct WGPUShaderModuleDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
}

/// webgpu.yml `s_type.shader_source_WGSL`.
const WGPUSType_ShaderSourceWGSL: i32 = 0x0000_0002;

/// `subscript-typegpu.h`: borrowed UTF-8 string view.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuStringView {
    /// Pointer to UTF-8 bytes.
    pub data: *const c_char,
    /// Byte length, or `usize::MAX` for null-terminated input.
    pub length: usize,
}

/// webgpu.h `WGPUStringView`.
#[repr(C)]
#[derive(Clone, Copy)]
struct WGPUStringView {
    data: *const c_char,
    length: usize,
}

fn wgpu_string_view(view: SubscriptTypegpuStringView) -> WGPUStringView {
    WGPUStringView {
        data: view.data,
        length: view.length,
    }
}

/// webgpu.yml constant `strlen` (`usize_max`): marks a
/// null-terminated `WGPUStringView`.
const WGPU_STRLEN: usize = usize::MAX;

/// webgpu.h request-device descriptor types used to install F14 callbacks.
type WGPUFeatureName = i32;
type WGPUDeviceLostCallback = Option<
    // SAFETY: the callback signature matches the pinned webgpu.h declaration.
    unsafe extern "C" fn(
        device: *const WGPUDevice,
        reason: i32,
        message: WGPUStringView,
        userdata1: *mut c_void,
        userdata2: *mut c_void,
    ),
>;
#[repr(C)]
struct WGPUDeviceLostCallbackInfo {
    next_in_chain: *mut WGPUChainedStruct,
    mode: i32,
    callback: WGPUDeviceLostCallback,
    userdata1: *mut c_void,
    userdata2: *mut c_void,
}
type WGPUUncapturedErrorCallback = Option<
    // SAFETY: the callback signature matches the pinned webgpu.h declaration.
    unsafe extern "C" fn(
        device: *const WGPUDevice,
        error_type: i32,
        message: WGPUStringView,
        userdata1: *mut c_void,
        userdata2: *mut c_void,
    ),
>;
#[repr(C)]
struct WGPUUncapturedErrorCallbackInfo {
    next_in_chain: *mut WGPUChainedStruct,
    callback: WGPUUncapturedErrorCallback,
    userdata1: *mut c_void,
    userdata2: *mut c_void,
}
#[repr(C)]
struct WGPUDeviceDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
    required_feature_count: usize,
    required_features: *const WGPUFeatureName,
    required_limits: *const WGPULimits,
    default_queue: WGPUQueueDescriptor,
    device_lost_callback_info: WGPUDeviceLostCallbackInfo,
    uncaptured_error_callback_info: WGPUUncapturedErrorCallbackInfo,
}
/// webgpu.h `WGPUPopErrorScopeCallback` callback and info.
type WGPUPopErrorScopeCallback = Option<
    // SAFETY: the callback signature matches the pinned webgpu.h declaration.
    unsafe extern "C" fn(
        status: i32,
        error_type: i32,
        message: WGPUStringView,
        userdata1: *mut c_void,
        userdata2: *mut c_void,
    ),
>;
#[repr(C)]
struct WGPUPopErrorScopeCallbackInfo {
    next_in_chain: *mut WGPUChainedStruct,
    mode: i32,
    callback: WGPUPopErrorScopeCallback,
    userdata1: *mut c_void,
    userdata2: *mut c_void,
}

/// webgpu.h `WGPUAdapterInfo`; the two subgroup fields remain private.
#[repr(C)]
struct WGPUAdapterInfo {
    next_in_chain: *mut WGPUChainedStruct,
    vendor: WGPUStringView,
    architecture: WGPUStringView,
    device: WGPUStringView,
    description: WGPUStringView,
    backend_type: i32,
    adapter_type: i32,
    vendor_id: u32,
    device_id: u32,
    subgroup_min_size: u32,
    subgroup_max_size: u32,
}

/// webgpu.h `WGPUFuture { uint64_t id; }`.
#[repr(C)]
struct WGPUFuture {
    id: u64,
}

/// webgpu.yml enum `callback_mode`, entry `allow_process_events`
/// — the only mode the facade registers (CLAUDE.md invariant 3).
const WGPUCallbackMode_AllowProcessEvents: i32 = 0x0000_0002;

/// webgpu.yml enum value (`success`).
const WGPURequestAdapterStatus_Success: i32 = 0x0000_0001;

/// webgpu.yml enum value (`success`).
const WGPURequestDeviceStatus_Success: i32 = 0x0000_0001;

/// webgpu.yml enum value (`success`).
const WGPUCreatePipelineAsyncStatus_Success: i32 = 0x0000_0001;

/// webgpu.yml enum value (`success`).
const WGPUQueueWorkDoneStatus_Success: i32 = 0x0000_0001;

/// webgpu.yml enum value (`success`).
const WGPUMapAsyncStatus_Success: i32 = 0x0000_0001;

/// webgpu.yml `bitflag.buffer_usage` value.
#[allow(dead_code)]
const WGPUBufferUsage_None: u64 = 0x0;
/// webgpu.yml `bitflag.buffer_usage` value.
#[allow(dead_code)]
const WGPUBufferUsage_MapRead: u64 = 0x1;
/// webgpu.yml `bitflag.buffer_usage` value.
#[allow(dead_code)]
const WGPUBufferUsage_MapWrite: u64 = 0x2;
/// webgpu.yml `bitflag.buffer_usage` value.
#[allow(dead_code)]
const WGPUBufferUsage_CopySrc: u64 = 0x4;
/// webgpu.yml `bitflag.buffer_usage` value.
#[allow(dead_code)]
const WGPUBufferUsage_CopyDst: u64 = 0x8;
/// webgpu.yml `bitflag.buffer_usage` value.
#[allow(dead_code)]
const WGPUBufferUsage_Index: u64 = 0x10;
/// webgpu.yml `bitflag.buffer_usage` value.
#[allow(dead_code)]
const WGPUBufferUsage_Vertex: u64 = 0x20;
/// webgpu.yml `bitflag.buffer_usage` value.
#[allow(dead_code)]
const WGPUBufferUsage_Uniform: u64 = 0x40;
/// webgpu.yml `bitflag.buffer_usage` value.
#[allow(dead_code)]
const WGPUBufferUsage_Storage: u64 = 0x80;
/// webgpu.yml `bitflag.buffer_usage` value.
#[allow(dead_code)]
const WGPUBufferUsage_Indirect: u64 = 0x100;
/// webgpu.yml `bitflag.buffer_usage` value.
#[allow(dead_code)]
const WGPUBufferUsage_QueryResolve: u64 = 0x200;

/// webgpu.yml `bitflag.map_mode` value.
#[allow(dead_code)]
const WGPUMapMode_None: u64 = 0x0;
/// webgpu.yml `bitflag.map_mode` value.
#[allow(dead_code)]
const WGPUMapMode_Read: u64 = 0x1;
/// webgpu.yml `bitflag.map_mode` value.
#[allow(dead_code)]
const WGPUMapMode_Write: u64 = 0x2;

/// webgpu.yml `enum.buffer_map_state` value.
#[allow(dead_code)]
const WGPUBufferMapState_Unmapped: i32 = 0x0000_0001;
/// webgpu.yml `enum.buffer_map_state` value.
#[allow(dead_code)]
const WGPUBufferMapState_Pending: i32 = 0x0000_0002;
/// webgpu.yml `enum.buffer_map_state` value.
#[allow(dead_code)]
const WGPUBufferMapState_Mapped: i32 = 0x0000_0003;

/// webgpu.yml `bitflag.texture_usage` value.
#[allow(dead_code)]
const WGPUTextureUsage_None: u64 = 0x0;
/// webgpu.yml `bitflag.texture_usage` value.
#[allow(dead_code)]
const WGPUTextureUsage_CopySrc: u64 = 0x1;
/// webgpu.yml `bitflag.texture_usage` value.
#[allow(dead_code)]
const WGPUTextureUsage_CopyDst: u64 = 0x2;
/// webgpu.yml `bitflag.texture_usage` value.
#[allow(dead_code)]
const WGPUTextureUsage_TextureBinding: u64 = 0x4;
/// webgpu.yml `bitflag.texture_usage` value.
#[allow(dead_code)]
const WGPUTextureUsage_StorageBinding: u64 = 0x8;
/// webgpu.yml `bitflag.texture_usage` value.
#[allow(dead_code)]
const WGPUTextureUsage_RenderAttachment: u64 = 0x10;
/// webgpu.yml `bitflag.texture_usage` value.
#[allow(dead_code)]
const WGPUTextureUsage_TransientAttachment: u64 = 0x20;

/// webgpu.yml `enum.texture_dimension` value.
#[allow(dead_code)]
const WGPUTextureDimension_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.texture_dimension` value.
#[allow(dead_code)]
const WGPUTextureDimension_1D: i32 = 0x0000_0001;
/// webgpu.yml `enum.texture_dimension` value.
#[allow(dead_code)]
const WGPUTextureDimension_2D: i32 = 0x0000_0002;
/// webgpu.yml `enum.texture_dimension` value.
#[allow(dead_code)]
const WGPUTextureDimension_3D: i32 = 0x0000_0003;

/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_R8Unorm: i32 = 0x0000_0001;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_R8Snorm: i32 = 0x0000_0002;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_R8Uint: i32 = 0x0000_0003;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_R8Sint: i32 = 0x0000_0004;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_R16Unorm: i32 = 0x0000_0005;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_R16Snorm: i32 = 0x0000_0006;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_R16Uint: i32 = 0x0000_0007;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_R16Sint: i32 = 0x0000_0008;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_R16Float: i32 = 0x0000_0009;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RG8Unorm: i32 = 0x0000_000A;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RG8Snorm: i32 = 0x0000_000B;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RG8Uint: i32 = 0x0000_000C;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RG8Sint: i32 = 0x0000_000D;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_R32Float: i32 = 0x0000_000E;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_R32Uint: i32 = 0x0000_000F;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_R32Sint: i32 = 0x0000_0010;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RG16Unorm: i32 = 0x0000_0011;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RG16Snorm: i32 = 0x0000_0012;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RG16Uint: i32 = 0x0000_0013;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RG16Sint: i32 = 0x0000_0014;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RG16Float: i32 = 0x0000_0015;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RGBA8Unorm: i32 = 0x0000_0016;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RGBA8UnormSrgb: i32 = 0x0000_0017;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RGBA8Snorm: i32 = 0x0000_0018;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RGBA8Uint: i32 = 0x0000_0019;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RGBA8Sint: i32 = 0x0000_001A;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_BGRA8Unorm: i32 = 0x0000_001B;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_BGRA8UnormSrgb: i32 = 0x0000_001C;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RGB10A2Uint: i32 = 0x0000_001D;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RGB10A2Unorm: i32 = 0x0000_001E;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RG11B10Ufloat: i32 = 0x0000_001F;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RGB9E5Ufloat: i32 = 0x0000_0020;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RG32Float: i32 = 0x0000_0021;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RG32Uint: i32 = 0x0000_0022;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RG32Sint: i32 = 0x0000_0023;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RGBA16Unorm: i32 = 0x0000_0024;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RGBA16Snorm: i32 = 0x0000_0025;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RGBA16Uint: i32 = 0x0000_0026;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RGBA16Sint: i32 = 0x0000_0027;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RGBA16Float: i32 = 0x0000_0028;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RGBA32Float: i32 = 0x0000_0029;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RGBA32Uint: i32 = 0x0000_002A;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_RGBA32Sint: i32 = 0x0000_002B;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_Stencil8: i32 = 0x0000_002C;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_Depth16Unorm: i32 = 0x0000_002D;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_Depth24Plus: i32 = 0x0000_002E;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_Depth24PlusStencil8: i32 = 0x0000_002F;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_Depth32Float: i32 = 0x0000_0030;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_Depth32FloatStencil8: i32 = 0x0000_0031;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_BC1RGBAUnorm: i32 = 0x0000_0032;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_BC1RGBAUnormSrgb: i32 = 0x0000_0033;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_BC2RGBAUnorm: i32 = 0x0000_0034;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_BC2RGBAUnormSrgb: i32 = 0x0000_0035;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_BC3RGBAUnorm: i32 = 0x0000_0036;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_BC3RGBAUnormSrgb: i32 = 0x0000_0037;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_BC4RUnorm: i32 = 0x0000_0038;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_BC4RSnorm: i32 = 0x0000_0039;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_BC5RGUnorm: i32 = 0x0000_003A;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_BC5RGSnorm: i32 = 0x0000_003B;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_BC6HRGBUfloat: i32 = 0x0000_003C;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_BC6HRGBFloat: i32 = 0x0000_003D;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_BC7RGBAUnorm: i32 = 0x0000_003E;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_BC7RGBAUnormSrgb: i32 = 0x0000_003F;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ETC2RGB8Unorm: i32 = 0x0000_0040;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ETC2RGB8UnormSrgb: i32 = 0x0000_0041;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ETC2RGB8A1Unorm: i32 = 0x0000_0042;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ETC2RGB8A1UnormSrgb: i32 = 0x0000_0043;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ETC2RGBA8Unorm: i32 = 0x0000_0044;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ETC2RGBA8UnormSrgb: i32 = 0x0000_0045;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_EACR11Unorm: i32 = 0x0000_0046;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_EACR11Snorm: i32 = 0x0000_0047;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_EACRG11Unorm: i32 = 0x0000_0048;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_EACRG11Snorm: i32 = 0x0000_0049;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC4x4Unorm: i32 = 0x0000_004A;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC4x4UnormSrgb: i32 = 0x0000_004B;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC5x4Unorm: i32 = 0x0000_004C;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC5x4UnormSrgb: i32 = 0x0000_004D;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC5x5Unorm: i32 = 0x0000_004E;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC5x5UnormSrgb: i32 = 0x0000_004F;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC6x5Unorm: i32 = 0x0000_0050;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC6x5UnormSrgb: i32 = 0x0000_0051;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC6x6Unorm: i32 = 0x0000_0052;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC6x6UnormSrgb: i32 = 0x0000_0053;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC8x5Unorm: i32 = 0x0000_0054;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC8x5UnormSrgb: i32 = 0x0000_0055;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC8x6Unorm: i32 = 0x0000_0056;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC8x6UnormSrgb: i32 = 0x0000_0057;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC8x8Unorm: i32 = 0x0000_0058;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC8x8UnormSrgb: i32 = 0x0000_0059;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC10x5Unorm: i32 = 0x0000_005A;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC10x5UnormSrgb: i32 = 0x0000_005B;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC10x6Unorm: i32 = 0x0000_005C;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC10x6UnormSrgb: i32 = 0x0000_005D;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC10x8Unorm: i32 = 0x0000_005E;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC10x8UnormSrgb: i32 = 0x0000_005F;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC10x10Unorm: i32 = 0x0000_0060;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC10x10UnormSrgb: i32 = 0x0000_0061;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC12x10Unorm: i32 = 0x0000_0062;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC12x10UnormSrgb: i32 = 0x0000_0063;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC12x12Unorm: i32 = 0x0000_0064;
/// webgpu.yml `enum.texture_format` value.
#[allow(dead_code)]
const WGPUTextureFormat_ASTC12x12UnormSrgb: i32 = 0x0000_0065;

/// webgpu.yml `enum.texture_view_dimension` value.
#[allow(dead_code)]
const WGPUTextureViewDimension_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.texture_view_dimension` value.
#[allow(dead_code)]
const WGPUTextureViewDimension_1D: i32 = 0x0000_0001;
/// webgpu.yml `enum.texture_view_dimension` value.
#[allow(dead_code)]
const WGPUTextureViewDimension_2D: i32 = 0x0000_0002;
/// webgpu.yml `enum.texture_view_dimension` value.
#[allow(dead_code)]
const WGPUTextureViewDimension_2DArray: i32 = 0x0000_0003;
/// webgpu.yml `enum.texture_view_dimension` value.
#[allow(dead_code)]
const WGPUTextureViewDimension_Cube: i32 = 0x0000_0004;
/// webgpu.yml `enum.texture_view_dimension` value.
#[allow(dead_code)]
const WGPUTextureViewDimension_CubeArray: i32 = 0x0000_0005;
/// webgpu.yml `enum.texture_view_dimension` value.
#[allow(dead_code)]
const WGPUTextureViewDimension_3D: i32 = 0x0000_0006;

/// webgpu.yml `enum.texture_aspect` value.
#[allow(dead_code)]
const WGPUTextureAspect_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.texture_aspect` value.
#[allow(dead_code)]
const WGPUTextureAspect_All: i32 = 0x0000_0001;
/// webgpu.yml `enum.texture_aspect` value.
#[allow(dead_code)]
const WGPUTextureAspect_StencilOnly: i32 = 0x0000_0002;
/// webgpu.yml `enum.texture_aspect` value.
#[allow(dead_code)]
const WGPUTextureAspect_DepthOnly: i32 = 0x0000_0003;

/// webgpu.yml `enum.address_mode` value.
#[allow(dead_code)]
const WGPUAddressMode_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.address_mode` value.
#[allow(dead_code)]
const WGPUAddressMode_ClampToEdge: i32 = 0x0000_0001;
/// webgpu.yml `enum.address_mode` value.
#[allow(dead_code)]
const WGPUAddressMode_Repeat: i32 = 0x0000_0002;
/// webgpu.yml `enum.address_mode` value.
#[allow(dead_code)]
const WGPUAddressMode_MirrorRepeat: i32 = 0x0000_0003;

/// webgpu.yml `enum.filter_mode` value.
#[allow(dead_code)]
const WGPUFilterMode_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.filter_mode` value.
#[allow(dead_code)]
const WGPUFilterMode_Nearest: i32 = 0x0000_0001;
/// webgpu.yml `enum.filter_mode` value.
#[allow(dead_code)]
const WGPUFilterMode_Linear: i32 = 0x0000_0002;

/// webgpu.yml `enum.mipmap_filter_mode` value.
#[allow(dead_code)]
const WGPUMipmapFilterMode_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.mipmap_filter_mode` value.
#[allow(dead_code)]
const WGPUMipmapFilterMode_Nearest: i32 = 0x0000_0001;
/// webgpu.yml `enum.mipmap_filter_mode` value.
#[allow(dead_code)]
const WGPUMipmapFilterMode_Linear: i32 = 0x0000_0002;

/// webgpu.yml `enum.compare_function` value.
#[allow(dead_code)]
const WGPUCompareFunction_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.compare_function` value.
#[allow(dead_code)]
const WGPUCompareFunction_Never: i32 = 0x0000_0001;
/// webgpu.yml `enum.compare_function` value.
#[allow(dead_code)]
const WGPUCompareFunction_Less: i32 = 0x0000_0002;
/// webgpu.yml `enum.compare_function` value.
#[allow(dead_code)]
const WGPUCompareFunction_Equal: i32 = 0x0000_0003;
/// webgpu.yml `enum.compare_function` value.
#[allow(dead_code)]
const WGPUCompareFunction_LessEqual: i32 = 0x0000_0004;
/// webgpu.yml `enum.compare_function` value.
#[allow(dead_code)]
const WGPUCompareFunction_Greater: i32 = 0x0000_0005;
/// webgpu.yml `enum.compare_function` value.
#[allow(dead_code)]
const WGPUCompareFunction_NotEqual: i32 = 0x0000_0006;
/// webgpu.yml `enum.compare_function` value.
#[allow(dead_code)]
const WGPUCompareFunction_GreaterEqual: i32 = 0x0000_0007;
/// webgpu.yml `enum.compare_function` value.
#[allow(dead_code)]
const WGPUCompareFunction_Always: i32 = 0x0000_0008;

/// webgpu.yml `bitflag.shader_stage` value.
#[allow(dead_code)]
const WGPUShaderStage_None: u64 = 0x0;
/// webgpu.yml `bitflag.shader_stage` value.
#[allow(dead_code)]
const WGPUShaderStage_Vertex: u64 = 0x1;
/// webgpu.yml `bitflag.shader_stage` value.
#[allow(dead_code)]
const WGPUShaderStage_Fragment: u64 = 0x2;
/// webgpu.yml `bitflag.shader_stage` value.
#[allow(dead_code)]
const WGPUShaderStage_Compute: u64 = 0x4;

/// webgpu.yml `enum.buffer_binding_type` value.
#[allow(dead_code)]
const WGPUBufferBindingType_BindingNotUsed: i32 = 0x0000_0000;
/// webgpu.yml `enum.buffer_binding_type` value.
#[allow(dead_code)]
const WGPUBufferBindingType_Undefined: i32 = 0x0000_0001;
/// webgpu.yml `enum.buffer_binding_type` value.
#[allow(dead_code)]
const WGPUBufferBindingType_Uniform: i32 = 0x0000_0002;
/// webgpu.yml `enum.buffer_binding_type` value.
#[allow(dead_code)]
const WGPUBufferBindingType_Storage: i32 = 0x0000_0003;
/// webgpu.yml `enum.buffer_binding_type` value.
#[allow(dead_code)]
const WGPUBufferBindingType_ReadOnlyStorage: i32 = 0x0000_0004;

/// webgpu.yml `enum.sampler_binding_type` value.
#[allow(dead_code)]
const WGPUSamplerBindingType_BindingNotUsed: i32 = 0x0000_0000;
/// webgpu.yml `enum.sampler_binding_type` value.
#[allow(dead_code)]
const WGPUSamplerBindingType_Undefined: i32 = 0x0000_0001;
/// webgpu.yml `enum.sampler_binding_type` value.
#[allow(dead_code)]
const WGPUSamplerBindingType_Filtering: i32 = 0x0000_0002;
/// webgpu.yml `enum.sampler_binding_type` value.
#[allow(dead_code)]
const WGPUSamplerBindingType_NonFiltering: i32 = 0x0000_0003;
/// webgpu.yml `enum.sampler_binding_type` value.
#[allow(dead_code)]
const WGPUSamplerBindingType_Comparison: i32 = 0x0000_0004;

/// webgpu.yml `enum.texture_sample_type` value.
#[allow(dead_code)]
const WGPUTextureSampleType_BindingNotUsed: i32 = 0x0000_0000;
/// webgpu.yml `enum.texture_sample_type` value.
#[allow(dead_code)]
const WGPUTextureSampleType_Undefined: i32 = 0x0000_0001;
/// webgpu.yml `enum.texture_sample_type` value.
#[allow(dead_code)]
const WGPUTextureSampleType_Float: i32 = 0x0000_0002;
/// webgpu.yml `enum.texture_sample_type` value.
#[allow(dead_code)]
const WGPUTextureSampleType_UnfilterableFloat: i32 = 0x0000_0003;
/// webgpu.yml `enum.texture_sample_type` value.
#[allow(dead_code)]
const WGPUTextureSampleType_Depth: i32 = 0x0000_0004;
/// webgpu.yml `enum.texture_sample_type` value.
#[allow(dead_code)]
const WGPUTextureSampleType_Sint: i32 = 0x0000_0005;
/// webgpu.yml `enum.texture_sample_type` value.
#[allow(dead_code)]
const WGPUTextureSampleType_Uint: i32 = 0x0000_0006;

/// webgpu.yml `enum.storage_texture_access` value.
#[allow(dead_code)]
const WGPUStorageTextureAccess_BindingNotUsed: i32 = 0x0000_0000;
/// webgpu.yml `enum.storage_texture_access` value.
#[allow(dead_code)]
const WGPUStorageTextureAccess_Undefined: i32 = 0x0000_0001;
/// webgpu.yml `enum.storage_texture_access` value.
#[allow(dead_code)]
const WGPUStorageTextureAccess_WriteOnly: i32 = 0x0000_0002;
/// webgpu.yml `enum.storage_texture_access` value.
#[allow(dead_code)]
const WGPUStorageTextureAccess_ReadOnly: i32 = 0x0000_0003;
/// webgpu.yml `enum.storage_texture_access` value.
#[allow(dead_code)]
const WGPUStorageTextureAccess_ReadWrite: i32 = 0x0000_0004;

/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Uint8: i32 = 0x0000_0001;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Uint8x2: i32 = 0x0000_0002;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Uint8x4: i32 = 0x0000_0003;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Sint8: i32 = 0x0000_0004;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Sint8x2: i32 = 0x0000_0005;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Sint8x4: i32 = 0x0000_0006;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Unorm8: i32 = 0x0000_0007;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Unorm8x2: i32 = 0x0000_0008;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Unorm8x4: i32 = 0x0000_0009;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Snorm8: i32 = 0x0000_000A;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Snorm8x2: i32 = 0x0000_000B;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Snorm8x4: i32 = 0x0000_000C;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Uint16: i32 = 0x0000_000D;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Uint16x2: i32 = 0x0000_000E;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Uint16x4: i32 = 0x0000_000F;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Sint16: i32 = 0x0000_0010;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Sint16x2: i32 = 0x0000_0011;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Sint16x4: i32 = 0x0000_0012;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Unorm16: i32 = 0x0000_0013;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Unorm16x2: i32 = 0x0000_0014;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Unorm16x4: i32 = 0x0000_0015;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Snorm16: i32 = 0x0000_0016;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Snorm16x2: i32 = 0x0000_0017;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Snorm16x4: i32 = 0x0000_0018;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Float16: i32 = 0x0000_0019;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Float16x2: i32 = 0x0000_001A;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Float16x4: i32 = 0x0000_001B;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Float32: i32 = 0x0000_001C;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Float32x2: i32 = 0x0000_001D;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Float32x3: i32 = 0x0000_001E;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Float32x4: i32 = 0x0000_001F;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Uint32: i32 = 0x0000_0020;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Uint32x2: i32 = 0x0000_0021;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Uint32x3: i32 = 0x0000_0022;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Uint32x4: i32 = 0x0000_0023;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Sint32: i32 = 0x0000_0024;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Sint32x2: i32 = 0x0000_0025;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Sint32x3: i32 = 0x0000_0026;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Sint32x4: i32 = 0x0000_0027;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Unorm10_10_10_2: i32 = 0x0000_0028;
/// webgpu.yml `enum.vertex_format` value.
#[allow(dead_code)]
const WGPUVertexFormat_Unorm8x4BGRA: i32 = 0x0000_0029;

/// webgpu.yml `enum.vertex_step_mode` value.
#[allow(dead_code)]
const WGPUVertexStepMode_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.vertex_step_mode` value.
#[allow(dead_code)]
const WGPUVertexStepMode_Vertex: i32 = 0x0000_0001;
/// webgpu.yml `enum.vertex_step_mode` value.
#[allow(dead_code)]
const WGPUVertexStepMode_Instance: i32 = 0x0000_0002;

/// webgpu.yml `enum.primitive_topology` value.
#[allow(dead_code)]
const WGPUPrimitiveTopology_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.primitive_topology` value.
#[allow(dead_code)]
const WGPUPrimitiveTopology_PointList: i32 = 0x0000_0001;
/// webgpu.yml `enum.primitive_topology` value.
#[allow(dead_code)]
const WGPUPrimitiveTopology_LineList: i32 = 0x0000_0002;
/// webgpu.yml `enum.primitive_topology` value.
#[allow(dead_code)]
const WGPUPrimitiveTopology_LineStrip: i32 = 0x0000_0003;
/// webgpu.yml `enum.primitive_topology` value.
#[allow(dead_code)]
const WGPUPrimitiveTopology_TriangleList: i32 = 0x0000_0004;
/// webgpu.yml `enum.primitive_topology` value.
#[allow(dead_code)]
const WGPUPrimitiveTopology_TriangleStrip: i32 = 0x0000_0005;

/// webgpu.yml `enum.index_format` value.
#[allow(dead_code)]
const WGPUIndexFormat_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.index_format` value.
#[allow(dead_code)]
const WGPUIndexFormat_Uint16: i32 = 0x0000_0001;
/// webgpu.yml `enum.index_format` value.
#[allow(dead_code)]
const WGPUIndexFormat_Uint32: i32 = 0x0000_0002;

/// webgpu.yml `enum.front_face` value.
#[allow(dead_code)]
const WGPUFrontFace_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.front_face` value.
#[allow(dead_code)]
const WGPUFrontFace_CCW: i32 = 0x0000_0001;
/// webgpu.yml `enum.front_face` value.
#[allow(dead_code)]
const WGPUFrontFace_CW: i32 = 0x0000_0002;

/// webgpu.yml `enum.cull_mode` value.
#[allow(dead_code)]
const WGPUCullMode_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.cull_mode` value.
#[allow(dead_code)]
const WGPUCullMode_None: i32 = 0x0000_0001;
/// webgpu.yml `enum.cull_mode` value.
#[allow(dead_code)]
const WGPUCullMode_Front: i32 = 0x0000_0002;
/// webgpu.yml `enum.cull_mode` value.
#[allow(dead_code)]
const WGPUCullMode_Back: i32 = 0x0000_0003;

/// webgpu.yml `enum.blend_operation` value.
#[allow(dead_code)]
const WGPUBlendOperation_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.blend_operation` value.
#[allow(dead_code)]
const WGPUBlendOperation_Add: i32 = 0x0000_0001;
/// webgpu.yml `enum.blend_operation` value.
#[allow(dead_code)]
const WGPUBlendOperation_Subtract: i32 = 0x0000_0002;
/// webgpu.yml `enum.blend_operation` value.
#[allow(dead_code)]
const WGPUBlendOperation_ReverseSubtract: i32 = 0x0000_0003;
/// webgpu.yml `enum.blend_operation` value.
#[allow(dead_code)]
const WGPUBlendOperation_Min: i32 = 0x0000_0004;
/// webgpu.yml `enum.blend_operation` value.
#[allow(dead_code)]
const WGPUBlendOperation_Max: i32 = 0x0000_0005;

/// webgpu.yml `enum.blend_factor` value.
#[allow(dead_code)]
const WGPUBlendFactor_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.blend_factor` value.
#[allow(dead_code)]
const WGPUBlendFactor_Zero: i32 = 0x0000_0001;
/// webgpu.yml `enum.blend_factor` value.
#[allow(dead_code)]
const WGPUBlendFactor_One: i32 = 0x0000_0002;
/// webgpu.yml `enum.blend_factor` value.
#[allow(dead_code)]
const WGPUBlendFactor_Src: i32 = 0x0000_0003;
/// webgpu.yml `enum.blend_factor` value.
#[allow(dead_code)]
const WGPUBlendFactor_OneMinusSrc: i32 = 0x0000_0004;
/// webgpu.yml `enum.blend_factor` value.
#[allow(dead_code)]
const WGPUBlendFactor_SrcAlpha: i32 = 0x0000_0005;
/// webgpu.yml `enum.blend_factor` value.
#[allow(dead_code)]
const WGPUBlendFactor_OneMinusSrcAlpha: i32 = 0x0000_0006;
/// webgpu.yml `enum.blend_factor` value.
#[allow(dead_code)]
const WGPUBlendFactor_Dst: i32 = 0x0000_0007;
/// webgpu.yml `enum.blend_factor` value.
#[allow(dead_code)]
const WGPUBlendFactor_OneMinusDst: i32 = 0x0000_0008;
/// webgpu.yml `enum.blend_factor` value.
#[allow(dead_code)]
const WGPUBlendFactor_DstAlpha: i32 = 0x0000_0009;
/// webgpu.yml `enum.blend_factor` value.
#[allow(dead_code)]
const WGPUBlendFactor_OneMinusDstAlpha: i32 = 0x0000_000A;
/// webgpu.yml `enum.blend_factor` value.
#[allow(dead_code)]
const WGPUBlendFactor_SrcAlphaSaturated: i32 = 0x0000_000B;
/// webgpu.yml `enum.blend_factor` value.
#[allow(dead_code)]
const WGPUBlendFactor_Constant: i32 = 0x0000_000C;
/// webgpu.yml `enum.blend_factor` value.
#[allow(dead_code)]
const WGPUBlendFactor_OneMinusConstant: i32 = 0x0000_000D;
/// webgpu.yml `enum.blend_factor` value.
#[allow(dead_code)]
const WGPUBlendFactor_Src1: i32 = 0x0000_000E;
/// webgpu.yml `enum.blend_factor` value.
#[allow(dead_code)]
const WGPUBlendFactor_OneMinusSrc1: i32 = 0x0000_000F;
/// webgpu.yml `enum.blend_factor` value.
#[allow(dead_code)]
const WGPUBlendFactor_Src1Alpha: i32 = 0x0000_0010;
/// webgpu.yml `enum.blend_factor` value.
#[allow(dead_code)]
const WGPUBlendFactor_OneMinusSrc1Alpha: i32 = 0x0000_0011;

/// webgpu.yml `bitflag.color_write_mask` value.
#[allow(dead_code)]
const WGPUColorWriteMask_None: u64 = 0x0;
/// webgpu.yml `bitflag.color_write_mask` value.
#[allow(dead_code)]
const WGPUColorWriteMask_Red: u64 = 0x1;
/// webgpu.yml `bitflag.color_write_mask` value.
#[allow(dead_code)]
const WGPUColorWriteMask_Green: u64 = 0x2;
/// webgpu.yml `bitflag.color_write_mask` value.
#[allow(dead_code)]
const WGPUColorWriteMask_Blue: u64 = 0x4;
/// webgpu.yml `bitflag.color_write_mask` value.
#[allow(dead_code)]
const WGPUColorWriteMask_Alpha: u64 = 0x8;
/// webgpu.yml `bitflag.color_write_mask` value.
#[allow(dead_code)]
const WGPUColorWriteMask_All: u64 = 0xF;

/// webgpu.yml `enum.optional_bool` value.
#[allow(dead_code)]
const WGPUOptionalBool_False: i32 = 0x0000_0000;
/// webgpu.yml `enum.optional_bool` value.
#[allow(dead_code)]
const WGPUOptionalBool_True: i32 = 0x0000_0001;
/// webgpu.yml `enum.optional_bool` value.
#[allow(dead_code)]
const WGPUOptionalBool_Undefined: i32 = 0x0000_0002;

/// webgpu.yml `enum.stencil_operation` value.
#[allow(dead_code)]
const WGPUStencilOperation_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.stencil_operation` value.
#[allow(dead_code)]
const WGPUStencilOperation_Keep: i32 = 0x0000_0001;
/// webgpu.yml `enum.stencil_operation` value.
#[allow(dead_code)]
const WGPUStencilOperation_Zero: i32 = 0x0000_0002;
/// webgpu.yml `enum.stencil_operation` value.
#[allow(dead_code)]
const WGPUStencilOperation_Replace: i32 = 0x0000_0003;
/// webgpu.yml `enum.stencil_operation` value.
#[allow(dead_code)]
const WGPUStencilOperation_Invert: i32 = 0x0000_0004;
/// webgpu.yml `enum.stencil_operation` value.
#[allow(dead_code)]
const WGPUStencilOperation_IncrementClamp: i32 = 0x0000_0005;
/// webgpu.yml `enum.stencil_operation` value.
#[allow(dead_code)]
const WGPUStencilOperation_DecrementClamp: i32 = 0x0000_0006;
/// webgpu.yml `enum.stencil_operation` value.
#[allow(dead_code)]
const WGPUStencilOperation_IncrementWrap: i32 = 0x0000_0007;
/// webgpu.yml `enum.stencil_operation` value.
#[allow(dead_code)]
const WGPUStencilOperation_DecrementWrap: i32 = 0x0000_0008;

/// webgpu.yml `enum.load_op` value.
#[allow(dead_code)]
const WGPULoadOp_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.load_op` value.
#[allow(dead_code)]
const WGPULoadOp_Load: i32 = 0x0000_0001;
/// webgpu.yml `enum.load_op` value.
#[allow(dead_code)]
const WGPULoadOp_Clear: i32 = 0x0000_0002;

/// webgpu.yml `enum.store_op` value.
#[allow(dead_code)]
const WGPUStoreOp_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.store_op` value.
#[allow(dead_code)]
const WGPUStoreOp_Store: i32 = 0x0000_0001;
/// webgpu.yml `enum.store_op` value.
#[allow(dead_code)]
const WGPUStoreOp_Discard: i32 = 0x0000_0002;

/// webgpu.yml `enum.query_type` value.
#[allow(dead_code)]
const WGPUQueryType_Occlusion: i32 = 0x0000_0001;
/// webgpu.yml `enum.query_type` value.
#[allow(dead_code)]
const WGPUQueryType_Timestamp: i32 = 0x0000_0002;

/// webgpu.yml `enum.error_filter` value.
#[allow(dead_code)]
const WGPUErrorFilter_Validation: i32 = 0x0000_0001;
/// webgpu.yml `enum.error_filter` value.
#[allow(dead_code)]
const WGPUErrorFilter_OutOfMemory: i32 = 0x0000_0002;
/// webgpu.yml `enum.error_filter` value.
#[allow(dead_code)]
const WGPUErrorFilter_Internal: i32 = 0x0000_0003;

/// webgpu.yml `enum.error_type` value.
#[allow(dead_code)]
const WGPUErrorType_NoError: i32 = 0x0000_0001;
/// webgpu.yml `enum.error_type` value.
#[allow(dead_code)]
const WGPUErrorType_Validation: i32 = 0x0000_0002;
/// webgpu.yml `enum.error_type` value.
#[allow(dead_code)]
const WGPUErrorType_OutOfMemory: i32 = 0x0000_0003;
/// webgpu.yml `enum.error_type` value.
#[allow(dead_code)]
const WGPUErrorType_Internal: i32 = 0x0000_0004;
/// webgpu.yml `enum.error_type` value.
#[allow(dead_code)]
const WGPUErrorType_Unknown: i32 = 0x0000_0005;

/// webgpu.yml `enum.device_lost_reason` value.
#[allow(dead_code)]
const WGPUDeviceLostReason_Unknown: i32 = 0x0000_0001;
/// webgpu.yml `enum.device_lost_reason` value.
#[allow(dead_code)]
const WGPUDeviceLostReason_Destroyed: i32 = 0x0000_0002;
/// webgpu.yml `enum.device_lost_reason` value.
#[allow(dead_code)]
const WGPUDeviceLostReason_CallbackCancelled: i32 = 0x0000_0003;
/// webgpu.yml `enum.device_lost_reason` value.
#[allow(dead_code)]
const WGPUDeviceLostReason_FailedCreation: i32 = 0x0000_0004;

/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_CoreFeaturesAndLimits: i32 = 0x0000_0001;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_DepthClipControl: i32 = 0x0000_0002;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_Depth32FloatStencil8: i32 = 0x0000_0003;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_TextureCompressionBC: i32 = 0x0000_0004;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_TextureCompressionBCSliced3D: i32 = 0x0000_0005;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_TextureCompressionETC2: i32 = 0x0000_0006;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_TextureCompressionASTC: i32 = 0x0000_0007;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_TextureCompressionASTCSliced3D: i32 = 0x0000_0008;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_TimestampQuery: i32 = 0x0000_0009;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_IndirectFirstInstance: i32 = 0x0000_000A;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_ShaderF16: i32 = 0x0000_000B;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_RG11B10UfloatRenderable: i32 = 0x0000_000C;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_BGRA8UnormStorage: i32 = 0x0000_000D;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_Float32Filterable: i32 = 0x0000_000E;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_Float32Blendable: i32 = 0x0000_000F;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_ClipDistances: i32 = 0x0000_0010;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_DualSourceBlending: i32 = 0x0000_0011;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_Subgroups: i32 = 0x0000_0012;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_TextureFormatsTier1: i32 = 0x0000_0013;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_TextureFormatsTier2: i32 = 0x0000_0014;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_PrimitiveIndex: i32 = 0x0000_0015;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_TextureComponentSwizzle: i32 = 0x0000_0016;
/// webgpu.yml `enum.feature_name` value.
#[allow(dead_code)]
const WGPUFeatureName_SubgroupSizeControl: i32 = 0x0000_0017;

/// webgpu.yml `enum.instance_feature_name` value.
#[allow(dead_code)]
const WGPUInstanceFeatureName_TimedWaitAny: i32 = 0x0000_0001;
/// webgpu.yml `enum.instance_feature_name` value.
#[allow(dead_code)]
const WGPUInstanceFeatureName_ShaderSourceSPIRV: i32 = 0x0000_0002;
/// webgpu.yml `enum.instance_feature_name` value.
#[allow(dead_code)]
const WGPUInstanceFeatureName_MultipleDevicesPerAdapter: i32 = 0x0000_0003;

/// webgpu.yml `enum.backend_type` value.
#[allow(dead_code)]
const WGPUBackendType_Undefined: i32 = 0x0000_0000;
/// webgpu.yml `enum.backend_type` value.
#[allow(dead_code)]
const WGPUBackendType_Null: i32 = 0x0000_0001;
/// webgpu.yml `enum.backend_type` value.
#[allow(dead_code)]
const WGPUBackendType_WebGPU: i32 = 0x0000_0002;
/// webgpu.yml `enum.backend_type` value.
#[allow(dead_code)]
const WGPUBackendType_D3D11: i32 = 0x0000_0003;
/// webgpu.yml `enum.backend_type` value.
#[allow(dead_code)]
const WGPUBackendType_D3D12: i32 = 0x0000_0004;
/// webgpu.yml `enum.backend_type` value.
#[allow(dead_code)]
const WGPUBackendType_Metal: i32 = 0x0000_0005;
/// webgpu.yml `enum.backend_type` value.
#[allow(dead_code)]
const WGPUBackendType_Vulkan: i32 = 0x0000_0006;
/// webgpu.yml `enum.backend_type` value.
#[allow(dead_code)]
const WGPUBackendType_OpenGL: i32 = 0x0000_0007;
/// webgpu.yml `enum.backend_type` value.
#[allow(dead_code)]
const WGPUBackendType_OpenGLES: i32 = 0x0000_0008;

/// webgpu.yml `enum.adapter_type` value.
#[allow(dead_code)]
const WGPUAdapterType_DiscreteGPU: i32 = 0x0000_0001;
/// webgpu.yml `enum.adapter_type` value.
#[allow(dead_code)]
const WGPUAdapterType_IntegratedGPU: i32 = 0x0000_0002;
/// webgpu.yml `enum.adapter_type` value.
#[allow(dead_code)]
const WGPUAdapterType_CPU: i32 = 0x0000_0003;
/// webgpu.yml `enum.adapter_type` value.
#[allow(dead_code)]
const WGPUAdapterType_Unknown: i32 = 0x0000_0004;

/// webgpu.yml `constant.limit_u64_undefined`; kept internal because it exceeds the exact script integer range.
const WGPU_LIMIT_U64_UNDEFINED: u64 = u64::MAX;

/// webgpu.yml `constant.whole_size`; kept internal because it exceeds the exact script integer range.
const WGPU_WHOLE_SIZE: u64 = u64::MAX;

/// webgpu.yml adapter-info success status.
const WGPUStatus_Success: i32 = 0x0000_0001;

/// webgpu.h `WGPURequestAdapterCallback` callback and userdata.
type WGPURequestAdapterCallback = Option<
    // SAFETY: the callback signature matches the pinned webgpu.h declaration.
    unsafe extern "C" fn(
        status: i32,
        adapter: WGPUAdapter,
        message: WGPUStringView,
        userdata1: *mut c_void,
        userdata2: *mut c_void,
    ),
>;

/// webgpu.h `WGPURequestAdapterCallbackInfo` (passed by value).
#[repr(C)]
struct WGPURequestAdapterCallbackInfo {
    next_in_chain: *mut WGPUChainedStruct,
    mode: i32,
    callback: WGPURequestAdapterCallback,
    userdata1: *mut c_void,
    userdata2: *mut c_void,
}

/// webgpu.h `WGPURequestDeviceCallback` callback and userdata.
type WGPURequestDeviceCallback = Option<
    // SAFETY: the callback signature matches the pinned webgpu.h declaration.
    unsafe extern "C" fn(
        status: i32,
        device: WGPUDevice,
        message: WGPUStringView,
        userdata1: *mut c_void,
        userdata2: *mut c_void,
    ),
>;

/// webgpu.h `WGPURequestDeviceCallbackInfo` (passed by value).
#[repr(C)]
struct WGPURequestDeviceCallbackInfo {
    next_in_chain: *mut WGPUChainedStruct,
    mode: i32,
    callback: WGPURequestDeviceCallback,
    userdata1: *mut c_void,
    userdata2: *mut c_void,
}

/// webgpu.h `WGPUCreateComputePipelineAsyncCallback` callback and userdata.
type WGPUCreateComputePipelineAsyncCallback = Option<
    // SAFETY: the callback signature matches the pinned webgpu.h declaration.
    unsafe extern "C" fn(
        status: i32,
        computePipeline: WGPUComputePipeline,
        message: WGPUStringView,
        userdata1: *mut c_void,
        userdata2: *mut c_void,
    ),
>;

/// webgpu.h `WGPUCreateComputePipelineAsyncCallbackInfo` (passed by value).
#[repr(C)]
struct WGPUCreateComputePipelineAsyncCallbackInfo {
    next_in_chain: *mut WGPUChainedStruct,
    mode: i32,
    callback: WGPUCreateComputePipelineAsyncCallback,
    userdata1: *mut c_void,
    userdata2: *mut c_void,
}

/// webgpu.h `WGPUCreateRenderPipelineAsyncCallback` callback and userdata.
type WGPUCreateRenderPipelineAsyncCallback = Option<
    // SAFETY: the callback signature matches the pinned webgpu.h declaration.
    unsafe extern "C" fn(
        status: i32,
        renderPipeline: WGPURenderPipeline,
        message: WGPUStringView,
        userdata1: *mut c_void,
        userdata2: *mut c_void,
    ),
>;

/// webgpu.h `WGPUCreateRenderPipelineAsyncCallbackInfo` (passed by value).
#[repr(C)]
struct WGPUCreateRenderPipelineAsyncCallbackInfo {
    next_in_chain: *mut WGPUChainedStruct,
    mode: i32,
    callback: WGPUCreateRenderPipelineAsyncCallback,
    userdata1: *mut c_void,
    userdata2: *mut c_void,
}

/// webgpu.h `WGPUQueueWorkDoneCallback` callback and userdata.
type WGPUQueueWorkDoneCallback = Option<
    // SAFETY: the callback signature matches the pinned webgpu.h declaration.
    unsafe extern "C" fn(
        status: i32,
        message: WGPUStringView,
        userdata1: *mut c_void,
        userdata2: *mut c_void,
    ),
>;

/// webgpu.h `WGPUQueueWorkDoneCallbackInfo` (passed by value).
#[repr(C)]
struct WGPUQueueWorkDoneCallbackInfo {
    next_in_chain: *mut WGPUChainedStruct,
    mode: i32,
    callback: WGPUQueueWorkDoneCallback,
    userdata1: *mut c_void,
    userdata2: *mut c_void,
}

/// webgpu.h `WGPUBufferMapCallback` callback and userdata.
type WGPUBufferMapCallback = Option<
    // SAFETY: the callback signature matches the pinned webgpu.h declaration.
    unsafe extern "C" fn(
        status: i32,
        message: WGPUStringView,
        userdata1: *mut c_void,
        userdata2: *mut c_void,
    ),
>;

/// webgpu.h `WGPUBufferMapCallbackInfo` (passed by value).
#[repr(C)]
struct WGPUBufferMapCallbackInfo {
    next_in_chain: *mut WGPUChainedStruct,
    mode: i32,
    callback: WGPUBufferMapCallback,
    userdata1: *mut c_void,
    userdata2: *mut c_void,
}

/// webgpu.h `WGPUInstanceLimits`.
#[repr(C)]
struct WGPUInstanceLimits {
    next_in_chain: *mut WGPUChainedStruct,
    timed_wait_any_max_count: usize,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuInstanceLimits {
    /// Struct field `timedWaitAnyMaxCount`.
    pub timed_wait_any_max_count: usize,
}

#[allow(dead_code)]
fn convert_instance_limits(source: SubscriptTypegpuInstanceLimits) -> WGPUInstanceLimits {
    WGPUInstanceLimits {
        next_in_chain: std::ptr::null_mut(),
        timed_wait_any_max_count: source.timed_wait_any_max_count,
    }
}

/// webgpu.h `WGPULimits`.
#[repr(C)]
struct WGPULimits {
    next_in_chain: *mut WGPUChainedStruct,
    max_texture_dimension_1D: u32,
    max_texture_dimension_2D: u32,
    max_texture_dimension_3D: u32,
    max_texture_array_layers: u32,
    max_bind_groups: u32,
    max_bind_groups_plus_vertex_buffers: u32,
    max_bindings_per_bind_group: u32,
    max_dynamic_uniform_buffers_per_pipeline_layout: u32,
    max_dynamic_storage_buffers_per_pipeline_layout: u32,
    max_sampled_textures_per_shader_stage: u32,
    max_samplers_per_shader_stage: u32,
    max_storage_buffers_per_shader_stage: u32,
    max_storage_textures_per_shader_stage: u32,
    max_uniform_buffers_per_shader_stage: u32,
    max_uniform_buffer_binding_size: u64,
    max_storage_buffer_binding_size: u64,
    min_uniform_buffer_offset_alignment: u32,
    min_storage_buffer_offset_alignment: u32,
    max_vertex_buffers: u32,
    max_buffer_size: u64,
    max_vertex_attributes: u32,
    max_vertex_buffer_array_stride: u32,
    max_inter_stage_shader_variables: u32,
    max_color_attachments: u32,
    max_color_attachment_bytes_per_sample: u32,
    max_compute_workgroup_storage_size: u32,
    max_compute_invocations_per_workgroup: u32,
    max_compute_workgroup_size_x: u32,
    max_compute_workgroup_size_y: u32,
    max_compute_workgroup_size_z: u32,
    max_compute_workgroups_per_dimension: u32,
    max_immediate_size: u32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuLimits {
    /// Struct field `maxTextureDimension1D`.
    pub max_texture_dimension_1D: u32,
    /// Struct field `maxTextureDimension2D`.
    pub max_texture_dimension_2D: u32,
    /// Struct field `maxTextureDimension3D`.
    pub max_texture_dimension_3D: u32,
    /// Struct field `maxTextureArrayLayers`.
    pub max_texture_array_layers: u32,
    /// Struct field `maxBindGroups`.
    pub max_bind_groups: u32,
    /// Struct field `maxBindGroupsPlusVertexBuffers`.
    pub max_bind_groups_plus_vertex_buffers: u32,
    /// Struct field `maxBindingsPerBindGroup`.
    pub max_bindings_per_bind_group: u32,
    /// Struct field `maxDynamicUniformBuffersPerPipelineLayout`.
    pub max_dynamic_uniform_buffers_per_pipeline_layout: u32,
    /// Struct field `maxDynamicStorageBuffersPerPipelineLayout`.
    pub max_dynamic_storage_buffers_per_pipeline_layout: u32,
    /// Struct field `maxSampledTexturesPerShaderStage`.
    pub max_sampled_textures_per_shader_stage: u32,
    /// Struct field `maxSamplersPerShaderStage`.
    pub max_samplers_per_shader_stage: u32,
    /// Struct field `maxStorageBuffersPerShaderStage`.
    pub max_storage_buffers_per_shader_stage: u32,
    /// Struct field `maxStorageTexturesPerShaderStage`.
    pub max_storage_textures_per_shader_stage: u32,
    /// Struct field `maxUniformBuffersPerShaderStage`.
    pub max_uniform_buffers_per_shader_stage: u32,
    /// Struct field `maxUniformBufferBindingSize`.
    pub max_uniform_buffer_binding_size: u64,
    /// Struct field `maxStorageBufferBindingSize`.
    pub max_storage_buffer_binding_size: u64,
    /// Struct field `minUniformBufferOffsetAlignment`.
    pub min_uniform_buffer_offset_alignment: u32,
    /// Struct field `minStorageBufferOffsetAlignment`.
    pub min_storage_buffer_offset_alignment: u32,
    /// Struct field `maxVertexBuffers`.
    pub max_vertex_buffers: u32,
    /// Struct field `maxBufferSize`.
    pub max_buffer_size: u64,
    /// Struct field `maxVertexAttributes`.
    pub max_vertex_attributes: u32,
    /// Struct field `maxVertexBufferArrayStride`.
    pub max_vertex_buffer_array_stride: u32,
    /// Struct field `maxInterStageShaderVariables`.
    pub max_inter_stage_shader_variables: u32,
    /// Struct field `maxColorAttachments`.
    pub max_color_attachments: u32,
    /// Struct field `maxColorAttachmentBytesPerSample`.
    pub max_color_attachment_bytes_per_sample: u32,
    /// Struct field `maxComputeWorkgroupStorageSize`.
    pub max_compute_workgroup_storage_size: u32,
    /// Struct field `maxComputeInvocationsPerWorkgroup`.
    pub max_compute_invocations_per_workgroup: u32,
    /// Struct field `maxComputeWorkgroupSizeX`.
    pub max_compute_workgroup_size_x: u32,
    /// Struct field `maxComputeWorkgroupSizeY`.
    pub max_compute_workgroup_size_y: u32,
    /// Struct field `maxComputeWorkgroupSizeZ`.
    pub max_compute_workgroup_size_z: u32,
    /// Struct field `maxComputeWorkgroupsPerDimension`.
    pub max_compute_workgroups_per_dimension: u32,
    /// Struct field `maxImmediateSize`.
    pub max_immediate_size: u32,
}

fn convert_limits_max_uniform_buffer_binding_size_zero_rule(value: u64) -> u64 {
    if value == 0 { WGPU_LIMIT_U64_UNDEFINED } else { value }
}

#[doc(hidden)]
pub fn subscript_typegpu_internal_limits_max_uniform_buffer_binding_size_for_test(value: u64) -> u64 {
    // SAFETY: generated SubscriptTypegpu descriptor fields all admit an all-zero value.
    let mut source: SubscriptTypegpuLimits = unsafe { std::mem::zeroed() };
    source.max_uniform_buffer_binding_size = value;
    let converted = convert_limits(source);
    converted.max_uniform_buffer_binding_size
}

fn convert_limits_max_storage_buffer_binding_size_zero_rule(value: u64) -> u64 {
    if value == 0 { WGPU_LIMIT_U64_UNDEFINED } else { value }
}

#[doc(hidden)]
pub fn subscript_typegpu_internal_limits_max_storage_buffer_binding_size_for_test(value: u64) -> u64 {
    // SAFETY: generated SubscriptTypegpu descriptor fields all admit an all-zero value.
    let mut source: SubscriptTypegpuLimits = unsafe { std::mem::zeroed() };
    source.max_storage_buffer_binding_size = value;
    let converted = convert_limits(source);
    converted.max_storage_buffer_binding_size
}

fn convert_limits_max_buffer_size_zero_rule(value: u64) -> u64 {
    if value == 0 { WGPU_LIMIT_U64_UNDEFINED } else { value }
}

#[doc(hidden)]
pub fn subscript_typegpu_internal_limits_max_buffer_size_for_test(value: u64) -> u64 {
    // SAFETY: generated SubscriptTypegpu descriptor fields all admit an all-zero value.
    let mut source: SubscriptTypegpuLimits = unsafe { std::mem::zeroed() };
    source.max_buffer_size = value;
    let converted = convert_limits(source);
    converted.max_buffer_size
}

#[allow(dead_code)]
fn convert_limits(source: SubscriptTypegpuLimits) -> WGPULimits {
    WGPULimits {
        next_in_chain: std::ptr::null_mut(),
        max_texture_dimension_1D: source.max_texture_dimension_1D,
        max_texture_dimension_2D: source.max_texture_dimension_2D,
        max_texture_dimension_3D: source.max_texture_dimension_3D,
        max_texture_array_layers: source.max_texture_array_layers,
        max_bind_groups: source.max_bind_groups,
        max_bind_groups_plus_vertex_buffers: source.max_bind_groups_plus_vertex_buffers,
        max_bindings_per_bind_group: source.max_bindings_per_bind_group,
        max_dynamic_uniform_buffers_per_pipeline_layout: source.max_dynamic_uniform_buffers_per_pipeline_layout,
        max_dynamic_storage_buffers_per_pipeline_layout: source.max_dynamic_storage_buffers_per_pipeline_layout,
        max_sampled_textures_per_shader_stage: source.max_sampled_textures_per_shader_stage,
        max_samplers_per_shader_stage: source.max_samplers_per_shader_stage,
        max_storage_buffers_per_shader_stage: source.max_storage_buffers_per_shader_stage,
        max_storage_textures_per_shader_stage: source.max_storage_textures_per_shader_stage,
        max_uniform_buffers_per_shader_stage: source.max_uniform_buffers_per_shader_stage,
        max_uniform_buffer_binding_size: convert_limits_max_uniform_buffer_binding_size_zero_rule(source.max_uniform_buffer_binding_size),
        max_storage_buffer_binding_size: convert_limits_max_storage_buffer_binding_size_zero_rule(source.max_storage_buffer_binding_size),
        min_uniform_buffer_offset_alignment: source.min_uniform_buffer_offset_alignment,
        min_storage_buffer_offset_alignment: source.min_storage_buffer_offset_alignment,
        max_vertex_buffers: source.max_vertex_buffers,
        max_buffer_size: convert_limits_max_buffer_size_zero_rule(source.max_buffer_size),
        max_vertex_attributes: source.max_vertex_attributes,
        max_vertex_buffer_array_stride: source.max_vertex_buffer_array_stride,
        max_inter_stage_shader_variables: source.max_inter_stage_shader_variables,
        max_color_attachments: source.max_color_attachments,
        max_color_attachment_bytes_per_sample: source.max_color_attachment_bytes_per_sample,
        max_compute_workgroup_storage_size: source.max_compute_workgroup_storage_size,
        max_compute_invocations_per_workgroup: source.max_compute_invocations_per_workgroup,
        max_compute_workgroup_size_x: source.max_compute_workgroup_size_x,
        max_compute_workgroup_size_y: source.max_compute_workgroup_size_y,
        max_compute_workgroup_size_z: source.max_compute_workgroup_size_z,
        max_compute_workgroups_per_dimension: source.max_compute_workgroups_per_dimension,
        max_immediate_size: source.max_immediate_size,
    }
}

/// webgpu.h `WGPUQueueDescriptor`.
#[repr(C)]
struct WGPUQueueDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuQueueDescriptor {
    /// Struct field `label`.
    pub label: SubscriptTypegpuStringView,
}

#[allow(dead_code)]
fn convert_queue_descriptor(source: SubscriptTypegpuQueueDescriptor) -> WGPUQueueDescriptor {
    WGPUQueueDescriptor {
        next_in_chain: std::ptr::null_mut(),
        label: wgpu_string_view(source.label),
    }
}

/// webgpu.h `WGPUBufferDescriptor`.
#[repr(C)]
struct WGPUBufferDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
    usage: u64,
    size: u64,
    mapped_at_creation: u32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuBufferDescriptor {
    /// Struct field `label`.
    pub label: SubscriptTypegpuStringView,
    /// Struct field `usage`.
    pub usage: u64,
    /// Struct field `size`.
    pub size: u64,
    /// Struct field `mappedAtCreation`.
    pub mapped_at_creation: bool,
}

#[allow(dead_code)]
fn convert_buffer_descriptor(source: SubscriptTypegpuBufferDescriptor) -> WGPUBufferDescriptor {
    WGPUBufferDescriptor {
        next_in_chain: std::ptr::null_mut(),
        label: wgpu_string_view(source.label),
        usage: source.usage,
        size: source.size,
        mapped_at_creation: u32::from(source.mapped_at_creation),
    }
}

/// webgpu.h `WGPUExtent3D`.
#[repr(C)]
struct WGPUExtent3D {
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuExtent3D {
    /// Struct field `width`.
    pub width: u32,
    /// Struct field `height`.
    pub height: u32,
    /// Struct field `depthOrArrayLayers`.
    pub depth_or_array_layers: u32,
}

#[allow(dead_code)]
fn convert_extent_3D(source: SubscriptTypegpuExtent3D) -> WGPUExtent3D {
    WGPUExtent3D {
        width: source.width,
        height: source.height,
        depth_or_array_layers: source.depth_or_array_layers,
    }
}

/// webgpu.h `WGPUTextureDescriptor`.
#[repr(C)]
struct WGPUTextureDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
    usage: u64,
    dimension: i32,
    size: WGPUExtent3D,
    format: i32,
    mip_level_count: u32,
    sample_count: u32,
    view_format_count: usize,
    view_formats: *const i32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuTextureDescriptor {
    /// Struct field `label`.
    pub label: SubscriptTypegpuStringView,
    /// Struct field `usage`.
    pub usage: u64,
    /// Struct field `dimension`.
    pub dimension: i32,
    /// Struct field `size`.
    pub size: SubscriptTypegpuExtent3D,
    /// Struct field `format`.
    pub format: i32,
    /// Struct field `mipLevelCount`.
    pub mip_level_count: u32,
    /// Struct field `sampleCount`.
    pub sample_count: u32,
    /// Element count for `viewFormats`.
    pub view_formats_count: usize,
    /// Struct field `viewFormats`.
    pub view_formats: *const i32,
}

#[allow(dead_code)]
fn convert_texture_descriptor(source: SubscriptTypegpuTextureDescriptor) -> WGPUTextureDescriptor {
    WGPUTextureDescriptor {
        next_in_chain: std::ptr::null_mut(),
        label: wgpu_string_view(source.label),
        usage: source.usage,
        dimension: source.dimension,
        size: convert_extent_3D(source.size),
        format: source.format,
        mip_level_count: source.mip_level_count,
        sample_count: source.sample_count,
        view_format_count: source.view_formats_count,
        view_formats: source.view_formats,
    }
}

/// webgpu.h `WGPUSamplerDescriptor`.
#[repr(C)]
struct WGPUSamplerDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
    address_mode_u: i32,
    address_mode_v: i32,
    address_mode_w: i32,
    mag_filter: i32,
    min_filter: i32,
    mipmap_filter: i32,
    lod_min_clamp: f32,
    lod_max_clamp: f32,
    compare: i32,
    max_anisotropy: u16,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuSamplerDescriptor {
    /// Struct field `label`.
    pub label: SubscriptTypegpuStringView,
    /// Struct field `addressModeU`.
    pub address_mode_u: i32,
    /// Struct field `addressModeV`.
    pub address_mode_v: i32,
    /// Struct field `addressModeW`.
    pub address_mode_w: i32,
    /// Struct field `magFilter`.
    pub mag_filter: i32,
    /// Struct field `minFilter`.
    pub min_filter: i32,
    /// Struct field `mipmapFilter`.
    pub mipmap_filter: i32,
    /// Struct field `lodMinClamp`.
    pub lod_min_clamp: f32,
    /// Struct field `lodMaxClamp`.
    pub lod_max_clamp: f32,
    /// Struct field `compare`.
    pub compare: i32,
    /// Struct field `maxAnisotropy`.
    pub max_anisotropy: u16,
}

#[allow(dead_code)]
fn convert_sampler_descriptor(source: SubscriptTypegpuSamplerDescriptor) -> WGPUSamplerDescriptor {
    WGPUSamplerDescriptor {
        next_in_chain: std::ptr::null_mut(),
        label: wgpu_string_view(source.label),
        address_mode_u: source.address_mode_u,
        address_mode_v: source.address_mode_v,
        address_mode_w: source.address_mode_w,
        mag_filter: source.mag_filter,
        min_filter: source.min_filter,
        mipmap_filter: source.mipmap_filter,
        lod_min_clamp: source.lod_min_clamp,
        lod_max_clamp: source.lod_max_clamp,
        compare: source.compare,
        max_anisotropy: source.max_anisotropy,
    }
}

/// webgpu.h `WGPUBufferBindingLayout`.
#[repr(C)]
struct WGPUBufferBindingLayout {
    next_in_chain: *mut WGPUChainedStruct,
    r#type: i32,
    has_dynamic_offset: u32,
    min_binding_size: u64,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuBufferBindingLayout {
    /// Struct field `type`.
    pub r#type: i32,
    /// Struct field `hasDynamicOffset`.
    pub has_dynamic_offset: bool,
    /// Struct field `minBindingSize`.
    pub min_binding_size: u64,
}

#[allow(dead_code)]
fn convert_buffer_binding_layout(source: SubscriptTypegpuBufferBindingLayout) -> WGPUBufferBindingLayout {
    WGPUBufferBindingLayout {
        next_in_chain: std::ptr::null_mut(),
        r#type: source.r#type,
        has_dynamic_offset: u32::from(source.has_dynamic_offset),
        min_binding_size: source.min_binding_size,
    }
}

/// webgpu.h `WGPUSamplerBindingLayout`.
#[repr(C)]
struct WGPUSamplerBindingLayout {
    next_in_chain: *mut WGPUChainedStruct,
    r#type: i32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuSamplerBindingLayout {
    /// Struct field `type`.
    pub r#type: i32,
}

#[allow(dead_code)]
fn convert_sampler_binding_layout(source: SubscriptTypegpuSamplerBindingLayout) -> WGPUSamplerBindingLayout {
    WGPUSamplerBindingLayout {
        next_in_chain: std::ptr::null_mut(),
        r#type: source.r#type,
    }
}

/// webgpu.h `WGPUTextureBindingLayout`.
#[repr(C)]
struct WGPUTextureBindingLayout {
    next_in_chain: *mut WGPUChainedStruct,
    sample_type: i32,
    view_dimension: i32,
    multisampled: u32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuTextureBindingLayout {
    /// Struct field `sampleType`.
    pub sample_type: i32,
    /// Struct field `viewDimension`.
    pub view_dimension: i32,
    /// Struct field `multisampled`.
    pub multisampled: bool,
}

#[allow(dead_code)]
fn convert_texture_binding_layout(source: SubscriptTypegpuTextureBindingLayout) -> WGPUTextureBindingLayout {
    WGPUTextureBindingLayout {
        next_in_chain: std::ptr::null_mut(),
        sample_type: source.sample_type,
        view_dimension: source.view_dimension,
        multisampled: u32::from(source.multisampled),
    }
}

/// webgpu.h `WGPUStorageTextureBindingLayout`.
#[repr(C)]
struct WGPUStorageTextureBindingLayout {
    next_in_chain: *mut WGPUChainedStruct,
    access: i32,
    format: i32,
    view_dimension: i32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuStorageTextureBindingLayout {
    /// Struct field `access`.
    pub access: i32,
    /// Struct field `format`.
    pub format: i32,
    /// Struct field `viewDimension`.
    pub view_dimension: i32,
}

#[allow(dead_code)]
fn convert_storage_texture_binding_layout(source: SubscriptTypegpuStorageTextureBindingLayout) -> WGPUStorageTextureBindingLayout {
    WGPUStorageTextureBindingLayout {
        next_in_chain: std::ptr::null_mut(),
        access: source.access,
        format: source.format,
        view_dimension: source.view_dimension,
    }
}

/// webgpu.h `WGPUBindGroupLayoutEntry`.
#[repr(C)]
struct WGPUBindGroupLayoutEntry {
    next_in_chain: *mut WGPUChainedStruct,
    binding: u32,
    visibility: u64,
    binding_array_size: u32,
    buffer: WGPUBufferBindingLayout,
    sampler: WGPUSamplerBindingLayout,
    texture: WGPUTextureBindingLayout,
    storage_texture: WGPUStorageTextureBindingLayout,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuBindGroupLayoutEntry {
    /// Struct field `binding`.
    pub binding: u32,
    /// Struct field `visibility`.
    pub visibility: u64,
    /// Struct field `bindingArraySize`.
    pub binding_array_size: u32,
    /// Struct field `buffer`.
    pub buffer: SubscriptTypegpuBufferBindingLayout,
    /// Struct field `sampler`.
    pub sampler: SubscriptTypegpuSamplerBindingLayout,
    /// Struct field `texture`.
    pub texture: SubscriptTypegpuTextureBindingLayout,
    /// Struct field `storageTexture`.
    pub storage_texture: SubscriptTypegpuStorageTextureBindingLayout,
}

#[allow(dead_code)]
fn convert_bind_group_layout_entry(source: SubscriptTypegpuBindGroupLayoutEntry) -> WGPUBindGroupLayoutEntry {
    WGPUBindGroupLayoutEntry {
        next_in_chain: std::ptr::null_mut(),
        binding: source.binding,
        visibility: source.visibility,
        binding_array_size: source.binding_array_size,
        buffer: convert_buffer_binding_layout(source.buffer),
        sampler: convert_sampler_binding_layout(source.sampler),
        texture: convert_texture_binding_layout(source.texture),
        storage_texture: convert_storage_texture_binding_layout(source.storage_texture),
    }
}

/// webgpu.h `WGPUBindGroupLayoutDescriptor`.
#[repr(C)]
struct WGPUBindGroupLayoutDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
    entry_count: usize,
    entries: *const WGPUBindGroupLayoutEntry,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuBindGroupLayoutDescriptor {
    /// Struct field `label`.
    pub label: SubscriptTypegpuStringView,
    /// Element count for `entries`.
    pub entries_count: usize,
    /// Struct field `entries`.
    pub entries: *const SubscriptTypegpuBindGroupLayoutEntry,
}

struct ConvertedBindGroupLayoutDescriptor {
    value: WGPUBindGroupLayoutDescriptor,
    _entries: Vec<WGPUBindGroupLayoutEntry>,
}

#[allow(dead_code)]
fn convert_bind_group_layout_descriptor(source: SubscriptTypegpuBindGroupLayoutDescriptor) -> ConvertedBindGroupLayoutDescriptor {
    let entries: Vec<WGPUBindGroupLayoutEntry> = if source.entries.is_null() {
        Vec::new()
    } else {
        // SAFETY: the boundary pair promises `count` readable elements.
        unsafe { std::slice::from_raw_parts(source.entries, source.entries_count) }
            .iter()
            .copied()
            .map(convert_bind_group_layout_entry)
            .collect()
    };
    let entries_ptr = if source.entries.is_null() {
        std::ptr::null()
    } else {
        entries.as_ptr()
    };
    let value = WGPUBindGroupLayoutDescriptor {
        next_in_chain: std::ptr::null_mut(),
        label: wgpu_string_view(source.label),
        entry_count: source.entries_count,
        entries: entries_ptr,
    };
    ConvertedBindGroupLayoutDescriptor {
        value,
        _entries: entries,
    }
}

/// webgpu.h `WGPUBindGroupEntry`.
#[repr(C)]
struct WGPUBindGroupEntry {
    next_in_chain: *mut WGPUChainedStruct,
    binding: u32,
    buffer: WGPUBuffer,
    offset: u64,
    size: u64,
    sampler: WGPUSampler,
    texture_view: WGPUTextureView,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuBindGroupEntry {
    /// Struct field `binding`.
    pub binding: u32,
    /// Struct field `buffer`.
    pub buffer: SubscriptTypegpuBuffer,
    /// Struct field `offset`.
    pub offset: u64,
    /// Struct field `size`.
    pub size: u64,
    /// Struct field `sampler`.
    pub sampler: SubscriptTypegpuSampler,
    /// Struct field `textureView`.
    pub texture_view: SubscriptTypegpuTextureView,
}

fn convert_bind_group_entry_size_zero_rule(value: u64) -> u64 {
    if value == 0 { WGPU_WHOLE_SIZE } else { value }
}

#[doc(hidden)]
pub fn subscript_typegpu_internal_bind_group_entry_size_for_test(value: u64) -> u64 {
    // SAFETY: generated SubscriptTypegpu descriptor fields all admit an all-zero value.
    let mut source: SubscriptTypegpuBindGroupEntry = unsafe { std::mem::zeroed() };
    source.size = value;
    let converted = convert_bind_group_entry(source);
    converted.size
}

#[allow(dead_code)]
fn convert_bind_group_entry(source: SubscriptTypegpuBindGroupEntry) -> WGPUBindGroupEntry {
    WGPUBindGroupEntry {
        next_in_chain: std::ptr::null_mut(),
        binding: source.binding,
        buffer: source.buffer.cast(),
        offset: source.offset,
        size: convert_bind_group_entry_size_zero_rule(source.size),
        sampler: source.sampler.cast(),
        texture_view: source.texture_view.cast(),
    }
}

/// webgpu.h `WGPUBindGroupDescriptor`.
#[repr(C)]
struct WGPUBindGroupDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
    layout: WGPUBindGroupLayout,
    entry_count: usize,
    entries: *const WGPUBindGroupEntry,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuBindGroupDescriptor {
    /// Struct field `label`.
    pub label: SubscriptTypegpuStringView,
    /// Struct field `layout`.
    pub layout: SubscriptTypegpuBindGroupLayout,
    /// Element count for `entries`.
    pub entries_count: usize,
    /// Struct field `entries`.
    pub entries: *const SubscriptTypegpuBindGroupEntry,
}

struct ConvertedBindGroupDescriptor {
    value: WGPUBindGroupDescriptor,
    _entries: Vec<WGPUBindGroupEntry>,
}

#[allow(dead_code)]
fn convert_bind_group_descriptor(source: SubscriptTypegpuBindGroupDescriptor) -> ConvertedBindGroupDescriptor {
    let entries: Vec<WGPUBindGroupEntry> = if source.entries.is_null() {
        Vec::new()
    } else {
        // SAFETY: the boundary pair promises `count` readable elements.
        unsafe { std::slice::from_raw_parts(source.entries, source.entries_count) }
            .iter()
            .copied()
            .map(convert_bind_group_entry)
            .collect()
    };
    let entries_ptr = if source.entries.is_null() {
        std::ptr::null()
    } else {
        entries.as_ptr()
    };
    let value = WGPUBindGroupDescriptor {
        next_in_chain: std::ptr::null_mut(),
        label: wgpu_string_view(source.label),
        layout: source.layout.cast(),
        entry_count: source.entries_count,
        entries: entries_ptr,
    };
    ConvertedBindGroupDescriptor {
        value,
        _entries: entries,
    }
}

/// webgpu.h `WGPUPipelineLayoutDescriptor`.
#[repr(C)]
struct WGPUPipelineLayoutDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
    bind_group_layout_count: usize,
    bind_group_layouts: *const WGPUBindGroupLayout,
    immediate_size: u32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuPipelineLayoutDescriptor {
    /// Struct field `label`.
    pub label: SubscriptTypegpuStringView,
    /// Element count for `bindGroupLayouts`.
    pub bind_group_layouts_count: usize,
    /// Struct field `bindGroupLayouts`.
    pub bind_group_layouts: *const SubscriptTypegpuBindGroupLayout,
    /// Struct field `immediateSize`.
    pub immediate_size: u32,
}

#[allow(dead_code)]
fn convert_pipeline_layout_descriptor(source: SubscriptTypegpuPipelineLayoutDescriptor) -> WGPUPipelineLayoutDescriptor {
    WGPUPipelineLayoutDescriptor {
        next_in_chain: std::ptr::null_mut(),
        label: wgpu_string_view(source.label),
        bind_group_layout_count: source.bind_group_layouts_count,
        bind_group_layouts: source.bind_group_layouts.cast(),
        immediate_size: source.immediate_size,
    }
}

/// webgpu.h `WGPUConstantEntry`.
#[repr(C)]
struct WGPUConstantEntry {
    next_in_chain: *mut WGPUChainedStruct,
    key: WGPUStringView,
    value: f64,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuConstantEntry {
    /// Struct field `key`.
    pub key: SubscriptTypegpuStringView,
    /// Struct field `value`.
    pub value: f64,
}

#[allow(dead_code)]
fn convert_constant_entry(source: SubscriptTypegpuConstantEntry) -> WGPUConstantEntry {
    WGPUConstantEntry {
        next_in_chain: std::ptr::null_mut(),
        key: wgpu_string_view(source.key),
        value: source.value,
    }
}

/// webgpu.h `WGPUComputeState`.
#[repr(C)]
#[derive(Clone, Copy)]
struct WGPUComputeState {
    next_in_chain: *mut WGPUChainedStruct,
    module: WGPUShaderModule,
    entry_point: WGPUStringView,
    constant_count: usize,
    constants: *const WGPUConstantEntry,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuComputeState {
    /// Struct field `module`.
    pub module: SubscriptTypegpuShaderModule,
    /// Struct field `entryPoint`.
    pub entry_point: SubscriptTypegpuStringView,
    /// Element count for `constants`.
    pub constants_count: usize,
    /// Struct field `constants`.
    pub constants: *const SubscriptTypegpuConstantEntry,
}

struct ConvertedComputeState {
    value: WGPUComputeState,
    _constants: Vec<WGPUConstantEntry>,
}

#[allow(dead_code)]
fn convert_compute_state(source: SubscriptTypegpuComputeState) -> ConvertedComputeState {
    let constants: Vec<WGPUConstantEntry> = if source.constants.is_null() {
        Vec::new()
    } else {
        // SAFETY: the boundary pair promises `count` readable elements.
        unsafe { std::slice::from_raw_parts(source.constants, source.constants_count) }
            .iter()
            .copied()
            .map(convert_constant_entry)
            .collect()
    };
    let constants_ptr = if source.constants.is_null() {
        std::ptr::null()
    } else {
        constants.as_ptr()
    };
    let value = WGPUComputeState {
        next_in_chain: std::ptr::null_mut(),
        module: source.module.cast(),
        entry_point: wgpu_string_view(source.entry_point),
        constant_count: source.constants_count,
        constants: constants_ptr,
    };
    ConvertedComputeState {
        value,
        _constants: constants,
    }
}

/// webgpu.h `WGPUComputePipelineDescriptor`.
#[repr(C)]
struct WGPUComputePipelineDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
    layout: WGPUPipelineLayout,
    compute: WGPUComputeState,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuComputePipelineDescriptor {
    /// Struct field `label`.
    pub label: SubscriptTypegpuStringView,
    /// Struct field `layout`.
    pub layout: SubscriptTypegpuPipelineLayout,
    /// Struct field `compute`.
    pub compute: SubscriptTypegpuComputeState,
}

struct ConvertedComputePipelineDescriptor {
    value: WGPUComputePipelineDescriptor,
    _compute: ConvertedComputeState,
}

#[allow(dead_code)]
fn convert_compute_pipeline_descriptor(source: SubscriptTypegpuComputePipelineDescriptor) -> ConvertedComputePipelineDescriptor {
    let compute = convert_compute_state(source.compute);
    let value = WGPUComputePipelineDescriptor {
        next_in_chain: std::ptr::null_mut(),
        label: wgpu_string_view(source.label),
        layout: source.layout.cast(),
        compute: compute.value,
    };
    ConvertedComputePipelineDescriptor {
        value,
        _compute: compute,
    }
}

/// webgpu.h `WGPUVertexAttribute`.
#[repr(C)]
struct WGPUVertexAttribute {
    next_in_chain: *mut WGPUChainedStruct,
    format: i32,
    offset: u64,
    shader_location: u32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuVertexAttribute {
    /// Struct field `format`.
    pub format: i32,
    /// Struct field `offset`.
    pub offset: u64,
    /// Struct field `shaderLocation`.
    pub shader_location: u32,
}

#[allow(dead_code)]
fn convert_vertex_attribute(source: SubscriptTypegpuVertexAttribute) -> WGPUVertexAttribute {
    WGPUVertexAttribute {
        next_in_chain: std::ptr::null_mut(),
        format: source.format,
        offset: source.offset,
        shader_location: source.shader_location,
    }
}

/// webgpu.h `WGPUVertexBufferLayout`.
#[repr(C)]
#[derive(Clone, Copy)]
struct WGPUVertexBufferLayout {
    next_in_chain: *mut WGPUChainedStruct,
    step_mode: i32,
    array_stride: u64,
    attribute_count: usize,
    attributes: *const WGPUVertexAttribute,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuVertexBufferLayout {
    /// Struct field `stepMode`.
    pub step_mode: i32,
    /// Struct field `arrayStride`.
    pub array_stride: u64,
    /// Element count for `attributes`.
    pub attributes_count: usize,
    /// Struct field `attributes`.
    pub attributes: *const SubscriptTypegpuVertexAttribute,
}

struct ConvertedVertexBufferLayout {
    value: WGPUVertexBufferLayout,
    _attributes: Vec<WGPUVertexAttribute>,
}

#[allow(dead_code)]
fn convert_vertex_buffer_layout(source: SubscriptTypegpuVertexBufferLayout) -> ConvertedVertexBufferLayout {
    let attributes: Vec<WGPUVertexAttribute> = if source.attributes.is_null() {
        Vec::new()
    } else {
        // SAFETY: the boundary pair promises `count` readable elements.
        unsafe { std::slice::from_raw_parts(source.attributes, source.attributes_count) }
            .iter()
            .copied()
            .map(convert_vertex_attribute)
            .collect()
    };
    let attributes_ptr = if source.attributes.is_null() {
        std::ptr::null()
    } else {
        attributes.as_ptr()
    };
    let value = WGPUVertexBufferLayout {
        next_in_chain: std::ptr::null_mut(),
        step_mode: source.step_mode,
        array_stride: source.array_stride,
        attribute_count: source.attributes_count,
        attributes: attributes_ptr,
    };
    ConvertedVertexBufferLayout {
        value,
        _attributes: attributes,
    }
}

/// webgpu.h `WGPUVertexState`.
#[repr(C)]
#[derive(Clone, Copy)]
struct WGPUVertexState {
    next_in_chain: *mut WGPUChainedStruct,
    module: WGPUShaderModule,
    entry_point: WGPUStringView,
    constant_count: usize,
    constants: *const WGPUConstantEntry,
    buffer_count: usize,
    buffers: *const WGPUVertexBufferLayout,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuVertexState {
    /// Struct field `module`.
    pub module: SubscriptTypegpuShaderModule,
    /// Struct field `entryPoint`.
    pub entry_point: SubscriptTypegpuStringView,
    /// Element count for `constants`.
    pub constants_count: usize,
    /// Struct field `constants`.
    pub constants: *const SubscriptTypegpuConstantEntry,
    /// Element count for `buffers`.
    pub buffers_count: usize,
    /// Struct field `buffers`.
    pub buffers: *const SubscriptTypegpuVertexBufferLayout,
}

struct ConvertedVertexState {
    value: WGPUVertexState,
    _constants: Vec<WGPUConstantEntry>,
    _buffers_converted: Vec<Box<ConvertedVertexBufferLayout>>,
    _buffers: Vec<WGPUVertexBufferLayout>,
}

#[allow(dead_code)]
fn convert_vertex_state(source: SubscriptTypegpuVertexState) -> ConvertedVertexState {
    let constants: Vec<WGPUConstantEntry> = if source.constants.is_null() {
        Vec::new()
    } else {
        // SAFETY: the boundary pair promises `count` readable elements.
        unsafe { std::slice::from_raw_parts(source.constants, source.constants_count) }
            .iter()
            .copied()
            .map(convert_constant_entry)
            .collect()
    };
    let constants_ptr = if source.constants.is_null() {
        std::ptr::null()
    } else {
        constants.as_ptr()
    };
    let buffers_converted: Vec<Box<ConvertedVertexBufferLayout>> = if source.buffers.is_null() {
        Vec::new()
    } else {
        // SAFETY: the boundary pair promises `count` readable elements.
        unsafe { std::slice::from_raw_parts(source.buffers, source.buffers_count) }
            .iter()
            .copied()
            .map(|item| Box::new(convert_vertex_buffer_layout(item)))
            .collect()
    };
    let buffers: Vec<WGPUVertexBufferLayout> = buffers_converted.iter().map(|item| item.value).collect();
    let buffers_ptr = if source.buffers.is_null() {
        std::ptr::null()
    } else {
        buffers.as_ptr()
    };
    let value = WGPUVertexState {
        next_in_chain: std::ptr::null_mut(),
        module: source.module.cast(),
        entry_point: wgpu_string_view(source.entry_point),
        constant_count: source.constants_count,
        constants: constants_ptr,
        buffer_count: source.buffers_count,
        buffers: buffers_ptr,
    };
    ConvertedVertexState {
        value,
        _constants: constants,
        _buffers_converted: buffers_converted,
        _buffers: buffers,
    }
}

/// webgpu.h `WGPUPrimitiveState`.
#[repr(C)]
struct WGPUPrimitiveState {
    next_in_chain: *mut WGPUChainedStruct,
    topology: i32,
    strip_index_format: i32,
    front_face: i32,
    cull_mode: i32,
    unclipped_depth: u32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuPrimitiveState {
    /// Struct field `topology`.
    pub topology: i32,
    /// Struct field `stripIndexFormat`.
    pub strip_index_format: i32,
    /// Struct field `frontFace`.
    pub front_face: i32,
    /// Struct field `cullMode`.
    pub cull_mode: i32,
    /// Struct field `unclippedDepth`.
    pub unclipped_depth: bool,
}

#[allow(dead_code)]
fn convert_primitive_state(source: SubscriptTypegpuPrimitiveState) -> WGPUPrimitiveState {
    WGPUPrimitiveState {
        next_in_chain: std::ptr::null_mut(),
        topology: source.topology,
        strip_index_format: source.strip_index_format,
        front_face: source.front_face,
        cull_mode: source.cull_mode,
        unclipped_depth: u32::from(source.unclipped_depth),
    }
}

/// webgpu.h `WGPUStencilFaceState`.
#[repr(C)]
struct WGPUStencilFaceState {
    compare: i32,
    fail_op: i32,
    depth_fail_op: i32,
    pass_op: i32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuStencilFaceState {
    /// Struct field `compare`.
    pub compare: i32,
    /// Struct field `failOp`.
    pub fail_op: i32,
    /// Struct field `depthFailOp`.
    pub depth_fail_op: i32,
    /// Struct field `passOp`.
    pub pass_op: i32,
}

#[allow(dead_code)]
fn convert_stencil_face_state(source: SubscriptTypegpuStencilFaceState) -> WGPUStencilFaceState {
    WGPUStencilFaceState {
        compare: source.compare,
        fail_op: source.fail_op,
        depth_fail_op: source.depth_fail_op,
        pass_op: source.pass_op,
    }
}

/// webgpu.h `WGPUDepthStencilState`.
#[repr(C)]
struct WGPUDepthStencilState {
    next_in_chain: *mut WGPUChainedStruct,
    format: i32,
    depth_write_enabled: i32,
    depth_compare: i32,
    stencil_front: WGPUStencilFaceState,
    stencil_back: WGPUStencilFaceState,
    stencil_read_mask: u32,
    stencil_write_mask: u32,
    depth_bias: i32,
    depth_bias_slope_scale: f32,
    depth_bias_clamp: f32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuDepthStencilState {
    /// Struct field `format`.
    pub format: i32,
    /// Struct field `depthWriteEnabled`.
    pub depth_write_enabled: i32,
    /// Struct field `depthCompare`.
    pub depth_compare: i32,
    /// Struct field `stencilFront`.
    pub stencil_front: SubscriptTypegpuStencilFaceState,
    /// Struct field `stencilBack`.
    pub stencil_back: SubscriptTypegpuStencilFaceState,
    /// Struct field `stencilReadMask`.
    pub stencil_read_mask: u32,
    /// Struct field `stencilWriteMask`.
    pub stencil_write_mask: u32,
    /// Struct field `depthBias`.
    pub depth_bias: i32,
    /// Struct field `depthBiasSlopeScale`.
    pub depth_bias_slope_scale: f32,
    /// Struct field `depthBiasClamp`.
    pub depth_bias_clamp: f32,
}

#[allow(dead_code)]
fn convert_depth_stencil_state(source: SubscriptTypegpuDepthStencilState) -> WGPUDepthStencilState {
    WGPUDepthStencilState {
        next_in_chain: std::ptr::null_mut(),
        format: source.format,
        depth_write_enabled: source.depth_write_enabled,
        depth_compare: source.depth_compare,
        stencil_front: convert_stencil_face_state(source.stencil_front),
        stencil_back: convert_stencil_face_state(source.stencil_back),
        stencil_read_mask: source.stencil_read_mask,
        stencil_write_mask: source.stencil_write_mask,
        depth_bias: source.depth_bias,
        depth_bias_slope_scale: source.depth_bias_slope_scale,
        depth_bias_clamp: source.depth_bias_clamp,
    }
}

/// webgpu.h `WGPUMultisampleState`.
#[repr(C)]
struct WGPUMultisampleState {
    next_in_chain: *mut WGPUChainedStruct,
    count: u32,
    mask: u32,
    alpha_to_coverage_enabled: u32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuMultisampleState {
    /// Struct field `count`.
    pub count: u32,
    /// Struct field `mask`.
    pub mask: u32,
    /// Struct field `alphaToCoverageEnabled`.
    pub alpha_to_coverage_enabled: bool,
}

#[allow(dead_code)]
fn convert_multisample_state(source: SubscriptTypegpuMultisampleState) -> WGPUMultisampleState {
    WGPUMultisampleState {
        next_in_chain: std::ptr::null_mut(),
        count: source.count,
        mask: source.mask,
        alpha_to_coverage_enabled: u32::from(source.alpha_to_coverage_enabled),
    }
}

/// webgpu.h `WGPUBlendComponent`.
#[repr(C)]
struct WGPUBlendComponent {
    operation: i32,
    src_factor: i32,
    dst_factor: i32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuBlendComponent {
    /// Struct field `operation`.
    pub operation: i32,
    /// Struct field `srcFactor`.
    pub src_factor: i32,
    /// Struct field `dstFactor`.
    pub dst_factor: i32,
}

#[allow(dead_code)]
fn convert_blend_component(source: SubscriptTypegpuBlendComponent) -> WGPUBlendComponent {
    WGPUBlendComponent {
        operation: source.operation,
        src_factor: source.src_factor,
        dst_factor: source.dst_factor,
    }
}

/// webgpu.h `WGPUBlendState`.
#[repr(C)]
struct WGPUBlendState {
    color: WGPUBlendComponent,
    alpha: WGPUBlendComponent,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuBlendState {
    /// Struct field `color`.
    pub color: SubscriptTypegpuBlendComponent,
    /// Struct field `alpha`.
    pub alpha: SubscriptTypegpuBlendComponent,
}

#[allow(dead_code)]
fn convert_blend_state(source: SubscriptTypegpuBlendState) -> WGPUBlendState {
    WGPUBlendState {
        color: convert_blend_component(source.color),
        alpha: convert_blend_component(source.alpha),
    }
}

/// webgpu.h `WGPUColorTargetState`.
#[repr(C)]
#[derive(Clone, Copy)]
struct WGPUColorTargetState {
    next_in_chain: *mut WGPUChainedStruct,
    format: i32,
    blend: *const WGPUBlendState,
    write_mask: u64,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuColorTargetState {
    /// Struct field `format`.
    pub format: i32,
    /// Struct field `blend`.
    pub blend: *const SubscriptTypegpuBlendState,
    /// Struct field `writeMask`.
    pub write_mask: u64,
}

struct ConvertedColorTargetState {
    value: WGPUColorTargetState,
    _blend: Option<Box<WGPUBlendState>>,
}

#[allow(dead_code)]
fn convert_color_target_state(source: SubscriptTypegpuColorTargetState) -> ConvertedColorTargetState {
    let blend = if source.blend.is_null() {
        None
    } else {
        // SAFETY: a non-null struct pointer is readable for this call.
        Some(Box::new(convert_blend_state(unsafe { *source.blend })))
    };
    let blend_ptr = blend.as_ref().map_or(std::ptr::null(), |value| value.as_ref() as *const _);
    let value = WGPUColorTargetState {
        next_in_chain: std::ptr::null_mut(),
        format: source.format,
        blend: blend_ptr,
        write_mask: source.write_mask,
    };
    ConvertedColorTargetState {
        value,
        _blend: blend,
    }
}

/// webgpu.h `WGPUFragmentState`.
#[repr(C)]
#[derive(Clone, Copy)]
struct WGPUFragmentState {
    next_in_chain: *mut WGPUChainedStruct,
    module: WGPUShaderModule,
    entry_point: WGPUStringView,
    constant_count: usize,
    constants: *const WGPUConstantEntry,
    target_count: usize,
    targets: *const WGPUColorTargetState,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuFragmentState {
    /// Struct field `module`.
    pub module: SubscriptTypegpuShaderModule,
    /// Struct field `entryPoint`.
    pub entry_point: SubscriptTypegpuStringView,
    /// Element count for `constants`.
    pub constants_count: usize,
    /// Struct field `constants`.
    pub constants: *const SubscriptTypegpuConstantEntry,
    /// Element count for `targets`.
    pub targets_count: usize,
    /// Struct field `targets`.
    pub targets: *const SubscriptTypegpuColorTargetState,
}

struct ConvertedFragmentState {
    value: WGPUFragmentState,
    _constants: Vec<WGPUConstantEntry>,
    _targets_converted: Vec<Box<ConvertedColorTargetState>>,
    _targets: Vec<WGPUColorTargetState>,
}

#[allow(dead_code)]
fn convert_fragment_state(source: SubscriptTypegpuFragmentState) -> ConvertedFragmentState {
    let constants: Vec<WGPUConstantEntry> = if source.constants.is_null() {
        Vec::new()
    } else {
        // SAFETY: the boundary pair promises `count` readable elements.
        unsafe { std::slice::from_raw_parts(source.constants, source.constants_count) }
            .iter()
            .copied()
            .map(convert_constant_entry)
            .collect()
    };
    let constants_ptr = if source.constants.is_null() {
        std::ptr::null()
    } else {
        constants.as_ptr()
    };
    let targets_converted: Vec<Box<ConvertedColorTargetState>> = if source.targets.is_null() {
        Vec::new()
    } else {
        // SAFETY: the boundary pair promises `count` readable elements.
        unsafe { std::slice::from_raw_parts(source.targets, source.targets_count) }
            .iter()
            .copied()
            .map(|item| Box::new(convert_color_target_state(item)))
            .collect()
    };
    let targets: Vec<WGPUColorTargetState> = targets_converted.iter().map(|item| item.value).collect();
    let targets_ptr = if source.targets.is_null() {
        std::ptr::null()
    } else {
        targets.as_ptr()
    };
    let value = WGPUFragmentState {
        next_in_chain: std::ptr::null_mut(),
        module: source.module.cast(),
        entry_point: wgpu_string_view(source.entry_point),
        constant_count: source.constants_count,
        constants: constants_ptr,
        target_count: source.targets_count,
        targets: targets_ptr,
    };
    ConvertedFragmentState {
        value,
        _constants: constants,
        _targets_converted: targets_converted,
        _targets: targets,
    }
}

/// webgpu.h `WGPURenderPipelineDescriptor`.
#[repr(C)]
struct WGPURenderPipelineDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
    layout: WGPUPipelineLayout,
    vertex: WGPUVertexState,
    primitive: WGPUPrimitiveState,
    depth_stencil: *const WGPUDepthStencilState,
    multisample: WGPUMultisampleState,
    fragment: *const WGPUFragmentState,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuRenderPipelineDescriptor {
    /// Struct field `label`.
    pub label: SubscriptTypegpuStringView,
    /// Struct field `layout`.
    pub layout: SubscriptTypegpuPipelineLayout,
    /// Struct field `vertex`.
    pub vertex: SubscriptTypegpuVertexState,
    /// Struct field `primitive`.
    pub primitive: SubscriptTypegpuPrimitiveState,
    /// Struct field `depthStencil`.
    pub depth_stencil: *const SubscriptTypegpuDepthStencilState,
    /// Struct field `multisample`.
    pub multisample: SubscriptTypegpuMultisampleState,
    /// Struct field `fragment`.
    pub fragment: *const SubscriptTypegpuFragmentState,
}

struct ConvertedRenderPipelineDescriptor {
    value: WGPURenderPipelineDescriptor,
    _vertex: ConvertedVertexState,
    _depth_stencil: Option<Box<WGPUDepthStencilState>>,
    _fragment: Option<Box<ConvertedFragmentState>>,
}

#[allow(dead_code)]
fn convert_render_pipeline_descriptor(source: SubscriptTypegpuRenderPipelineDescriptor) -> ConvertedRenderPipelineDescriptor {
    let vertex = convert_vertex_state(source.vertex);
    let depth_stencil = if source.depth_stencil.is_null() {
        None
    } else {
        // SAFETY: a non-null struct pointer is readable for this call.
        Some(Box::new(convert_depth_stencil_state(unsafe { *source.depth_stencil })))
    };
    let depth_stencil_ptr = depth_stencil.as_ref().map_or(std::ptr::null(), |value| value.as_ref() as *const _);
    let fragment = if source.fragment.is_null() {
        None
    } else {
        // SAFETY: a non-null struct pointer is readable for this call.
        Some(Box::new(convert_fragment_state(unsafe { *source.fragment })))
    };
    let fragment_ptr = fragment.as_ref().map_or(std::ptr::null(), |value| &value.value as *const _);
    let value = WGPURenderPipelineDescriptor {
        next_in_chain: std::ptr::null_mut(),
        label: wgpu_string_view(source.label),
        layout: source.layout.cast(),
        vertex: vertex.value,
        primitive: convert_primitive_state(source.primitive),
        depth_stencil: depth_stencil_ptr,
        multisample: convert_multisample_state(source.multisample),
        fragment: fragment_ptr,
    };
    ConvertedRenderPipelineDescriptor {
        value,
        _vertex: vertex,
        _depth_stencil: depth_stencil,
        _fragment: fragment,
    }
}

/// webgpu.h `WGPUCommandEncoderDescriptor`.
#[repr(C)]
struct WGPUCommandEncoderDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuCommandEncoderDescriptor {
    /// Struct field `label`.
    pub label: SubscriptTypegpuStringView,
}

#[allow(dead_code)]
fn convert_command_encoder_descriptor(source: SubscriptTypegpuCommandEncoderDescriptor) -> WGPUCommandEncoderDescriptor {
    WGPUCommandEncoderDescriptor {
        next_in_chain: std::ptr::null_mut(),
        label: wgpu_string_view(source.label),
    }
}

/// webgpu.h `WGPURenderBundleEncoderDescriptor`.
#[repr(C)]
struct WGPURenderBundleEncoderDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
    color_format_count: usize,
    color_formats: *const i32,
    depth_stencil_format: i32,
    sample_count: u32,
    depth_read_only: u32,
    stencil_read_only: u32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuRenderBundleEncoderDescriptor {
    /// Struct field `label`.
    pub label: SubscriptTypegpuStringView,
    /// Element count for `colorFormats`.
    pub color_formats_count: usize,
    /// Struct field `colorFormats`.
    pub color_formats: *const i32,
    /// Struct field `depthStencilFormat`.
    pub depth_stencil_format: i32,
    /// Struct field `sampleCount`.
    pub sample_count: u32,
    /// Struct field `depthReadOnly`.
    pub depth_read_only: bool,
    /// Struct field `stencilReadOnly`.
    pub stencil_read_only: bool,
}

#[allow(dead_code)]
fn convert_render_bundle_encoder_descriptor(source: SubscriptTypegpuRenderBundleEncoderDescriptor) -> WGPURenderBundleEncoderDescriptor {
    WGPURenderBundleEncoderDescriptor {
        next_in_chain: std::ptr::null_mut(),
        label: wgpu_string_view(source.label),
        color_format_count: source.color_formats_count,
        color_formats: source.color_formats,
        depth_stencil_format: source.depth_stencil_format,
        sample_count: source.sample_count,
        depth_read_only: u32::from(source.depth_read_only),
        stencil_read_only: u32::from(source.stencil_read_only),
    }
}

/// webgpu.h `WGPUQuerySetDescriptor`.
#[repr(C)]
struct WGPUQuerySetDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
    r#type: i32,
    count: u32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuQuerySetDescriptor {
    /// Struct field `label`.
    pub label: SubscriptTypegpuStringView,
    /// Struct field `type`.
    pub r#type: i32,
    /// Struct field `count`.
    pub count: u32,
}

#[allow(dead_code)]
fn convert_query_set_descriptor(source: SubscriptTypegpuQuerySetDescriptor) -> WGPUQuerySetDescriptor {
    WGPUQuerySetDescriptor {
        next_in_chain: std::ptr::null_mut(),
        label: wgpu_string_view(source.label),
        r#type: source.r#type,
        count: source.count,
    }
}

/// webgpu.h `WGPUOrigin3D`.
#[repr(C)]
struct WGPUOrigin3D {
    x: u32,
    y: u32,
    z: u32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuOrigin3D {
    /// Struct field `x`.
    pub x: u32,
    /// Struct field `y`.
    pub y: u32,
    /// Struct field `z`.
    pub z: u32,
}

#[allow(dead_code)]
fn convert_origin_3D(source: SubscriptTypegpuOrigin3D) -> WGPUOrigin3D {
    WGPUOrigin3D {
        x: source.x,
        y: source.y,
        z: source.z,
    }
}

/// webgpu.h `WGPUTexelCopyTextureInfo`.
#[repr(C)]
struct WGPUTexelCopyTextureInfo {
    texture: WGPUTexture,
    mip_level: u32,
    origin: WGPUOrigin3D,
    aspect: i32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuTexelCopyTextureInfo {
    /// Struct field `texture`.
    pub texture: SubscriptTypegpuTexture,
    /// Struct field `mipLevel`.
    pub mip_level: u32,
    /// Struct field `origin`.
    pub origin: SubscriptTypegpuOrigin3D,
    /// Struct field `aspect`.
    pub aspect: i32,
}

#[allow(dead_code)]
fn convert_texel_copy_texture_info(source: SubscriptTypegpuTexelCopyTextureInfo) -> WGPUTexelCopyTextureInfo {
    WGPUTexelCopyTextureInfo {
        texture: source.texture.cast(),
        mip_level: source.mip_level,
        origin: convert_origin_3D(source.origin),
        aspect: source.aspect,
    }
}

/// webgpu.h `WGPUTexelCopyBufferLayout`.
#[repr(C)]
struct WGPUTexelCopyBufferLayout {
    offset: u64,
    bytes_per_row: u32,
    rows_per_image: u32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuTexelCopyBufferLayout {
    /// Struct field `offset`.
    pub offset: u64,
    /// Struct field `bytesPerRow`.
    pub bytes_per_row: u32,
    /// Struct field `rowsPerImage`.
    pub rows_per_image: u32,
}

#[allow(dead_code)]
fn convert_texel_copy_buffer_layout(source: SubscriptTypegpuTexelCopyBufferLayout) -> WGPUTexelCopyBufferLayout {
    WGPUTexelCopyBufferLayout {
        offset: source.offset,
        bytes_per_row: source.bytes_per_row,
        rows_per_image: source.rows_per_image,
    }
}

/// webgpu.h `WGPUTextureViewDescriptor`.
#[repr(C)]
struct WGPUTextureViewDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
    format: i32,
    dimension: i32,
    base_mip_level: u32,
    mip_level_count: u32,
    base_array_layer: u32,
    array_layer_count: u32,
    aspect: i32,
    usage: u64,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuTextureViewDescriptor {
    /// Struct field `label`.
    pub label: SubscriptTypegpuStringView,
    /// Struct field `format`.
    pub format: i32,
    /// Struct field `dimension`.
    pub dimension: i32,
    /// Struct field `baseMipLevel`.
    pub base_mip_level: u32,
    /// Struct field `mipLevelCount`.
    pub mip_level_count: u32,
    /// Struct field `baseArrayLayer`.
    pub base_array_layer: u32,
    /// Struct field `arrayLayerCount`.
    pub array_layer_count: u32,
    /// Struct field `aspect`.
    pub aspect: i32,
    /// Struct field `usage`.
    pub usage: u64,
}

#[allow(dead_code)]
fn convert_texture_view_descriptor(source: SubscriptTypegpuTextureViewDescriptor) -> WGPUTextureViewDescriptor {
    WGPUTextureViewDescriptor {
        next_in_chain: std::ptr::null_mut(),
        label: wgpu_string_view(source.label),
        format: source.format,
        dimension: source.dimension,
        base_mip_level: source.base_mip_level,
        mip_level_count: source.mip_level_count,
        base_array_layer: source.base_array_layer,
        array_layer_count: source.array_layer_count,
        aspect: source.aspect,
        usage: source.usage,
    }
}

/// webgpu.h `WGPUCommandBufferDescriptor`.
#[repr(C)]
struct WGPUCommandBufferDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuCommandBufferDescriptor {
    /// Struct field `label`.
    pub label: SubscriptTypegpuStringView,
}

#[allow(dead_code)]
fn convert_command_buffer_descriptor(source: SubscriptTypegpuCommandBufferDescriptor) -> WGPUCommandBufferDescriptor {
    WGPUCommandBufferDescriptor {
        next_in_chain: std::ptr::null_mut(),
        label: wgpu_string_view(source.label),
    }
}

/// webgpu.h `WGPUPassTimestampWrites`.
#[repr(C)]
struct WGPUPassTimestampWrites {
    next_in_chain: *mut WGPUChainedStruct,
    query_set: WGPUQuerySet,
    beginning_of_pass_write_index: u32,
    end_of_pass_write_index: u32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuPassTimestampWrites {
    /// Struct field `querySet`.
    pub query_set: SubscriptTypegpuQuerySet,
    /// Struct field `beginningOfPassWriteIndex`.
    pub beginning_of_pass_write_index: u32,
    /// Struct field `endOfPassWriteIndex`.
    pub end_of_pass_write_index: u32,
}

#[allow(dead_code)]
fn convert_pass_timestamp_writes(source: SubscriptTypegpuPassTimestampWrites) -> WGPUPassTimestampWrites {
    WGPUPassTimestampWrites {
        next_in_chain: std::ptr::null_mut(),
        query_set: source.query_set.cast(),
        beginning_of_pass_write_index: source.beginning_of_pass_write_index,
        end_of_pass_write_index: source.end_of_pass_write_index,
    }
}

/// webgpu.h `WGPUComputePassDescriptor`.
#[repr(C)]
struct WGPUComputePassDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
    timestamp_writes: *const WGPUPassTimestampWrites,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuComputePassDescriptor {
    /// Struct field `label`.
    pub label: SubscriptTypegpuStringView,
    /// Struct field `timestampWrites`.
    pub timestamp_writes: *const SubscriptTypegpuPassTimestampWrites,
}

struct ConvertedComputePassDescriptor {
    value: WGPUComputePassDescriptor,
    _timestamp_writes: Option<Box<WGPUPassTimestampWrites>>,
}

#[allow(dead_code)]
fn convert_compute_pass_descriptor(source: SubscriptTypegpuComputePassDescriptor) -> ConvertedComputePassDescriptor {
    let timestamp_writes = if source.timestamp_writes.is_null() {
        None
    } else {
        // SAFETY: a non-null struct pointer is readable for this call.
        Some(Box::new(convert_pass_timestamp_writes(unsafe { *source.timestamp_writes })))
    };
    let timestamp_writes_ptr = timestamp_writes.as_ref().map_or(std::ptr::null(), |value| value.as_ref() as *const _);
    let value = WGPUComputePassDescriptor {
        next_in_chain: std::ptr::null_mut(),
        label: wgpu_string_view(source.label),
        timestamp_writes: timestamp_writes_ptr,
    };
    ConvertedComputePassDescriptor {
        value,
        _timestamp_writes: timestamp_writes,
    }
}

/// webgpu.h `WGPUColor`.
#[repr(C)]
struct WGPUColor {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuColor {
    /// Struct field `r`.
    pub r: f64,
    /// Struct field `g`.
    pub g: f64,
    /// Struct field `b`.
    pub b: f64,
    /// Struct field `a`.
    pub a: f64,
}

#[allow(dead_code)]
fn convert_color(source: SubscriptTypegpuColor) -> WGPUColor {
    WGPUColor {
        r: source.r,
        g: source.g,
        b: source.b,
        a: source.a,
    }
}

/// webgpu.h `WGPURenderPassColorAttachment`.
#[repr(C)]
struct WGPURenderPassColorAttachment {
    next_in_chain: *mut WGPUChainedStruct,
    view: WGPUTextureView,
    depth_slice: u32,
    resolve_target: WGPUTextureView,
    load_op: i32,
    store_op: i32,
    clear_value: WGPUColor,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuRenderPassColorAttachment {
    /// Struct field `view`.
    pub view: SubscriptTypegpuTextureView,
    /// Struct field `depthSlice`.
    pub depth_slice: u32,
    /// Struct field `resolveTarget`.
    pub resolve_target: SubscriptTypegpuTextureView,
    /// Struct field `loadOp`.
    pub load_op: i32,
    /// Struct field `storeOp`.
    pub store_op: i32,
    /// Struct field `clearValue`.
    pub clear_value: SubscriptTypegpuColor,
}

#[allow(dead_code)]
fn convert_render_pass_color_attachment(source: SubscriptTypegpuRenderPassColorAttachment) -> WGPURenderPassColorAttachment {
    WGPURenderPassColorAttachment {
        next_in_chain: std::ptr::null_mut(),
        view: source.view.cast(),
        depth_slice: source.depth_slice,
        resolve_target: source.resolve_target.cast(),
        load_op: source.load_op,
        store_op: source.store_op,
        clear_value: convert_color(source.clear_value),
    }
}

/// webgpu.h `WGPURenderPassDepthStencilAttachment`.
#[repr(C)]
struct WGPURenderPassDepthStencilAttachment {
    next_in_chain: *mut WGPUChainedStruct,
    view: WGPUTextureView,
    depth_load_op: i32,
    depth_store_op: i32,
    depth_clear_value: f32,
    depth_read_only: u32,
    stencil_load_op: i32,
    stencil_store_op: i32,
    stencil_clear_value: u32,
    stencil_read_only: u32,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuRenderPassDepthStencilAttachment {
    /// Struct field `view`.
    pub view: SubscriptTypegpuTextureView,
    /// Struct field `depthLoadOp`.
    pub depth_load_op: i32,
    /// Struct field `depthStoreOp`.
    pub depth_store_op: i32,
    /// Struct field `depthClearValue`.
    pub depth_clear_value: f32,
    /// Struct field `depthReadOnly`.
    pub depth_read_only: bool,
    /// Struct field `stencilLoadOp`.
    pub stencil_load_op: i32,
    /// Struct field `stencilStoreOp`.
    pub stencil_store_op: i32,
    /// Struct field `stencilClearValue`.
    pub stencil_clear_value: u32,
    /// Struct field `stencilReadOnly`.
    pub stencil_read_only: bool,
}

#[allow(dead_code)]
fn convert_render_pass_depth_stencil_attachment(source: SubscriptTypegpuRenderPassDepthStencilAttachment) -> WGPURenderPassDepthStencilAttachment {
    WGPURenderPassDepthStencilAttachment {
        next_in_chain: std::ptr::null_mut(),
        view: source.view.cast(),
        depth_load_op: source.depth_load_op,
        depth_store_op: source.depth_store_op,
        depth_clear_value: source.depth_clear_value,
        depth_read_only: u32::from(source.depth_read_only),
        stencil_load_op: source.stencil_load_op,
        stencil_store_op: source.stencil_store_op,
        stencil_clear_value: source.stencil_clear_value,
        stencil_read_only: u32::from(source.stencil_read_only),
    }
}

/// webgpu.h `WGPURenderPassDescriptor`.
#[repr(C)]
struct WGPURenderPassDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
    color_attachment_count: usize,
    color_attachments: *const WGPURenderPassColorAttachment,
    depth_stencil_attachment: *const WGPURenderPassDepthStencilAttachment,
    occlusion_query_set: WGPUQuerySet,
    timestamp_writes: *const WGPUPassTimestampWrites,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuRenderPassDescriptor {
    /// Struct field `label`.
    pub label: SubscriptTypegpuStringView,
    /// Element count for `colorAttachments`.
    pub color_attachments_count: usize,
    /// Struct field `colorAttachments`.
    pub color_attachments: *const SubscriptTypegpuRenderPassColorAttachment,
    /// Struct field `depthStencilAttachment`.
    pub depth_stencil_attachment: *const SubscriptTypegpuRenderPassDepthStencilAttachment,
    /// Struct field `occlusionQuerySet`.
    pub occlusion_query_set: SubscriptTypegpuQuerySet,
    /// Struct field `timestampWrites`.
    pub timestamp_writes: *const SubscriptTypegpuPassTimestampWrites,
}

struct ConvertedRenderPassDescriptor {
    value: WGPURenderPassDescriptor,
    _color_attachments: Vec<WGPURenderPassColorAttachment>,
    _depth_stencil_attachment: Option<Box<WGPURenderPassDepthStencilAttachment>>,
    _timestamp_writes: Option<Box<WGPUPassTimestampWrites>>,
}

#[allow(dead_code)]
fn convert_render_pass_descriptor(source: SubscriptTypegpuRenderPassDescriptor) -> ConvertedRenderPassDescriptor {
    let color_attachments: Vec<WGPURenderPassColorAttachment> = if source.color_attachments.is_null() {
        Vec::new()
    } else {
        // SAFETY: the boundary pair promises `count` readable elements.
        unsafe { std::slice::from_raw_parts(source.color_attachments, source.color_attachments_count) }
            .iter()
            .copied()
            .map(convert_render_pass_color_attachment)
            .collect()
    };
    let color_attachments_ptr = if source.color_attachments.is_null() {
        std::ptr::null()
    } else {
        color_attachments.as_ptr()
    };
    let depth_stencil_attachment = if source.depth_stencil_attachment.is_null() {
        None
    } else {
        // SAFETY: a non-null struct pointer is readable for this call.
        Some(Box::new(convert_render_pass_depth_stencil_attachment(unsafe { *source.depth_stencil_attachment })))
    };
    let depth_stencil_attachment_ptr = depth_stencil_attachment.as_ref().map_or(std::ptr::null(), |value| value.as_ref() as *const _);
    let timestamp_writes = if source.timestamp_writes.is_null() {
        None
    } else {
        // SAFETY: a non-null struct pointer is readable for this call.
        Some(Box::new(convert_pass_timestamp_writes(unsafe { *source.timestamp_writes })))
    };
    let timestamp_writes_ptr = timestamp_writes.as_ref().map_or(std::ptr::null(), |value| value.as_ref() as *const _);
    let value = WGPURenderPassDescriptor {
        next_in_chain: std::ptr::null_mut(),
        label: wgpu_string_view(source.label),
        color_attachment_count: source.color_attachments_count,
        color_attachments: color_attachments_ptr,
        depth_stencil_attachment: depth_stencil_attachment_ptr,
        occlusion_query_set: source.occlusion_query_set.cast(),
        timestamp_writes: timestamp_writes_ptr,
    };
    ConvertedRenderPassDescriptor {
        value,
        _color_attachments: color_attachments,
        _depth_stencil_attachment: depth_stencil_attachment,
        _timestamp_writes: timestamp_writes,
    }
}

/// webgpu.h `WGPUTexelCopyBufferInfo`.
#[repr(C)]
struct WGPUTexelCopyBufferInfo {
    layout: WGPUTexelCopyBufferLayout,
    buffer: WGPUBuffer,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuTexelCopyBufferInfo {
    /// Struct field `layout`.
    pub layout: SubscriptTypegpuTexelCopyBufferLayout,
    /// Struct field `buffer`.
    pub buffer: SubscriptTypegpuBuffer,
}

#[allow(dead_code)]
fn convert_texel_copy_buffer_info(source: SubscriptTypegpuTexelCopyBufferInfo) -> WGPUTexelCopyBufferInfo {
    WGPUTexelCopyBufferInfo {
        layout: convert_texel_copy_buffer_layout(source.layout),
        buffer: source.buffer.cast(),
    }
}

/// webgpu.h `WGPURenderBundleDescriptor`.
#[repr(C)]
struct WGPURenderBundleDescriptor {
    next_in_chain: *mut WGPUChainedStruct,
    label: WGPUStringView,
}

/// `subscript-typegpu.h`: chain-free struct.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuRenderBundleDescriptor {
    /// Struct field `label`.
    pub label: SubscriptTypegpuStringView,
}

#[allow(dead_code)]
fn convert_render_bundle_descriptor(source: SubscriptTypegpuRenderBundleDescriptor) -> WGPURenderBundleDescriptor {
    WGPURenderBundleDescriptor {
        next_in_chain: std::ptr::null_mut(),
        label: wgpu_string_view(source.label),
    }
}

pub(crate) struct WebgpuTable {
    pub(crate) _library: libloading::Library,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuCreateInstance: unsafe extern "C" fn(*const WGPUInstanceDescriptor)-> WGPUInstance,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuInstanceProcessEvents: unsafe extern "C" fn(WGPUInstance),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuInstanceRelease: unsafe extern "C" fn(WGPUInstance),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuInstanceRequestAdapter: unsafe extern "C" fn(WGPUInstance, *const WGPURequestAdapterOptions, WGPURequestAdapterCallbackInfo)-> WGPUFuture,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuAdapterGetLimits: unsafe extern "C" fn(WGPUAdapter, *mut WGPULimits)-> i32,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuAdapterGetInfo: unsafe extern "C" fn(WGPUAdapter, *mut WGPUAdapterInfo)-> i32,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuAdapterHasFeature: unsafe extern "C" fn(WGPUAdapter, i32)-> u32,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuAdapterRequestDevice: unsafe extern "C" fn(WGPUAdapter, *const WGPUDeviceDescriptor, WGPURequestDeviceCallbackInfo)-> WGPUFuture,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceGetQueue: unsafe extern "C" fn(WGPUDevice)-> WGPUQueue,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceDestroy: unsafe extern "C" fn(WGPUDevice),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceSetLabel: unsafe extern "C" fn(WGPUDevice, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDevicePushErrorScope: unsafe extern "C" fn(WGPUDevice, i32),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDevicePopErrorScope: unsafe extern "C" fn(WGPUDevice, WGPUPopErrorScopeCallbackInfo)-> WGPUFuture,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceGetLimits: unsafe extern "C" fn(WGPUDevice, *mut WGPULimits)-> i32,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceGetAdapterInfo: unsafe extern "C" fn(WGPUDevice, *mut WGPUAdapterInfo)-> i32,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceHasFeature: unsafe extern "C" fn(WGPUDevice, i32)-> u32,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceCreateBuffer: unsafe extern "C" fn(WGPUDevice, *const WGPUBufferDescriptor)-> WGPUBuffer,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceCreateTexture: unsafe extern "C" fn(WGPUDevice, *const WGPUTextureDescriptor)-> WGPUTexture,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceCreateSampler: unsafe extern "C" fn(WGPUDevice, *const WGPUSamplerDescriptor)-> WGPUSampler,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceCreateBindGroupLayout: unsafe extern "C" fn(WGPUDevice, *const WGPUBindGroupLayoutDescriptor)-> WGPUBindGroupLayout,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceCreateBindGroup: unsafe extern "C" fn(WGPUDevice, *const WGPUBindGroupDescriptor)-> WGPUBindGroup,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceCreatePipelineLayout: unsafe extern "C" fn(WGPUDevice, *const WGPUPipelineLayoutDescriptor)-> WGPUPipelineLayout,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceCreateShaderModule: unsafe extern "C" fn(WGPUDevice, *const WGPUShaderModuleDescriptor)-> WGPUShaderModule,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceCreateComputePipeline: unsafe extern "C" fn(WGPUDevice, *const WGPUComputePipelineDescriptor)-> WGPUComputePipeline,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceCreateComputePipelineAsync: unsafe extern "C" fn(WGPUDevice, *const WGPUComputePipelineDescriptor, WGPUCreateComputePipelineAsyncCallbackInfo)-> WGPUFuture,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceCreateRenderPipeline: unsafe extern "C" fn(WGPUDevice, *const WGPURenderPipelineDescriptor)-> WGPURenderPipeline,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceCreateRenderPipelineAsync: unsafe extern "C" fn(WGPUDevice, *const WGPURenderPipelineDescriptor, WGPUCreateRenderPipelineAsyncCallbackInfo)-> WGPUFuture,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceCreateCommandEncoder: unsafe extern "C" fn(WGPUDevice, *const WGPUCommandEncoderDescriptor)-> WGPUCommandEncoder,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceCreateRenderBundleEncoder: unsafe extern "C" fn(WGPUDevice, *const WGPURenderBundleEncoderDescriptor)-> WGPURenderBundleEncoder,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceCreateQuerySet: unsafe extern "C" fn(WGPUDevice, *const WGPUQuerySetDescriptor)-> WGPUQuerySet,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuQueueSubmit: unsafe extern "C" fn(WGPUQueue, usize, *const WGPUCommandBuffer),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuQueueOnSubmittedWorkDone: unsafe extern "C" fn(WGPUQueue, WGPUQueueWorkDoneCallbackInfo)-> WGPUFuture,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuQueueWriteBuffer: unsafe extern "C" fn(WGPUQueue, WGPUBuffer, u64, *const c_void, usize),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuQueueWriteTexture: unsafe extern "C" fn(WGPUQueue, *const WGPUTexelCopyTextureInfo, *const std::ffi::c_void, usize, *const WGPUTexelCopyBufferLayout, *const WGPUExtent3D),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuQueueSetLabel: unsafe extern "C" fn(WGPUQueue, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuBufferMapAsync: unsafe extern "C" fn(WGPUBuffer, u64, usize, usize, WGPUBufferMapCallbackInfo)-> WGPUFuture,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuBufferReadMappedRange: unsafe extern "C" fn(WGPUBuffer, usize, *mut c_void, usize)-> i32,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuBufferWriteMappedRange: unsafe extern "C" fn(WGPUBuffer, usize, *const c_void, usize)-> i32,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuBufferSetLabel: unsafe extern "C" fn(WGPUBuffer, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuBufferGetUsage: unsafe extern "C" fn(WGPUBuffer)-> u64,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuBufferGetSize: unsafe extern "C" fn(WGPUBuffer)-> u64,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuBufferGetMapState: unsafe extern "C" fn(WGPUBuffer)-> i32,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuBufferUnmap: unsafe extern "C" fn(WGPUBuffer),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuBufferDestroy: unsafe extern "C" fn(WGPUBuffer),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuTextureCreateView: unsafe extern "C" fn(WGPUTexture, *const WGPUTextureViewDescriptor)-> WGPUTextureView,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuTextureSetLabel: unsafe extern "C" fn(WGPUTexture, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuTextureGetWidth: unsafe extern "C" fn(WGPUTexture)-> u32,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuTextureGetHeight: unsafe extern "C" fn(WGPUTexture)-> u32,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuTextureGetDepthOrArrayLayers: unsafe extern "C" fn(WGPUTexture)-> u32,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuTextureGetMipLevelCount: unsafe extern "C" fn(WGPUTexture)-> u32,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuTextureGetSampleCount: unsafe extern "C" fn(WGPUTexture)-> u32,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuTextureGetDimension: unsafe extern "C" fn(WGPUTexture)-> i32,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuTextureGetFormat: unsafe extern "C" fn(WGPUTexture)-> i32,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuTextureGetUsage: unsafe extern "C" fn(WGPUTexture)-> u64,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuTextureDestroy: unsafe extern "C" fn(WGPUTexture),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuTextureViewSetLabel: unsafe extern "C" fn(WGPUTextureView, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuSamplerSetLabel: unsafe extern "C" fn(WGPUSampler, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuBindGroupLayoutSetLabel: unsafe extern "C" fn(WGPUBindGroupLayout, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuBindGroupSetLabel: unsafe extern "C" fn(WGPUBindGroup, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuPipelineLayoutSetLabel: unsafe extern "C" fn(WGPUPipelineLayout, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuShaderModuleSetLabel: unsafe extern "C" fn(WGPUShaderModule, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuComputePipelineGetBindGroupLayout: unsafe extern "C" fn(WGPUComputePipeline, u32)-> WGPUBindGroupLayout,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuComputePipelineSetLabel: unsafe extern "C" fn(WGPUComputePipeline, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPipelineGetBindGroupLayout: unsafe extern "C" fn(WGPURenderPipeline, u32)-> WGPUBindGroupLayout,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPipelineSetLabel: unsafe extern "C" fn(WGPURenderPipeline, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuCommandEncoderFinish: unsafe extern "C" fn(WGPUCommandEncoder, *const WGPUCommandBufferDescriptor)-> WGPUCommandBuffer,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuCommandEncoderBeginComputePass: unsafe extern "C" fn(WGPUCommandEncoder, *const WGPUComputePassDescriptor)-> WGPUComputePassEncoder,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuCommandEncoderBeginRenderPass: unsafe extern "C" fn(WGPUCommandEncoder, *const WGPURenderPassDescriptor)-> WGPURenderPassEncoder,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuCommandEncoderCopyBufferToBuffer: unsafe extern "C" fn(WGPUCommandEncoder, WGPUBuffer, u64, WGPUBuffer, u64, u64),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuCommandEncoderCopyBufferToTexture: unsafe extern "C" fn(WGPUCommandEncoder, *const WGPUTexelCopyBufferInfo, *const WGPUTexelCopyTextureInfo, *const WGPUExtent3D),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuCommandEncoderCopyTextureToBuffer: unsafe extern "C" fn(WGPUCommandEncoder, *const WGPUTexelCopyTextureInfo, *const WGPUTexelCopyBufferInfo, *const WGPUExtent3D),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuCommandEncoderCopyTextureToTexture: unsafe extern "C" fn(WGPUCommandEncoder, *const WGPUTexelCopyTextureInfo, *const WGPUTexelCopyTextureInfo, *const WGPUExtent3D),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuCommandEncoderClearBuffer: unsafe extern "C" fn(WGPUCommandEncoder, WGPUBuffer, u64, u64),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuCommandEncoderResolveQuerySet: unsafe extern "C" fn(WGPUCommandEncoder, WGPUQuerySet, u32, u32, WGPUBuffer, u64),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuCommandEncoderInsertDebugMarker: unsafe extern "C" fn(WGPUCommandEncoder, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuCommandEncoderPushDebugGroup: unsafe extern "C" fn(WGPUCommandEncoder, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuCommandEncoderPopDebugGroup: unsafe extern "C" fn(WGPUCommandEncoder),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuCommandEncoderSetLabel: unsafe extern "C" fn(WGPUCommandEncoder, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuComputePassEncoderSetPipeline: unsafe extern "C" fn(WGPUComputePassEncoder, WGPUComputePipeline),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuComputePassEncoderSetBindGroup: unsafe extern "C" fn(WGPUComputePassEncoder, u32, WGPUBindGroup, usize, *const u32),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuComputePassEncoderDispatchWorkgroups: unsafe extern "C" fn(WGPUComputePassEncoder, u32, u32, u32),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuComputePassEncoderDispatchWorkgroupsIndirect: unsafe extern "C" fn(WGPUComputePassEncoder, WGPUBuffer, u64),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuComputePassEncoderInsertDebugMarker: unsafe extern "C" fn(WGPUComputePassEncoder, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuComputePassEncoderPushDebugGroup: unsafe extern "C" fn(WGPUComputePassEncoder, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuComputePassEncoderPopDebugGroup: unsafe extern "C" fn(WGPUComputePassEncoder),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuComputePassEncoderEnd: unsafe extern "C" fn(WGPUComputePassEncoder),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuComputePassEncoderSetLabel: unsafe extern "C" fn(WGPUComputePassEncoder, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderSetPipeline: unsafe extern "C" fn(WGPURenderPassEncoder, WGPURenderPipeline),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderSetBindGroup: unsafe extern "C" fn(WGPURenderPassEncoder, u32, WGPUBindGroup, usize, *const u32),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderSetVertexBuffer: unsafe extern "C" fn(WGPURenderPassEncoder, u32, WGPUBuffer, u64, u64),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderSetIndexBuffer: unsafe extern "C" fn(WGPURenderPassEncoder, WGPUBuffer, i32, u64, u64),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderDraw: unsafe extern "C" fn(WGPURenderPassEncoder, u32, u32, u32, u32),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderDrawIndexed: unsafe extern "C" fn(WGPURenderPassEncoder, u32, u32, u32, i32, u32),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderDrawIndirect: unsafe extern "C" fn(WGPURenderPassEncoder, WGPUBuffer, u64),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderDrawIndexedIndirect: unsafe extern "C" fn(WGPURenderPassEncoder, WGPUBuffer, u64),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderSetViewport: unsafe extern "C" fn(WGPURenderPassEncoder, f32, f32, f32, f32, f32, f32),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderSetScissorRect: unsafe extern "C" fn(WGPURenderPassEncoder, u32, u32, u32, u32),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderSetBlendConstant: unsafe extern "C" fn(WGPURenderPassEncoder, *const WGPUColor),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderSetStencilReference: unsafe extern "C" fn(WGPURenderPassEncoder, u32),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderBeginOcclusionQuery: unsafe extern "C" fn(WGPURenderPassEncoder, u32),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderEndOcclusionQuery: unsafe extern "C" fn(WGPURenderPassEncoder),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderExecuteBundles: unsafe extern "C" fn(WGPURenderPassEncoder, usize, *const WGPURenderBundle),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderInsertDebugMarker: unsafe extern "C" fn(WGPURenderPassEncoder, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderPushDebugGroup: unsafe extern "C" fn(WGPURenderPassEncoder, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderPopDebugGroup: unsafe extern "C" fn(WGPURenderPassEncoder),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderEnd: unsafe extern "C" fn(WGPURenderPassEncoder),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderSetLabel: unsafe extern "C" fn(WGPURenderPassEncoder, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuCommandBufferSetLabel: unsafe extern "C" fn(WGPUCommandBuffer, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderBundleEncoderSetPipeline: unsafe extern "C" fn(WGPURenderBundleEncoder, WGPURenderPipeline),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderBundleEncoderSetBindGroup: unsafe extern "C" fn(WGPURenderBundleEncoder, u32, WGPUBindGroup, usize, *const u32),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderBundleEncoderSetVertexBuffer: unsafe extern "C" fn(WGPURenderBundleEncoder, u32, WGPUBuffer, u64, u64),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderBundleEncoderSetIndexBuffer: unsafe extern "C" fn(WGPURenderBundleEncoder, WGPUBuffer, i32, u64, u64),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderBundleEncoderDraw: unsafe extern "C" fn(WGPURenderBundleEncoder, u32, u32, u32, u32),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderBundleEncoderDrawIndexed: unsafe extern "C" fn(WGPURenderBundleEncoder, u32, u32, u32, i32, u32),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderBundleEncoderDrawIndirect: unsafe extern "C" fn(WGPURenderBundleEncoder, WGPUBuffer, u64),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderBundleEncoderDrawIndexedIndirect: unsafe extern "C" fn(WGPURenderBundleEncoder, WGPUBuffer, u64),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderBundleEncoderInsertDebugMarker: unsafe extern "C" fn(WGPURenderBundleEncoder, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderBundleEncoderPushDebugGroup: unsafe extern "C" fn(WGPURenderBundleEncoder, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderBundleEncoderPopDebugGroup: unsafe extern "C" fn(WGPURenderBundleEncoder),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderBundleEncoderFinish: unsafe extern "C" fn(WGPURenderBundleEncoder, *const WGPURenderBundleDescriptor)-> WGPURenderBundle,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderBundleEncoderSetLabel: unsafe extern "C" fn(WGPURenderBundleEncoder, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderBundleSetLabel: unsafe extern "C" fn(WGPURenderBundle, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuQuerySetGetType: unsafe extern "C" fn(WGPUQuerySet)-> i32,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuQuerySetGetCount: unsafe extern "C" fn(WGPUQuerySet)-> u32,
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuQuerySetDestroy: unsafe extern "C" fn(WGPUQuerySet),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuQuerySetSetLabel: unsafe extern "C" fn(WGPUQuerySet, WGPUStringView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuAdapterInfoFreeMembers: unsafe extern "C" fn(WGPUAdapterInfo),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuQuerySetRelease: unsafe extern "C" fn(WGPUQuerySet),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderBundleRelease: unsafe extern "C" fn(WGPURenderBundle),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderBundleEncoderRelease: unsafe extern "C" fn(WGPURenderBundleEncoder),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuCommandBufferRelease: unsafe extern "C" fn(WGPUCommandBuffer),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPassEncoderRelease: unsafe extern "C" fn(WGPURenderPassEncoder),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuComputePassEncoderRelease: unsafe extern "C" fn(WGPUComputePassEncoder),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuCommandEncoderRelease: unsafe extern "C" fn(WGPUCommandEncoder),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuRenderPipelineRelease: unsafe extern "C" fn(WGPURenderPipeline),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuComputePipelineRelease: unsafe extern "C" fn(WGPUComputePipeline),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuShaderModuleRelease: unsafe extern "C" fn(WGPUShaderModule),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuPipelineLayoutRelease: unsafe extern "C" fn(WGPUPipelineLayout),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuBindGroupRelease: unsafe extern "C" fn(WGPUBindGroup),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuBindGroupLayoutRelease: unsafe extern "C" fn(WGPUBindGroupLayout),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuSamplerRelease: unsafe extern "C" fn(WGPUSampler),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuTextureViewRelease: unsafe extern "C" fn(WGPUTextureView),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuTextureRelease: unsafe extern "C" fn(WGPUTexture),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuBufferRelease: unsafe extern "C" fn(WGPUBuffer),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuQueueRelease: unsafe extern "C" fn(WGPUQueue),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuDeviceRelease: unsafe extern "C" fn(WGPUDevice),
    // SAFETY: the function pointer signature matches the pinned webgpu.h declaration.
wgpuAdapterRelease: unsafe extern "C" fn(WGPUAdapter),
}

impl WebgpuTable {
    pub(crate) fn load(path: &std::path::Path) -> Result<Self, String> {
        #[cfg(windows)]
        // SAFETY: The library stays owned by the returned table.
        // The Windows flag searches the backend directory for dependent libraries.
        let library = unsafe {
            libloading::os::windows::Library::load_with_flags(
                path,
                libloading::os::windows::LOAD_WITH_ALTERED_SEARCH_PATH,
            )
        }
        .map(libloading::Library::from);
        #[cfg(not(windows))]
        // SAFETY: The library stays owned by the returned table.
        let library = unsafe { libloading::Library::new(path) };
        let library = library
            .map_err(|error| format!("load {}: {error}", path.display()))?;
        fn symbol<T: Copy>(
            library: &libloading::Library,
            path: &std::path::Path,
            name: &'static [u8],
        ) -> Result<T, String> {
            // SAFETY: each call uses the pinned webgpu.h signature for this symbol.
            unsafe { library.get::<T>(name) }
                .map(|value| *value)
                .map_err(|error| {
                    let name = std::str::from_utf8(&name[..name.len() - 1]).unwrap_or("<invalid>");
                    format!("missing symbol {name} in {}: {error}", path.display())
                })
        }
        Ok(Self {
            wgpuCreateInstance: symbol(&library, path, b"wgpuCreateInstance\0")?,
            wgpuInstanceProcessEvents: symbol(&library, path, b"wgpuInstanceProcessEvents\0")?,
            wgpuInstanceRelease: symbol(&library, path, b"wgpuInstanceRelease\0")?,
            wgpuInstanceRequestAdapter: symbol(&library, path, b"wgpuInstanceRequestAdapter\0")?,
            wgpuAdapterGetLimits: symbol(&library, path, b"wgpuAdapterGetLimits\0")?,
            wgpuAdapterGetInfo: symbol(&library, path, b"wgpuAdapterGetInfo\0")?,
            wgpuAdapterHasFeature: symbol(&library, path, b"wgpuAdapterHasFeature\0")?,
            wgpuAdapterRequestDevice: symbol(&library, path, b"wgpuAdapterRequestDevice\0")?,
            wgpuDeviceGetQueue: symbol(&library, path, b"wgpuDeviceGetQueue\0")?,
            wgpuDeviceDestroy: symbol(&library, path, b"wgpuDeviceDestroy\0")?,
            wgpuDeviceSetLabel: symbol(&library, path, b"wgpuDeviceSetLabel\0")?,
            wgpuDevicePushErrorScope: symbol(&library, path, b"wgpuDevicePushErrorScope\0")?,
            wgpuDevicePopErrorScope: symbol(&library, path, b"wgpuDevicePopErrorScope\0")?,
            wgpuDeviceGetLimits: symbol(&library, path, b"wgpuDeviceGetLimits\0")?,
            wgpuDeviceGetAdapterInfo: symbol(&library, path, b"wgpuDeviceGetAdapterInfo\0")?,
            wgpuDeviceHasFeature: symbol(&library, path, b"wgpuDeviceHasFeature\0")?,
            wgpuDeviceCreateBuffer: symbol(&library, path, b"wgpuDeviceCreateBuffer\0")?,
            wgpuDeviceCreateTexture: symbol(&library, path, b"wgpuDeviceCreateTexture\0")?,
            wgpuDeviceCreateSampler: symbol(&library, path, b"wgpuDeviceCreateSampler\0")?,
            wgpuDeviceCreateBindGroupLayout: symbol(&library, path, b"wgpuDeviceCreateBindGroupLayout\0")?,
            wgpuDeviceCreateBindGroup: symbol(&library, path, b"wgpuDeviceCreateBindGroup\0")?,
            wgpuDeviceCreatePipelineLayout: symbol(&library, path, b"wgpuDeviceCreatePipelineLayout\0")?,
            wgpuDeviceCreateShaderModule: symbol(&library, path, b"wgpuDeviceCreateShaderModule\0")?,
            wgpuDeviceCreateComputePipeline: symbol(&library, path, b"wgpuDeviceCreateComputePipeline\0")?,
            wgpuDeviceCreateComputePipelineAsync: symbol(&library, path, b"wgpuDeviceCreateComputePipelineAsync\0")?,
            wgpuDeviceCreateRenderPipeline: symbol(&library, path, b"wgpuDeviceCreateRenderPipeline\0")?,
            wgpuDeviceCreateRenderPipelineAsync: symbol(&library, path, b"wgpuDeviceCreateRenderPipelineAsync\0")?,
            wgpuDeviceCreateCommandEncoder: symbol(&library, path, b"wgpuDeviceCreateCommandEncoder\0")?,
            wgpuDeviceCreateRenderBundleEncoder: symbol(&library, path, b"wgpuDeviceCreateRenderBundleEncoder\0")?,
            wgpuDeviceCreateQuerySet: symbol(&library, path, b"wgpuDeviceCreateQuerySet\0")?,
            wgpuQueueSubmit: symbol(&library, path, b"wgpuQueueSubmit\0")?,
            wgpuQueueOnSubmittedWorkDone: symbol(&library, path, b"wgpuQueueOnSubmittedWorkDone\0")?,
            wgpuQueueWriteBuffer: symbol(&library, path, b"wgpuQueueWriteBuffer\0")?,
            wgpuQueueWriteTexture: symbol(&library, path, b"wgpuQueueWriteTexture\0")?,
            wgpuQueueSetLabel: symbol(&library, path, b"wgpuQueueSetLabel\0")?,
            wgpuBufferMapAsync: symbol(&library, path, b"wgpuBufferMapAsync\0")?,
            wgpuBufferReadMappedRange: symbol(&library, path, b"wgpuBufferReadMappedRange\0")?,
            wgpuBufferWriteMappedRange: symbol(&library, path, b"wgpuBufferWriteMappedRange\0")?,
            wgpuBufferSetLabel: symbol(&library, path, b"wgpuBufferSetLabel\0")?,
            wgpuBufferGetUsage: symbol(&library, path, b"wgpuBufferGetUsage\0")?,
            wgpuBufferGetSize: symbol(&library, path, b"wgpuBufferGetSize\0")?,
            wgpuBufferGetMapState: symbol(&library, path, b"wgpuBufferGetMapState\0")?,
            wgpuBufferUnmap: symbol(&library, path, b"wgpuBufferUnmap\0")?,
            wgpuBufferDestroy: symbol(&library, path, b"wgpuBufferDestroy\0")?,
            wgpuTextureCreateView: symbol(&library, path, b"wgpuTextureCreateView\0")?,
            wgpuTextureSetLabel: symbol(&library, path, b"wgpuTextureSetLabel\0")?,
            wgpuTextureGetWidth: symbol(&library, path, b"wgpuTextureGetWidth\0")?,
            wgpuTextureGetHeight: symbol(&library, path, b"wgpuTextureGetHeight\0")?,
            wgpuTextureGetDepthOrArrayLayers: symbol(&library, path, b"wgpuTextureGetDepthOrArrayLayers\0")?,
            wgpuTextureGetMipLevelCount: symbol(&library, path, b"wgpuTextureGetMipLevelCount\0")?,
            wgpuTextureGetSampleCount: symbol(&library, path, b"wgpuTextureGetSampleCount\0")?,
            wgpuTextureGetDimension: symbol(&library, path, b"wgpuTextureGetDimension\0")?,
            wgpuTextureGetFormat: symbol(&library, path, b"wgpuTextureGetFormat\0")?,
            wgpuTextureGetUsage: symbol(&library, path, b"wgpuTextureGetUsage\0")?,
            wgpuTextureDestroy: symbol(&library, path, b"wgpuTextureDestroy\0")?,
            wgpuTextureViewSetLabel: symbol(&library, path, b"wgpuTextureViewSetLabel\0")?,
            wgpuSamplerSetLabel: symbol(&library, path, b"wgpuSamplerSetLabel\0")?,
            wgpuBindGroupLayoutSetLabel: symbol(&library, path, b"wgpuBindGroupLayoutSetLabel\0")?,
            wgpuBindGroupSetLabel: symbol(&library, path, b"wgpuBindGroupSetLabel\0")?,
            wgpuPipelineLayoutSetLabel: symbol(&library, path, b"wgpuPipelineLayoutSetLabel\0")?,
            wgpuShaderModuleSetLabel: symbol(&library, path, b"wgpuShaderModuleSetLabel\0")?,
            wgpuComputePipelineGetBindGroupLayout: symbol(&library, path, b"wgpuComputePipelineGetBindGroupLayout\0")?,
            wgpuComputePipelineSetLabel: symbol(&library, path, b"wgpuComputePipelineSetLabel\0")?,
            wgpuRenderPipelineGetBindGroupLayout: symbol(&library, path, b"wgpuRenderPipelineGetBindGroupLayout\0")?,
            wgpuRenderPipelineSetLabel: symbol(&library, path, b"wgpuRenderPipelineSetLabel\0")?,
            wgpuCommandEncoderFinish: symbol(&library, path, b"wgpuCommandEncoderFinish\0")?,
            wgpuCommandEncoderBeginComputePass: symbol(&library, path, b"wgpuCommandEncoderBeginComputePass\0")?,
            wgpuCommandEncoderBeginRenderPass: symbol(&library, path, b"wgpuCommandEncoderBeginRenderPass\0")?,
            wgpuCommandEncoderCopyBufferToBuffer: symbol(&library, path, b"wgpuCommandEncoderCopyBufferToBuffer\0")?,
            wgpuCommandEncoderCopyBufferToTexture: symbol(&library, path, b"wgpuCommandEncoderCopyBufferToTexture\0")?,
            wgpuCommandEncoderCopyTextureToBuffer: symbol(&library, path, b"wgpuCommandEncoderCopyTextureToBuffer\0")?,
            wgpuCommandEncoderCopyTextureToTexture: symbol(&library, path, b"wgpuCommandEncoderCopyTextureToTexture\0")?,
            wgpuCommandEncoderClearBuffer: symbol(&library, path, b"wgpuCommandEncoderClearBuffer\0")?,
            wgpuCommandEncoderResolveQuerySet: symbol(&library, path, b"wgpuCommandEncoderResolveQuerySet\0")?,
            wgpuCommandEncoderInsertDebugMarker: symbol(&library, path, b"wgpuCommandEncoderInsertDebugMarker\0")?,
            wgpuCommandEncoderPushDebugGroup: symbol(&library, path, b"wgpuCommandEncoderPushDebugGroup\0")?,
            wgpuCommandEncoderPopDebugGroup: symbol(&library, path, b"wgpuCommandEncoderPopDebugGroup\0")?,
            wgpuCommandEncoderSetLabel: symbol(&library, path, b"wgpuCommandEncoderSetLabel\0")?,
            wgpuComputePassEncoderSetPipeline: symbol(&library, path, b"wgpuComputePassEncoderSetPipeline\0")?,
            wgpuComputePassEncoderSetBindGroup: symbol(&library, path, b"wgpuComputePassEncoderSetBindGroup\0")?,
            wgpuComputePassEncoderDispatchWorkgroups: symbol(&library, path, b"wgpuComputePassEncoderDispatchWorkgroups\0")?,
            wgpuComputePassEncoderDispatchWorkgroupsIndirect: symbol(&library, path, b"wgpuComputePassEncoderDispatchWorkgroupsIndirect\0")?,
            wgpuComputePassEncoderInsertDebugMarker: symbol(&library, path, b"wgpuComputePassEncoderInsertDebugMarker\0")?,
            wgpuComputePassEncoderPushDebugGroup: symbol(&library, path, b"wgpuComputePassEncoderPushDebugGroup\0")?,
            wgpuComputePassEncoderPopDebugGroup: symbol(&library, path, b"wgpuComputePassEncoderPopDebugGroup\0")?,
            wgpuComputePassEncoderEnd: symbol(&library, path, b"wgpuComputePassEncoderEnd\0")?,
            wgpuComputePassEncoderSetLabel: symbol(&library, path, b"wgpuComputePassEncoderSetLabel\0")?,
            wgpuRenderPassEncoderSetPipeline: symbol(&library, path, b"wgpuRenderPassEncoderSetPipeline\0")?,
            wgpuRenderPassEncoderSetBindGroup: symbol(&library, path, b"wgpuRenderPassEncoderSetBindGroup\0")?,
            wgpuRenderPassEncoderSetVertexBuffer: symbol(&library, path, b"wgpuRenderPassEncoderSetVertexBuffer\0")?,
            wgpuRenderPassEncoderSetIndexBuffer: symbol(&library, path, b"wgpuRenderPassEncoderSetIndexBuffer\0")?,
            wgpuRenderPassEncoderDraw: symbol(&library, path, b"wgpuRenderPassEncoderDraw\0")?,
            wgpuRenderPassEncoderDrawIndexed: symbol(&library, path, b"wgpuRenderPassEncoderDrawIndexed\0")?,
            wgpuRenderPassEncoderDrawIndirect: symbol(&library, path, b"wgpuRenderPassEncoderDrawIndirect\0")?,
            wgpuRenderPassEncoderDrawIndexedIndirect: symbol(&library, path, b"wgpuRenderPassEncoderDrawIndexedIndirect\0")?,
            wgpuRenderPassEncoderSetViewport: symbol(&library, path, b"wgpuRenderPassEncoderSetViewport\0")?,
            wgpuRenderPassEncoderSetScissorRect: symbol(&library, path, b"wgpuRenderPassEncoderSetScissorRect\0")?,
            wgpuRenderPassEncoderSetBlendConstant: symbol(&library, path, b"wgpuRenderPassEncoderSetBlendConstant\0")?,
            wgpuRenderPassEncoderSetStencilReference: symbol(&library, path, b"wgpuRenderPassEncoderSetStencilReference\0")?,
            wgpuRenderPassEncoderBeginOcclusionQuery: symbol(&library, path, b"wgpuRenderPassEncoderBeginOcclusionQuery\0")?,
            wgpuRenderPassEncoderEndOcclusionQuery: symbol(&library, path, b"wgpuRenderPassEncoderEndOcclusionQuery\0")?,
            wgpuRenderPassEncoderExecuteBundles: symbol(&library, path, b"wgpuRenderPassEncoderExecuteBundles\0")?,
            wgpuRenderPassEncoderInsertDebugMarker: symbol(&library, path, b"wgpuRenderPassEncoderInsertDebugMarker\0")?,
            wgpuRenderPassEncoderPushDebugGroup: symbol(&library, path, b"wgpuRenderPassEncoderPushDebugGroup\0")?,
            wgpuRenderPassEncoderPopDebugGroup: symbol(&library, path, b"wgpuRenderPassEncoderPopDebugGroup\0")?,
            wgpuRenderPassEncoderEnd: symbol(&library, path, b"wgpuRenderPassEncoderEnd\0")?,
            wgpuRenderPassEncoderSetLabel: symbol(&library, path, b"wgpuRenderPassEncoderSetLabel\0")?,
            wgpuCommandBufferSetLabel: symbol(&library, path, b"wgpuCommandBufferSetLabel\0")?,
            wgpuRenderBundleEncoderSetPipeline: symbol(&library, path, b"wgpuRenderBundleEncoderSetPipeline\0")?,
            wgpuRenderBundleEncoderSetBindGroup: symbol(&library, path, b"wgpuRenderBundleEncoderSetBindGroup\0")?,
            wgpuRenderBundleEncoderSetVertexBuffer: symbol(&library, path, b"wgpuRenderBundleEncoderSetVertexBuffer\0")?,
            wgpuRenderBundleEncoderSetIndexBuffer: symbol(&library, path, b"wgpuRenderBundleEncoderSetIndexBuffer\0")?,
            wgpuRenderBundleEncoderDraw: symbol(&library, path, b"wgpuRenderBundleEncoderDraw\0")?,
            wgpuRenderBundleEncoderDrawIndexed: symbol(&library, path, b"wgpuRenderBundleEncoderDrawIndexed\0")?,
            wgpuRenderBundleEncoderDrawIndirect: symbol(&library, path, b"wgpuRenderBundleEncoderDrawIndirect\0")?,
            wgpuRenderBundleEncoderDrawIndexedIndirect: symbol(&library, path, b"wgpuRenderBundleEncoderDrawIndexedIndirect\0")?,
            wgpuRenderBundleEncoderInsertDebugMarker: symbol(&library, path, b"wgpuRenderBundleEncoderInsertDebugMarker\0")?,
            wgpuRenderBundleEncoderPushDebugGroup: symbol(&library, path, b"wgpuRenderBundleEncoderPushDebugGroup\0")?,
            wgpuRenderBundleEncoderPopDebugGroup: symbol(&library, path, b"wgpuRenderBundleEncoderPopDebugGroup\0")?,
            wgpuRenderBundleEncoderFinish: symbol(&library, path, b"wgpuRenderBundleEncoderFinish\0")?,
            wgpuRenderBundleEncoderSetLabel: symbol(&library, path, b"wgpuRenderBundleEncoderSetLabel\0")?,
            wgpuRenderBundleSetLabel: symbol(&library, path, b"wgpuRenderBundleSetLabel\0")?,
            wgpuQuerySetGetType: symbol(&library, path, b"wgpuQuerySetGetType\0")?,
            wgpuQuerySetGetCount: symbol(&library, path, b"wgpuQuerySetGetCount\0")?,
            wgpuQuerySetDestroy: symbol(&library, path, b"wgpuQuerySetDestroy\0")?,
            wgpuQuerySetSetLabel: symbol(&library, path, b"wgpuQuerySetSetLabel\0")?,
            wgpuAdapterInfoFreeMembers: symbol(&library, path, b"wgpuAdapterInfoFreeMembers\0")?,
            wgpuQuerySetRelease: symbol(&library, path, b"wgpuQuerySetRelease\0")?,
            wgpuRenderBundleRelease: symbol(&library, path, b"wgpuRenderBundleRelease\0")?,
            wgpuRenderBundleEncoderRelease: symbol(&library, path, b"wgpuRenderBundleEncoderRelease\0")?,
            wgpuCommandBufferRelease: symbol(&library, path, b"wgpuCommandBufferRelease\0")?,
            wgpuRenderPassEncoderRelease: symbol(&library, path, b"wgpuRenderPassEncoderRelease\0")?,
            wgpuComputePassEncoderRelease: symbol(&library, path, b"wgpuComputePassEncoderRelease\0")?,
            wgpuCommandEncoderRelease: symbol(&library, path, b"wgpuCommandEncoderRelease\0")?,
            wgpuRenderPipelineRelease: symbol(&library, path, b"wgpuRenderPipelineRelease\0")?,
            wgpuComputePipelineRelease: symbol(&library, path, b"wgpuComputePipelineRelease\0")?,
            wgpuShaderModuleRelease: symbol(&library, path, b"wgpuShaderModuleRelease\0")?,
            wgpuPipelineLayoutRelease: symbol(&library, path, b"wgpuPipelineLayoutRelease\0")?,
            wgpuBindGroupRelease: symbol(&library, path, b"wgpuBindGroupRelease\0")?,
            wgpuBindGroupLayoutRelease: symbol(&library, path, b"wgpuBindGroupLayoutRelease\0")?,
            wgpuSamplerRelease: symbol(&library, path, b"wgpuSamplerRelease\0")?,
            wgpuTextureViewRelease: symbol(&library, path, b"wgpuTextureViewRelease\0")?,
            wgpuTextureRelease: symbol(&library, path, b"wgpuTextureRelease\0")?,
            wgpuBufferRelease: symbol(&library, path, b"wgpuBufferRelease\0")?,
            wgpuQueueRelease: symbol(&library, path, b"wgpuQueueRelease\0")?,
            wgpuDeviceRelease: symbol(&library, path, b"wgpuDeviceRelease\0")?,
            wgpuAdapterRelease: symbol(&library, path, b"wgpuAdapterRelease\0")?,
            _library: library,
        })
    }
}

unsafe fn wgpuCreateInstance(descriptor: *const WGPUInstanceDescriptor)-> WGPUInstance {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuCreateInstance: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuCreateInstance)(descriptor) }
}

unsafe fn wgpuInstanceProcessEvents(instance: WGPUInstance) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuInstanceProcessEvents: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuInstanceProcessEvents)(instance) }
}

unsafe fn wgpuInstanceRelease(instance: WGPUInstance) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuInstanceRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuInstanceRelease)(instance) }
}

unsafe fn wgpuInstanceRequestAdapter(instance: WGPUInstance, options: *const WGPURequestAdapterOptions, callback_info: WGPURequestAdapterCallbackInfo,)-> WGPUFuture {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuInstanceRequestAdapter: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuInstanceRequestAdapter)(instance, options, callback_info) }
}

unsafe fn wgpuAdapterGetLimits(adapter: WGPUAdapter, out: *mut WGPULimits)-> i32 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuAdapterGetLimits: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuAdapterGetLimits)(adapter, out) }
}

unsafe fn wgpuAdapterGetInfo(adapter: WGPUAdapter, out: *mut WGPUAdapterInfo)-> i32 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuAdapterGetInfo: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuAdapterGetInfo)(adapter, out) }
}

unsafe fn wgpuAdapterHasFeature(adapter: WGPUAdapter, feature: i32)-> u32 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuAdapterHasFeature: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuAdapterHasFeature)(adapter, feature) }
}

unsafe fn wgpuAdapterRequestDevice(adapter: WGPUAdapter, descriptor: *const WGPUDeviceDescriptor, callback_info: WGPURequestDeviceCallbackInfo,)-> WGPUFuture {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuAdapterRequestDevice: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuAdapterRequestDevice)(adapter, descriptor, callback_info) }
}

unsafe fn wgpuDeviceGetQueue(device: WGPUDevice)-> WGPUQueue {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceGetQueue: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceGetQueue)(device) }
}

unsafe fn wgpuDeviceDestroy(device: WGPUDevice) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceDestroy: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceDestroy)(device) }
}

unsafe fn wgpuDeviceSetLabel(device: WGPUDevice, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceSetLabel)(device, label) }
}

unsafe fn wgpuDevicePushErrorScope(device: WGPUDevice, filter: i32) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDevicePushErrorScope: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDevicePushErrorScope)(device, filter) }
}

unsafe fn wgpuDevicePopErrorScope(device: WGPUDevice, callback_info: WGPUPopErrorScopeCallbackInfo)-> WGPUFuture {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDevicePopErrorScope: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDevicePopErrorScope)(device, callback_info) }
}

unsafe fn wgpuDeviceGetLimits(device: WGPUDevice, out: *mut WGPULimits)-> i32 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceGetLimits: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceGetLimits)(device, out) }
}

unsafe fn wgpuDeviceGetAdapterInfo(device: WGPUDevice, out: *mut WGPUAdapterInfo)-> i32 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceGetAdapterInfo: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceGetAdapterInfo)(device, out) }
}

unsafe fn wgpuDeviceHasFeature(device: WGPUDevice, feature: i32)-> u32 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceHasFeature: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceHasFeature)(device, feature) }
}

unsafe fn wgpuDeviceCreateBuffer(device: WGPUDevice, descriptor: *const WGPUBufferDescriptor)-> WGPUBuffer {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceCreateBuffer: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceCreateBuffer)(device, descriptor) }
}

unsafe fn wgpuDeviceCreateTexture(device: WGPUDevice, descriptor: *const WGPUTextureDescriptor)-> WGPUTexture {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceCreateTexture: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceCreateTexture)(device, descriptor) }
}

unsafe fn wgpuDeviceCreateSampler(device: WGPUDevice, descriptor: *const WGPUSamplerDescriptor)-> WGPUSampler {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceCreateSampler: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceCreateSampler)(device, descriptor) }
}

unsafe fn wgpuDeviceCreateBindGroupLayout(device: WGPUDevice, descriptor: *const WGPUBindGroupLayoutDescriptor)-> WGPUBindGroupLayout {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceCreateBindGroupLayout: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceCreateBindGroupLayout)(device, descriptor) }
}

unsafe fn wgpuDeviceCreateBindGroup(device: WGPUDevice, descriptor: *const WGPUBindGroupDescriptor)-> WGPUBindGroup {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceCreateBindGroup: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceCreateBindGroup)(device, descriptor) }
}

unsafe fn wgpuDeviceCreatePipelineLayout(device: WGPUDevice, descriptor: *const WGPUPipelineLayoutDescriptor)-> WGPUPipelineLayout {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceCreatePipelineLayout: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceCreatePipelineLayout)(device, descriptor) }
}

unsafe fn wgpuDeviceCreateShaderModule(device: WGPUDevice, descriptor: *const WGPUShaderModuleDescriptor)-> WGPUShaderModule {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceCreateShaderModule: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceCreateShaderModule)(device, descriptor) }
}

unsafe fn wgpuDeviceCreateComputePipeline(device: WGPUDevice, descriptor: *const WGPUComputePipelineDescriptor)-> WGPUComputePipeline {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceCreateComputePipeline: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceCreateComputePipeline)(device, descriptor) }
}

unsafe fn wgpuDeviceCreateComputePipelineAsync(device: WGPUDevice, descriptor: *const WGPUComputePipelineDescriptor, callback_info: WGPUCreateComputePipelineAsyncCallbackInfo)-> WGPUFuture {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceCreateComputePipelineAsync: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceCreateComputePipelineAsync)(device, descriptor, callback_info) }
}

unsafe fn wgpuDeviceCreateRenderPipeline(device: WGPUDevice, descriptor: *const WGPURenderPipelineDescriptor)-> WGPURenderPipeline {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceCreateRenderPipeline: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceCreateRenderPipeline)(device, descriptor) }
}

unsafe fn wgpuDeviceCreateRenderPipelineAsync(device: WGPUDevice, descriptor: *const WGPURenderPipelineDescriptor, callback_info: WGPUCreateRenderPipelineAsyncCallbackInfo)-> WGPUFuture {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceCreateRenderPipelineAsync: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceCreateRenderPipelineAsync)(device, descriptor, callback_info) }
}

unsafe fn wgpuDeviceCreateCommandEncoder(device: WGPUDevice, descriptor: *const WGPUCommandEncoderDescriptor)-> WGPUCommandEncoder {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceCreateCommandEncoder: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceCreateCommandEncoder)(device, descriptor) }
}

unsafe fn wgpuDeviceCreateRenderBundleEncoder(device: WGPUDevice, descriptor: *const WGPURenderBundleEncoderDescriptor)-> WGPURenderBundleEncoder {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceCreateRenderBundleEncoder: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceCreateRenderBundleEncoder)(device, descriptor) }
}

unsafe fn wgpuDeviceCreateQuerySet(device: WGPUDevice, descriptor: *const WGPUQuerySetDescriptor)-> WGPUQuerySet {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceCreateQuerySet: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceCreateQuerySet)(device, descriptor) }
}

unsafe fn wgpuQueueSubmit(queue: WGPUQueue, commandCount: usize, commands: *const WGPUCommandBuffer) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuQueueSubmit: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuQueueSubmit)(queue, commandCount, commands) }
}

unsafe fn wgpuQueueOnSubmittedWorkDone(queue: WGPUQueue, callback_info: WGPUQueueWorkDoneCallbackInfo,)-> WGPUFuture {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuQueueOnSubmittedWorkDone: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuQueueOnSubmittedWorkDone)(queue, callback_info) }
}

unsafe fn wgpuQueueWriteBuffer(queue: WGPUQueue, buffer: WGPUBuffer, bufferOffset: u64, data: *const c_void, size: usize,) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuQueueWriteBuffer: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuQueueWriteBuffer)(queue, buffer, bufferOffset, data, size) }
}

unsafe fn wgpuQueueWriteTexture(queue: WGPUQueue, destination: *const WGPUTexelCopyTextureInfo, data: *const std::ffi::c_void, data_size: usize, data_layout: *const WGPUTexelCopyBufferLayout, write_size: *const WGPUExtent3D,) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuQueueWriteTexture: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuQueueWriteTexture)(queue, destination, data, data_size, data_layout, write_size) }
}

unsafe fn wgpuQueueSetLabel(queue: WGPUQueue, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuQueueSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuQueueSetLabel)(queue, label) }
}

unsafe fn wgpuBufferMapAsync(buffer: WGPUBuffer, mode: u64, offset: usize, size: usize, callback_info: WGPUBufferMapCallbackInfo,)-> WGPUFuture {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuBufferMapAsync: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuBufferMapAsync)(buffer, mode, offset, size, callback_info) }
}

unsafe fn wgpuBufferReadMappedRange(buffer: WGPUBuffer, offset: usize, data: *mut c_void, size: usize,)-> i32 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuBufferReadMappedRange: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuBufferReadMappedRange)(buffer, offset, data, size) }
}

unsafe fn wgpuBufferWriteMappedRange(buffer: WGPUBuffer, offset: usize, data: *const c_void, size: usize,)-> i32 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuBufferWriteMappedRange: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuBufferWriteMappedRange)(buffer, offset, data, size) }
}

unsafe fn wgpuBufferSetLabel(buffer: WGPUBuffer, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuBufferSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuBufferSetLabel)(buffer, label) }
}

unsafe fn wgpuBufferGetUsage(buffer: WGPUBuffer)-> u64 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuBufferGetUsage: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuBufferGetUsage)(buffer) }
}

unsafe fn wgpuBufferGetSize(buffer: WGPUBuffer)-> u64 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuBufferGetSize: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuBufferGetSize)(buffer) }
}

unsafe fn wgpuBufferGetMapState(buffer: WGPUBuffer)-> i32 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuBufferGetMapState: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuBufferGetMapState)(buffer) }
}

unsafe fn wgpuBufferUnmap(buffer: WGPUBuffer) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuBufferUnmap: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuBufferUnmap)(buffer) }
}

unsafe fn wgpuBufferDestroy(buffer: WGPUBuffer) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuBufferDestroy: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuBufferDestroy)(buffer) }
}

unsafe fn wgpuTextureCreateView(texture: WGPUTexture, descriptor: *const WGPUTextureViewDescriptor)-> WGPUTextureView {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuTextureCreateView: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuTextureCreateView)(texture, descriptor) }
}

unsafe fn wgpuTextureSetLabel(texture: WGPUTexture, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuTextureSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuTextureSetLabel)(texture, label) }
}

unsafe fn wgpuTextureGetWidth(texture: WGPUTexture)-> u32 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuTextureGetWidth: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuTextureGetWidth)(texture) }
}

unsafe fn wgpuTextureGetHeight(texture: WGPUTexture)-> u32 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuTextureGetHeight: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuTextureGetHeight)(texture) }
}

unsafe fn wgpuTextureGetDepthOrArrayLayers(texture: WGPUTexture)-> u32 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuTextureGetDepthOrArrayLayers: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuTextureGetDepthOrArrayLayers)(texture) }
}

unsafe fn wgpuTextureGetMipLevelCount(texture: WGPUTexture)-> u32 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuTextureGetMipLevelCount: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuTextureGetMipLevelCount)(texture) }
}

unsafe fn wgpuTextureGetSampleCount(texture: WGPUTexture)-> u32 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuTextureGetSampleCount: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuTextureGetSampleCount)(texture) }
}

unsafe fn wgpuTextureGetDimension(texture: WGPUTexture)-> i32 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuTextureGetDimension: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuTextureGetDimension)(texture) }
}

unsafe fn wgpuTextureGetFormat(texture: WGPUTexture)-> i32 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuTextureGetFormat: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuTextureGetFormat)(texture) }
}

unsafe fn wgpuTextureGetUsage(texture: WGPUTexture)-> u64 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuTextureGetUsage: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuTextureGetUsage)(texture) }
}

unsafe fn wgpuTextureDestroy(texture: WGPUTexture) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuTextureDestroy: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuTextureDestroy)(texture) }
}

unsafe fn wgpuTextureViewSetLabel(textureView: WGPUTextureView, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuTextureViewSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuTextureViewSetLabel)(textureView, label) }
}

unsafe fn wgpuSamplerSetLabel(sampler: WGPUSampler, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuSamplerSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuSamplerSetLabel)(sampler, label) }
}

unsafe fn wgpuBindGroupLayoutSetLabel(bindGroupLayout: WGPUBindGroupLayout, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuBindGroupLayoutSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuBindGroupLayoutSetLabel)(bindGroupLayout, label) }
}

unsafe fn wgpuBindGroupSetLabel(bindGroup: WGPUBindGroup, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuBindGroupSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuBindGroupSetLabel)(bindGroup, label) }
}

unsafe fn wgpuPipelineLayoutSetLabel(pipelineLayout: WGPUPipelineLayout, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuPipelineLayoutSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuPipelineLayoutSetLabel)(pipelineLayout, label) }
}

unsafe fn wgpuShaderModuleSetLabel(shaderModule: WGPUShaderModule, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuShaderModuleSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuShaderModuleSetLabel)(shaderModule, label) }
}

unsafe fn wgpuComputePipelineGetBindGroupLayout(computePipeline: WGPUComputePipeline, groupIndex: u32)-> WGPUBindGroupLayout {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuComputePipelineGetBindGroupLayout: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuComputePipelineGetBindGroupLayout)(computePipeline, groupIndex) }
}

unsafe fn wgpuComputePipelineSetLabel(computePipeline: WGPUComputePipeline, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuComputePipelineSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuComputePipelineSetLabel)(computePipeline, label) }
}

unsafe fn wgpuRenderPipelineGetBindGroupLayout(renderPipeline: WGPURenderPipeline, groupIndex: u32)-> WGPUBindGroupLayout {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPipelineGetBindGroupLayout: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPipelineGetBindGroupLayout)(renderPipeline, groupIndex) }
}

unsafe fn wgpuRenderPipelineSetLabel(renderPipeline: WGPURenderPipeline, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPipelineSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPipelineSetLabel)(renderPipeline, label) }
}

unsafe fn wgpuCommandEncoderFinish(commandEncoder: WGPUCommandEncoder, descriptor: *const WGPUCommandBufferDescriptor)-> WGPUCommandBuffer {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuCommandEncoderFinish: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuCommandEncoderFinish)(commandEncoder, descriptor) }
}

unsafe fn wgpuCommandEncoderBeginComputePass(commandEncoder: WGPUCommandEncoder, descriptor: *const WGPUComputePassDescriptor)-> WGPUComputePassEncoder {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuCommandEncoderBeginComputePass: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuCommandEncoderBeginComputePass)(commandEncoder, descriptor) }
}

unsafe fn wgpuCommandEncoderBeginRenderPass(commandEncoder: WGPUCommandEncoder, descriptor: *const WGPURenderPassDescriptor)-> WGPURenderPassEncoder {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuCommandEncoderBeginRenderPass: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuCommandEncoderBeginRenderPass)(commandEncoder, descriptor) }
}

unsafe fn wgpuCommandEncoderCopyBufferToBuffer(commandEncoder: WGPUCommandEncoder, source: WGPUBuffer, sourceOffset: u64, destination: WGPUBuffer, destinationOffset: u64, size: u64) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuCommandEncoderCopyBufferToBuffer: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuCommandEncoderCopyBufferToBuffer)(commandEncoder, source, sourceOffset, destination, destinationOffset, size) }
}

unsafe fn wgpuCommandEncoderCopyBufferToTexture(commandEncoder: WGPUCommandEncoder, source: *const WGPUTexelCopyBufferInfo, destination: *const WGPUTexelCopyTextureInfo, copySize: *const WGPUExtent3D) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuCommandEncoderCopyBufferToTexture: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuCommandEncoderCopyBufferToTexture)(commandEncoder, source, destination, copySize) }
}

unsafe fn wgpuCommandEncoderCopyTextureToBuffer(commandEncoder: WGPUCommandEncoder, source: *const WGPUTexelCopyTextureInfo, destination: *const WGPUTexelCopyBufferInfo, copySize: *const WGPUExtent3D) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuCommandEncoderCopyTextureToBuffer: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuCommandEncoderCopyTextureToBuffer)(commandEncoder, source, destination, copySize) }
}

unsafe fn wgpuCommandEncoderCopyTextureToTexture(commandEncoder: WGPUCommandEncoder, source: *const WGPUTexelCopyTextureInfo, destination: *const WGPUTexelCopyTextureInfo, copySize: *const WGPUExtent3D) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuCommandEncoderCopyTextureToTexture: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuCommandEncoderCopyTextureToTexture)(commandEncoder, source, destination, copySize) }
}

unsafe fn wgpuCommandEncoderClearBuffer(commandEncoder: WGPUCommandEncoder, buffer: WGPUBuffer, offset: u64, size: u64) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuCommandEncoderClearBuffer: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuCommandEncoderClearBuffer)(commandEncoder, buffer, offset, size) }
}

unsafe fn wgpuCommandEncoderResolveQuerySet(commandEncoder: WGPUCommandEncoder, querySet: WGPUQuerySet, firstQuery: u32, queryCount: u32, destination: WGPUBuffer, destinationOffset: u64) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuCommandEncoderResolveQuerySet: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuCommandEncoderResolveQuerySet)(commandEncoder, querySet, firstQuery, queryCount, destination, destinationOffset) }
}

unsafe fn wgpuCommandEncoderInsertDebugMarker(commandEncoder: WGPUCommandEncoder, markerLabel: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuCommandEncoderInsertDebugMarker: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuCommandEncoderInsertDebugMarker)(commandEncoder, markerLabel) }
}

unsafe fn wgpuCommandEncoderPushDebugGroup(commandEncoder: WGPUCommandEncoder, groupLabel: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuCommandEncoderPushDebugGroup: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuCommandEncoderPushDebugGroup)(commandEncoder, groupLabel) }
}

unsafe fn wgpuCommandEncoderPopDebugGroup(commandEncoder: WGPUCommandEncoder) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuCommandEncoderPopDebugGroup: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuCommandEncoderPopDebugGroup)(commandEncoder) }
}

unsafe fn wgpuCommandEncoderSetLabel(commandEncoder: WGPUCommandEncoder, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuCommandEncoderSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuCommandEncoderSetLabel)(commandEncoder, label) }
}

unsafe fn wgpuComputePassEncoderSetPipeline(computePassEncoder: WGPUComputePassEncoder, pipeline: WGPUComputePipeline) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuComputePassEncoderSetPipeline: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuComputePassEncoderSetPipeline)(computePassEncoder, pipeline) }
}

unsafe fn wgpuComputePassEncoderSetBindGroup(computePassEncoder: WGPUComputePassEncoder, groupIndex: u32, group: WGPUBindGroup, dynamicOffsetCount: usize, dynamicOffsets: *const u32) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuComputePassEncoderSetBindGroup: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuComputePassEncoderSetBindGroup)(computePassEncoder, groupIndex, group, dynamicOffsetCount, dynamicOffsets) }
}

unsafe fn wgpuComputePassEncoderDispatchWorkgroups(computePassEncoder: WGPUComputePassEncoder, workgroupCountX: u32, workgroupCountY: u32, workgroupCountZ: u32) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuComputePassEncoderDispatchWorkgroups: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuComputePassEncoderDispatchWorkgroups)(computePassEncoder, workgroupCountX, workgroupCountY, workgroupCountZ) }
}

unsafe fn wgpuComputePassEncoderDispatchWorkgroupsIndirect(computePassEncoder: WGPUComputePassEncoder, indirectBuffer: WGPUBuffer, indirectOffset: u64) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuComputePassEncoderDispatchWorkgroupsIndirect: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuComputePassEncoderDispatchWorkgroupsIndirect)(computePassEncoder, indirectBuffer, indirectOffset) }
}

unsafe fn wgpuComputePassEncoderInsertDebugMarker(computePassEncoder: WGPUComputePassEncoder, markerLabel: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuComputePassEncoderInsertDebugMarker: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuComputePassEncoderInsertDebugMarker)(computePassEncoder, markerLabel) }
}

unsafe fn wgpuComputePassEncoderPushDebugGroup(computePassEncoder: WGPUComputePassEncoder, groupLabel: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuComputePassEncoderPushDebugGroup: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuComputePassEncoderPushDebugGroup)(computePassEncoder, groupLabel) }
}

unsafe fn wgpuComputePassEncoderPopDebugGroup(computePassEncoder: WGPUComputePassEncoder) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuComputePassEncoderPopDebugGroup: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuComputePassEncoderPopDebugGroup)(computePassEncoder) }
}

unsafe fn wgpuComputePassEncoderEnd(computePassEncoder: WGPUComputePassEncoder) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuComputePassEncoderEnd: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuComputePassEncoderEnd)(computePassEncoder) }
}

unsafe fn wgpuComputePassEncoderSetLabel(computePassEncoder: WGPUComputePassEncoder, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuComputePassEncoderSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuComputePassEncoderSetLabel)(computePassEncoder, label) }
}

unsafe fn wgpuRenderPassEncoderSetPipeline(renderPassEncoder: WGPURenderPassEncoder, pipeline: WGPURenderPipeline) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderSetPipeline: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderSetPipeline)(renderPassEncoder, pipeline) }
}

unsafe fn wgpuRenderPassEncoderSetBindGroup(renderPassEncoder: WGPURenderPassEncoder, groupIndex: u32, group: WGPUBindGroup, dynamicOffsetCount: usize, dynamicOffsets: *const u32) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderSetBindGroup: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderSetBindGroup)(renderPassEncoder, groupIndex, group, dynamicOffsetCount, dynamicOffsets) }
}

unsafe fn wgpuRenderPassEncoderSetVertexBuffer(renderPassEncoder: WGPURenderPassEncoder, slot: u32, buffer: WGPUBuffer, offset: u64, size: u64) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderSetVertexBuffer: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderSetVertexBuffer)(renderPassEncoder, slot, buffer, offset, size) }
}

unsafe fn wgpuRenderPassEncoderSetIndexBuffer(renderPassEncoder: WGPURenderPassEncoder, buffer: WGPUBuffer, format: i32, offset: u64, size: u64) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderSetIndexBuffer: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderSetIndexBuffer)(renderPassEncoder, buffer, format, offset, size) }
}

unsafe fn wgpuRenderPassEncoderDraw(renderPassEncoder: WGPURenderPassEncoder, vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderDraw: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderDraw)(renderPassEncoder, vertexCount, instanceCount, firstVertex, firstInstance) }
}

unsafe fn wgpuRenderPassEncoderDrawIndexed(renderPassEncoder: WGPURenderPassEncoder, indexCount: u32, instanceCount: u32, firstIndex: u32, baseVertex: i32, firstInstance: u32) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderDrawIndexed: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderDrawIndexed)(renderPassEncoder, indexCount, instanceCount, firstIndex, baseVertex, firstInstance) }
}

unsafe fn wgpuRenderPassEncoderDrawIndirect(renderPassEncoder: WGPURenderPassEncoder, indirectBuffer: WGPUBuffer, indirectOffset: u64) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderDrawIndirect: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderDrawIndirect)(renderPassEncoder, indirectBuffer, indirectOffset) }
}

unsafe fn wgpuRenderPassEncoderDrawIndexedIndirect(renderPassEncoder: WGPURenderPassEncoder, indirectBuffer: WGPUBuffer, indirectOffset: u64) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderDrawIndexedIndirect: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderDrawIndexedIndirect)(renderPassEncoder, indirectBuffer, indirectOffset) }
}

unsafe fn wgpuRenderPassEncoderSetViewport(renderPassEncoder: WGPURenderPassEncoder, x: f32, y: f32, width: f32, height: f32, minDepth: f32, maxDepth: f32) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderSetViewport: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderSetViewport)(renderPassEncoder, x, y, width, height, minDepth, maxDepth) }
}

unsafe fn wgpuRenderPassEncoderSetScissorRect(renderPassEncoder: WGPURenderPassEncoder, x: u32, y: u32, width: u32, height: u32) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderSetScissorRect: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderSetScissorRect)(renderPassEncoder, x, y, width, height) }
}

unsafe fn wgpuRenderPassEncoderSetBlendConstant(renderPassEncoder: WGPURenderPassEncoder, color: *const WGPUColor) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderSetBlendConstant: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderSetBlendConstant)(renderPassEncoder, color) }
}

unsafe fn wgpuRenderPassEncoderSetStencilReference(renderPassEncoder: WGPURenderPassEncoder, reference: u32) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderSetStencilReference: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderSetStencilReference)(renderPassEncoder, reference) }
}

unsafe fn wgpuRenderPassEncoderBeginOcclusionQuery(renderPassEncoder: WGPURenderPassEncoder, queryIndex: u32) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderBeginOcclusionQuery: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderBeginOcclusionQuery)(renderPassEncoder, queryIndex) }
}

unsafe fn wgpuRenderPassEncoderEndOcclusionQuery(renderPassEncoder: WGPURenderPassEncoder) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderEndOcclusionQuery: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderEndOcclusionQuery)(renderPassEncoder) }
}

unsafe fn wgpuRenderPassEncoderExecuteBundles(renderPassEncoder: WGPURenderPassEncoder, bundleCount: usize, bundles: *const WGPURenderBundle) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderExecuteBundles: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderExecuteBundles)(renderPassEncoder, bundleCount, bundles) }
}

unsafe fn wgpuRenderPassEncoderInsertDebugMarker(renderPassEncoder: WGPURenderPassEncoder, markerLabel: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderInsertDebugMarker: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderInsertDebugMarker)(renderPassEncoder, markerLabel) }
}

unsafe fn wgpuRenderPassEncoderPushDebugGroup(renderPassEncoder: WGPURenderPassEncoder, groupLabel: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderPushDebugGroup: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderPushDebugGroup)(renderPassEncoder, groupLabel) }
}

unsafe fn wgpuRenderPassEncoderPopDebugGroup(renderPassEncoder: WGPURenderPassEncoder) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderPopDebugGroup: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderPopDebugGroup)(renderPassEncoder) }
}

unsafe fn wgpuRenderPassEncoderEnd(renderPassEncoder: WGPURenderPassEncoder) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderEnd: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderEnd)(renderPassEncoder) }
}

unsafe fn wgpuRenderPassEncoderSetLabel(renderPassEncoder: WGPURenderPassEncoder, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderSetLabel)(renderPassEncoder, label) }
}

unsafe fn wgpuCommandBufferSetLabel(commandBuffer: WGPUCommandBuffer, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuCommandBufferSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuCommandBufferSetLabel)(commandBuffer, label) }
}

unsafe fn wgpuRenderBundleEncoderSetPipeline(renderBundleEncoder: WGPURenderBundleEncoder, pipeline: WGPURenderPipeline) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderBundleEncoderSetPipeline: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderBundleEncoderSetPipeline)(renderBundleEncoder, pipeline) }
}

unsafe fn wgpuRenderBundleEncoderSetBindGroup(renderBundleEncoder: WGPURenderBundleEncoder, groupIndex: u32, group: WGPUBindGroup, dynamicOffsetCount: usize, dynamicOffsets: *const u32) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderBundleEncoderSetBindGroup: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderBundleEncoderSetBindGroup)(renderBundleEncoder, groupIndex, group, dynamicOffsetCount, dynamicOffsets) }
}

unsafe fn wgpuRenderBundleEncoderSetVertexBuffer(renderBundleEncoder: WGPURenderBundleEncoder, slot: u32, buffer: WGPUBuffer, offset: u64, size: u64) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderBundleEncoderSetVertexBuffer: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderBundleEncoderSetVertexBuffer)(renderBundleEncoder, slot, buffer, offset, size) }
}

unsafe fn wgpuRenderBundleEncoderSetIndexBuffer(renderBundleEncoder: WGPURenderBundleEncoder, buffer: WGPUBuffer, format: i32, offset: u64, size: u64) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderBundleEncoderSetIndexBuffer: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderBundleEncoderSetIndexBuffer)(renderBundleEncoder, buffer, format, offset, size) }
}

unsafe fn wgpuRenderBundleEncoderDraw(renderBundleEncoder: WGPURenderBundleEncoder, vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderBundleEncoderDraw: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderBundleEncoderDraw)(renderBundleEncoder, vertexCount, instanceCount, firstVertex, firstInstance) }
}

unsafe fn wgpuRenderBundleEncoderDrawIndexed(renderBundleEncoder: WGPURenderBundleEncoder, indexCount: u32, instanceCount: u32, firstIndex: u32, baseVertex: i32, firstInstance: u32) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderBundleEncoderDrawIndexed: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderBundleEncoderDrawIndexed)(renderBundleEncoder, indexCount, instanceCount, firstIndex, baseVertex, firstInstance) }
}

unsafe fn wgpuRenderBundleEncoderDrawIndirect(renderBundleEncoder: WGPURenderBundleEncoder, indirectBuffer: WGPUBuffer, indirectOffset: u64) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderBundleEncoderDrawIndirect: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderBundleEncoderDrawIndirect)(renderBundleEncoder, indirectBuffer, indirectOffset) }
}

unsafe fn wgpuRenderBundleEncoderDrawIndexedIndirect(renderBundleEncoder: WGPURenderBundleEncoder, indirectBuffer: WGPUBuffer, indirectOffset: u64) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderBundleEncoderDrawIndexedIndirect: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderBundleEncoderDrawIndexedIndirect)(renderBundleEncoder, indirectBuffer, indirectOffset) }
}

unsafe fn wgpuRenderBundleEncoderInsertDebugMarker(renderBundleEncoder: WGPURenderBundleEncoder, markerLabel: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderBundleEncoderInsertDebugMarker: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderBundleEncoderInsertDebugMarker)(renderBundleEncoder, markerLabel) }
}

unsafe fn wgpuRenderBundleEncoderPushDebugGroup(renderBundleEncoder: WGPURenderBundleEncoder, groupLabel: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderBundleEncoderPushDebugGroup: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderBundleEncoderPushDebugGroup)(renderBundleEncoder, groupLabel) }
}

unsafe fn wgpuRenderBundleEncoderPopDebugGroup(renderBundleEncoder: WGPURenderBundleEncoder) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderBundleEncoderPopDebugGroup: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderBundleEncoderPopDebugGroup)(renderBundleEncoder) }
}

unsafe fn wgpuRenderBundleEncoderFinish(renderBundleEncoder: WGPURenderBundleEncoder, descriptor: *const WGPURenderBundleDescriptor)-> WGPURenderBundle {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderBundleEncoderFinish: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderBundleEncoderFinish)(renderBundleEncoder, descriptor) }
}

unsafe fn wgpuRenderBundleEncoderSetLabel(renderBundleEncoder: WGPURenderBundleEncoder, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderBundleEncoderSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderBundleEncoderSetLabel)(renderBundleEncoder, label) }
}

unsafe fn wgpuRenderBundleSetLabel(renderBundle: WGPURenderBundle, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderBundleSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderBundleSetLabel)(renderBundle, label) }
}

unsafe fn wgpuQuerySetGetType(querySet: WGPUQuerySet)-> i32 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuQuerySetGetType: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuQuerySetGetType)(querySet) }
}

unsafe fn wgpuQuerySetGetCount(querySet: WGPUQuerySet)-> u32 {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuQuerySetGetCount: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuQuerySetGetCount)(querySet) }
}

unsafe fn wgpuQuerySetDestroy(querySet: WGPUQuerySet) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuQuerySetDestroy: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuQuerySetDestroy)(querySet) }
}

unsafe fn wgpuQuerySetSetLabel(querySet: WGPUQuerySet, label: WGPUStringView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuQuerySetSetLabel: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuQuerySetSetLabel)(querySet, label) }
}

unsafe fn wgpuAdapterInfoFreeMembers(info: WGPUAdapterInfo) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuAdapterInfoFreeMembers: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuAdapterInfoFreeMembers)(info) }
}

unsafe fn wgpuQuerySetRelease(querySet: WGPUQuerySet) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuQuerySetRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuQuerySetRelease)(querySet) }
}

unsafe fn wgpuRenderBundleRelease(renderBundle: WGPURenderBundle) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderBundleRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderBundleRelease)(renderBundle) }
}

unsafe fn wgpuRenderBundleEncoderRelease(renderBundleEncoder: WGPURenderBundleEncoder) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderBundleEncoderRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderBundleEncoderRelease)(renderBundleEncoder) }
}

unsafe fn wgpuCommandBufferRelease(commandBuffer: WGPUCommandBuffer) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuCommandBufferRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuCommandBufferRelease)(commandBuffer) }
}

unsafe fn wgpuRenderPassEncoderRelease(renderPassEncoder: WGPURenderPassEncoder) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPassEncoderRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPassEncoderRelease)(renderPassEncoder) }
}

unsafe fn wgpuComputePassEncoderRelease(computePassEncoder: WGPUComputePassEncoder) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuComputePassEncoderRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuComputePassEncoderRelease)(computePassEncoder) }
}

unsafe fn wgpuCommandEncoderRelease(commandEncoder: WGPUCommandEncoder) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuCommandEncoderRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuCommandEncoderRelease)(commandEncoder) }
}

unsafe fn wgpuRenderPipelineRelease(renderPipeline: WGPURenderPipeline) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuRenderPipelineRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuRenderPipelineRelease)(renderPipeline) }
}

unsafe fn wgpuComputePipelineRelease(computePipeline: WGPUComputePipeline) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuComputePipelineRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuComputePipelineRelease)(computePipeline) }
}

unsafe fn wgpuShaderModuleRelease(shaderModule: WGPUShaderModule) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuShaderModuleRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuShaderModuleRelease)(shaderModule) }
}

unsafe fn wgpuPipelineLayoutRelease(pipelineLayout: WGPUPipelineLayout) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuPipelineLayoutRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuPipelineLayoutRelease)(pipelineLayout) }
}

unsafe fn wgpuBindGroupRelease(bindGroup: WGPUBindGroup) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuBindGroupRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuBindGroupRelease)(bindGroup) }
}

unsafe fn wgpuBindGroupLayoutRelease(bindGroupLayout: WGPUBindGroupLayout) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuBindGroupLayoutRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuBindGroupLayoutRelease)(bindGroupLayout) }
}

unsafe fn wgpuSamplerRelease(sampler: WGPUSampler) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuSamplerRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuSamplerRelease)(sampler) }
}

unsafe fn wgpuTextureViewRelease(textureView: WGPUTextureView) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuTextureViewRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuTextureViewRelease)(textureView) }
}

unsafe fn wgpuTextureRelease(texture: WGPUTexture) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuTextureRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuTextureRelease)(texture) }
}

unsafe fn wgpuBufferRelease(buffer: WGPUBuffer) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuBufferRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuBufferRelease)(buffer) }
}

unsafe fn wgpuQueueRelease(queue: WGPUQueue) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuQueueRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuQueueRelease)(queue) }
}

unsafe fn wgpuDeviceRelease(device: WGPUDevice) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuDeviceRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuDeviceRelease)(device) }
}

unsafe fn wgpuAdapterRelease(adapter: WGPUAdapter) {
    let Some(table) = crate::runtime::table() else {
        eprintln!("subscript-typegpu: cannot call wgpuAdapterRelease: set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::process::abort();
    };
    // SAFETY: the table stores the pinned signature for this symbol.
    unsafe { (table.wgpuAdapterRelease)(adapter) }
}


// ---------------------------------------------------------------------
// subscript-typegpu.h surface and panic-free export bodies.
// ---------------------------------------------------------------------

opaque!(
    pub
    SubscriptTypegpuInstanceImpl,
    SubscriptTypegpuAdapterImpl,
    SubscriptTypegpuDeviceImpl,
    SubscriptTypegpuQueueImpl,
    SubscriptTypegpuBufferImpl,
    SubscriptTypegpuTextureImpl,
    SubscriptTypegpuTextureViewImpl,
    SubscriptTypegpuSamplerImpl,
    SubscriptTypegpuBindGroupLayoutImpl,
    SubscriptTypegpuBindGroupImpl,
    SubscriptTypegpuPipelineLayoutImpl,
    SubscriptTypegpuShaderModuleImpl,
    SubscriptTypegpuComputePipelineImpl,
    SubscriptTypegpuRenderPipelineImpl,
    SubscriptTypegpuCommandEncoderImpl,
    SubscriptTypegpuComputePassEncoderImpl,
    SubscriptTypegpuRenderPassEncoderImpl,
    SubscriptTypegpuCommandBufferImpl,
    SubscriptTypegpuRenderBundleEncoderImpl,
    SubscriptTypegpuRenderBundleImpl,
    SubscriptTypegpuQuerySetImpl,
);

/// `subscript-typegpu.h`: opaque instance handle.
pub type SubscriptTypegpuInstance = *mut SubscriptTypegpuInstanceImpl;
/// `subscript-typegpu.h`: opaque adapter handle.
pub type SubscriptTypegpuAdapter = *mut SubscriptTypegpuAdapterImpl;
/// `subscript-typegpu.h`: opaque device handle.
pub type SubscriptTypegpuDevice = *mut SubscriptTypegpuDeviceImpl;
/// `subscript-typegpu.h`: opaque queue handle.
pub type SubscriptTypegpuQueue = *mut SubscriptTypegpuQueueImpl;
/// `subscript-typegpu.h`: opaque buffer handle.
pub type SubscriptTypegpuBuffer = *mut SubscriptTypegpuBufferImpl;
/// `subscript-typegpu.h`: opaque texture handle.
pub type SubscriptTypegpuTexture = *mut SubscriptTypegpuTextureImpl;
/// `subscript-typegpu.h`: opaque texture view handle.
pub type SubscriptTypegpuTextureView = *mut SubscriptTypegpuTextureViewImpl;
/// `subscript-typegpu.h`: opaque sampler handle.
pub type SubscriptTypegpuSampler = *mut SubscriptTypegpuSamplerImpl;
/// `subscript-typegpu.h`: opaque bind group layout handle.
pub type SubscriptTypegpuBindGroupLayout = *mut SubscriptTypegpuBindGroupLayoutImpl;
/// `subscript-typegpu.h`: opaque bind group handle.
pub type SubscriptTypegpuBindGroup = *mut SubscriptTypegpuBindGroupImpl;
/// `subscript-typegpu.h`: opaque pipeline layout handle.
pub type SubscriptTypegpuPipelineLayout = *mut SubscriptTypegpuPipelineLayoutImpl;
/// `subscript-typegpu.h`: opaque shader module handle.
pub type SubscriptTypegpuShaderModule = *mut SubscriptTypegpuShaderModuleImpl;
/// `subscript-typegpu.h`: opaque compute pipeline handle.
pub type SubscriptTypegpuComputePipeline = *mut SubscriptTypegpuComputePipelineImpl;
/// `subscript-typegpu.h`: opaque render pipeline handle.
pub type SubscriptTypegpuRenderPipeline = *mut SubscriptTypegpuRenderPipelineImpl;
/// `subscript-typegpu.h`: opaque command encoder handle.
pub type SubscriptTypegpuCommandEncoder = *mut SubscriptTypegpuCommandEncoderImpl;
/// `subscript-typegpu.h`: opaque compute pass encoder handle.
pub type SubscriptTypegpuComputePassEncoder = *mut SubscriptTypegpuComputePassEncoderImpl;
/// `subscript-typegpu.h`: opaque render pass encoder handle.
pub type SubscriptTypegpuRenderPassEncoder = *mut SubscriptTypegpuRenderPassEncoderImpl;
/// `subscript-typegpu.h`: opaque command buffer handle.
pub type SubscriptTypegpuCommandBuffer = *mut SubscriptTypegpuCommandBufferImpl;
/// `subscript-typegpu.h`: opaque render bundle encoder handle.
pub type SubscriptTypegpuRenderBundleEncoder = *mut SubscriptTypegpuRenderBundleEncoderImpl;
/// `subscript-typegpu.h`: opaque render bundle handle.
pub type SubscriptTypegpuRenderBundle = *mut SubscriptTypegpuRenderBundleImpl;
/// `subscript-typegpu.h`: opaque query set handle.
pub type SubscriptTypegpuQuerySet = *mut SubscriptTypegpuQuerySetImpl;

/// `subscript-typegpu.h`: facade-filled error type and message.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuErrorRecord {
    /// Pinned `SubscriptTypegpuErrorType` value.
    pub r#type: SubscriptTypegpuErrorType,
    /// Facade-owned UTF-8 bytes, valid until the next fill on the parent.
    pub message: SubscriptTypegpuStringView,
}

/// `subscript-typegpu.h`: facade-filled device-lost reason and message.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuLostRecord {
    /// Pinned `SubscriptTypegpuDeviceLostReason` value.
    pub reason: SubscriptTypegpuDeviceLostReason,
    /// Facade-owned UTF-8 bytes, valid until the next fill on the device.
    pub message: SubscriptTypegpuStringView,
}

/// `subscript-typegpu.h`: facade-filled adapter information.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuAdapterInfo {
    /// Facade-owned vendor string.
    pub vendor: SubscriptTypegpuStringView,
    /// Facade-owned architecture string.
    pub architecture: SubscriptTypegpuStringView,
    /// Facade-owned device string.
    pub device: SubscriptTypegpuStringView,
    /// Facade-owned description string.
    pub description: SubscriptTypegpuStringView,
    /// Pinned backend-type enum value.
    pub backend_type: SubscriptTypegpuBackendType,
    /// Pinned adapter-type enum value.
    pub adapter_type: SubscriptTypegpuAdapterType,
    /// PCI vendor identifier when reported.
    pub vendor_id: u32,
    /// PCI device identifier when reported.
    pub device_id: u32,
}

/// `subscript-typegpu.h`: request-device descriptor without callback fields.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuDeviceDescriptor {
    /// Debug label.
    pub label: SubscriptTypegpuStringView,
    /// Number of required feature enum values.
    pub required_features_count: usize,
    /// Required feature enum values.
    pub required_features: *const SubscriptTypegpuFeatureName,
    /// Optional required-limits record.
    pub required_limits: *const SubscriptTypegpuLimits,
    /// Default queue descriptor.
    pub default_queue: SubscriptTypegpuQueueDescriptor,
}

/// Facade-test probe for the H2 required-limits sentinel rules.
#[doc(hidden)]
pub fn subscript_typegpu_internal_required_limits_for_test(
    max_bind_groups: u32,
    max_uniform_buffer_binding_size: u64,
    max_storage_buffer_binding_size: u64,
    max_buffer_size: u64,
) -> (u32, u64, u64, u64) {
    // SAFETY: every facade limits field admits zero.
    let mut source: SubscriptTypegpuLimits = unsafe { std::mem::zeroed() };
    source.max_bind_groups = max_bind_groups;
    source.max_uniform_buffer_binding_size = max_uniform_buffer_binding_size;
    source.max_storage_buffer_binding_size = max_storage_buffer_binding_size;
    source.max_buffer_size = max_buffer_size;
    let converted = convert_limits(source);
    (
        converted.max_bind_groups,
        converted.max_uniform_buffer_binding_size,
        converted.max_storage_buffer_binding_size,
        converted.max_buffer_size,
    )
}

/// `subscript-typegpu.h`: WGSL shader module descriptor with its source chain flattened.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriptTypegpuShaderModuleDescriptor {
    /// Shader module label.
    pub label: SubscriptTypegpuStringView,
    /// WGSL source text.
    pub code: SubscriptTypegpuStringView,
}

/// `subscript-typegpu.h`: `bitflag.buffer_usage` scalar set.
pub type SubscriptTypegpuBufferUsage = u64;
/// `subscript-typegpu.h`: `bitflag.buffer_usage` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_NONE: SubscriptTypegpuBufferUsage = 0x0;
/// `subscript-typegpu.h`: `bitflag.buffer_usage` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_MAP_READ: SubscriptTypegpuBufferUsage = 0x1;
/// `subscript-typegpu.h`: `bitflag.buffer_usage` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_MAP_WRITE: SubscriptTypegpuBufferUsage = 0x2;
/// `subscript-typegpu.h`: `bitflag.buffer_usage` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_COPY_SRC: SubscriptTypegpuBufferUsage = 0x4;
/// `subscript-typegpu.h`: `bitflag.buffer_usage` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_COPY_DST: SubscriptTypegpuBufferUsage = 0x8;
/// `subscript-typegpu.h`: `bitflag.buffer_usage` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_INDEX: SubscriptTypegpuBufferUsage = 0x10;
/// `subscript-typegpu.h`: `bitflag.buffer_usage` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_VERTEX: SubscriptTypegpuBufferUsage = 0x20;
/// `subscript-typegpu.h`: `bitflag.buffer_usage` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_UNIFORM: SubscriptTypegpuBufferUsage = 0x40;
/// `subscript-typegpu.h`: `bitflag.buffer_usage` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_STORAGE: SubscriptTypegpuBufferUsage = 0x80;
/// `subscript-typegpu.h`: `bitflag.buffer_usage` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_INDIRECT: SubscriptTypegpuBufferUsage = 0x100;
/// `subscript-typegpu.h`: `bitflag.buffer_usage` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_QUERY_RESOLVE: SubscriptTypegpuBufferUsage = 0x200;

/// `subscript-typegpu.h`: `bitflag.map_mode` scalar set.
pub type SubscriptTypegpuMapMode = u64;
/// `subscript-typegpu.h`: `bitflag.map_mode` value.
pub const SUBSCRIPT_TYPEGPU_MAP_MODE_NONE: SubscriptTypegpuMapMode = 0x0;
/// `subscript-typegpu.h`: `bitflag.map_mode` value.
pub const SUBSCRIPT_TYPEGPU_MAP_MODE_READ: SubscriptTypegpuMapMode = 0x1;
/// `subscript-typegpu.h`: `bitflag.map_mode` value.
pub const SUBSCRIPT_TYPEGPU_MAP_MODE_WRITE: SubscriptTypegpuMapMode = 0x2;

/// `subscript-typegpu.h`: `enum.buffer_map_state` scalar set.
pub type SubscriptTypegpuBufferMapState = i32;
/// `subscript-typegpu.h`: `enum.buffer_map_state` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_UNMAPPED: SubscriptTypegpuBufferMapState = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.buffer_map_state` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_PENDING: SubscriptTypegpuBufferMapState = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.buffer_map_state` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_MAPPED: SubscriptTypegpuBufferMapState = 0x0000_0003;

/// `subscript-typegpu.h`: `bitflag.texture_usage` scalar set.
pub type SubscriptTypegpuTextureUsage = u64;
/// `subscript-typegpu.h`: `bitflag.texture_usage` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_USAGE_NONE: SubscriptTypegpuTextureUsage = 0x0;
/// `subscript-typegpu.h`: `bitflag.texture_usage` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_USAGE_COPY_SRC: SubscriptTypegpuTextureUsage = 0x1;
/// `subscript-typegpu.h`: `bitflag.texture_usage` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_USAGE_COPY_DST: SubscriptTypegpuTextureUsage = 0x2;
/// `subscript-typegpu.h`: `bitflag.texture_usage` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_USAGE_TEXTURE_BINDING: SubscriptTypegpuTextureUsage = 0x4;
/// `subscript-typegpu.h`: `bitflag.texture_usage` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_USAGE_STORAGE_BINDING: SubscriptTypegpuTextureUsage = 0x8;
/// `subscript-typegpu.h`: `bitflag.texture_usage` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_USAGE_RENDER_ATTACHMENT: SubscriptTypegpuTextureUsage = 0x10;
/// `subscript-typegpu.h`: `bitflag.texture_usage` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_USAGE_TRANSIENT_ATTACHMENT: SubscriptTypegpuTextureUsage = 0x20;

/// `subscript-typegpu.h`: `enum.texture_dimension` scalar set.
pub type SubscriptTypegpuTextureDimension = i32;
/// `subscript-typegpu.h`: `enum.texture_dimension` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_DIMENSION_UNDEFINED: SubscriptTypegpuTextureDimension = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.texture_dimension` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_DIMENSION_1_D: SubscriptTypegpuTextureDimension = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.texture_dimension` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_DIMENSION_2_D: SubscriptTypegpuTextureDimension = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.texture_dimension` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_DIMENSION_3_D: SubscriptTypegpuTextureDimension = 0x0000_0003;

/// `subscript-typegpu.h`: `enum.texture_format` scalar set.
pub type SubscriptTypegpuTextureFormat = i32;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_UNDEFINED: SubscriptTypegpuTextureFormat = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_R8_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_R8_SNORM: SubscriptTypegpuTextureFormat = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_R8_UINT: SubscriptTypegpuTextureFormat = 0x0000_0003;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_R8_SINT: SubscriptTypegpuTextureFormat = 0x0000_0004;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_R16_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0005;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_R16_SNORM: SubscriptTypegpuTextureFormat = 0x0000_0006;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_R16_UINT: SubscriptTypegpuTextureFormat = 0x0000_0007;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_R16_SINT: SubscriptTypegpuTextureFormat = 0x0000_0008;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_R16_FLOAT: SubscriptTypegpuTextureFormat = 0x0000_0009;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RG8_UNORM: SubscriptTypegpuTextureFormat = 0x0000_000A;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RG8_SNORM: SubscriptTypegpuTextureFormat = 0x0000_000B;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RG8_UINT: SubscriptTypegpuTextureFormat = 0x0000_000C;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RG8_SINT: SubscriptTypegpuTextureFormat = 0x0000_000D;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_R32_FLOAT: SubscriptTypegpuTextureFormat = 0x0000_000E;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_R32_UINT: SubscriptTypegpuTextureFormat = 0x0000_000F;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_R32_SINT: SubscriptTypegpuTextureFormat = 0x0000_0010;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RG16_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0011;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RG16_SNORM: SubscriptTypegpuTextureFormat = 0x0000_0012;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RG16_UINT: SubscriptTypegpuTextureFormat = 0x0000_0013;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RG16_SINT: SubscriptTypegpuTextureFormat = 0x0000_0014;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RG16_FLOAT: SubscriptTypegpuTextureFormat = 0x0000_0015;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RGBA8_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0016;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RGBA8_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_0017;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RGBA8_SNORM: SubscriptTypegpuTextureFormat = 0x0000_0018;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RGBA8_UINT: SubscriptTypegpuTextureFormat = 0x0000_0019;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RGBA8_SINT: SubscriptTypegpuTextureFormat = 0x0000_001A;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_BGRA8_UNORM: SubscriptTypegpuTextureFormat = 0x0000_001B;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_BGRA8_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_001C;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RGB10_A2_UINT: SubscriptTypegpuTextureFormat = 0x0000_001D;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RGB10_A2_UNORM: SubscriptTypegpuTextureFormat = 0x0000_001E;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RG11_B10_UFLOAT: SubscriptTypegpuTextureFormat = 0x0000_001F;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RGB9_E5_UFLOAT: SubscriptTypegpuTextureFormat = 0x0000_0020;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RG32_FLOAT: SubscriptTypegpuTextureFormat = 0x0000_0021;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RG32_UINT: SubscriptTypegpuTextureFormat = 0x0000_0022;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RG32_SINT: SubscriptTypegpuTextureFormat = 0x0000_0023;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RGBA16_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0024;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RGBA16_SNORM: SubscriptTypegpuTextureFormat = 0x0000_0025;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RGBA16_UINT: SubscriptTypegpuTextureFormat = 0x0000_0026;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RGBA16_SINT: SubscriptTypegpuTextureFormat = 0x0000_0027;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RGBA16_FLOAT: SubscriptTypegpuTextureFormat = 0x0000_0028;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RGBA32_FLOAT: SubscriptTypegpuTextureFormat = 0x0000_0029;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RGBA32_UINT: SubscriptTypegpuTextureFormat = 0x0000_002A;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RGBA32_SINT: SubscriptTypegpuTextureFormat = 0x0000_002B;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_STENCIL8: SubscriptTypegpuTextureFormat = 0x0000_002C;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_DEPTH16_UNORM: SubscriptTypegpuTextureFormat = 0x0000_002D;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_DEPTH24_PLUS: SubscriptTypegpuTextureFormat = 0x0000_002E;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_DEPTH24_PLUS_STENCIL8: SubscriptTypegpuTextureFormat = 0x0000_002F;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_DEPTH32_FLOAT: SubscriptTypegpuTextureFormat = 0x0000_0030;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_DEPTH32_FLOAT_STENCIL8: SubscriptTypegpuTextureFormat = 0x0000_0031;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_BC1_RGBA_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0032;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_BC1_RGBA_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_0033;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_BC2_RGBA_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0034;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_BC2_RGBA_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_0035;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_BC3_RGBA_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0036;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_BC3_RGBA_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_0037;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_BC4_R_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0038;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_BC4_R_SNORM: SubscriptTypegpuTextureFormat = 0x0000_0039;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_BC5_RG_UNORM: SubscriptTypegpuTextureFormat = 0x0000_003A;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_BC5_RG_SNORM: SubscriptTypegpuTextureFormat = 0x0000_003B;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_BC6_HRGB_UFLOAT: SubscriptTypegpuTextureFormat = 0x0000_003C;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_BC6_HRGB_FLOAT: SubscriptTypegpuTextureFormat = 0x0000_003D;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_BC7_RGBA_UNORM: SubscriptTypegpuTextureFormat = 0x0000_003E;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_BC7_RGBA_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_003F;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ETC2_RGB8_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0040;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ETC2_RGB8_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_0041;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ETC2_RGB8_A1_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0042;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ETC2_RGB8_A1_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_0043;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ETC2_RGBA8_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0044;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ETC2_RGBA8_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_0045;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_EACR11_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0046;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_EACR11_SNORM: SubscriptTypegpuTextureFormat = 0x0000_0047;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_EACRG11_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0048;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_EACRG11_SNORM: SubscriptTypegpuTextureFormat = 0x0000_0049;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC4X4_UNORM: SubscriptTypegpuTextureFormat = 0x0000_004A;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC4X4_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_004B;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC5X4_UNORM: SubscriptTypegpuTextureFormat = 0x0000_004C;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC5X4_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_004D;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC5X5_UNORM: SubscriptTypegpuTextureFormat = 0x0000_004E;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC5X5_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_004F;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC6X5_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0050;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC6X5_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_0051;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC6X6_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0052;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC6X6_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_0053;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC8X5_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0054;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC8X5_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_0055;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC8X6_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0056;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC8X6_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_0057;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC8X8_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0058;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC8X8_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_0059;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC10X5_UNORM: SubscriptTypegpuTextureFormat = 0x0000_005A;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC10X5_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_005B;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC10X6_UNORM: SubscriptTypegpuTextureFormat = 0x0000_005C;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC10X6_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_005D;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC10X8_UNORM: SubscriptTypegpuTextureFormat = 0x0000_005E;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC10X8_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_005F;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC10X10_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0060;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC10X10_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_0061;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC12X10_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0062;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC12X10_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_0063;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC12X12_UNORM: SubscriptTypegpuTextureFormat = 0x0000_0064;
/// `subscript-typegpu.h`: `enum.texture_format` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_ASTC12X12_UNORM_SRGB: SubscriptTypegpuTextureFormat = 0x0000_0065;

/// `subscript-typegpu.h`: `enum.texture_view_dimension` scalar set.
pub type SubscriptTypegpuTextureViewDimension = i32;
/// `subscript-typegpu.h`: `enum.texture_view_dimension` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_VIEW_DIMENSION_UNDEFINED: SubscriptTypegpuTextureViewDimension = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.texture_view_dimension` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_VIEW_DIMENSION_1_D: SubscriptTypegpuTextureViewDimension = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.texture_view_dimension` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_VIEW_DIMENSION_2_D: SubscriptTypegpuTextureViewDimension = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.texture_view_dimension` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_VIEW_DIMENSION_2_D_ARRAY: SubscriptTypegpuTextureViewDimension = 0x0000_0003;
/// `subscript-typegpu.h`: `enum.texture_view_dimension` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_VIEW_DIMENSION_CUBE: SubscriptTypegpuTextureViewDimension = 0x0000_0004;
/// `subscript-typegpu.h`: `enum.texture_view_dimension` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_VIEW_DIMENSION_CUBE_ARRAY: SubscriptTypegpuTextureViewDimension = 0x0000_0005;
/// `subscript-typegpu.h`: `enum.texture_view_dimension` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_VIEW_DIMENSION_3_D: SubscriptTypegpuTextureViewDimension = 0x0000_0006;

/// `subscript-typegpu.h`: `enum.texture_aspect` scalar set.
pub type SubscriptTypegpuTextureAspect = i32;
/// `subscript-typegpu.h`: `enum.texture_aspect` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_ASPECT_UNDEFINED: SubscriptTypegpuTextureAspect = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.texture_aspect` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_ASPECT_ALL: SubscriptTypegpuTextureAspect = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.texture_aspect` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_ASPECT_STENCIL_ONLY: SubscriptTypegpuTextureAspect = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.texture_aspect` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_ASPECT_DEPTH_ONLY: SubscriptTypegpuTextureAspect = 0x0000_0003;

/// `subscript-typegpu.h`: `enum.address_mode` scalar set.
pub type SubscriptTypegpuAddressMode = i32;
/// `subscript-typegpu.h`: `enum.address_mode` value.
pub const SUBSCRIPT_TYPEGPU_ADDRESS_MODE_UNDEFINED: SubscriptTypegpuAddressMode = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.address_mode` value.
pub const SUBSCRIPT_TYPEGPU_ADDRESS_MODE_CLAMP_TO_EDGE: SubscriptTypegpuAddressMode = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.address_mode` value.
pub const SUBSCRIPT_TYPEGPU_ADDRESS_MODE_REPEAT: SubscriptTypegpuAddressMode = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.address_mode` value.
pub const SUBSCRIPT_TYPEGPU_ADDRESS_MODE_MIRROR_REPEAT: SubscriptTypegpuAddressMode = 0x0000_0003;

/// `subscript-typegpu.h`: `enum.filter_mode` scalar set.
pub type SubscriptTypegpuFilterMode = i32;
/// `subscript-typegpu.h`: `enum.filter_mode` value.
pub const SUBSCRIPT_TYPEGPU_FILTER_MODE_UNDEFINED: SubscriptTypegpuFilterMode = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.filter_mode` value.
pub const SUBSCRIPT_TYPEGPU_FILTER_MODE_NEAREST: SubscriptTypegpuFilterMode = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.filter_mode` value.
pub const SUBSCRIPT_TYPEGPU_FILTER_MODE_LINEAR: SubscriptTypegpuFilterMode = 0x0000_0002;

/// `subscript-typegpu.h`: `enum.mipmap_filter_mode` scalar set.
pub type SubscriptTypegpuMipmapFilterMode = i32;
/// `subscript-typegpu.h`: `enum.mipmap_filter_mode` value.
pub const SUBSCRIPT_TYPEGPU_MIPMAP_FILTER_MODE_UNDEFINED: SubscriptTypegpuMipmapFilterMode = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.mipmap_filter_mode` value.
pub const SUBSCRIPT_TYPEGPU_MIPMAP_FILTER_MODE_NEAREST: SubscriptTypegpuMipmapFilterMode = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.mipmap_filter_mode` value.
pub const SUBSCRIPT_TYPEGPU_MIPMAP_FILTER_MODE_LINEAR: SubscriptTypegpuMipmapFilterMode = 0x0000_0002;

/// `subscript-typegpu.h`: `enum.compare_function` scalar set.
pub type SubscriptTypegpuCompareFunction = i32;
/// `subscript-typegpu.h`: `enum.compare_function` value.
pub const SUBSCRIPT_TYPEGPU_COMPARE_FUNCTION_UNDEFINED: SubscriptTypegpuCompareFunction = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.compare_function` value.
pub const SUBSCRIPT_TYPEGPU_COMPARE_FUNCTION_NEVER: SubscriptTypegpuCompareFunction = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.compare_function` value.
pub const SUBSCRIPT_TYPEGPU_COMPARE_FUNCTION_LESS: SubscriptTypegpuCompareFunction = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.compare_function` value.
pub const SUBSCRIPT_TYPEGPU_COMPARE_FUNCTION_EQUAL: SubscriptTypegpuCompareFunction = 0x0000_0003;
/// `subscript-typegpu.h`: `enum.compare_function` value.
pub const SUBSCRIPT_TYPEGPU_COMPARE_FUNCTION_LESS_EQUAL: SubscriptTypegpuCompareFunction = 0x0000_0004;
/// `subscript-typegpu.h`: `enum.compare_function` value.
pub const SUBSCRIPT_TYPEGPU_COMPARE_FUNCTION_GREATER: SubscriptTypegpuCompareFunction = 0x0000_0005;
/// `subscript-typegpu.h`: `enum.compare_function` value.
pub const SUBSCRIPT_TYPEGPU_COMPARE_FUNCTION_NOT_EQUAL: SubscriptTypegpuCompareFunction = 0x0000_0006;
/// `subscript-typegpu.h`: `enum.compare_function` value.
pub const SUBSCRIPT_TYPEGPU_COMPARE_FUNCTION_GREATER_EQUAL: SubscriptTypegpuCompareFunction = 0x0000_0007;
/// `subscript-typegpu.h`: `enum.compare_function` value.
pub const SUBSCRIPT_TYPEGPU_COMPARE_FUNCTION_ALWAYS: SubscriptTypegpuCompareFunction = 0x0000_0008;

/// `subscript-typegpu.h`: `bitflag.shader_stage` scalar set.
pub type SubscriptTypegpuShaderStage = u64;
/// `subscript-typegpu.h`: `bitflag.shader_stage` value.
pub const SUBSCRIPT_TYPEGPU_SHADER_STAGE_NONE: SubscriptTypegpuShaderStage = 0x0;
/// `subscript-typegpu.h`: `bitflag.shader_stage` value.
pub const SUBSCRIPT_TYPEGPU_SHADER_STAGE_VERTEX: SubscriptTypegpuShaderStage = 0x1;
/// `subscript-typegpu.h`: `bitflag.shader_stage` value.
pub const SUBSCRIPT_TYPEGPU_SHADER_STAGE_FRAGMENT: SubscriptTypegpuShaderStage = 0x2;
/// `subscript-typegpu.h`: `bitflag.shader_stage` value.
pub const SUBSCRIPT_TYPEGPU_SHADER_STAGE_COMPUTE: SubscriptTypegpuShaderStage = 0x4;

/// `subscript-typegpu.h`: `enum.buffer_binding_type` scalar set.
pub type SubscriptTypegpuBufferBindingType = i32;
/// `subscript-typegpu.h`: `enum.buffer_binding_type` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_BINDING_TYPE_BINDING_NOT_USED: SubscriptTypegpuBufferBindingType = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.buffer_binding_type` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_BINDING_TYPE_UNDEFINED: SubscriptTypegpuBufferBindingType = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.buffer_binding_type` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_BINDING_TYPE_UNIFORM: SubscriptTypegpuBufferBindingType = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.buffer_binding_type` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_BINDING_TYPE_STORAGE: SubscriptTypegpuBufferBindingType = 0x0000_0003;
/// `subscript-typegpu.h`: `enum.buffer_binding_type` value.
pub const SUBSCRIPT_TYPEGPU_BUFFER_BINDING_TYPE_READ_ONLY_STORAGE: SubscriptTypegpuBufferBindingType = 0x0000_0004;

/// `subscript-typegpu.h`: `enum.sampler_binding_type` scalar set.
pub type SubscriptTypegpuSamplerBindingType = i32;
/// `subscript-typegpu.h`: `enum.sampler_binding_type` value.
pub const SUBSCRIPT_TYPEGPU_SAMPLER_BINDING_TYPE_BINDING_NOT_USED: SubscriptTypegpuSamplerBindingType = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.sampler_binding_type` value.
pub const SUBSCRIPT_TYPEGPU_SAMPLER_BINDING_TYPE_UNDEFINED: SubscriptTypegpuSamplerBindingType = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.sampler_binding_type` value.
pub const SUBSCRIPT_TYPEGPU_SAMPLER_BINDING_TYPE_FILTERING: SubscriptTypegpuSamplerBindingType = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.sampler_binding_type` value.
pub const SUBSCRIPT_TYPEGPU_SAMPLER_BINDING_TYPE_NON_FILTERING: SubscriptTypegpuSamplerBindingType = 0x0000_0003;
/// `subscript-typegpu.h`: `enum.sampler_binding_type` value.
pub const SUBSCRIPT_TYPEGPU_SAMPLER_BINDING_TYPE_COMPARISON: SubscriptTypegpuSamplerBindingType = 0x0000_0004;

/// `subscript-typegpu.h`: `enum.texture_sample_type` scalar set.
pub type SubscriptTypegpuTextureSampleType = i32;
/// `subscript-typegpu.h`: `enum.texture_sample_type` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_SAMPLE_TYPE_BINDING_NOT_USED: SubscriptTypegpuTextureSampleType = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.texture_sample_type` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_SAMPLE_TYPE_UNDEFINED: SubscriptTypegpuTextureSampleType = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.texture_sample_type` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_SAMPLE_TYPE_FLOAT: SubscriptTypegpuTextureSampleType = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.texture_sample_type` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_SAMPLE_TYPE_UNFILTERABLE_FLOAT: SubscriptTypegpuTextureSampleType = 0x0000_0003;
/// `subscript-typegpu.h`: `enum.texture_sample_type` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_SAMPLE_TYPE_DEPTH: SubscriptTypegpuTextureSampleType = 0x0000_0004;
/// `subscript-typegpu.h`: `enum.texture_sample_type` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_SAMPLE_TYPE_SINT: SubscriptTypegpuTextureSampleType = 0x0000_0005;
/// `subscript-typegpu.h`: `enum.texture_sample_type` value.
pub const SUBSCRIPT_TYPEGPU_TEXTURE_SAMPLE_TYPE_UINT: SubscriptTypegpuTextureSampleType = 0x0000_0006;

/// `subscript-typegpu.h`: `enum.storage_texture_access` scalar set.
pub type SubscriptTypegpuStorageTextureAccess = i32;
/// `subscript-typegpu.h`: `enum.storage_texture_access` value.
pub const SUBSCRIPT_TYPEGPU_STORAGE_TEXTURE_ACCESS_BINDING_NOT_USED: SubscriptTypegpuStorageTextureAccess = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.storage_texture_access` value.
pub const SUBSCRIPT_TYPEGPU_STORAGE_TEXTURE_ACCESS_UNDEFINED: SubscriptTypegpuStorageTextureAccess = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.storage_texture_access` value.
pub const SUBSCRIPT_TYPEGPU_STORAGE_TEXTURE_ACCESS_WRITE_ONLY: SubscriptTypegpuStorageTextureAccess = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.storage_texture_access` value.
pub const SUBSCRIPT_TYPEGPU_STORAGE_TEXTURE_ACCESS_READ_ONLY: SubscriptTypegpuStorageTextureAccess = 0x0000_0003;
/// `subscript-typegpu.h`: `enum.storage_texture_access` value.
pub const SUBSCRIPT_TYPEGPU_STORAGE_TEXTURE_ACCESS_READ_WRITE: SubscriptTypegpuStorageTextureAccess = 0x0000_0004;

/// `subscript-typegpu.h`: `enum.vertex_format` scalar set.
pub type SubscriptTypegpuVertexFormat = i32;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UINT8: SubscriptTypegpuVertexFormat = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UINT8X2: SubscriptTypegpuVertexFormat = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UINT8X4: SubscriptTypegpuVertexFormat = 0x0000_0003;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_SINT8: SubscriptTypegpuVertexFormat = 0x0000_0004;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_SINT8X2: SubscriptTypegpuVertexFormat = 0x0000_0005;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_SINT8X4: SubscriptTypegpuVertexFormat = 0x0000_0006;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UNORM8: SubscriptTypegpuVertexFormat = 0x0000_0007;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UNORM8X2: SubscriptTypegpuVertexFormat = 0x0000_0008;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UNORM8X4: SubscriptTypegpuVertexFormat = 0x0000_0009;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_SNORM8: SubscriptTypegpuVertexFormat = 0x0000_000A;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_SNORM8X2: SubscriptTypegpuVertexFormat = 0x0000_000B;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_SNORM8X4: SubscriptTypegpuVertexFormat = 0x0000_000C;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UINT16: SubscriptTypegpuVertexFormat = 0x0000_000D;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UINT16X2: SubscriptTypegpuVertexFormat = 0x0000_000E;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UINT16X4: SubscriptTypegpuVertexFormat = 0x0000_000F;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_SINT16: SubscriptTypegpuVertexFormat = 0x0000_0010;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_SINT16X2: SubscriptTypegpuVertexFormat = 0x0000_0011;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_SINT16X4: SubscriptTypegpuVertexFormat = 0x0000_0012;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UNORM16: SubscriptTypegpuVertexFormat = 0x0000_0013;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UNORM16X2: SubscriptTypegpuVertexFormat = 0x0000_0014;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UNORM16X4: SubscriptTypegpuVertexFormat = 0x0000_0015;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_SNORM16: SubscriptTypegpuVertexFormat = 0x0000_0016;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_SNORM16X2: SubscriptTypegpuVertexFormat = 0x0000_0017;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_SNORM16X4: SubscriptTypegpuVertexFormat = 0x0000_0018;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_FLOAT16: SubscriptTypegpuVertexFormat = 0x0000_0019;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_FLOAT16X2: SubscriptTypegpuVertexFormat = 0x0000_001A;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_FLOAT16X4: SubscriptTypegpuVertexFormat = 0x0000_001B;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_FLOAT32: SubscriptTypegpuVertexFormat = 0x0000_001C;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_FLOAT32X2: SubscriptTypegpuVertexFormat = 0x0000_001D;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_FLOAT32X3: SubscriptTypegpuVertexFormat = 0x0000_001E;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_FLOAT32X4: SubscriptTypegpuVertexFormat = 0x0000_001F;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UINT32: SubscriptTypegpuVertexFormat = 0x0000_0020;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UINT32X2: SubscriptTypegpuVertexFormat = 0x0000_0021;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UINT32X3: SubscriptTypegpuVertexFormat = 0x0000_0022;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UINT32X4: SubscriptTypegpuVertexFormat = 0x0000_0023;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_SINT32: SubscriptTypegpuVertexFormat = 0x0000_0024;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_SINT32X2: SubscriptTypegpuVertexFormat = 0x0000_0025;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_SINT32X3: SubscriptTypegpuVertexFormat = 0x0000_0026;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_SINT32X4: SubscriptTypegpuVertexFormat = 0x0000_0027;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UNORM10_10_10_2: SubscriptTypegpuVertexFormat = 0x0000_0028;
/// `subscript-typegpu.h`: `enum.vertex_format` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_FORMAT_UNORM8X4_BGRA: SubscriptTypegpuVertexFormat = 0x0000_0029;

/// `subscript-typegpu.h`: `enum.vertex_step_mode` scalar set.
pub type SubscriptTypegpuVertexStepMode = i32;
/// `subscript-typegpu.h`: `enum.vertex_step_mode` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_STEP_MODE_UNDEFINED: SubscriptTypegpuVertexStepMode = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.vertex_step_mode` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_STEP_MODE_VERTEX: SubscriptTypegpuVertexStepMode = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.vertex_step_mode` value.
pub const SUBSCRIPT_TYPEGPU_VERTEX_STEP_MODE_INSTANCE: SubscriptTypegpuVertexStepMode = 0x0000_0002;

/// `subscript-typegpu.h`: `enum.primitive_topology` scalar set.
pub type SubscriptTypegpuPrimitiveTopology = i32;
/// `subscript-typegpu.h`: `enum.primitive_topology` value.
pub const SUBSCRIPT_TYPEGPU_PRIMITIVE_TOPOLOGY_UNDEFINED: SubscriptTypegpuPrimitiveTopology = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.primitive_topology` value.
pub const SUBSCRIPT_TYPEGPU_PRIMITIVE_TOPOLOGY_POINT_LIST: SubscriptTypegpuPrimitiveTopology = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.primitive_topology` value.
pub const SUBSCRIPT_TYPEGPU_PRIMITIVE_TOPOLOGY_LINE_LIST: SubscriptTypegpuPrimitiveTopology = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.primitive_topology` value.
pub const SUBSCRIPT_TYPEGPU_PRIMITIVE_TOPOLOGY_LINE_STRIP: SubscriptTypegpuPrimitiveTopology = 0x0000_0003;
/// `subscript-typegpu.h`: `enum.primitive_topology` value.
pub const SUBSCRIPT_TYPEGPU_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST: SubscriptTypegpuPrimitiveTopology = 0x0000_0004;
/// `subscript-typegpu.h`: `enum.primitive_topology` value.
pub const SUBSCRIPT_TYPEGPU_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP: SubscriptTypegpuPrimitiveTopology = 0x0000_0005;

/// `subscript-typegpu.h`: `enum.index_format` scalar set.
pub type SubscriptTypegpuIndexFormat = i32;
/// `subscript-typegpu.h`: `enum.index_format` value.
pub const SUBSCRIPT_TYPEGPU_INDEX_FORMAT_UNDEFINED: SubscriptTypegpuIndexFormat = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.index_format` value.
pub const SUBSCRIPT_TYPEGPU_INDEX_FORMAT_UINT16: SubscriptTypegpuIndexFormat = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.index_format` value.
pub const SUBSCRIPT_TYPEGPU_INDEX_FORMAT_UINT32: SubscriptTypegpuIndexFormat = 0x0000_0002;

/// `subscript-typegpu.h`: `enum.front_face` scalar set.
pub type SubscriptTypegpuFrontFace = i32;
/// `subscript-typegpu.h`: `enum.front_face` value.
pub const SUBSCRIPT_TYPEGPU_FRONT_FACE_UNDEFINED: SubscriptTypegpuFrontFace = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.front_face` value.
pub const SUBSCRIPT_TYPEGPU_FRONT_FACE_CCW: SubscriptTypegpuFrontFace = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.front_face` value.
pub const SUBSCRIPT_TYPEGPU_FRONT_FACE_CW: SubscriptTypegpuFrontFace = 0x0000_0002;

/// `subscript-typegpu.h`: `enum.cull_mode` scalar set.
pub type SubscriptTypegpuCullMode = i32;
/// `subscript-typegpu.h`: `enum.cull_mode` value.
pub const SUBSCRIPT_TYPEGPU_CULL_MODE_UNDEFINED: SubscriptTypegpuCullMode = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.cull_mode` value.
pub const SUBSCRIPT_TYPEGPU_CULL_MODE_NONE: SubscriptTypegpuCullMode = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.cull_mode` value.
pub const SUBSCRIPT_TYPEGPU_CULL_MODE_FRONT: SubscriptTypegpuCullMode = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.cull_mode` value.
pub const SUBSCRIPT_TYPEGPU_CULL_MODE_BACK: SubscriptTypegpuCullMode = 0x0000_0003;

/// `subscript-typegpu.h`: `enum.blend_operation` scalar set.
pub type SubscriptTypegpuBlendOperation = i32;
/// `subscript-typegpu.h`: `enum.blend_operation` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_OPERATION_UNDEFINED: SubscriptTypegpuBlendOperation = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.blend_operation` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_OPERATION_ADD: SubscriptTypegpuBlendOperation = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.blend_operation` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_OPERATION_SUBTRACT: SubscriptTypegpuBlendOperation = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.blend_operation` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_OPERATION_REVERSE_SUBTRACT: SubscriptTypegpuBlendOperation = 0x0000_0003;
/// `subscript-typegpu.h`: `enum.blend_operation` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_OPERATION_MIN: SubscriptTypegpuBlendOperation = 0x0000_0004;
/// `subscript-typegpu.h`: `enum.blend_operation` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_OPERATION_MAX: SubscriptTypegpuBlendOperation = 0x0000_0005;

/// `subscript-typegpu.h`: `enum.blend_factor` scalar set.
pub type SubscriptTypegpuBlendFactor = i32;
/// `subscript-typegpu.h`: `enum.blend_factor` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_FACTOR_UNDEFINED: SubscriptTypegpuBlendFactor = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.blend_factor` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_FACTOR_ZERO: SubscriptTypegpuBlendFactor = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.blend_factor` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_FACTOR_ONE: SubscriptTypegpuBlendFactor = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.blend_factor` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_FACTOR_SRC: SubscriptTypegpuBlendFactor = 0x0000_0003;
/// `subscript-typegpu.h`: `enum.blend_factor` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_FACTOR_ONE_MINUS_SRC: SubscriptTypegpuBlendFactor = 0x0000_0004;
/// `subscript-typegpu.h`: `enum.blend_factor` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_FACTOR_SRC_ALPHA: SubscriptTypegpuBlendFactor = 0x0000_0005;
/// `subscript-typegpu.h`: `enum.blend_factor` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA: SubscriptTypegpuBlendFactor = 0x0000_0006;
/// `subscript-typegpu.h`: `enum.blend_factor` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_FACTOR_DST: SubscriptTypegpuBlendFactor = 0x0000_0007;
/// `subscript-typegpu.h`: `enum.blend_factor` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_FACTOR_ONE_MINUS_DST: SubscriptTypegpuBlendFactor = 0x0000_0008;
/// `subscript-typegpu.h`: `enum.blend_factor` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_FACTOR_DST_ALPHA: SubscriptTypegpuBlendFactor = 0x0000_0009;
/// `subscript-typegpu.h`: `enum.blend_factor` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_FACTOR_ONE_MINUS_DST_ALPHA: SubscriptTypegpuBlendFactor = 0x0000_000A;
/// `subscript-typegpu.h`: `enum.blend_factor` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_FACTOR_SRC_ALPHA_SATURATED: SubscriptTypegpuBlendFactor = 0x0000_000B;
/// `subscript-typegpu.h`: `enum.blend_factor` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_FACTOR_CONSTANT: SubscriptTypegpuBlendFactor = 0x0000_000C;
/// `subscript-typegpu.h`: `enum.blend_factor` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_FACTOR_ONE_MINUS_CONSTANT: SubscriptTypegpuBlendFactor = 0x0000_000D;
/// `subscript-typegpu.h`: `enum.blend_factor` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_FACTOR_SRC1: SubscriptTypegpuBlendFactor = 0x0000_000E;
/// `subscript-typegpu.h`: `enum.blend_factor` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_FACTOR_ONE_MINUS_SRC1: SubscriptTypegpuBlendFactor = 0x0000_000F;
/// `subscript-typegpu.h`: `enum.blend_factor` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_FACTOR_SRC1_ALPHA: SubscriptTypegpuBlendFactor = 0x0000_0010;
/// `subscript-typegpu.h`: `enum.blend_factor` value.
pub const SUBSCRIPT_TYPEGPU_BLEND_FACTOR_ONE_MINUS_SRC1_ALPHA: SubscriptTypegpuBlendFactor = 0x0000_0011;

/// `subscript-typegpu.h`: `bitflag.color_write_mask` scalar set.
pub type SubscriptTypegpuColorWriteMask = u64;
/// `subscript-typegpu.h`: `bitflag.color_write_mask` value.
pub const SUBSCRIPT_TYPEGPU_COLOR_WRITE_MASK_NONE: SubscriptTypegpuColorWriteMask = 0x0;
/// `subscript-typegpu.h`: `bitflag.color_write_mask` value.
pub const SUBSCRIPT_TYPEGPU_COLOR_WRITE_MASK_RED: SubscriptTypegpuColorWriteMask = 0x1;
/// `subscript-typegpu.h`: `bitflag.color_write_mask` value.
pub const SUBSCRIPT_TYPEGPU_COLOR_WRITE_MASK_GREEN: SubscriptTypegpuColorWriteMask = 0x2;
/// `subscript-typegpu.h`: `bitflag.color_write_mask` value.
pub const SUBSCRIPT_TYPEGPU_COLOR_WRITE_MASK_BLUE: SubscriptTypegpuColorWriteMask = 0x4;
/// `subscript-typegpu.h`: `bitflag.color_write_mask` value.
pub const SUBSCRIPT_TYPEGPU_COLOR_WRITE_MASK_ALPHA: SubscriptTypegpuColorWriteMask = 0x8;
/// `subscript-typegpu.h`: `bitflag.color_write_mask` value.
pub const SUBSCRIPT_TYPEGPU_COLOR_WRITE_MASK_ALL: SubscriptTypegpuColorWriteMask = 0xF;

/// `subscript-typegpu.h`: `enum.optional_bool` scalar set.
pub type SubscriptTypegpuOptionalBool = i32;
/// `subscript-typegpu.h`: `enum.optional_bool` value.
pub const SUBSCRIPT_TYPEGPU_OPTIONAL_BOOL_FALSE: SubscriptTypegpuOptionalBool = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.optional_bool` value.
pub const SUBSCRIPT_TYPEGPU_OPTIONAL_BOOL_TRUE: SubscriptTypegpuOptionalBool = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.optional_bool` value.
pub const SUBSCRIPT_TYPEGPU_OPTIONAL_BOOL_UNDEFINED: SubscriptTypegpuOptionalBool = 0x0000_0002;

/// `subscript-typegpu.h`: `enum.stencil_operation` scalar set.
pub type SubscriptTypegpuStencilOperation = i32;
/// `subscript-typegpu.h`: `enum.stencil_operation` value.
pub const SUBSCRIPT_TYPEGPU_STENCIL_OPERATION_UNDEFINED: SubscriptTypegpuStencilOperation = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.stencil_operation` value.
pub const SUBSCRIPT_TYPEGPU_STENCIL_OPERATION_KEEP: SubscriptTypegpuStencilOperation = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.stencil_operation` value.
pub const SUBSCRIPT_TYPEGPU_STENCIL_OPERATION_ZERO: SubscriptTypegpuStencilOperation = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.stencil_operation` value.
pub const SUBSCRIPT_TYPEGPU_STENCIL_OPERATION_REPLACE: SubscriptTypegpuStencilOperation = 0x0000_0003;
/// `subscript-typegpu.h`: `enum.stencil_operation` value.
pub const SUBSCRIPT_TYPEGPU_STENCIL_OPERATION_INVERT: SubscriptTypegpuStencilOperation = 0x0000_0004;
/// `subscript-typegpu.h`: `enum.stencil_operation` value.
pub const SUBSCRIPT_TYPEGPU_STENCIL_OPERATION_INCREMENT_CLAMP: SubscriptTypegpuStencilOperation = 0x0000_0005;
/// `subscript-typegpu.h`: `enum.stencil_operation` value.
pub const SUBSCRIPT_TYPEGPU_STENCIL_OPERATION_DECREMENT_CLAMP: SubscriptTypegpuStencilOperation = 0x0000_0006;
/// `subscript-typegpu.h`: `enum.stencil_operation` value.
pub const SUBSCRIPT_TYPEGPU_STENCIL_OPERATION_INCREMENT_WRAP: SubscriptTypegpuStencilOperation = 0x0000_0007;
/// `subscript-typegpu.h`: `enum.stencil_operation` value.
pub const SUBSCRIPT_TYPEGPU_STENCIL_OPERATION_DECREMENT_WRAP: SubscriptTypegpuStencilOperation = 0x0000_0008;

/// `subscript-typegpu.h`: `enum.load_op` scalar set.
pub type SubscriptTypegpuLoadOp = i32;
/// `subscript-typegpu.h`: `enum.load_op` value.
pub const SUBSCRIPT_TYPEGPU_LOAD_OP_UNDEFINED: SubscriptTypegpuLoadOp = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.load_op` value.
pub const SUBSCRIPT_TYPEGPU_LOAD_OP_LOAD: SubscriptTypegpuLoadOp = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.load_op` value.
pub const SUBSCRIPT_TYPEGPU_LOAD_OP_CLEAR: SubscriptTypegpuLoadOp = 0x0000_0002;

/// `subscript-typegpu.h`: `enum.store_op` scalar set.
pub type SubscriptTypegpuStoreOp = i32;
/// `subscript-typegpu.h`: `enum.store_op` value.
pub const SUBSCRIPT_TYPEGPU_STORE_OP_UNDEFINED: SubscriptTypegpuStoreOp = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.store_op` value.
pub const SUBSCRIPT_TYPEGPU_STORE_OP_STORE: SubscriptTypegpuStoreOp = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.store_op` value.
pub const SUBSCRIPT_TYPEGPU_STORE_OP_DISCARD: SubscriptTypegpuStoreOp = 0x0000_0002;

/// `subscript-typegpu.h`: `enum.query_type` scalar set.
pub type SubscriptTypegpuQueryType = i32;
/// `subscript-typegpu.h`: `enum.query_type` value.
pub const SUBSCRIPT_TYPEGPU_QUERY_TYPE_OCCLUSION: SubscriptTypegpuQueryType = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.query_type` value.
pub const SUBSCRIPT_TYPEGPU_QUERY_TYPE_TIMESTAMP: SubscriptTypegpuQueryType = 0x0000_0002;

/// `subscript-typegpu.h`: `enum.error_filter` scalar set.
pub type SubscriptTypegpuErrorFilter = i32;
/// `subscript-typegpu.h`: `enum.error_filter` value.
pub const SUBSCRIPT_TYPEGPU_ERROR_FILTER_VALIDATION: SubscriptTypegpuErrorFilter = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.error_filter` value.
pub const SUBSCRIPT_TYPEGPU_ERROR_FILTER_OUT_OF_MEMORY: SubscriptTypegpuErrorFilter = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.error_filter` value.
pub const SUBSCRIPT_TYPEGPU_ERROR_FILTER_INTERNAL: SubscriptTypegpuErrorFilter = 0x0000_0003;

/// `subscript-typegpu.h`: `enum.error_type` scalar set.
pub type SubscriptTypegpuErrorType = i32;
/// `subscript-typegpu.h`: `enum.error_type` value.
pub const SUBSCRIPT_TYPEGPU_ERROR_TYPE_NO_ERROR: SubscriptTypegpuErrorType = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.error_type` value.
pub const SUBSCRIPT_TYPEGPU_ERROR_TYPE_VALIDATION: SubscriptTypegpuErrorType = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.error_type` value.
pub const SUBSCRIPT_TYPEGPU_ERROR_TYPE_OUT_OF_MEMORY: SubscriptTypegpuErrorType = 0x0000_0003;
/// `subscript-typegpu.h`: `enum.error_type` value.
pub const SUBSCRIPT_TYPEGPU_ERROR_TYPE_INTERNAL: SubscriptTypegpuErrorType = 0x0000_0004;
/// `subscript-typegpu.h`: `enum.error_type` value.
pub const SUBSCRIPT_TYPEGPU_ERROR_TYPE_UNKNOWN: SubscriptTypegpuErrorType = 0x0000_0005;

/// `subscript-typegpu.h`: `enum.device_lost_reason` scalar set.
pub type SubscriptTypegpuDeviceLostReason = i32;
/// `subscript-typegpu.h`: `enum.device_lost_reason` value.
pub const SUBSCRIPT_TYPEGPU_DEVICE_LOST_REASON_UNKNOWN: SubscriptTypegpuDeviceLostReason = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.device_lost_reason` value.
pub const SUBSCRIPT_TYPEGPU_DEVICE_LOST_REASON_DESTROYED: SubscriptTypegpuDeviceLostReason = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.device_lost_reason` value.
pub const SUBSCRIPT_TYPEGPU_DEVICE_LOST_REASON_CALLBACK_CANCELLED: SubscriptTypegpuDeviceLostReason = 0x0000_0003;
/// `subscript-typegpu.h`: `enum.device_lost_reason` value.
pub const SUBSCRIPT_TYPEGPU_DEVICE_LOST_REASON_FAILED_CREATION: SubscriptTypegpuDeviceLostReason = 0x0000_0004;

/// `subscript-typegpu.h`: `enum.feature_name` scalar set.
pub type SubscriptTypegpuFeatureName = i32;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_CORE_FEATURES_AND_LIMITS: SubscriptTypegpuFeatureName = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_DEPTH_CLIP_CONTROL: SubscriptTypegpuFeatureName = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_DEPTH32_FLOAT_STENCIL8: SubscriptTypegpuFeatureName = 0x0000_0003;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_TEXTURE_COMPRESSION_BC: SubscriptTypegpuFeatureName = 0x0000_0004;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_TEXTURE_COMPRESSION_BC_SLICED3_D: SubscriptTypegpuFeatureName = 0x0000_0005;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_TEXTURE_COMPRESSION_ETC2: SubscriptTypegpuFeatureName = 0x0000_0006;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_TEXTURE_COMPRESSION_ASTC: SubscriptTypegpuFeatureName = 0x0000_0007;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_TEXTURE_COMPRESSION_ASTC_SLICED3_D: SubscriptTypegpuFeatureName = 0x0000_0008;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_TIMESTAMP_QUERY: SubscriptTypegpuFeatureName = 0x0000_0009;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_INDIRECT_FIRST_INSTANCE: SubscriptTypegpuFeatureName = 0x0000_000A;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_SHADER_F16: SubscriptTypegpuFeatureName = 0x0000_000B;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_RG11_B10_UFLOAT_RENDERABLE: SubscriptTypegpuFeatureName = 0x0000_000C;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_BGRA8_UNORM_STORAGE: SubscriptTypegpuFeatureName = 0x0000_000D;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_FLOAT32_FILTERABLE: SubscriptTypegpuFeatureName = 0x0000_000E;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_FLOAT32_BLENDABLE: SubscriptTypegpuFeatureName = 0x0000_000F;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_CLIP_DISTANCES: SubscriptTypegpuFeatureName = 0x0000_0010;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_DUAL_SOURCE_BLENDING: SubscriptTypegpuFeatureName = 0x0000_0011;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_SUBGROUPS: SubscriptTypegpuFeatureName = 0x0000_0012;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_TEXTURE_FORMATS_TIER1: SubscriptTypegpuFeatureName = 0x0000_0013;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_TEXTURE_FORMATS_TIER2: SubscriptTypegpuFeatureName = 0x0000_0014;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_PRIMITIVE_INDEX: SubscriptTypegpuFeatureName = 0x0000_0015;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_TEXTURE_COMPONENT_SWIZZLE: SubscriptTypegpuFeatureName = 0x0000_0016;
/// `subscript-typegpu.h`: `enum.feature_name` value.
pub const SUBSCRIPT_TYPEGPU_FEATURE_NAME_SUBGROUP_SIZE_CONTROL: SubscriptTypegpuFeatureName = 0x0000_0017;

/// `subscript-typegpu.h`: `enum.instance_feature_name` scalar set.
pub type SubscriptTypegpuInstanceFeatureName = i32;
/// `subscript-typegpu.h`: `enum.instance_feature_name` value.
pub const SUBSCRIPT_TYPEGPU_INSTANCE_FEATURE_NAME_TIMED_WAIT_ANY: SubscriptTypegpuInstanceFeatureName = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.instance_feature_name` value.
pub const SUBSCRIPT_TYPEGPU_INSTANCE_FEATURE_NAME_SHADER_SOURCE_SPIRV: SubscriptTypegpuInstanceFeatureName = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.instance_feature_name` value.
pub const SUBSCRIPT_TYPEGPU_INSTANCE_FEATURE_NAME_MULTIPLE_DEVICES_PER_ADAPTER: SubscriptTypegpuInstanceFeatureName = 0x0000_0003;

/// `subscript-typegpu.h`: `enum.backend_type` scalar set.
pub type SubscriptTypegpuBackendType = i32;
/// `subscript-typegpu.h`: `enum.backend_type` value.
pub const SUBSCRIPT_TYPEGPU_BACKEND_TYPE_UNDEFINED: SubscriptTypegpuBackendType = 0x0000_0000;
/// `subscript-typegpu.h`: `enum.backend_type` value.
pub const SUBSCRIPT_TYPEGPU_BACKEND_TYPE_NULL: SubscriptTypegpuBackendType = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.backend_type` value.
pub const SUBSCRIPT_TYPEGPU_BACKEND_TYPE_WEB_GPU: SubscriptTypegpuBackendType = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.backend_type` value.
pub const SUBSCRIPT_TYPEGPU_BACKEND_TYPE_D3_D11: SubscriptTypegpuBackendType = 0x0000_0003;
/// `subscript-typegpu.h`: `enum.backend_type` value.
pub const SUBSCRIPT_TYPEGPU_BACKEND_TYPE_D3_D12: SubscriptTypegpuBackendType = 0x0000_0004;
/// `subscript-typegpu.h`: `enum.backend_type` value.
pub const SUBSCRIPT_TYPEGPU_BACKEND_TYPE_METAL: SubscriptTypegpuBackendType = 0x0000_0005;
/// `subscript-typegpu.h`: `enum.backend_type` value.
pub const SUBSCRIPT_TYPEGPU_BACKEND_TYPE_VULKAN: SubscriptTypegpuBackendType = 0x0000_0006;
/// `subscript-typegpu.h`: `enum.backend_type` value.
pub const SUBSCRIPT_TYPEGPU_BACKEND_TYPE_OPEN_GL: SubscriptTypegpuBackendType = 0x0000_0007;
/// `subscript-typegpu.h`: `enum.backend_type` value.
pub const SUBSCRIPT_TYPEGPU_BACKEND_TYPE_OPEN_GLES: SubscriptTypegpuBackendType = 0x0000_0008;

/// `subscript-typegpu.h`: `enum.adapter_type` scalar set.
pub type SubscriptTypegpuAdapterType = i32;
/// `subscript-typegpu.h`: `enum.adapter_type` value.
pub const SUBSCRIPT_TYPEGPU_ADAPTER_TYPE_DISCRETE_GPU: SubscriptTypegpuAdapterType = 0x0000_0001;
/// `subscript-typegpu.h`: `enum.adapter_type` value.
pub const SUBSCRIPT_TYPEGPU_ADAPTER_TYPE_INTEGRATED_GPU: SubscriptTypegpuAdapterType = 0x0000_0002;
/// `subscript-typegpu.h`: `enum.adapter_type` value.
pub const SUBSCRIPT_TYPEGPU_ADAPTER_TYPE_CPU: SubscriptTypegpuAdapterType = 0x0000_0003;
/// `subscript-typegpu.h`: `enum.adapter_type` value.
pub const SUBSCRIPT_TYPEGPU_ADAPTER_TYPE_UNKNOWN: SubscriptTypegpuAdapterType = 0x0000_0004;

/// `subscript-typegpu.h`: facade-owned future id.
pub type SubscriptTypegpuFutureId = u64;

/// Runtime slot-kind tag for `request_adapter` futures.
const SLOT_KIND_REQUEST_ADAPTER: u32 = 0;

/// Runtime slot-kind tag for `request_device` futures.
const SLOT_KIND_REQUEST_DEVICE: u32 = 1;

/// Runtime slot-kind tag for `create_compute_pipeline_async` futures.
const SLOT_KIND_CREATE_COMPUTE_PIPELINE_ASYNC: u32 = 3;

/// Runtime slot-kind tag for `create_render_pipeline_async` futures.
const SLOT_KIND_CREATE_RENDER_PIPELINE_ASYNC: u32 = 4;

/// Runtime slot-kind tag for `queue_work_done` futures.
const SLOT_KIND_QUEUE_WORK_DONE: u32 = 5;

/// Runtime slot-kind tag for `buffer_map` futures.
const SLOT_KIND_BUFFER_MAP: u32 = 6;

/// Copies a callback-scope string view before the callback returns.
///
/// # Safety
///
/// `view` is valid for the callback duration.
unsafe fn copy_string_view(view: WGPUStringView) -> String {
    if view.data.is_null() {
        return String::new();
    }
    let bytes: &[u8] = if view.length == WGPU_STRLEN {
        CStr::from_ptr(view.data).to_bytes()
    } else {
        std::slice::from_raw_parts(view.data.cast::<u8>(), view.length)
    };
    String::from_utf8_lossy(bytes).into_owned()
}

// SAFETY: the callback signature matches the pinned webgpu.h declaration.
unsafe extern "C" fn request_adapter_callback(
    status: i32,
    adapter: WGPUAdapter,
    message: WGPUStringView,
    userdata1: *mut c_void,
    _userdata2: *mut c_void,
) {
    // SAFETY: callback pointers and views remain valid for this callback.
    runtime::callback_guard(|| unsafe {
        let message = copy_string_view(message);
        runtime::complete_from_callback(
            userdata1,
            SLOT_KIND_REQUEST_ADAPTER,
            status == WGPURequestAdapterStatus_Success,
            status,
            adapter as usize,
            message,
        );
    });
}

// SAFETY: the callback signature matches the pinned webgpu.h declaration.
unsafe extern "C" fn request_device_callback(
    status: i32,
    device: WGPUDevice,
    message: WGPUStringView,
    userdata1: *mut c_void,
    userdata2: *mut c_void,
) {
    // SAFETY: callback pointers and views remain valid for this callback.
    runtime::callback_guard(|| unsafe {
        let message = copy_string_view(message);
        let event_id = userdata2 as usize;
        if status == WGPURequestDeviceStatus_Success && device as usize != 0 {
            runtime::associate_device_events(event_id, device as usize);
        } else {
            runtime::discard_device_event_slot(event_id);
        }
        runtime::complete_from_callback(
            userdata1,
            SLOT_KIND_REQUEST_DEVICE,
            status == WGPURequestDeviceStatus_Success,
            status,
            device as usize,
            message,
        );
    });
}

// SAFETY: the callback signature matches the pinned webgpu.h declaration.
unsafe extern "C" fn create_compute_pipeline_async_callback(
    status: i32,
    computePipeline: WGPUComputePipeline,
    message: WGPUStringView,
    userdata1: *mut c_void,
    _userdata2: *mut c_void,
) {
    // SAFETY: callback pointers and views remain valid for this callback.
    runtime::callback_guard(|| unsafe {
        let message = copy_string_view(message);
        runtime::complete_from_callback(
            userdata1,
            SLOT_KIND_CREATE_COMPUTE_PIPELINE_ASYNC,
            status == WGPUCreatePipelineAsyncStatus_Success,
            status,
            computePipeline as usize,
            message,
        );
    });
}

// SAFETY: the callback signature matches the pinned webgpu.h declaration.
unsafe extern "C" fn create_render_pipeline_async_callback(
    status: i32,
    renderPipeline: WGPURenderPipeline,
    message: WGPUStringView,
    userdata1: *mut c_void,
    _userdata2: *mut c_void,
) {
    // SAFETY: callback pointers and views remain valid for this callback.
    runtime::callback_guard(|| unsafe {
        let message = copy_string_view(message);
        runtime::complete_from_callback(
            userdata1,
            SLOT_KIND_CREATE_RENDER_PIPELINE_ASYNC,
            status == WGPUCreatePipelineAsyncStatus_Success,
            status,
            renderPipeline as usize,
            message,
        );
    });
}

// SAFETY: the callback signature matches the pinned webgpu.h declaration.
unsafe extern "C" fn queue_work_done_callback(
    status: i32,
    message: WGPUStringView,
    userdata1: *mut c_void,
    _userdata2: *mut c_void,
) {
    // SAFETY: callback pointers and views remain valid for this callback.
    runtime::callback_guard(|| unsafe {
        let message = copy_string_view(message);
        runtime::complete_from_callback(
            userdata1,
            SLOT_KIND_QUEUE_WORK_DONE,
            status == WGPUQueueWorkDoneStatus_Success,
            status,
            0,
            message,
        );
    });
}

// SAFETY: the callback signature matches the pinned webgpu.h declaration.
unsafe extern "C" fn buffer_map_callback(
    status: i32,
    message: WGPUStringView,
    userdata1: *mut c_void,
    _userdata2: *mut c_void,
) {
    // SAFETY: callback pointers and views remain valid for this callback.
    runtime::callback_guard(|| unsafe {
        let message = copy_string_view(message);
        runtime::complete_from_callback(
            userdata1,
            SLOT_KIND_BUFFER_MAP,
            status == WGPUMapAsyncStatus_Success,
            status,
            0,
            message,
        );
    });
}

/// webgpu.yml pop-error-scope success value.
const WGPUPopErrorScopeStatus_Success: i32 = 0x0000_0001;
/// Runtime slot-kind tag for pop-error-scope futures.
const SLOT_KIND_POP_ERROR_SCOPE: u32 = 2;

// SAFETY: the callback signature matches the pinned webgpu.h declaration.
unsafe extern "C" fn device_lost_callback(
    _device: *const WGPUDevice,
    reason: i32,
    message: WGPUStringView,
    userdata1: *mut c_void,
    _userdata2: *mut c_void,
) {
    // SAFETY: callback pointers and views remain valid for this callback.
    runtime::callback_guard(|| unsafe {
        runtime::record_device_lost(
            userdata1 as usize,
            reason,
            copy_string_view(message),
        );
    });
}

// SAFETY: the callback signature matches the pinned webgpu.h declaration.
unsafe extern "C" fn uncaptured_error_callback(
    _device: *const WGPUDevice,
    error_type: i32,
    message: WGPUStringView,
    userdata1: *mut c_void,
    _userdata2: *mut c_void,
) {
    // SAFETY: callback pointers and views remain valid for this callback.
    runtime::callback_guard(|| unsafe {
        runtime::enqueue_uncaptured_error(
            userdata1 as usize,
            error_type,
            copy_string_view(message),
        );
    });
}

// SAFETY: the callback signature matches the pinned webgpu.h declaration.
unsafe extern "C" fn pop_error_scope_callback(
    status: i32,
    error_type: i32,
    message: WGPUStringView,
    userdata1: *mut c_void,
    _userdata2: *mut c_void,
) {
    // SAFETY: callback pointers and views remain valid for this callback.
    runtime::callback_guard(|| unsafe {
        runtime::complete_record_from_callback(
            userdata1,
            SLOT_KIND_POP_ERROR_SCOPE,
            status == WGPUPopErrorScopeStatus_Success,
            status,
            error_type,
            copy_string_view(message),
        );
    });
}

fn release_owned_handle(handle: runtime::OwnedHandle) {
    match handle.kind {
        SLOT_KIND_REQUEST_ADAPTER => {
            // SAFETY: the owned handle matches this slot kind and is released once.
            unsafe { wgpuAdapterRelease(handle.value as WGPUAdapter) };
            runtime::note_owned_handle_release();
        }
        SLOT_KIND_REQUEST_DEVICE => {
            // SAFETY: the owned handle matches this slot kind and is released once.
            unsafe { wgpuDeviceRelease(handle.value as WGPUDevice) };
            runtime::release_device_events(handle.value);
            runtime::note_owned_handle_release();
        }
        SLOT_KIND_CREATE_COMPUTE_PIPELINE_ASYNC => {
            // SAFETY: the owned handle matches this slot kind and is released once.
            unsafe { wgpuComputePipelineRelease(handle.value as WGPUComputePipeline) };
            runtime::note_owned_handle_release();
        }
        SLOT_KIND_CREATE_RENDER_PIPELINE_ASYNC => {
            // SAFETY: the owned handle matches this slot kind and is released once.
            unsafe { wgpuRenderPipelineRelease(handle.value as WGPURenderPipeline) };
            runtime::note_owned_handle_release();
        }
        _ => {}
    }
}

fn release_deferred_handles() {
    for handle in runtime::drain_deferred_handles() {
        release_owned_handle(handle);
    }
}

/// `subscript-typegpu.h`: Creates an instance with no descriptor.
#[no_mangle]
pub extern "C" fn subscript_typegpu_create_instance() -> SubscriptTypegpuInstance {
    let requested_backend = std::env::var_os("SUBSCRIPT_TYPEGPU_BACKEND");
    let backend = match requested_backend.as_deref().and_then(std::ffi::OsStr::to_str) {
        None if requested_backend.is_none() => None,
        Some("metal") => Some(("metal", YAWGPU_INSTANCE_BACKEND_METAL)),
        Some("vulkan") => Some(("vulkan", YAWGPU_INSTANCE_BACKEND_VULKAN)),
        Some("gles") => Some(("gles", YAWGPU_INSTANCE_BACKEND_GLES)),
        _ => {
            let value = requested_backend.as_deref().map_or_else(
                || "<non-UTF-8>".into(),
                |value| value.to_string_lossy(),
            );
            eprintln!("subscript-typegpu: unknown SUBSCRIPT_TYPEGPU_BACKEND value `{value}`; expected metal, vulkan, or gles");
            return std::ptr::null_mut();
        }
    };
    if !runtime::initialize_table() {
        return std::ptr::null_mut();
    }
    let mut select = backend.map(|(_, backend)| YawgpuInstanceBackendSelect {
        chain: YawgpuChainedStruct {
            next: std::ptr::null_mut(),
            s_type: YAWGPU_STYPE_INSTANCE_BACKEND_SELECT,
        },
        backend,
    });
    let descriptor = select.as_mut().map(|select| WGPUInstanceDescriptor {
        next_in_chain: &mut select.chain,
        required_feature_count: 0,
        required_features: std::ptr::null(),
        required_limits: std::ptr::null(),
    });
    let descriptor = descriptor.as_ref().map_or(std::ptr::null(), |value| value);
    // SAFETY: the optional descriptor and chain live through the backend call.
    let instance: SubscriptTypegpuInstance = unsafe { wgpuCreateInstance(descriptor).cast() };
    if instance.is_null() {
        if let Some((request, _)) = backend {
            let path = std::env::var_os("SUBSCRIPT_TYPEGPU_BACKEND_LIB")
                .map(std::path::PathBuf::from)
                .map_or_else(|| "<unset>".into(), |path| path.display().to_string());
            eprintln!("subscript-typegpu: backend request `{request}` returned a null instance from {path}");
        }
        return std::ptr::null_mut();
    }
    runtime::register_instance(instance as usize);
    instance
}

/// `subscript-typegpu.h`: forwards to `wgpuInstanceProcessEvents`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_instance_process_events(instance: SubscriptTypegpuInstance) {
    if instance.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuInstanceProcessEvents(instance.cast()) }
    release_deferred_handles();
}

/// `subscript-typegpu.h`: releases the instance and every remaining future slot.
#[no_mangle]
pub extern "C" fn subscript_typegpu_instance_release(instance: SubscriptTypegpuInstance) {
    if instance.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuInstanceRelease(instance.cast()) }
    for handle in runtime::release_all_slots(instance as usize) {
        release_owned_handle(handle);
    }
}

/// `subscript-typegpu.h`: begins the `wgpuInstanceRequestAdapter` request; poll after pumping.
#[no_mangle]
pub extern "C" fn subscript_typegpu_instance_request_adapter(
    instance: SubscriptTypegpuInstance,
) -> SubscriptTypegpuFutureId {
    if instance.is_null() {
        return 0;
    }
    let (id, userdata1) = runtime::new_pending_slot(instance as usize, SLOT_KIND_REQUEST_ADAPTER);
    let info = WGPURequestAdapterCallbackInfo {
        next_in_chain: std::ptr::null_mut(),
        mode: WGPUCallbackMode_AllowProcessEvents,
        callback: Some(request_adapter_callback),
        userdata1,
        userdata2: std::ptr::null_mut(),
    };
    // SAFETY: non-null receiver; NULL options is allowed and
    // callback userdata remains live until completion or release.
    let _ = unsafe { wgpuInstanceRequestAdapter(instance.cast(), std::ptr::null(), info) };
    id
}

/// `subscript-typegpu.h`: 0 pending / 1 success / negative failure / -100 unknown.
#[no_mangle]
pub extern "C" fn subscript_typegpu_future_status(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId) -> i32 {
    runtime::future_status(instance as usize, future)
}

/// `subscript-typegpu.h`: drops a future slot; pending slots become doomed.
#[no_mangle]
pub extern "C" fn subscript_typegpu_future_drop(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId) {
    if let Some(handle) = runtime::drop_future(instance as usize, future) {
        release_owned_handle(handle);
    }
}

/// `subscript-typegpu.h`: takes the adapter once and frees its slot.
#[no_mangle]
pub extern "C" fn subscript_typegpu_request_adapter_take(
    instance: SubscriptTypegpuInstance,
    future: SubscriptTypegpuFutureId,
) -> SubscriptTypegpuAdapter {
    runtime::take_handle(instance as usize, future, SLOT_KIND_REQUEST_ADAPTER) as SubscriptTypegpuAdapter
}

/// `subscript-typegpu.h`: fills backend-reported limits and returns status verbatim.
#[no_mangle]
pub extern "C" fn subscript_typegpu_adapter_get_limits(adapter: SubscriptTypegpuAdapter, out: *mut SubscriptTypegpuLimits) -> i32 {
    if adapter.is_null() {
        return 0;
    }
    if out.is_null() {
        return 0;
    }
    // SAFETY: this scalar-only out struct admits all-zero initialization.
    let mut backend: WGPULimits = unsafe { std::mem::zeroed() };
    // SAFETY: the optional receiver is non-null and `backend` is writable.
    let status = unsafe { wgpuAdapterGetLimits(adapter.cast(), &mut backend) };
    // SAFETY: `out` was checked and every field is copied verbatim.
    unsafe {
        out.write(SubscriptTypegpuLimits {
            max_texture_dimension_1D: backend.max_texture_dimension_1D,
            max_texture_dimension_2D: backend.max_texture_dimension_2D,
            max_texture_dimension_3D: backend.max_texture_dimension_3D,
            max_texture_array_layers: backend.max_texture_array_layers,
            max_bind_groups: backend.max_bind_groups,
            max_bind_groups_plus_vertex_buffers: backend.max_bind_groups_plus_vertex_buffers,
            max_bindings_per_bind_group: backend.max_bindings_per_bind_group,
            max_dynamic_uniform_buffers_per_pipeline_layout: backend.max_dynamic_uniform_buffers_per_pipeline_layout,
            max_dynamic_storage_buffers_per_pipeline_layout: backend.max_dynamic_storage_buffers_per_pipeline_layout,
            max_sampled_textures_per_shader_stage: backend.max_sampled_textures_per_shader_stage,
            max_samplers_per_shader_stage: backend.max_samplers_per_shader_stage,
            max_storage_buffers_per_shader_stage: backend.max_storage_buffers_per_shader_stage,
            max_storage_textures_per_shader_stage: backend.max_storage_textures_per_shader_stage,
            max_uniform_buffers_per_shader_stage: backend.max_uniform_buffers_per_shader_stage,
            max_uniform_buffer_binding_size: backend.max_uniform_buffer_binding_size,
            max_storage_buffer_binding_size: backend.max_storage_buffer_binding_size,
            min_uniform_buffer_offset_alignment: backend.min_uniform_buffer_offset_alignment,
            min_storage_buffer_offset_alignment: backend.min_storage_buffer_offset_alignment,
            max_vertex_buffers: backend.max_vertex_buffers,
            max_buffer_size: backend.max_buffer_size,
            max_vertex_attributes: backend.max_vertex_attributes,
            max_vertex_buffer_array_stride: backend.max_vertex_buffer_array_stride,
            max_inter_stage_shader_variables: backend.max_inter_stage_shader_variables,
            max_color_attachments: backend.max_color_attachments,
            max_color_attachment_bytes_per_sample: backend.max_color_attachment_bytes_per_sample,
            max_compute_workgroup_storage_size: backend.max_compute_workgroup_storage_size,
            max_compute_invocations_per_workgroup: backend.max_compute_invocations_per_workgroup,
            max_compute_workgroup_size_x: backend.max_compute_workgroup_size_x,
            max_compute_workgroup_size_y: backend.max_compute_workgroup_size_y,
            max_compute_workgroup_size_z: backend.max_compute_workgroup_size_z,
            max_compute_workgroups_per_dimension: backend.max_compute_workgroups_per_dimension,
            max_immediate_size: backend.max_immediate_size,
        });
    }
    status
}

/// `subscript-typegpu.h`: fills copied adapter information and frees backend members.
#[no_mangle]
pub extern "C" fn subscript_typegpu_adapter_get_info(
    adapter: SubscriptTypegpuAdapter,
    out: *mut SubscriptTypegpuAdapterInfo,
) -> bool {
    if adapter.is_null() || out.is_null() {
        return false;
    }
    // SAFETY: WGPUAdapterInfo's initial state is its all-zero initializer.
    let mut info: WGPUAdapterInfo = unsafe { std::mem::zeroed() };
    // SAFETY: receiver is non-null and `info` is writable.
    let status = unsafe { wgpuAdapterGetInfo(adapter.cast(), &mut info) };
    let strings = runtime::store_adapter_info_strings(
        adapter as usize,
        [
            // SAFETY: backend string views remain valid until free-members runs.
            unsafe { copy_string_view(info.vendor) },
            // SAFETY: backend string views remain valid until free-members runs.
            unsafe { copy_string_view(info.architecture) },
            // SAFETY: backend string views remain valid until free-members runs.
            unsafe { copy_string_view(info.device) },
            // SAFETY: backend string views remain valid until free-members runs.
            unsafe { copy_string_view(info.description) },
        ],
    );
    let result = SubscriptTypegpuAdapterInfo {
        vendor: SubscriptTypegpuStringView { data: strings[0].data as *const c_char, length: strings[0].length },
        architecture: SubscriptTypegpuStringView { data: strings[1].data as *const c_char, length: strings[1].length },
        device: SubscriptTypegpuStringView { data: strings[2].data as *const c_char, length: strings[2].length },
        description: SubscriptTypegpuStringView { data: strings[3].data as *const c_char, length: strings[3].length },
        backend_type: info.backend_type,
        adapter_type: info.adapter_type,
        vendor_id: info.vendor_id,
        device_id: info.device_id,
    };
    // SAFETY: ownership of every backend output string is returned once.
    unsafe { wgpuAdapterInfoFreeMembers(info) };
    // SAFETY: `out` was checked above.
    unsafe { out.write(result) };
    status == WGPUStatus_Success
}

/// `subscript-typegpu.h`: reports whether one pinned feature enum is present.
#[no_mangle]
pub extern "C" fn subscript_typegpu_adapter_has_feature(adapter: SubscriptTypegpuAdapter, feature: i32) -> bool {
    if adapter.is_null() {
        return false;
    }
    // SAFETY: optional receiver is non-null and the enum is passed verbatim.
    unsafe { wgpuAdapterHasFeature(adapter.cast(), feature) != 0 }
}

/// `subscript-typegpu.h`: begins the `wgpuAdapterRequestDevice` request; poll after pumping.
#[no_mangle]
pub extern "C" fn subscript_typegpu_adapter_request_device_with_descriptor(
    instance: SubscriptTypegpuInstance,
    adapter: SubscriptTypegpuAdapter,
    descriptor: *const SubscriptTypegpuDeviceDescriptor,
) -> SubscriptTypegpuFutureId {
    if adapter.is_null() {
        return 0;
    }
    let (id, userdata1) = runtime::new_pending_slot(instance as usize, SLOT_KIND_REQUEST_DEVICE);
    let event_id = runtime::new_device_event_slot();
    runtime::attach_device_event_to_future(id, event_id);
    let empty_view = WGPUStringView {
        data: std::ptr::null(),
        length: 0,
    };
    let public_descriptor = if descriptor.is_null() {
        None
    } else {
        // SAFETY: a non-null descriptor is readable for this call.
        Some(unsafe { *descriptor })
    };
    let required_limits = public_descriptor.as_ref().and_then(|source| {
        if source.required_limits.is_null() {
            None
        } else {
            // SAFETY: the nested limits pointer is readable for this call.
            Some(convert_limits(unsafe { *source.required_limits }))
        }
    });
    let default_queue = public_descriptor.as_ref().map_or(
        WGPUQueueDescriptor {
            next_in_chain: std::ptr::null_mut(),
            label: empty_view,
        },
        |source| convert_queue_descriptor(source.default_queue),
    );
    let descriptor = WGPUDeviceDescriptor {
        next_in_chain: std::ptr::null_mut(),
        label: public_descriptor.as_ref().map_or(
            empty_view, |source| wgpu_string_view(source.label),
        ),
        required_feature_count: public_descriptor.as_ref().map_or(
            0, |source| source.required_features_count,
        ),
        required_features: public_descriptor.as_ref().map_or(
            std::ptr::null(), |source| source.required_features,
        ),
        required_limits: required_limits.as_ref().map_or(
            std::ptr::null(), |limits| limits as *const _,
        ),
        default_queue,
        device_lost_callback_info: WGPUDeviceLostCallbackInfo {
            next_in_chain: std::ptr::null_mut(),
            mode: WGPUCallbackMode_AllowProcessEvents,
            callback: Some(device_lost_callback),
            userdata1: event_id as *mut c_void,
            userdata2: std::ptr::null_mut(),
        },
        uncaptured_error_callback_info: WGPUUncapturedErrorCallbackInfo {
            next_in_chain: std::ptr::null_mut(),
            callback: Some(uncaptured_error_callback),
            userdata1: event_id as *mut c_void,
            userdata2: std::ptr::null_mut(),
        },
    };
    let info = WGPURequestDeviceCallbackInfo {
        next_in_chain: std::ptr::null_mut(),
        mode: WGPUCallbackMode_AllowProcessEvents,
        callback: Some(request_device_callback),
        userdata1,
        userdata2: event_id as *mut c_void,
    };
    // SAFETY: non-null receiver; NULL descriptor is allowed and
    // callback userdata remains live until completion or release.
    let _ = unsafe { wgpuAdapterRequestDevice(adapter.cast(), &descriptor, info) };
    id
}

/// `subscript-typegpu.h`: takes the device once and frees its slot.
#[no_mangle]
pub extern "C" fn subscript_typegpu_request_device_take(
    instance: SubscriptTypegpuInstance,
    future: SubscriptTypegpuFutureId,
) -> SubscriptTypegpuDevice {
    runtime::take_handle(instance as usize, future, SLOT_KIND_REQUEST_DEVICE) as SubscriptTypegpuDevice
}

/// `subscript-typegpu.h`: forwards to `wgpuDeviceGetQueue`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_get_queue(device: SubscriptTypegpuDevice) -> SubscriptTypegpuQueue {
    if device.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuDeviceGetQueue(device.cast()).cast() }
}

/// `subscript-typegpu.h`: forwards to `wgpuDeviceDestroy`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_destroy(device: SubscriptTypegpuDevice) {
    if device.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuDeviceDestroy(device.cast()) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_set_label(device: SubscriptTypegpuDevice, label: SubscriptTypegpuStringView) {
    if device.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuDeviceSetLabel(device.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: forwards to `wgpuDevicePushErrorScope`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_push_error_scope(device: SubscriptTypegpuDevice, filter: i32) {
    if device.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuDevicePushErrorScope(device.cast(), filter) }
}

/// `subscript-typegpu.h`: pops the current error scope into an F6 future.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_pop_error_scope(device: SubscriptTypegpuDevice) -> SubscriptTypegpuFutureId {
    if device.is_null() {
        return 0;
    }
    let instance = runtime::instance_for_handle(device as usize);
    if instance == 0 {
        return 0;
    }
    let (id, userdata1) = runtime::new_pending_slot(instance, SLOT_KIND_POP_ERROR_SCOPE);
    let info = WGPUPopErrorScopeCallbackInfo {
        next_in_chain: std::ptr::null_mut(),
        mode: WGPUCallbackMode_AllowProcessEvents,
        callback: Some(pop_error_scope_callback),
        userdata1,
        userdata2: std::ptr::null_mut(),
    };
    // SAFETY: the device is non-null and callback userdata remains live.
    let _ = unsafe { wgpuDevicePopErrorScope(device.cast(), info) };
    id
}

/// `subscript-typegpu.h`: consumes a successful pop future and fills `out`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_pop_error_scope_take(
    instance: SubscriptTypegpuInstance,
    future: SubscriptTypegpuFutureId,
    out: *mut SubscriptTypegpuErrorRecord,
) -> bool {
    if out.is_null() {
        return false;
    }
    let Some(record) = runtime::take_record(
        instance as usize, future, SLOT_KIND_POP_ERROR_SCOPE,
    ) else {
        return false;
    };
    // SAFETY: out is non-null and writable for one record.
    unsafe {
        out.write(SubscriptTypegpuErrorRecord {
            r#type: record.value,
            message: record_string_view(record),
        });
    }
    true
}

fn record_string_view(record: runtime::RecordFill) -> SubscriptTypegpuStringView {
    SubscriptTypegpuStringView {
        data: record.data as *const c_char,
        length: record.length,
    }
}

/// `subscript-typegpu.h`: drains the next uncaptured error in FIFO order.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_next_uncaptured_error(
    device: SubscriptTypegpuDevice,
    out: *mut SubscriptTypegpuErrorRecord,
) -> bool {
    if device.is_null() || out.is_null() {
        return false;
    }
    let Some(record) = runtime::next_uncaptured_error(device as usize) else {
        return false;
    };
    // SAFETY: out is non-null and writable for one record.
    unsafe {
        out.write(SubscriptTypegpuErrorRecord {
            r#type: record.value,
            message: record_string_view(record),
        });
    }
    true
}

/// `subscript-typegpu.h`: fills the recorded device-lost information when present.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_lost_info(
    device: SubscriptTypegpuDevice,
    out: *mut SubscriptTypegpuLostRecord,
) -> bool {
    if device.is_null() || out.is_null() {
        return false;
    }
    let Some(record) = runtime::device_lost_info(device as usize) else {
        return false;
    };
    // SAFETY: out is non-null and writable for one record.
    unsafe {
        out.write(SubscriptTypegpuLostRecord {
            reason: record.value,
            message: record_string_view(record),
        });
    }
    true
}

/// Facade-test injection for the F11 string-byte lifetime rule.
#[doc(hidden)]
pub fn subscript_typegpu_internal_enqueue_uncaptured_error_for_test(
    device: SubscriptTypegpuDevice,
    error_type: i32,
    message: &str,
) -> bool {
    runtime::enqueue_uncaptured_for_device(
        device as usize, error_type, message.to_owned(),
    )
}

/// `subscript-typegpu.h`: fills backend-reported limits and returns status verbatim.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_get_limits(device: SubscriptTypegpuDevice, out: *mut SubscriptTypegpuLimits) -> i32 {
    if device.is_null() {
        return 0;
    }
    if out.is_null() {
        return 0;
    }
    // SAFETY: this scalar-only out struct admits all-zero initialization.
    let mut backend: WGPULimits = unsafe { std::mem::zeroed() };
    // SAFETY: the optional receiver is non-null and `backend` is writable.
    let status = unsafe { wgpuDeviceGetLimits(device.cast(), &mut backend) };
    // SAFETY: `out` was checked and every field is copied verbatim.
    unsafe {
        out.write(SubscriptTypegpuLimits {
            max_texture_dimension_1D: backend.max_texture_dimension_1D,
            max_texture_dimension_2D: backend.max_texture_dimension_2D,
            max_texture_dimension_3D: backend.max_texture_dimension_3D,
            max_texture_array_layers: backend.max_texture_array_layers,
            max_bind_groups: backend.max_bind_groups,
            max_bind_groups_plus_vertex_buffers: backend.max_bind_groups_plus_vertex_buffers,
            max_bindings_per_bind_group: backend.max_bindings_per_bind_group,
            max_dynamic_uniform_buffers_per_pipeline_layout: backend.max_dynamic_uniform_buffers_per_pipeline_layout,
            max_dynamic_storage_buffers_per_pipeline_layout: backend.max_dynamic_storage_buffers_per_pipeline_layout,
            max_sampled_textures_per_shader_stage: backend.max_sampled_textures_per_shader_stage,
            max_samplers_per_shader_stage: backend.max_samplers_per_shader_stage,
            max_storage_buffers_per_shader_stage: backend.max_storage_buffers_per_shader_stage,
            max_storage_textures_per_shader_stage: backend.max_storage_textures_per_shader_stage,
            max_uniform_buffers_per_shader_stage: backend.max_uniform_buffers_per_shader_stage,
            max_uniform_buffer_binding_size: backend.max_uniform_buffer_binding_size,
            max_storage_buffer_binding_size: backend.max_storage_buffer_binding_size,
            min_uniform_buffer_offset_alignment: backend.min_uniform_buffer_offset_alignment,
            min_storage_buffer_offset_alignment: backend.min_storage_buffer_offset_alignment,
            max_vertex_buffers: backend.max_vertex_buffers,
            max_buffer_size: backend.max_buffer_size,
            max_vertex_attributes: backend.max_vertex_attributes,
            max_vertex_buffer_array_stride: backend.max_vertex_buffer_array_stride,
            max_inter_stage_shader_variables: backend.max_inter_stage_shader_variables,
            max_color_attachments: backend.max_color_attachments,
            max_color_attachment_bytes_per_sample: backend.max_color_attachment_bytes_per_sample,
            max_compute_workgroup_storage_size: backend.max_compute_workgroup_storage_size,
            max_compute_invocations_per_workgroup: backend.max_compute_invocations_per_workgroup,
            max_compute_workgroup_size_x: backend.max_compute_workgroup_size_x,
            max_compute_workgroup_size_y: backend.max_compute_workgroup_size_y,
            max_compute_workgroup_size_z: backend.max_compute_workgroup_size_z,
            max_compute_workgroups_per_dimension: backend.max_compute_workgroups_per_dimension,
            max_immediate_size: backend.max_immediate_size,
        });
    }
    status
}

/// `subscript-typegpu.h`: fills copied adapter information and frees backend members.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_get_adapter_info(
    device: SubscriptTypegpuDevice,
    out: *mut SubscriptTypegpuAdapterInfo,
) -> bool {
    if device.is_null() || out.is_null() {
        return false;
    }
    // SAFETY: WGPUAdapterInfo's initial state is its all-zero initializer.
    let mut info: WGPUAdapterInfo = unsafe { std::mem::zeroed() };
    // SAFETY: receiver is non-null and `info` is writable.
    let status = unsafe { wgpuDeviceGetAdapterInfo(device.cast(), &mut info) };
    let strings = runtime::store_adapter_info_strings(
        device as usize,
        [
            // SAFETY: backend string views remain valid until free-members runs.
            unsafe { copy_string_view(info.vendor) },
            // SAFETY: backend string views remain valid until free-members runs.
            unsafe { copy_string_view(info.architecture) },
            // SAFETY: backend string views remain valid until free-members runs.
            unsafe { copy_string_view(info.device) },
            // SAFETY: backend string views remain valid until free-members runs.
            unsafe { copy_string_view(info.description) },
        ],
    );
    let result = SubscriptTypegpuAdapterInfo {
        vendor: SubscriptTypegpuStringView { data: strings[0].data as *const c_char, length: strings[0].length },
        architecture: SubscriptTypegpuStringView { data: strings[1].data as *const c_char, length: strings[1].length },
        device: SubscriptTypegpuStringView { data: strings[2].data as *const c_char, length: strings[2].length },
        description: SubscriptTypegpuStringView { data: strings[3].data as *const c_char, length: strings[3].length },
        backend_type: info.backend_type,
        adapter_type: info.adapter_type,
        vendor_id: info.vendor_id,
        device_id: info.device_id,
    };
    // SAFETY: ownership of every backend output string is returned once.
    unsafe { wgpuAdapterInfoFreeMembers(info) };
    // SAFETY: `out` was checked above.
    unsafe { out.write(result) };
    status == WGPUStatus_Success
}

/// `subscript-typegpu.h`: reports whether one pinned feature enum is present.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_has_feature(device: SubscriptTypegpuDevice, feature: i32) -> bool {
    if device.is_null() {
        return false;
    }
    // SAFETY: optional receiver is non-null and the enum is passed verbatim.
    unsafe { wgpuDeviceHasFeature(device.cast(), feature) != 0 }
}

/// `subscript-typegpu.h`: creates an object from a chain-free descriptor.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_create_buffer(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuBufferDescriptor,
) -> SubscriptTypegpuBuffer {
    if device.is_null() {
        return std::ptr::null_mut();
    }
    if descriptor.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let descriptor = convert_buffer_descriptor(source);
    // SAFETY: the receiver is non-null and the converted descriptor
    // outlives the backend call.
    let created = unsafe { wgpuDeviceCreateBuffer(device.cast(), &descriptor).cast() };
    runtime::inherit_handle_instance(device as usize, created as usize);
    created
}

/// `subscript-typegpu.h`: creates an object from a chain-free descriptor.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_create_texture(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuTextureDescriptor,
) -> SubscriptTypegpuTexture {
    if device.is_null() {
        return std::ptr::null_mut();
    }
    if descriptor.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let descriptor = convert_texture_descriptor(source);
    // SAFETY: the receiver is non-null and the converted descriptor
    // outlives the backend call.
    let created = unsafe { wgpuDeviceCreateTexture(device.cast(), &descriptor).cast() };
    runtime::inherit_handle_instance(device as usize, created as usize);
    created
}

/// `subscript-typegpu.h`: creates an object from a chain-free descriptor.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_create_sampler(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuSamplerDescriptor,
) -> SubscriptTypegpuSampler {
    if device.is_null() {
        return std::ptr::null_mut();
    }
    if descriptor.is_null() {
        // SAFETY: webgpu.yml marks this descriptor optional.
        let created = unsafe { wgpuDeviceCreateSampler(device.cast(), std::ptr::null()).cast() };
        runtime::inherit_handle_instance(device as usize, created as usize);
        return created;
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let descriptor = convert_sampler_descriptor(source);
    // SAFETY: the receiver is non-null and the converted descriptor
    // outlives the backend call.
    let created = unsafe { wgpuDeviceCreateSampler(device.cast(), &descriptor).cast() };
    runtime::inherit_handle_instance(device as usize, created as usize);
    created
}

/// `subscript-typegpu.h`: creates an object from a chain-free descriptor.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_create_bind_group_layout(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuBindGroupLayoutDescriptor,
) -> SubscriptTypegpuBindGroupLayout {
    if device.is_null() {
        return std::ptr::null_mut();
    }
    if descriptor.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let descriptor = convert_bind_group_layout_descriptor(source);
    // SAFETY: the receiver is non-null and the converted descriptor
    // outlives the backend call.
    let created = unsafe { wgpuDeviceCreateBindGroupLayout(device.cast(), &descriptor.value).cast() };
    runtime::inherit_handle_instance(device as usize, created as usize);
    created
}

/// `subscript-typegpu.h`: creates an object from a chain-free descriptor.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_create_bind_group(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuBindGroupDescriptor,
) -> SubscriptTypegpuBindGroup {
    if device.is_null() {
        return std::ptr::null_mut();
    }
    if descriptor.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let descriptor = convert_bind_group_descriptor(source);
    // SAFETY: the receiver is non-null and the converted descriptor
    // outlives the backend call.
    let created = unsafe { wgpuDeviceCreateBindGroup(device.cast(), &descriptor.value).cast() };
    runtime::inherit_handle_instance(device as usize, created as usize);
    created
}

/// `subscript-typegpu.h`: creates an object from a chain-free descriptor.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_create_pipeline_layout(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuPipelineLayoutDescriptor,
) -> SubscriptTypegpuPipelineLayout {
    if device.is_null() {
        return std::ptr::null_mut();
    }
    if descriptor.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let descriptor = convert_pipeline_layout_descriptor(source);
    // SAFETY: the receiver is non-null and the converted descriptor
    // outlives the backend call.
    let created = unsafe { wgpuDeviceCreatePipelineLayout(device.cast(), &descriptor).cast() };
    runtime::inherit_handle_instance(device as usize, created as usize);
    created
}

/// `subscript-typegpu.h`: creates a WGSL shader module through a private source chain.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_create_shader_module(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuShaderModuleDescriptor,
) -> SubscriptTypegpuShaderModule {
    if device.is_null() || descriptor.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let wgsl = WGPUShaderSourceWGSL {
        chain: WGPUChainedStruct {
            next: std::ptr::null_mut(),
            s_type: WGPUSType_ShaderSourceWGSL,
        },
        code: wgpu_string_view(source.code),
    };
    let descriptor = WGPUShaderModuleDescriptor {
        next_in_chain: (&wgsl.chain as *const WGPUChainedStruct).cast_mut(),
        label: wgpu_string_view(source.label),
    };
    // SAFETY: receiver and descriptor are non-null; the WGSL chain lives
    // through the backend call.
    let created = unsafe { wgpuDeviceCreateShaderModule(device.cast(), &descriptor).cast() };
    runtime::inherit_handle_instance(device as usize, created as usize);
    created
}

/// `subscript-typegpu.h`: creates an object from a chain-free descriptor.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_create_compute_pipeline(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuComputePipelineDescriptor,
) -> SubscriptTypegpuComputePipeline {
    if device.is_null() {
        return std::ptr::null_mut();
    }
    if descriptor.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let descriptor = convert_compute_pipeline_descriptor(source);
    // SAFETY: the receiver is non-null and the converted descriptor
    // outlives the backend call.
    let created = unsafe { wgpuDeviceCreateComputePipeline(device.cast(), &descriptor.value).cast() };
    runtime::inherit_handle_instance(device as usize, created as usize);
    created
}

/// `subscript-typegpu.h`: begins descriptor-backed `wgpuDeviceCreateComputePipelineAsync`; poll after pumping.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_create_compute_pipeline_async_begin(
    instance: SubscriptTypegpuInstance,
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuComputePipelineDescriptor,
) -> SubscriptTypegpuFutureId {
    if instance.is_null() || device.is_null() || descriptor.is_null() {
        return 0;
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let descriptor = convert_compute_pipeline_descriptor(source);
    let (id, userdata1) = runtime::new_pending_slot(instance as usize, SLOT_KIND_CREATE_COMPUTE_PIPELINE_ASYNC);
    let info = WGPUCreateComputePipelineAsyncCallbackInfo {
        next_in_chain: std::ptr::null_mut(),
        mode: WGPUCallbackMode_AllowProcessEvents,
        callback: Some(create_compute_pipeline_async_callback),
        userdata1,
        userdata2: std::ptr::null_mut(),
    };
    // SAFETY: handles and descriptor are non-null, converted storage lives
    // through the backend request call, and callback userdata stays live.
    let _ = unsafe { wgpuDeviceCreateComputePipelineAsync(device.cast(), &descriptor.value, info) };
    id
}

/// `subscript-typegpu.h`: takes the compute pipeline once and frees its slot.
#[no_mangle]
pub extern "C" fn subscript_typegpu_create_compute_pipeline_async_take(
    instance: SubscriptTypegpuInstance,
    future: SubscriptTypegpuFutureId,
) -> SubscriptTypegpuComputePipeline {
    runtime::take_handle(instance as usize, future, SLOT_KIND_CREATE_COMPUTE_PIPELINE_ASYNC) as SubscriptTypegpuComputePipeline
}

/// `subscript-typegpu.h`: creates an object from a chain-free descriptor.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_create_render_pipeline(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuRenderPipelineDescriptor,
) -> SubscriptTypegpuRenderPipeline {
    if device.is_null() {
        return std::ptr::null_mut();
    }
    if descriptor.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let descriptor = convert_render_pipeline_descriptor(source);
    // SAFETY: the receiver is non-null and the converted descriptor
    // outlives the backend call.
    let created = unsafe { wgpuDeviceCreateRenderPipeline(device.cast(), &descriptor.value).cast() };
    runtime::inherit_handle_instance(device as usize, created as usize);
    created
}

/// `subscript-typegpu.h`: begins descriptor-backed `wgpuDeviceCreateRenderPipelineAsync`; poll after pumping.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_create_render_pipeline_async_begin(
    instance: SubscriptTypegpuInstance,
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuRenderPipelineDescriptor,
) -> SubscriptTypegpuFutureId {
    if instance.is_null() || device.is_null() || descriptor.is_null() {
        return 0;
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let descriptor = convert_render_pipeline_descriptor(source);
    let (id, userdata1) = runtime::new_pending_slot(instance as usize, SLOT_KIND_CREATE_RENDER_PIPELINE_ASYNC);
    let info = WGPUCreateRenderPipelineAsyncCallbackInfo {
        next_in_chain: std::ptr::null_mut(),
        mode: WGPUCallbackMode_AllowProcessEvents,
        callback: Some(create_render_pipeline_async_callback),
        userdata1,
        userdata2: std::ptr::null_mut(),
    };
    // SAFETY: handles and descriptor are non-null, converted storage lives
    // through the backend request call, and callback userdata stays live.
    let _ = unsafe { wgpuDeviceCreateRenderPipelineAsync(device.cast(), &descriptor.value, info) };
    id
}

/// `subscript-typegpu.h`: takes the render pipeline once and frees its slot.
#[no_mangle]
pub extern "C" fn subscript_typegpu_create_render_pipeline_async_take(
    instance: SubscriptTypegpuInstance,
    future: SubscriptTypegpuFutureId,
) -> SubscriptTypegpuRenderPipeline {
    runtime::take_handle(instance as usize, future, SLOT_KIND_CREATE_RENDER_PIPELINE_ASYNC) as SubscriptTypegpuRenderPipeline
}

/// `subscript-typegpu.h`: creates an object from a chain-free descriptor.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_create_command_encoder(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuCommandEncoderDescriptor,
) -> SubscriptTypegpuCommandEncoder {
    if device.is_null() {
        return std::ptr::null_mut();
    }
    if descriptor.is_null() {
        // SAFETY: webgpu.yml marks this descriptor optional.
        let created = unsafe { wgpuDeviceCreateCommandEncoder(device.cast(), std::ptr::null()).cast() };
        runtime::inherit_handle_instance(device as usize, created as usize);
        return created;
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let descriptor = convert_command_encoder_descriptor(source);
    // SAFETY: the receiver is non-null and the converted descriptor
    // outlives the backend call.
    let created = unsafe { wgpuDeviceCreateCommandEncoder(device.cast(), &descriptor).cast() };
    runtime::inherit_handle_instance(device as usize, created as usize);
    created
}

/// `subscript-typegpu.h`: creates an object from a chain-free descriptor.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_create_render_bundle_encoder(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuRenderBundleEncoderDescriptor,
) -> SubscriptTypegpuRenderBundleEncoder {
    if device.is_null() {
        return std::ptr::null_mut();
    }
    if descriptor.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let descriptor = convert_render_bundle_encoder_descriptor(source);
    // SAFETY: the receiver is non-null and the converted descriptor
    // outlives the backend call.
    let created = unsafe { wgpuDeviceCreateRenderBundleEncoder(device.cast(), &descriptor).cast() };
    runtime::inherit_handle_instance(device as usize, created as usize);
    created
}

/// `subscript-typegpu.h`: creates an object from a chain-free descriptor.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_create_query_set(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuQuerySetDescriptor,
) -> SubscriptTypegpuQuerySet {
    if device.is_null() {
        return std::ptr::null_mut();
    }
    if descriptor.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let descriptor = convert_query_set_descriptor(source);
    // SAFETY: the receiver is non-null and the converted descriptor
    // outlives the backend call.
    let created = unsafe { wgpuDeviceCreateQuerySet(device.cast(), &descriptor).cast() };
    runtime::inherit_handle_instance(device as usize, created as usize);
    created
}

/// `subscript-typegpu.h`: forwards a count-first input array.
#[no_mangle]
pub extern "C" fn subscript_typegpu_queue_submit(queue: SubscriptTypegpuQueue, commands_count: usize, commands: *const SubscriptTypegpuCommandBuffer) {
    if queue.is_null() {
        return;
    }
    if commands_count != 0 && commands.is_null() {
        return;
    }
    // SAFETY: non-null receiver and the pair promises `count` readable elements.
    unsafe { wgpuQueueSubmit(queue.cast(), commands_count, commands.cast()) }
}

/// `subscript-typegpu.h`: begins the `wgpuQueueOnSubmittedWorkDone` request; poll after pumping.
#[no_mangle]
pub extern "C" fn subscript_typegpu_queue_on_submitted_work_done(
    instance: SubscriptTypegpuInstance,
    queue: SubscriptTypegpuQueue,
) -> SubscriptTypegpuFutureId {
    if queue.is_null() {
        return 0;
    }
    let (id, userdata1) = runtime::new_pending_slot(instance as usize, SLOT_KIND_QUEUE_WORK_DONE);
    let info = WGPUQueueWorkDoneCallbackInfo {
        next_in_chain: std::ptr::null_mut(),
        mode: WGPUCallbackMode_AllowProcessEvents,
        callback: Some(queue_work_done_callback),
        userdata1,
        userdata2: std::ptr::null_mut(),
    };
    // SAFETY: non-null receiver; NULL descriptor is allowed and
    // callback userdata remains live until completion or release.
    let _ = unsafe { wgpuQueueOnSubmittedWorkDone(queue.cast(), info) };
    id
}

/// `subscript-typegpu.h`: forwards a count-first byte array (F20).
#[no_mangle]
pub extern "C" fn subscript_typegpu_queue_write_buffer(
    queue: SubscriptTypegpuQueue,
    buffer: SubscriptTypegpuBuffer,
    bufferOffset: u64,
    dataCount: usize,
    data: *const u8,
) {
    if queue.is_null() || buffer.is_null() || (dataCount != 0 && data.is_null()) {
        return;
    }
    // SAFETY: handles are non-null; a non-empty array has a non-null
    // pointer valid for `dataCount` bytes for this call.
    unsafe {
        wgpuQueueWriteBuffer(
            queue.cast(),
            buffer.cast(),
            bufferOffset,
            data.cast(),
            dataCount,
        )
    }
}

/// `subscript-typegpu.h`: `bufferOffsetBytes` counts bytes; `dataCount` counts f32 elements.
#[no_mangle]
pub extern "C" fn subscript_typegpu_queue_write_buffer_f32(
    queue: SubscriptTypegpuQueue,
    buffer: SubscriptTypegpuBuffer,
    bufferOffsetBytes: u64,
    dataCount: usize,
    data: *const f32,
) {
    if queue.is_null() || buffer.is_null() || (dataCount != 0 && data.is_null()) {
        return;
    }
    let Some(byteCount) = dataCount.checked_mul(std::mem::size_of::<f32>()) else {
        return;
    };
    // SAFETY: handles are non-null; a non-empty array has a non-null
    // pointer valid for `dataCount` f32 elements, or `byteCount` bytes.
    unsafe {
        wgpuQueueWriteBuffer(
            queue.cast(),
            buffer.cast(),
            bufferOffsetBytes,
            data.cast(),
            byteCount,
        )
    }
}

/// `subscript-typegpu.h`: uploads a texture region with a count-first byte array.
#[no_mangle]
pub extern "C" fn subscript_typegpu_queue_write_texture(
    queue: SubscriptTypegpuQueue,
    dst: *const SubscriptTypegpuTexelCopyTextureInfo,
    layout: *const SubscriptTypegpuTexelCopyBufferLayout,
    extent: *const SubscriptTypegpuExtent3D,
    dataCount: usize,
    data: *const u8,
) {
    if queue.is_null() || dst.is_null() || layout.is_null() || extent.is_null()
        || (dataCount != 0 && data.is_null()) {
        return;
    }
    // SAFETY: public pointer checks above establish live input structs.
    let dst = convert_texel_copy_texture_info(unsafe { *dst });
    // SAFETY: as above.
    let layout = convert_texel_copy_buffer_layout(unsafe { *layout });
    // SAFETY: as above.
    let extent = convert_extent_3D(unsafe { *extent });
    // SAFETY: converted structs outlive the call; a non-empty byte
    // array has a non-null pointer valid for `dataCount` bytes.
    unsafe {
        wgpuQueueWriteTexture(
            queue.cast(),
            &dst,
            data.cast(),
            dataCount,
            &layout,
            &extent,
        );
    }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_queue_set_label(queue: SubscriptTypegpuQueue, label: SubscriptTypegpuStringView) {
    if queue.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuQueueSetLabel(queue.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: begins a buffer map request; poll after pumping.
#[no_mangle]
pub extern "C" fn subscript_typegpu_buffer_map_async(
    buffer: SubscriptTypegpuBuffer,
    mode: u64,
    offset: usize,
    size: usize,
) -> SubscriptTypegpuFutureId {
    if buffer.is_null() {
        return 0;
    }
    let instance = runtime::instance_for_handle(buffer as usize);
    if instance == 0 {
        return 0;
    }
    let (id, userdata1) = runtime::new_pending_slot(instance, SLOT_KIND_BUFFER_MAP);
    let info = WGPUBufferMapCallbackInfo {
        next_in_chain: std::ptr::null_mut(),
        mode: WGPUCallbackMode_AllowProcessEvents,
        callback: Some(buffer_map_callback),
        userdata1,
        userdata2: std::ptr::null_mut(),
    };
    // SAFETY: the receiver is non-null; callback userdata stays live
    // until completion or instance release.
    let _ = unsafe { wgpuBufferMapAsync(buffer.cast(), mode, offset, size, info) };
    id
}

/// `subscript-typegpu.h`: forwards a count-first byte array (F20).
#[no_mangle]
pub extern "C" fn subscript_typegpu_buffer_read_mapped_range(
    buffer: SubscriptTypegpuBuffer,
    offset: usize,
    outCount: usize,
    out: *mut u8,
) -> i32 {
    if buffer.is_null() || (outCount != 0 && out.is_null()) {
        return 2;
    }
    // SAFETY: handles are non-null; a non-empty array has a non-null
    // pointer valid for `outCount` bytes for this call.
    unsafe {
        wgpuBufferReadMappedRange(buffer.cast(), offset, out.cast(), outCount)
    }
}

/// `subscript-typegpu.h`: `offsetBytes` counts bytes; `outCount` counts f32 elements.
#[no_mangle]
pub extern "C" fn subscript_typegpu_buffer_read_mapped_range_f32(
    buffer: SubscriptTypegpuBuffer,
    offsetBytes: usize,
    outCount: usize,
    out: *mut f32,
) -> i32 {
    if buffer.is_null() || (outCount != 0 && out.is_null()) {
        return 2;
    }
    let Some(byteCount) = outCount.checked_mul(std::mem::size_of::<f32>()) else {
        return 2;
    };
    // SAFETY: handles are non-null; a non-empty array has a non-null
    // pointer valid for `outCount` f32 elements, or `byteCount` bytes.
    unsafe {
        wgpuBufferReadMappedRange(buffer.cast(), offsetBytes, out.cast(), byteCount)
    }
}

/// `subscript-typegpu.h`: forwards a count-first byte array (F20).
#[no_mangle]
pub extern "C" fn subscript_typegpu_buffer_write_mapped_range(
    buffer: SubscriptTypegpuBuffer,
    offset: usize,
    dataCount: usize,
    data: *const u8,
) -> i32 {
    if buffer.is_null() || (dataCount != 0 && data.is_null()) {
        return 2;
    }
    // SAFETY: handles are non-null; a non-empty array has a non-null
    // pointer valid for `dataCount` bytes for this call.
    unsafe {
        wgpuBufferWriteMappedRange(buffer.cast(), offset, data.cast(), dataCount)
    }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_buffer_set_label(buffer: SubscriptTypegpuBuffer, label: SubscriptTypegpuStringView) {
    if buffer.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuBufferSetLabel(buffer.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: forwards to `wgpuBufferGetUsage`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_buffer_get_usage(buffer: SubscriptTypegpuBuffer) -> u64 {
    if buffer.is_null() {
        return 0;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuBufferGetUsage(buffer.cast()) }
}

/// `subscript-typegpu.h`: forwards to `wgpuBufferGetSize`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_buffer_get_size(buffer: SubscriptTypegpuBuffer) -> u64 {
    if buffer.is_null() {
        return 0;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuBufferGetSize(buffer.cast()) }
}

/// `subscript-typegpu.h`: forwards to `wgpuBufferGetMapState`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_buffer_get_map_state(buffer: SubscriptTypegpuBuffer) -> i32 {
    if buffer.is_null() {
        return 0;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuBufferGetMapState(buffer.cast()) }
}

/// `subscript-typegpu.h`: forwards to `wgpuBufferUnmap`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_buffer_unmap(buffer: SubscriptTypegpuBuffer) {
    if buffer.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuBufferUnmap(buffer.cast()) }
}

/// `subscript-typegpu.h`: forwards to `wgpuBufferDestroy`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_buffer_destroy(buffer: SubscriptTypegpuBuffer) {
    if buffer.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuBufferDestroy(buffer.cast()) }
}

/// `subscript-typegpu.h`: creates an object from a chain-free descriptor.
#[no_mangle]
pub extern "C" fn subscript_typegpu_texture_create_view(
    texture: SubscriptTypegpuTexture,
    descriptor: *const SubscriptTypegpuTextureViewDescriptor,
) -> SubscriptTypegpuTextureView {
    if texture.is_null() {
        return std::ptr::null_mut();
    }
    if descriptor.is_null() {
        // SAFETY: webgpu.yml marks this descriptor optional.
        let created = unsafe { wgpuTextureCreateView(texture.cast(), std::ptr::null()).cast() };
        runtime::inherit_handle_instance(texture as usize, created as usize);
        return created;
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let descriptor = convert_texture_view_descriptor(source);
    // SAFETY: the receiver is non-null and the converted descriptor
    // outlives the backend call.
    let created = unsafe { wgpuTextureCreateView(texture.cast(), &descriptor).cast() };
    runtime::inherit_handle_instance(texture as usize, created as usize);
    created
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_texture_set_label(texture: SubscriptTypegpuTexture, label: SubscriptTypegpuStringView) {
    if texture.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuTextureSetLabel(texture.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: forwards to `wgpuTextureGetWidth`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_texture_get_width(texture: SubscriptTypegpuTexture) -> u32 {
    if texture.is_null() {
        return 0;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuTextureGetWidth(texture.cast()) }
}

/// `subscript-typegpu.h`: forwards to `wgpuTextureGetHeight`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_texture_get_height(texture: SubscriptTypegpuTexture) -> u32 {
    if texture.is_null() {
        return 0;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuTextureGetHeight(texture.cast()) }
}

/// `subscript-typegpu.h`: forwards to `wgpuTextureGetDepthOrArrayLayers`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_texture_get_depth_or_array_layers(texture: SubscriptTypegpuTexture) -> u32 {
    if texture.is_null() {
        return 0;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuTextureGetDepthOrArrayLayers(texture.cast()) }
}

/// `subscript-typegpu.h`: forwards to `wgpuTextureGetMipLevelCount`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_texture_get_mip_level_count(texture: SubscriptTypegpuTexture) -> u32 {
    if texture.is_null() {
        return 0;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuTextureGetMipLevelCount(texture.cast()) }
}

/// `subscript-typegpu.h`: forwards to `wgpuTextureGetSampleCount`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_texture_get_sample_count(texture: SubscriptTypegpuTexture) -> u32 {
    if texture.is_null() {
        return 0;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuTextureGetSampleCount(texture.cast()) }
}

/// `subscript-typegpu.h`: forwards to `wgpuTextureGetDimension`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_texture_get_dimension(texture: SubscriptTypegpuTexture) -> i32 {
    if texture.is_null() {
        return 0;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuTextureGetDimension(texture.cast()) }
}

/// `subscript-typegpu.h`: forwards to `wgpuTextureGetFormat`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_texture_get_format(texture: SubscriptTypegpuTexture) -> i32 {
    if texture.is_null() {
        return 0;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuTextureGetFormat(texture.cast()) }
}

/// `subscript-typegpu.h`: forwards to `wgpuTextureGetUsage`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_texture_get_usage(texture: SubscriptTypegpuTexture) -> u64 {
    if texture.is_null() {
        return 0;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuTextureGetUsage(texture.cast()) }
}

/// `subscript-typegpu.h`: forwards to `wgpuTextureDestroy`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_texture_destroy(texture: SubscriptTypegpuTexture) {
    if texture.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuTextureDestroy(texture.cast()) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_texture_view_set_label(textureView: SubscriptTypegpuTextureView, label: SubscriptTypegpuStringView) {
    if textureView.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuTextureViewSetLabel(textureView.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_sampler_set_label(sampler: SubscriptTypegpuSampler, label: SubscriptTypegpuStringView) {
    if sampler.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuSamplerSetLabel(sampler.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_bind_group_layout_set_label(bindGroupLayout: SubscriptTypegpuBindGroupLayout, label: SubscriptTypegpuStringView) {
    if bindGroupLayout.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuBindGroupLayoutSetLabel(bindGroupLayout.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_bind_group_set_label(bindGroup: SubscriptTypegpuBindGroup, label: SubscriptTypegpuStringView) {
    if bindGroup.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuBindGroupSetLabel(bindGroup.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_pipeline_layout_set_label(pipelineLayout: SubscriptTypegpuPipelineLayout, label: SubscriptTypegpuStringView) {
    if pipelineLayout.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuPipelineLayoutSetLabel(pipelineLayout.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_shader_module_set_label(shaderModule: SubscriptTypegpuShaderModule, label: SubscriptTypegpuStringView) {
    if shaderModule.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuShaderModuleSetLabel(shaderModule.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: forwards to `wgpuComputePipelineGetBindGroupLayout`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_compute_pipeline_get_bind_group_layout(computePipeline: SubscriptTypegpuComputePipeline, groupIndex: u32) -> SubscriptTypegpuBindGroupLayout {
    if computePipeline.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuComputePipelineGetBindGroupLayout(computePipeline.cast(), groupIndex).cast() }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_compute_pipeline_set_label(computePipeline: SubscriptTypegpuComputePipeline, label: SubscriptTypegpuStringView) {
    if computePipeline.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuComputePipelineSetLabel(computePipeline.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderPipelineGetBindGroupLayout`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pipeline_get_bind_group_layout(renderPipeline: SubscriptTypegpuRenderPipeline, groupIndex: u32) -> SubscriptTypegpuBindGroupLayout {
    if renderPipeline.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderPipelineGetBindGroupLayout(renderPipeline.cast(), groupIndex).cast() }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pipeline_set_label(renderPipeline: SubscriptTypegpuRenderPipeline, label: SubscriptTypegpuStringView) {
    if renderPipeline.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuRenderPipelineSetLabel(renderPipeline.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: creates an object from a chain-free descriptor.
#[no_mangle]
pub extern "C" fn subscript_typegpu_command_encoder_finish(
    commandEncoder: SubscriptTypegpuCommandEncoder,
    descriptor: *const SubscriptTypegpuCommandBufferDescriptor,
) -> SubscriptTypegpuCommandBuffer {
    if commandEncoder.is_null() {
        return std::ptr::null_mut();
    }
    if descriptor.is_null() {
        // SAFETY: webgpu.yml marks this descriptor optional.
        let created = unsafe { wgpuCommandEncoderFinish(commandEncoder.cast(), std::ptr::null()).cast() };
        runtime::inherit_handle_instance(commandEncoder as usize, created as usize);
        return created;
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let descriptor = convert_command_buffer_descriptor(source);
    // SAFETY: the receiver is non-null and the converted descriptor
    // outlives the backend call.
    let created = unsafe { wgpuCommandEncoderFinish(commandEncoder.cast(), &descriptor).cast() };
    runtime::inherit_handle_instance(commandEncoder as usize, created as usize);
    created
}

/// `subscript-typegpu.h`: creates an object from a chain-free descriptor.
#[no_mangle]
pub extern "C" fn subscript_typegpu_command_encoder_begin_compute_pass(
    commandEncoder: SubscriptTypegpuCommandEncoder,
    descriptor: *const SubscriptTypegpuComputePassDescriptor,
) -> SubscriptTypegpuComputePassEncoder {
    if commandEncoder.is_null() {
        return std::ptr::null_mut();
    }
    if descriptor.is_null() {
        // SAFETY: webgpu.yml marks this descriptor optional.
        let created = unsafe { wgpuCommandEncoderBeginComputePass(commandEncoder.cast(), std::ptr::null()).cast() };
        runtime::inherit_handle_instance(commandEncoder as usize, created as usize);
        return created;
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let descriptor = convert_compute_pass_descriptor(source);
    // SAFETY: the receiver is non-null and the converted descriptor
    // outlives the backend call.
    let created = unsafe { wgpuCommandEncoderBeginComputePass(commandEncoder.cast(), &descriptor.value).cast() };
    runtime::inherit_handle_instance(commandEncoder as usize, created as usize);
    created
}

/// `subscript-typegpu.h`: creates an object from a chain-free descriptor.
#[no_mangle]
pub extern "C" fn subscript_typegpu_command_encoder_begin_render_pass(
    commandEncoder: SubscriptTypegpuCommandEncoder,
    descriptor: *const SubscriptTypegpuRenderPassDescriptor,
) -> SubscriptTypegpuRenderPassEncoder {
    if commandEncoder.is_null() {
        return std::ptr::null_mut();
    }
    if descriptor.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let descriptor = convert_render_pass_descriptor(source);
    // SAFETY: the receiver is non-null and the converted descriptor
    // outlives the backend call.
    let created = unsafe { wgpuCommandEncoderBeginRenderPass(commandEncoder.cast(), &descriptor.value).cast() };
    runtime::inherit_handle_instance(commandEncoder as usize, created as usize);
    created
}

/// `subscript-typegpu.h`: forwards to `wgpuCommandEncoderCopyBufferToBuffer`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_command_encoder_copy_buffer_to_buffer(commandEncoder: SubscriptTypegpuCommandEncoder, source: SubscriptTypegpuBuffer, sourceOffset: u64, destination: SubscriptTypegpuBuffer, destinationOffset: u64, size: u64) {
    if commandEncoder.is_null() {
        return;
    }
    if source.is_null() {
        return;
    }
    if destination.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuCommandEncoderCopyBufferToBuffer(commandEncoder.cast(), source.cast(), sourceOffset, destination.cast(), destinationOffset, size) }
}

/// `subscript-typegpu.h`: forwards to `wgpuCommandEncoderCopyBufferToTexture`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_command_encoder_copy_buffer_to_texture(commandEncoder: SubscriptTypegpuCommandEncoder, source: *const SubscriptTypegpuTexelCopyBufferInfo, destination: *const SubscriptTypegpuTexelCopyTextureInfo, copySize: *const SubscriptTypegpuExtent3D) {
    if commandEncoder.is_null() {
        return;
    }
    if source.is_null() {
        return;
    }
    if destination.is_null() {
        return;
    }
    if copySize.is_null() {
        return;
    }
    // SAFETY: the non-null input pointer is readable for this call.
    let converted_source = convert_texel_copy_buffer_info(unsafe { *source });
    // SAFETY: the non-null input pointer is readable for this call.
    let converted_destination = convert_texel_copy_texture_info(unsafe { *destination });
    // SAFETY: the non-null input pointer is readable for this call.
    let converted_copySize = convert_extent_3D(unsafe { *copySize });
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuCommandEncoderCopyBufferToTexture(commandEncoder.cast(), &converted_source, &converted_destination, &converted_copySize) }
}

/// `subscript-typegpu.h`: forwards to `wgpuCommandEncoderCopyTextureToBuffer`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_command_encoder_copy_texture_to_buffer(commandEncoder: SubscriptTypegpuCommandEncoder, source: *const SubscriptTypegpuTexelCopyTextureInfo, destination: *const SubscriptTypegpuTexelCopyBufferInfo, copySize: *const SubscriptTypegpuExtent3D) {
    if commandEncoder.is_null() {
        return;
    }
    if source.is_null() {
        return;
    }
    if destination.is_null() {
        return;
    }
    if copySize.is_null() {
        return;
    }
    // SAFETY: the non-null input pointer is readable for this call.
    let converted_source = convert_texel_copy_texture_info(unsafe { *source });
    // SAFETY: the non-null input pointer is readable for this call.
    let converted_destination = convert_texel_copy_buffer_info(unsafe { *destination });
    // SAFETY: the non-null input pointer is readable for this call.
    let converted_copySize = convert_extent_3D(unsafe { *copySize });
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuCommandEncoderCopyTextureToBuffer(commandEncoder.cast(), &converted_source, &converted_destination, &converted_copySize) }
}

/// `subscript-typegpu.h`: forwards to `wgpuCommandEncoderCopyTextureToTexture`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_command_encoder_copy_texture_to_texture(commandEncoder: SubscriptTypegpuCommandEncoder, source: *const SubscriptTypegpuTexelCopyTextureInfo, destination: *const SubscriptTypegpuTexelCopyTextureInfo, copySize: *const SubscriptTypegpuExtent3D) {
    if commandEncoder.is_null() {
        return;
    }
    if source.is_null() {
        return;
    }
    if destination.is_null() {
        return;
    }
    if copySize.is_null() {
        return;
    }
    // SAFETY: the non-null input pointer is readable for this call.
    let converted_source = convert_texel_copy_texture_info(unsafe { *source });
    // SAFETY: the non-null input pointer is readable for this call.
    let converted_destination = convert_texel_copy_texture_info(unsafe { *destination });
    // SAFETY: the non-null input pointer is readable for this call.
    let converted_copySize = convert_extent_3D(unsafe { *copySize });
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuCommandEncoderCopyTextureToTexture(commandEncoder.cast(), &converted_source, &converted_destination, &converted_copySize) }
}

/// `subscript-typegpu.h`: forwards to `wgpuCommandEncoderClearBuffer`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_command_encoder_clear_buffer(commandEncoder: SubscriptTypegpuCommandEncoder, buffer: SubscriptTypegpuBuffer, offset: u64, size: u64) {
    if commandEncoder.is_null() {
        return;
    }
    if buffer.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuCommandEncoderClearBuffer(commandEncoder.cast(), buffer.cast(), offset, size) }
}

/// `subscript-typegpu.h`: forwards to `wgpuCommandEncoderResolveQuerySet`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_command_encoder_resolve_query_set(commandEncoder: SubscriptTypegpuCommandEncoder, querySet: SubscriptTypegpuQuerySet, firstQuery: u32, queryCount: u32, destination: SubscriptTypegpuBuffer, destinationOffset: u64) {
    if commandEncoder.is_null() {
        return;
    }
    if querySet.is_null() {
        return;
    }
    if destination.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuCommandEncoderResolveQuerySet(commandEncoder.cast(), querySet.cast(), firstQuery, queryCount, destination.cast(), destinationOffset) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_command_encoder_insert_debug_marker(commandEncoder: SubscriptTypegpuCommandEncoder, markerLabel: SubscriptTypegpuStringView) {
    if commandEncoder.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuCommandEncoderInsertDebugMarker(commandEncoder.cast(), wgpu_string_view(markerLabel)) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_command_encoder_push_debug_group(commandEncoder: SubscriptTypegpuCommandEncoder, groupLabel: SubscriptTypegpuStringView) {
    if commandEncoder.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuCommandEncoderPushDebugGroup(commandEncoder.cast(), wgpu_string_view(groupLabel)) }
}

/// `subscript-typegpu.h`: forwards to `wgpuCommandEncoderPopDebugGroup`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_command_encoder_pop_debug_group(commandEncoder: SubscriptTypegpuCommandEncoder) {
    if commandEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuCommandEncoderPopDebugGroup(commandEncoder.cast()) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_command_encoder_set_label(commandEncoder: SubscriptTypegpuCommandEncoder, label: SubscriptTypegpuStringView) {
    if commandEncoder.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuCommandEncoderSetLabel(commandEncoder.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: forwards to `wgpuComputePassEncoderSetPipeline`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_compute_pass_encoder_set_pipeline(computePassEncoder: SubscriptTypegpuComputePassEncoder, pipeline: SubscriptTypegpuComputePipeline) {
    if computePassEncoder.is_null() {
        return;
    }
    if pipeline.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuComputePassEncoderSetPipeline(computePassEncoder.cast(), pipeline.cast()) }
}

/// `subscript-typegpu.h`: forwards a count-first input array.
#[no_mangle]
pub extern "C" fn subscript_typegpu_compute_pass_encoder_set_bind_group(computePassEncoder: SubscriptTypegpuComputePassEncoder, groupIndex: u32, group: SubscriptTypegpuBindGroup, dynamicOffsets_count: usize, dynamicOffsets: *const u32) {
    if computePassEncoder.is_null() {
        return;
    }
    if dynamicOffsets_count != 0 && dynamicOffsets.is_null() {
        return;
    }
    // SAFETY: non-null receiver and the pair promises `count` readable elements.
    unsafe { wgpuComputePassEncoderSetBindGroup(computePassEncoder.cast(), groupIndex, group.cast(), dynamicOffsets_count, dynamicOffsets) }
}

/// `subscript-typegpu.h`: forwards to `wgpuComputePassEncoderDispatchWorkgroups`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_compute_pass_encoder_dispatch_workgroups(computePassEncoder: SubscriptTypegpuComputePassEncoder, workgroupCountX: u32, workgroupCountY: u32, workgroupCountZ: u32) {
    if computePassEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuComputePassEncoderDispatchWorkgroups(computePassEncoder.cast(), workgroupCountX, workgroupCountY, workgroupCountZ) }
}

/// `subscript-typegpu.h`: forwards to `wgpuComputePassEncoderDispatchWorkgroupsIndirect`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_compute_pass_encoder_dispatch_workgroups_indirect(computePassEncoder: SubscriptTypegpuComputePassEncoder, indirectBuffer: SubscriptTypegpuBuffer, indirectOffset: u64) {
    if computePassEncoder.is_null() {
        return;
    }
    if indirectBuffer.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuComputePassEncoderDispatchWorkgroupsIndirect(computePassEncoder.cast(), indirectBuffer.cast(), indirectOffset) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_compute_pass_encoder_insert_debug_marker(computePassEncoder: SubscriptTypegpuComputePassEncoder, markerLabel: SubscriptTypegpuStringView) {
    if computePassEncoder.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuComputePassEncoderInsertDebugMarker(computePassEncoder.cast(), wgpu_string_view(markerLabel)) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_compute_pass_encoder_push_debug_group(computePassEncoder: SubscriptTypegpuComputePassEncoder, groupLabel: SubscriptTypegpuStringView) {
    if computePassEncoder.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuComputePassEncoderPushDebugGroup(computePassEncoder.cast(), wgpu_string_view(groupLabel)) }
}

/// `subscript-typegpu.h`: forwards to `wgpuComputePassEncoderPopDebugGroup`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_compute_pass_encoder_pop_debug_group(computePassEncoder: SubscriptTypegpuComputePassEncoder) {
    if computePassEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuComputePassEncoderPopDebugGroup(computePassEncoder.cast()) }
}

/// `subscript-typegpu.h`: forwards to `wgpuComputePassEncoderEnd`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_compute_pass_encoder_end(computePassEncoder: SubscriptTypegpuComputePassEncoder) {
    if computePassEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuComputePassEncoderEnd(computePassEncoder.cast()) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_compute_pass_encoder_set_label(computePassEncoder: SubscriptTypegpuComputePassEncoder, label: SubscriptTypegpuStringView) {
    if computePassEncoder.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuComputePassEncoderSetLabel(computePassEncoder.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderPassEncoderSetPipeline`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_set_pipeline(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, pipeline: SubscriptTypegpuRenderPipeline) {
    if renderPassEncoder.is_null() {
        return;
    }
    if pipeline.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderPassEncoderSetPipeline(renderPassEncoder.cast(), pipeline.cast()) }
}

/// `subscript-typegpu.h`: forwards a count-first input array.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_set_bind_group(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, groupIndex: u32, group: SubscriptTypegpuBindGroup, dynamicOffsets_count: usize, dynamicOffsets: *const u32) {
    if renderPassEncoder.is_null() {
        return;
    }
    if dynamicOffsets_count != 0 && dynamicOffsets.is_null() {
        return;
    }
    // SAFETY: non-null receiver and the pair promises `count` readable elements.
    unsafe { wgpuRenderPassEncoderSetBindGroup(renderPassEncoder.cast(), groupIndex, group.cast(), dynamicOffsets_count, dynamicOffsets) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderPassEncoderSetVertexBuffer`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_set_vertex_buffer(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, slot: u32, buffer: SubscriptTypegpuBuffer, offset: u64, size: u64) {
    if renderPassEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderPassEncoderSetVertexBuffer(renderPassEncoder.cast(), slot, buffer.cast(), offset, size) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderPassEncoderSetIndexBuffer`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_set_index_buffer(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, buffer: SubscriptTypegpuBuffer, format: i32, offset: u64, size: u64) {
    if renderPassEncoder.is_null() {
        return;
    }
    if buffer.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderPassEncoderSetIndexBuffer(renderPassEncoder.cast(), buffer.cast(), format, offset, size) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderPassEncoderDraw`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_draw(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32) {
    if renderPassEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderPassEncoderDraw(renderPassEncoder.cast(), vertexCount, instanceCount, firstVertex, firstInstance) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderPassEncoderDrawIndexed`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_draw_indexed(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, indexCount: u32, instanceCount: u32, firstIndex: u32, baseVertex: i32, firstInstance: u32) {
    if renderPassEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderPassEncoderDrawIndexed(renderPassEncoder.cast(), indexCount, instanceCount, firstIndex, baseVertex, firstInstance) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderPassEncoderDrawIndirect`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_draw_indirect(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, indirectBuffer: SubscriptTypegpuBuffer, indirectOffset: u64) {
    if renderPassEncoder.is_null() {
        return;
    }
    if indirectBuffer.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderPassEncoderDrawIndirect(renderPassEncoder.cast(), indirectBuffer.cast(), indirectOffset) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderPassEncoderDrawIndexedIndirect`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_draw_indexed_indirect(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, indirectBuffer: SubscriptTypegpuBuffer, indirectOffset: u64) {
    if renderPassEncoder.is_null() {
        return;
    }
    if indirectBuffer.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderPassEncoderDrawIndexedIndirect(renderPassEncoder.cast(), indirectBuffer.cast(), indirectOffset) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderPassEncoderSetViewport`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_set_viewport(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, x: f32, y: f32, width: f32, height: f32, minDepth: f32, maxDepth: f32) {
    if renderPassEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderPassEncoderSetViewport(renderPassEncoder.cast(), x, y, width, height, minDepth, maxDepth) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderPassEncoderSetScissorRect`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_set_scissor_rect(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, x: u32, y: u32, width: u32, height: u32) {
    if renderPassEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderPassEncoderSetScissorRect(renderPassEncoder.cast(), x, y, width, height) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderPassEncoderSetBlendConstant`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_set_blend_constant(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, color: *const SubscriptTypegpuColor) {
    if renderPassEncoder.is_null() {
        return;
    }
    if color.is_null() {
        return;
    }
    // SAFETY: the non-null input pointer is readable for this call.
    let converted_color = convert_color(unsafe { *color });
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderPassEncoderSetBlendConstant(renderPassEncoder.cast(), &converted_color) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderPassEncoderSetStencilReference`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_set_stencil_reference(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, reference: u32) {
    if renderPassEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderPassEncoderSetStencilReference(renderPassEncoder.cast(), reference) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderPassEncoderBeginOcclusionQuery`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_begin_occlusion_query(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, queryIndex: u32) {
    if renderPassEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderPassEncoderBeginOcclusionQuery(renderPassEncoder.cast(), queryIndex) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderPassEncoderEndOcclusionQuery`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_end_occlusion_query(renderPassEncoder: SubscriptTypegpuRenderPassEncoder) {
    if renderPassEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderPassEncoderEndOcclusionQuery(renderPassEncoder.cast()) }
}

/// `subscript-typegpu.h`: forwards a count-first input array.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_execute_bundles(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, bundles_count: usize, bundles: *const SubscriptTypegpuRenderBundle) {
    if renderPassEncoder.is_null() {
        return;
    }
    if bundles_count != 0 && bundles.is_null() {
        return;
    }
    // SAFETY: non-null receiver and the pair promises `count` readable elements.
    unsafe { wgpuRenderPassEncoderExecuteBundles(renderPassEncoder.cast(), bundles_count, bundles.cast()) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_insert_debug_marker(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, markerLabel: SubscriptTypegpuStringView) {
    if renderPassEncoder.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuRenderPassEncoderInsertDebugMarker(renderPassEncoder.cast(), wgpu_string_view(markerLabel)) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_push_debug_group(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, groupLabel: SubscriptTypegpuStringView) {
    if renderPassEncoder.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuRenderPassEncoderPushDebugGroup(renderPassEncoder.cast(), wgpu_string_view(groupLabel)) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderPassEncoderPopDebugGroup`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_pop_debug_group(renderPassEncoder: SubscriptTypegpuRenderPassEncoder) {
    if renderPassEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderPassEncoderPopDebugGroup(renderPassEncoder.cast()) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderPassEncoderEnd`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_end(renderPassEncoder: SubscriptTypegpuRenderPassEncoder) {
    if renderPassEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderPassEncoderEnd(renderPassEncoder.cast()) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_set_label(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, label: SubscriptTypegpuStringView) {
    if renderPassEncoder.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuRenderPassEncoderSetLabel(renderPassEncoder.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_command_buffer_set_label(commandBuffer: SubscriptTypegpuCommandBuffer, label: SubscriptTypegpuStringView) {
    if commandBuffer.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuCommandBufferSetLabel(commandBuffer.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderBundleEncoderSetPipeline`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_bundle_encoder_set_pipeline(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, pipeline: SubscriptTypegpuRenderPipeline) {
    if renderBundleEncoder.is_null() {
        return;
    }
    if pipeline.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderBundleEncoderSetPipeline(renderBundleEncoder.cast(), pipeline.cast()) }
}

/// `subscript-typegpu.h`: forwards a count-first input array.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_bundle_encoder_set_bind_group(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, groupIndex: u32, group: SubscriptTypegpuBindGroup, dynamicOffsets_count: usize, dynamicOffsets: *const u32) {
    if renderBundleEncoder.is_null() {
        return;
    }
    if dynamicOffsets_count != 0 && dynamicOffsets.is_null() {
        return;
    }
    // SAFETY: non-null receiver and the pair promises `count` readable elements.
    unsafe { wgpuRenderBundleEncoderSetBindGroup(renderBundleEncoder.cast(), groupIndex, group.cast(), dynamicOffsets_count, dynamicOffsets) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderBundleEncoderSetVertexBuffer`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_bundle_encoder_set_vertex_buffer(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, slot: u32, buffer: SubscriptTypegpuBuffer, offset: u64, size: u64) {
    if renderBundleEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderBundleEncoderSetVertexBuffer(renderBundleEncoder.cast(), slot, buffer.cast(), offset, size) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderBundleEncoderSetIndexBuffer`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_bundle_encoder_set_index_buffer(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, buffer: SubscriptTypegpuBuffer, format: i32, offset: u64, size: u64) {
    if renderBundleEncoder.is_null() {
        return;
    }
    if buffer.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderBundleEncoderSetIndexBuffer(renderBundleEncoder.cast(), buffer.cast(), format, offset, size) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderBundleEncoderDraw`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_bundle_encoder_draw(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32) {
    if renderBundleEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderBundleEncoderDraw(renderBundleEncoder.cast(), vertexCount, instanceCount, firstVertex, firstInstance) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderBundleEncoderDrawIndexed`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_bundle_encoder_draw_indexed(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, indexCount: u32, instanceCount: u32, firstIndex: u32, baseVertex: i32, firstInstance: u32) {
    if renderBundleEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderBundleEncoderDrawIndexed(renderBundleEncoder.cast(), indexCount, instanceCount, firstIndex, baseVertex, firstInstance) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderBundleEncoderDrawIndirect`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_bundle_encoder_draw_indirect(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, indirectBuffer: SubscriptTypegpuBuffer, indirectOffset: u64) {
    if renderBundleEncoder.is_null() {
        return;
    }
    if indirectBuffer.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderBundleEncoderDrawIndirect(renderBundleEncoder.cast(), indirectBuffer.cast(), indirectOffset) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderBundleEncoderDrawIndexedIndirect`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_bundle_encoder_draw_indexed_indirect(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, indirectBuffer: SubscriptTypegpuBuffer, indirectOffset: u64) {
    if renderBundleEncoder.is_null() {
        return;
    }
    if indirectBuffer.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderBundleEncoderDrawIndexedIndirect(renderBundleEncoder.cast(), indirectBuffer.cast(), indirectOffset) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_bundle_encoder_insert_debug_marker(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, markerLabel: SubscriptTypegpuStringView) {
    if renderBundleEncoder.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuRenderBundleEncoderInsertDebugMarker(renderBundleEncoder.cast(), wgpu_string_view(markerLabel)) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_bundle_encoder_push_debug_group(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, groupLabel: SubscriptTypegpuStringView) {
    if renderBundleEncoder.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuRenderBundleEncoderPushDebugGroup(renderBundleEncoder.cast(), wgpu_string_view(groupLabel)) }
}

/// `subscript-typegpu.h`: forwards to `wgpuRenderBundleEncoderPopDebugGroup`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_bundle_encoder_pop_debug_group(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder) {
    if renderBundleEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderBundleEncoderPopDebugGroup(renderBundleEncoder.cast()) }
}

/// `subscript-typegpu.h`: creates an object from a chain-free descriptor.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_bundle_encoder_finish(
    renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder,
    descriptor: *const SubscriptTypegpuRenderBundleDescriptor,
) -> SubscriptTypegpuRenderBundle {
    if renderBundleEncoder.is_null() {
        return std::ptr::null_mut();
    }
    if descriptor.is_null() {
        // SAFETY: webgpu.yml marks this descriptor optional.
        let created = unsafe { wgpuRenderBundleEncoderFinish(renderBundleEncoder.cast(), std::ptr::null()).cast() };
        runtime::inherit_handle_instance(renderBundleEncoder as usize, created as usize);
        return created;
    }
    // SAFETY: the caller supplies a live descriptor for this call.
    let source = unsafe { *descriptor };
    let descriptor = convert_render_bundle_descriptor(source);
    // SAFETY: the receiver is non-null and the converted descriptor
    // outlives the backend call.
    let created = unsafe { wgpuRenderBundleEncoderFinish(renderBundleEncoder.cast(), &descriptor).cast() };
    runtime::inherit_handle_instance(renderBundleEncoder as usize, created as usize);
    created
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_bundle_encoder_set_label(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, label: SubscriptTypegpuStringView) {
    if renderBundleEncoder.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuRenderBundleEncoderSetLabel(renderBundleEncoder.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_bundle_set_label(renderBundle: SubscriptTypegpuRenderBundle, label: SubscriptTypegpuStringView) {
    if renderBundle.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuRenderBundleSetLabel(renderBundle.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: forwards to `wgpuQuerySetGetType`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_query_set_get_type(querySet: SubscriptTypegpuQuerySet) -> i32 {
    if querySet.is_null() {
        return 0;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuQuerySetGetType(querySet.cast()) }
}

/// `subscript-typegpu.h`: forwards to `wgpuQuerySetGetCount`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_query_set_get_count(querySet: SubscriptTypegpuQuerySet) -> u32 {
    if querySet.is_null() {
        return 0;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuQuerySetGetCount(querySet.cast()) }
}

/// `subscript-typegpu.h`: forwards to `wgpuQuerySetDestroy`.
#[no_mangle]
pub extern "C" fn subscript_typegpu_query_set_destroy(querySet: SubscriptTypegpuQuerySet) {
    if querySet.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuQuerySetDestroy(querySet.cast()) }
}

/// `subscript-typegpu.h`: forwards a borrowed label string view.
#[no_mangle]
pub extern "C" fn subscript_typegpu_query_set_set_label(querySet: SubscriptTypegpuQuerySet, label: SubscriptTypegpuStringView) {
    if querySet.is_null() {
        return;
    }
    // SAFETY: the receiver is non-null and the input view is borrowed
    // only for this call.
    unsafe { wgpuQuerySetSetLabel(querySet.cast(), wgpu_string_view(label)) }
}

/// `subscript-typegpu.h`: releases the query set handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_query_set_release(querySet: SubscriptTypegpuQuerySet) {
    if querySet.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuQuerySetRelease(querySet.cast()) }
}

/// `subscript-typegpu.h`: releases the render bundle handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_bundle_release(renderBundle: SubscriptTypegpuRenderBundle) {
    if renderBundle.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderBundleRelease(renderBundle.cast()) }
}

/// `subscript-typegpu.h`: releases the render bundle encoder handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_bundle_encoder_release(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder) {
    if renderBundleEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderBundleEncoderRelease(renderBundleEncoder.cast()) }
}

/// `subscript-typegpu.h`: releases the command buffer handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_command_buffer_release(commandBuffer: SubscriptTypegpuCommandBuffer) {
    if commandBuffer.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuCommandBufferRelease(commandBuffer.cast()) }
}

/// `subscript-typegpu.h`: releases the render pass encoder handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pass_encoder_release(renderPassEncoder: SubscriptTypegpuRenderPassEncoder) {
    if renderPassEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderPassEncoderRelease(renderPassEncoder.cast()) }
}

/// `subscript-typegpu.h`: releases the compute pass encoder handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_compute_pass_encoder_release(computePassEncoder: SubscriptTypegpuComputePassEncoder) {
    if computePassEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuComputePassEncoderRelease(computePassEncoder.cast()) }
}

/// `subscript-typegpu.h`: releases the command encoder handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_command_encoder_release(commandEncoder: SubscriptTypegpuCommandEncoder) {
    if commandEncoder.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuCommandEncoderRelease(commandEncoder.cast()) }
}

/// `subscript-typegpu.h`: releases the render pipeline handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_render_pipeline_release(renderPipeline: SubscriptTypegpuRenderPipeline) {
    if renderPipeline.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuRenderPipelineRelease(renderPipeline.cast()) }
}

/// `subscript-typegpu.h`: releases the compute pipeline handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_compute_pipeline_release(computePipeline: SubscriptTypegpuComputePipeline) {
    if computePipeline.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuComputePipelineRelease(computePipeline.cast()) }
}

/// `subscript-typegpu.h`: releases the shader module handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_shader_module_release(shaderModule: SubscriptTypegpuShaderModule) {
    if shaderModule.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuShaderModuleRelease(shaderModule.cast()) }
}

/// `subscript-typegpu.h`: releases the pipeline layout handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_pipeline_layout_release(pipelineLayout: SubscriptTypegpuPipelineLayout) {
    if pipelineLayout.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuPipelineLayoutRelease(pipelineLayout.cast()) }
}

/// `subscript-typegpu.h`: releases the bind group handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_bind_group_release(bindGroup: SubscriptTypegpuBindGroup) {
    if bindGroup.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuBindGroupRelease(bindGroup.cast()) }
}

/// `subscript-typegpu.h`: releases the bind group layout handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_bind_group_layout_release(bindGroupLayout: SubscriptTypegpuBindGroupLayout) {
    if bindGroupLayout.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuBindGroupLayoutRelease(bindGroupLayout.cast()) }
}

/// `subscript-typegpu.h`: releases the sampler handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_sampler_release(sampler: SubscriptTypegpuSampler) {
    if sampler.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuSamplerRelease(sampler.cast()) }
}

/// `subscript-typegpu.h`: releases the texture view handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_texture_view_release(textureView: SubscriptTypegpuTextureView) {
    if textureView.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuTextureViewRelease(textureView.cast()) }
}

/// `subscript-typegpu.h`: releases the texture handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_texture_release(texture: SubscriptTypegpuTexture) {
    if texture.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuTextureRelease(texture.cast()) }
}

/// `subscript-typegpu.h`: releases the buffer handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_buffer_release(buffer: SubscriptTypegpuBuffer) {
    if buffer.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuBufferRelease(buffer.cast()) }
}

/// `subscript-typegpu.h`: releases the queue handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_queue_release(queue: SubscriptTypegpuQueue) {
    if queue.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuQueueRelease(queue.cast()) }
}

/// `subscript-typegpu.h`: releases the device handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_device_release(device: SubscriptTypegpuDevice) {
    if device.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuDeviceRelease(device.cast()) }
    runtime::release_device_events(device as usize);
    runtime::release_adapter_info_strings(device as usize);
}

/// `subscript-typegpu.h`: releases the adapter handle.
#[no_mangle]
pub extern "C" fn subscript_typegpu_adapter_release(adapter: SubscriptTypegpuAdapter) {
    if adapter.is_null() {
        return;
    }
    // SAFETY: non-null handle owned by the caller.
    unsafe { wgpuAdapterRelease(adapter.cast()) }
    runtime::release_adapter_info_strings(adapter as usize);
}
