// GENERATED FILE — DO NOT EDIT.
//
// Ambient boundary mirror produced by this project's `bindgen` from
// `subscript-typegpu.h`. Hand edits are overwritten; the byte-identical
// regeneration test (specs/blocks/compiler.md §12.2) fails on drift. Fix
// the generator, never this file (CLAUDE.md core principle 6).
//
// Boundary typing follows the Q13 rules (specs/blocks/collisions.md §2):
// opaque handles are branded interfaces; struct pointers and
// value-class-with-null are `X | null`; (pointer,count) descriptors are
// `T[]`; length-carrying string views are `string`; callback userdata
// slots are `object | null`. These declarations are global ambient (no
// import/export), like the language prelude.

// @subscript-c-header include="subscript-typegpu.h"
// @subscript-c-cenum typedef="SubscriptTypegpuBufferMapState" alias="GPUBufferMapState"
// @subscript-c-cenum typedef="SubscriptTypegpuTextureDimension" alias="GPUTextureDimension"
// @subscript-c-cenum typedef="SubscriptTypegpuTextureFormat" alias="GPUTextureFormat"
// @subscript-c-cenum typedef="SubscriptTypegpuTextureViewDimension" alias="GPUTextureViewDimension"
// @subscript-c-cenum typedef="SubscriptTypegpuTextureAspect" alias="GPUTextureAspect"
// @subscript-c-cenum typedef="SubscriptTypegpuAddressMode" alias="GPUAddressMode"
// @subscript-c-cenum typedef="SubscriptTypegpuFilterMode" alias="GPUFilterMode"
// @subscript-c-cenum typedef="SubscriptTypegpuMipmapFilterMode" alias="GPUMipmapFilterMode"
// @subscript-c-cenum typedef="SubscriptTypegpuCompareFunction" alias="GPUCompareFunction"
// @subscript-c-cenum typedef="SubscriptTypegpuBufferBindingType" alias="GPUBufferBindingType"
// @subscript-c-cenum typedef="SubscriptTypegpuSamplerBindingType" alias="GPUSamplerBindingType"
// @subscript-c-cenum typedef="SubscriptTypegpuTextureSampleType" alias="GPUTextureSampleType"
// @subscript-c-cenum typedef="SubscriptTypegpuStorageTextureAccess" alias="GPUStorageTextureAccess"
// @subscript-c-cenum typedef="SubscriptTypegpuVertexFormat" alias="GPUVertexFormat"
// @subscript-c-cenum typedef="SubscriptTypegpuVertexStepMode" alias="GPUVertexStepMode"
// @subscript-c-cenum typedef="SubscriptTypegpuPrimitiveTopology" alias="GPUPrimitiveTopology"
// @subscript-c-cenum typedef="SubscriptTypegpuIndexFormat" alias="GPUIndexFormat"
// @subscript-c-cenum typedef="SubscriptTypegpuFrontFace" alias="GPUFrontFace"
// @subscript-c-cenum typedef="SubscriptTypegpuCullMode" alias="GPUCullMode"
// @subscript-c-cenum typedef="SubscriptTypegpuBlendOperation" alias="GPUBlendOperation"
// @subscript-c-cenum typedef="SubscriptTypegpuBlendFactor" alias="GPUBlendFactor"
// @subscript-c-cenum typedef="SubscriptTypegpuStencilOperation" alias="GPUStencilOperation"
// @subscript-c-cenum typedef="SubscriptTypegpuLoadOp" alias="GPULoadOp"
// @subscript-c-cenum typedef="SubscriptTypegpuStoreOp" alias="GPUStoreOp"
// @subscript-c-cenum typedef="SubscriptTypegpuQueryType" alias="GPUQueryType"
// @subscript-c-cenum typedef="SubscriptTypegpuErrorFilter" alias="GPUErrorFilter"
// @subscript-c-cenum typedef="SubscriptTypegpuDeviceLostReason" alias="GPUDeviceLostReason"
// @subscript-c-cenum typedef="SubscriptTypegpuFeatureName" alias="GPUFeatureName"
// @subscript-c-string-view function="subscript_typegpu_device_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"
// @subscript-c-scalar-pair function="subscript_typegpu_queue_submit" parameter="commands" element="SubscriptTypegpuCommandBuffer" const=true
// @subscript-c-scalar-pair function="subscript_typegpu_queue_write_buffer" parameter="data" element="uint8_t" const=true
// @subscript-c-scalar-pair function="subscript_typegpu_queue_write_buffer_f32" parameter="data" element="float" const=true
// @subscript-c-scalar-pair function="subscript_typegpu_queue_write_texture" parameter="data" element="uint8_t" const=true
// @subscript-c-string-view function="subscript_typegpu_queue_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"
// @subscript-c-scalar-pair function="subscript_typegpu_buffer_read_mapped_range" parameter="out" element="uint8_t" const=false
// @subscript-c-scalar-pair function="subscript_typegpu_buffer_read_mapped_range_f32" parameter="out" element="float" const=false
// @subscript-c-scalar-pair function="subscript_typegpu_buffer_write_mapped_range" parameter="data" element="uint8_t" const=true
// @subscript-c-string-view function="subscript_typegpu_buffer_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_texture_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_texture_view_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_sampler_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_bind_group_layout_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_bind_group_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_pipeline_layout_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_shader_module_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_compute_pipeline_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_render_pipeline_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_command_encoder_insert_debug_marker" parameter="markerLabel" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_command_encoder_push_debug_group" parameter="groupLabel" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_command_encoder_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"
// @subscript-c-scalar-pair function="subscript_typegpu_compute_pass_encoder_set_bind_group" parameter="dynamicOffsets" element="uint32_t" const=true
// @subscript-c-string-view function="subscript_typegpu_compute_pass_encoder_insert_debug_marker" parameter="markerLabel" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_compute_pass_encoder_push_debug_group" parameter="groupLabel" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_compute_pass_encoder_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"
// @subscript-c-scalar-pair function="subscript_typegpu_render_pass_encoder_set_bind_group" parameter="dynamicOffsets" element="uint32_t" const=true
// @subscript-c-scalar-pair function="subscript_typegpu_render_pass_encoder_execute_bundles" parameter="bundles" element="SubscriptTypegpuRenderBundle" const=true
// @subscript-c-string-view function="subscript_typegpu_render_pass_encoder_insert_debug_marker" parameter="markerLabel" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_render_pass_encoder_push_debug_group" parameter="groupLabel" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_render_pass_encoder_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_command_buffer_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"
// @subscript-c-scalar-pair function="subscript_typegpu_render_bundle_encoder_set_bind_group" parameter="dynamicOffsets" element="uint32_t" const=true
// @subscript-c-string-view function="subscript_typegpu_render_bundle_encoder_insert_debug_marker" parameter="markerLabel" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_render_bundle_encoder_push_debug_group" parameter="groupLabel" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_render_bundle_encoder_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_render_bundle_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"
// @subscript-c-string-view function="subscript_typegpu_query_set_set_label" parameter="label" aggregate="SubscriptTypegpuStringView"

interface SubscriptTypegpuInstance {
  readonly __sub_handle_SubscriptTypegpuInstance: never;
}

interface SubscriptTypegpuAdapter {
  readonly __sub_handle_SubscriptTypegpuAdapter: never;
}

interface SubscriptTypegpuDevice {
  readonly __sub_handle_SubscriptTypegpuDevice: never;
}

interface SubscriptTypegpuQueue {
  readonly __sub_handle_SubscriptTypegpuQueue: never;
}

interface SubscriptTypegpuBuffer {
  readonly __sub_handle_SubscriptTypegpuBuffer: never;
}

interface SubscriptTypegpuTexture {
  readonly __sub_handle_SubscriptTypegpuTexture: never;
}

interface SubscriptTypegpuTextureView {
  readonly __sub_handle_SubscriptTypegpuTextureView: never;
}

interface SubscriptTypegpuSampler {
  readonly __sub_handle_SubscriptTypegpuSampler: never;
}

interface SubscriptTypegpuBindGroupLayout {
  readonly __sub_handle_SubscriptTypegpuBindGroupLayout: never;
}

interface SubscriptTypegpuBindGroup {
  readonly __sub_handle_SubscriptTypegpuBindGroup: never;
}

interface SubscriptTypegpuPipelineLayout {
  readonly __sub_handle_SubscriptTypegpuPipelineLayout: never;
}

interface SubscriptTypegpuShaderModule {
  readonly __sub_handle_SubscriptTypegpuShaderModule: never;
}

interface SubscriptTypegpuComputePipeline {
  readonly __sub_handle_SubscriptTypegpuComputePipeline: never;
}

interface SubscriptTypegpuRenderPipeline {
  readonly __sub_handle_SubscriptTypegpuRenderPipeline: never;
}

interface SubscriptTypegpuCommandEncoder {
  readonly __sub_handle_SubscriptTypegpuCommandEncoder: never;
}

interface SubscriptTypegpuComputePassEncoder {
  readonly __sub_handle_SubscriptTypegpuComputePassEncoder: never;
}

interface SubscriptTypegpuRenderPassEncoder {
  readonly __sub_handle_SubscriptTypegpuRenderPassEncoder: never;
}

interface SubscriptTypegpuCommandBuffer {
  readonly __sub_handle_SubscriptTypegpuCommandBuffer: never;
}

interface SubscriptTypegpuRenderBundleEncoder {
  readonly __sub_handle_SubscriptTypegpuRenderBundleEncoder: never;
}

interface SubscriptTypegpuRenderBundle {
  readonly __sub_handle_SubscriptTypegpuRenderBundle: never;
}

interface SubscriptTypegpuQuerySet {
  readonly __sub_handle_SubscriptTypegpuQuerySet: never;
}

declare enum SubscriptTypegpuOptionalBool {
  SUBSCRIPT_TYPEGPU_OPTIONAL_BOOL_FALSE = 0,
  SUBSCRIPT_TYPEGPU_OPTIONAL_BOOL_TRUE = 1,
  SUBSCRIPT_TYPEGPU_OPTIONAL_BOOL_UNDEFINED = 2,
}

declare enum SubscriptTypegpuErrorType {
  SUBSCRIPT_TYPEGPU_ERROR_TYPE_NO_ERROR = 1,
  SUBSCRIPT_TYPEGPU_ERROR_TYPE_VALIDATION = 2,
  SUBSCRIPT_TYPEGPU_ERROR_TYPE_OUT_OF_MEMORY = 3,
  SUBSCRIPT_TYPEGPU_ERROR_TYPE_INTERNAL = 4,
  SUBSCRIPT_TYPEGPU_ERROR_TYPE_UNKNOWN = 5,
}

declare enum SubscriptTypegpuInstanceFeatureName {
  SUBSCRIPT_TYPEGPU_INSTANCE_FEATURE_NAME_TIMED_WAIT_ANY = 1,
  SUBSCRIPT_TYPEGPU_INSTANCE_FEATURE_NAME_SHADER_SOURCE_SPIRV = 2,
  SUBSCRIPT_TYPEGPU_INSTANCE_FEATURE_NAME_MULTIPLE_DEVICES_PER_ADAPTER = 3,
}

declare enum SubscriptTypegpuBackendType {
  SUBSCRIPT_TYPEGPU_BACKEND_TYPE_UNDEFINED = 0,
  SUBSCRIPT_TYPEGPU_BACKEND_TYPE_NULL = 1,
  SUBSCRIPT_TYPEGPU_BACKEND_TYPE_WEB_GPU = 2,
  SUBSCRIPT_TYPEGPU_BACKEND_TYPE_D3_D11 = 3,
  SUBSCRIPT_TYPEGPU_BACKEND_TYPE_D3_D12 = 4,
  SUBSCRIPT_TYPEGPU_BACKEND_TYPE_METAL = 5,
  SUBSCRIPT_TYPEGPU_BACKEND_TYPE_VULKAN = 6,
  SUBSCRIPT_TYPEGPU_BACKEND_TYPE_OPEN_GL = 7,
  SUBSCRIPT_TYPEGPU_BACKEND_TYPE_OPEN_GLES = 8,
}

declare enum SubscriptTypegpuAdapterType {
  SUBSCRIPT_TYPEGPU_ADAPTER_TYPE_DISCRETE_GPU = 1,
  SUBSCRIPT_TYPEGPU_ADAPTER_TYPE_INTEGRATED_GPU = 2,
  SUBSCRIPT_TYPEGPU_ADAPTER_TYPE_CPU = 3,
  SUBSCRIPT_TYPEGPU_ADAPTER_TYPE_UNKNOWN = 4,
}

declare class SubscriptTypegpuErrorRecord {
  type: SubscriptTypegpuErrorType;
  message: string;
  constructor(type: SubscriptTypegpuErrorType, message: string);
}

declare class SubscriptTypegpuLostRecord {
  reason: GPUDeviceLostReason;
  message: string;
  constructor(reason: GPUDeviceLostReason, message: string);
}

declare class SubscriptTypegpuAdapterInfo {
  vendor: string;
  architecture: string;
  device: string;
  description: string;
  backendType: SubscriptTypegpuBackendType;
  adapterType: SubscriptTypegpuAdapterType;
  vendorID: u32;
  deviceID: u32;
  constructor(vendor: string, architecture: string, device: string, description: string, backendType: SubscriptTypegpuBackendType, adapterType: SubscriptTypegpuAdapterType, vendorID: u32, deviceID: u32);
}

declare class SubscriptTypegpuInstanceLimits {
  timedWaitAnyMaxCount: u64;
  constructor(timedWaitAnyMaxCount: u64);
}

declare class SubscriptTypegpuLimits {
  maxTextureDimension1D: u32;
  maxTextureDimension2D: u32;
  maxTextureDimension3D: u32;
  maxTextureArrayLayers: u32;
  maxBindGroups: u32;
  maxBindGroupsPlusVertexBuffers: u32;
  maxBindingsPerBindGroup: u32;
  maxDynamicUniformBuffersPerPipelineLayout: u32;
  maxDynamicStorageBuffersPerPipelineLayout: u32;
  maxSampledTexturesPerShaderStage: u32;
  maxSamplersPerShaderStage: u32;
  maxStorageBuffersPerShaderStage: u32;
  maxStorageTexturesPerShaderStage: u32;
  maxUniformBuffersPerShaderStage: u32;
  maxUniformBufferBindingSize: u64;
  maxStorageBufferBindingSize: u64;
  minUniformBufferOffsetAlignment: u32;
  minStorageBufferOffsetAlignment: u32;
  maxVertexBuffers: u32;
  maxBufferSize: u64;
  maxVertexAttributes: u32;
  maxVertexBufferArrayStride: u32;
  maxInterStageShaderVariables: u32;
  maxColorAttachments: u32;
  maxColorAttachmentBytesPerSample: u32;
  maxComputeWorkgroupStorageSize: u32;
  maxComputeInvocationsPerWorkgroup: u32;
  maxComputeWorkgroupSizeX: u32;
  maxComputeWorkgroupSizeY: u32;
  maxComputeWorkgroupSizeZ: u32;
  maxComputeWorkgroupsPerDimension: u32;
  maxImmediateSize: u32;
  constructor(maxTextureDimension1D: u32, maxTextureDimension2D: u32, maxTextureDimension3D: u32, maxTextureArrayLayers: u32, maxBindGroups: u32, maxBindGroupsPlusVertexBuffers: u32, maxBindingsPerBindGroup: u32, maxDynamicUniformBuffersPerPipelineLayout: u32, maxDynamicStorageBuffersPerPipelineLayout: u32, maxSampledTexturesPerShaderStage: u32, maxSamplersPerShaderStage: u32, maxStorageBuffersPerShaderStage: u32, maxStorageTexturesPerShaderStage: u32, maxUniformBuffersPerShaderStage: u32, maxUniformBufferBindingSize: u64, maxStorageBufferBindingSize: u64, minUniformBufferOffsetAlignment: u32, minStorageBufferOffsetAlignment: u32, maxVertexBuffers: u32, maxBufferSize: u64, maxVertexAttributes: u32, maxVertexBufferArrayStride: u32, maxInterStageShaderVariables: u32, maxColorAttachments: u32, maxColorAttachmentBytesPerSample: u32, maxComputeWorkgroupStorageSize: u32, maxComputeInvocationsPerWorkgroup: u32, maxComputeWorkgroupSizeX: u32, maxComputeWorkgroupSizeY: u32, maxComputeWorkgroupSizeZ: u32, maxComputeWorkgroupsPerDimension: u32, maxImmediateSize: u32);
}

declare class SubscriptTypegpuQueueDescriptor {
  label: string;
  constructor(label: string);
}

declare class SubscriptTypegpuBufferDescriptor {
  label: string;
  usage: SubscriptTypegpuBufferUsage;
  size: u64;
  mappedAtCreation: boolean;
  constructor(label: string, usage: SubscriptTypegpuBufferUsage, size: u64, mappedAtCreation: boolean);
}

declare class SubscriptTypegpuExtent3D {
  width: u32;
  height: u32;
  depthOrArrayLayers: u32;
  constructor(width: u32, height: u32, depthOrArrayLayers: u32);
}

declare class SubscriptTypegpuTextureDescriptor {
  label: string;
  usage: SubscriptTypegpuTextureUsage;
  dimension: GPUTextureDimension;
  size: SubscriptTypegpuExtent3D;
  format: GPUTextureFormat;
  mipLevelCount: u32;
  sampleCount: u32;
  viewFormats: GPUTextureFormat[];
  constructor(label: string, usage: SubscriptTypegpuTextureUsage, dimension: GPUTextureDimension, size: SubscriptTypegpuExtent3D, format: GPUTextureFormat, mipLevelCount: u32, sampleCount: u32, viewFormats: GPUTextureFormat[]);
}

declare class SubscriptTypegpuSamplerDescriptor {
  label: string;
  addressModeU: GPUAddressMode;
  addressModeV: GPUAddressMode;
  addressModeW: GPUAddressMode;
  magFilter: GPUFilterMode;
  minFilter: GPUFilterMode;
  mipmapFilter: GPUMipmapFilterMode;
  lodMinClamp: f32;
  lodMaxClamp: f32;
  compare: GPUCompareFunction;
  maxAnisotropy: u16;
  constructor(label: string, addressModeU: GPUAddressMode, addressModeV: GPUAddressMode, addressModeW: GPUAddressMode, magFilter: GPUFilterMode, minFilter: GPUFilterMode, mipmapFilter: GPUMipmapFilterMode, lodMinClamp: f32, lodMaxClamp: f32, compare: GPUCompareFunction, maxAnisotropy: u16);
}

declare class SubscriptTypegpuBufferBindingLayout {
  type: GPUBufferBindingType;
  hasDynamicOffset: boolean;
  minBindingSize: u64;
  constructor(type: GPUBufferBindingType, hasDynamicOffset: boolean, minBindingSize: u64);
}

declare class SubscriptTypegpuSamplerBindingLayout {
  type: GPUSamplerBindingType;
  constructor(type: GPUSamplerBindingType);
}

declare class SubscriptTypegpuTextureBindingLayout {
  sampleType: GPUTextureSampleType;
  viewDimension: GPUTextureViewDimension;
  multisampled: boolean;
  constructor(sampleType: GPUTextureSampleType, viewDimension: GPUTextureViewDimension, multisampled: boolean);
}

declare class SubscriptTypegpuStorageTextureBindingLayout {
  access: GPUStorageTextureAccess;
  format: GPUTextureFormat;
  viewDimension: GPUTextureViewDimension;
  constructor(access: GPUStorageTextureAccess, format: GPUTextureFormat, viewDimension: GPUTextureViewDimension);
}

declare class SubscriptTypegpuBindGroupLayoutEntry {
  binding: u32;
  visibility: SubscriptTypegpuShaderStage;
  bindingArraySize: u32;
  buffer: SubscriptTypegpuBufferBindingLayout;
  sampler: SubscriptTypegpuSamplerBindingLayout;
  texture: SubscriptTypegpuTextureBindingLayout;
  storageTexture: SubscriptTypegpuStorageTextureBindingLayout;
  constructor(binding: u32, visibility: SubscriptTypegpuShaderStage, bindingArraySize: u32, buffer: SubscriptTypegpuBufferBindingLayout, sampler: SubscriptTypegpuSamplerBindingLayout, texture: SubscriptTypegpuTextureBindingLayout, storageTexture: SubscriptTypegpuStorageTextureBindingLayout);
}

declare class SubscriptTypegpuBindGroupLayoutDescriptor {
  label: string;
  entries: SubscriptTypegpuBindGroupLayoutEntry[];
  constructor(label: string, entries: SubscriptTypegpuBindGroupLayoutEntry[]);
}

declare class SubscriptTypegpuBindGroupEntry {
  binding: u32;
  buffer: SubscriptTypegpuBuffer | null;
  offset: u64;
  size: u64;
  sampler: SubscriptTypegpuSampler | null;
  textureView: SubscriptTypegpuTextureView | null;
  constructor(binding: u32, buffer: SubscriptTypegpuBuffer | null, offset: u64, size: u64, sampler: SubscriptTypegpuSampler | null, textureView: SubscriptTypegpuTextureView | null);
}

declare class SubscriptTypegpuBindGroupDescriptor {
  label: string;
  layout: SubscriptTypegpuBindGroupLayout;
  entries: SubscriptTypegpuBindGroupEntry[];
  constructor(label: string, layout: SubscriptTypegpuBindGroupLayout, entries: SubscriptTypegpuBindGroupEntry[]);
}

declare class SubscriptTypegpuPipelineLayoutDescriptor {
  label: string;
  bindGroupLayouts: SubscriptTypegpuBindGroupLayout[];
  immediateSize: u32;
  constructor(label: string, bindGroupLayouts: SubscriptTypegpuBindGroupLayout[], immediateSize: u32);
}

declare class SubscriptTypegpuConstantEntry {
  key: string;
  value: f64;
  constructor(key: string, value: f64);
}

declare class SubscriptTypegpuComputeState {
  module: SubscriptTypegpuShaderModule;
  entryPoint: string;
  constants: SubscriptTypegpuConstantEntry[];
  constructor(module: SubscriptTypegpuShaderModule, entryPoint: string, constants: SubscriptTypegpuConstantEntry[]);
}

declare class SubscriptTypegpuComputePipelineDescriptor {
  label: string;
  layout: SubscriptTypegpuPipelineLayout | null;
  compute: SubscriptTypegpuComputeState;
  constructor(label: string, layout: SubscriptTypegpuPipelineLayout | null, compute: SubscriptTypegpuComputeState);
}

declare class SubscriptTypegpuVertexAttribute {
  format: GPUVertexFormat;
  offset: u64;
  shaderLocation: u32;
  constructor(format: GPUVertexFormat, offset: u64, shaderLocation: u32);
}

declare class SubscriptTypegpuVertexBufferLayout {
  stepMode: GPUVertexStepMode;
  arrayStride: u64;
  attributes: SubscriptTypegpuVertexAttribute[];
  constructor(stepMode: GPUVertexStepMode, arrayStride: u64, attributes: SubscriptTypegpuVertexAttribute[]);
}

declare class SubscriptTypegpuVertexState {
  module: SubscriptTypegpuShaderModule;
  entryPoint: string;
  constants: SubscriptTypegpuConstantEntry[];
  buffers: SubscriptTypegpuVertexBufferLayout[];
  constructor(module: SubscriptTypegpuShaderModule, entryPoint: string, constants: SubscriptTypegpuConstantEntry[], buffers: SubscriptTypegpuVertexBufferLayout[]);
}

declare class SubscriptTypegpuPrimitiveState {
  topology: GPUPrimitiveTopology;
  stripIndexFormat: GPUIndexFormat;
  frontFace: GPUFrontFace;
  cullMode: GPUCullMode;
  unclippedDepth: boolean;
  constructor(topology: GPUPrimitiveTopology, stripIndexFormat: GPUIndexFormat, frontFace: GPUFrontFace, cullMode: GPUCullMode, unclippedDepth: boolean);
}

declare class SubscriptTypegpuStencilFaceState {
  compare: GPUCompareFunction;
  failOp: GPUStencilOperation;
  depthFailOp: GPUStencilOperation;
  passOp: GPUStencilOperation;
  constructor(compare: GPUCompareFunction, failOp: GPUStencilOperation, depthFailOp: GPUStencilOperation, passOp: GPUStencilOperation);
}

declare class SubscriptTypegpuDepthStencilState {
  format: GPUTextureFormat;
  depthWriteEnabled: SubscriptTypegpuOptionalBool;
  depthCompare: GPUCompareFunction;
  stencilFront: SubscriptTypegpuStencilFaceState;
  stencilBack: SubscriptTypegpuStencilFaceState;
  stencilReadMask: u32;
  stencilWriteMask: u32;
  depthBias: i32;
  depthBiasSlopeScale: f32;
  depthBiasClamp: f32;
  constructor(format: GPUTextureFormat, depthWriteEnabled: SubscriptTypegpuOptionalBool, depthCompare: GPUCompareFunction, stencilFront: SubscriptTypegpuStencilFaceState, stencilBack: SubscriptTypegpuStencilFaceState, stencilReadMask: u32, stencilWriteMask: u32, depthBias: i32, depthBiasSlopeScale: f32, depthBiasClamp: f32);
}

declare class SubscriptTypegpuMultisampleState {
  count: u32;
  mask: u32;
  alphaToCoverageEnabled: boolean;
  constructor(count: u32, mask: u32, alphaToCoverageEnabled: boolean);
}

declare class SubscriptTypegpuBlendComponent {
  operation: GPUBlendOperation;
  srcFactor: GPUBlendFactor;
  dstFactor: GPUBlendFactor;
  constructor(operation: GPUBlendOperation, srcFactor: GPUBlendFactor, dstFactor: GPUBlendFactor);
}

declare class SubscriptTypegpuBlendState {
  color: SubscriptTypegpuBlendComponent;
  alpha: SubscriptTypegpuBlendComponent;
  constructor(color: SubscriptTypegpuBlendComponent, alpha: SubscriptTypegpuBlendComponent);
}

declare class SubscriptTypegpuColorTargetState {
  format: GPUTextureFormat;
  blend: SubscriptTypegpuBlendState | null;
  writeMask: SubscriptTypegpuColorWriteMask;
  constructor(format: GPUTextureFormat, blend: SubscriptTypegpuBlendState | null, writeMask: SubscriptTypegpuColorWriteMask);
}

declare class SubscriptTypegpuFragmentState {
  module: SubscriptTypegpuShaderModule;
  entryPoint: string;
  constants: SubscriptTypegpuConstantEntry[];
  targets: SubscriptTypegpuColorTargetState[];
  constructor(module: SubscriptTypegpuShaderModule, entryPoint: string, constants: SubscriptTypegpuConstantEntry[], targets: SubscriptTypegpuColorTargetState[]);
}

declare class SubscriptTypegpuRenderPipelineDescriptor {
  label: string;
  layout: SubscriptTypegpuPipelineLayout | null;
  vertex: SubscriptTypegpuVertexState;
  primitive: SubscriptTypegpuPrimitiveState;
  depthStencil: SubscriptTypegpuDepthStencilState | null;
  multisample: SubscriptTypegpuMultisampleState;
  fragment: SubscriptTypegpuFragmentState | null;
  constructor(label: string, layout: SubscriptTypegpuPipelineLayout | null, vertex: SubscriptTypegpuVertexState, primitive: SubscriptTypegpuPrimitiveState, depthStencil: SubscriptTypegpuDepthStencilState | null, multisample: SubscriptTypegpuMultisampleState, fragment: SubscriptTypegpuFragmentState | null);
}

declare class SubscriptTypegpuCommandEncoderDescriptor {
  label: string;
  constructor(label: string);
}

declare class SubscriptTypegpuRenderBundleEncoderDescriptor {
  label: string;
  colorFormats: GPUTextureFormat[];
  depthStencilFormat: GPUTextureFormat;
  sampleCount: u32;
  depthReadOnly: boolean;
  stencilReadOnly: boolean;
  constructor(label: string, colorFormats: GPUTextureFormat[], depthStencilFormat: GPUTextureFormat, sampleCount: u32, depthReadOnly: boolean, stencilReadOnly: boolean);
}

declare class SubscriptTypegpuQuerySetDescriptor {
  label: string;
  type: GPUQueryType;
  count: u32;
  constructor(label: string, type: GPUQueryType, count: u32);
}

declare class SubscriptTypegpuOrigin3D {
  x: u32;
  y: u32;
  z: u32;
  constructor(x: u32, y: u32, z: u32);
}

declare class SubscriptTypegpuTexelCopyTextureInfo {
  texture: SubscriptTypegpuTexture;
  mipLevel: u32;
  origin: SubscriptTypegpuOrigin3D;
  aspect: GPUTextureAspect;
  constructor(texture: SubscriptTypegpuTexture, mipLevel: u32, origin: SubscriptTypegpuOrigin3D, aspect: GPUTextureAspect);
}

declare class SubscriptTypegpuTexelCopyBufferLayout {
  offset: u64;
  bytesPerRow: u32;
  rowsPerImage: u32;
  constructor(offset: u64, bytesPerRow: u32, rowsPerImage: u32);
}

declare class SubscriptTypegpuTextureViewDescriptor {
  label: string;
  format: GPUTextureFormat;
  dimension: GPUTextureViewDimension;
  baseMipLevel: u32;
  mipLevelCount: u32;
  baseArrayLayer: u32;
  arrayLayerCount: u32;
  aspect: GPUTextureAspect;
  usage: SubscriptTypegpuTextureUsage;
  constructor(label: string, format: GPUTextureFormat, dimension: GPUTextureViewDimension, baseMipLevel: u32, mipLevelCount: u32, baseArrayLayer: u32, arrayLayerCount: u32, aspect: GPUTextureAspect, usage: SubscriptTypegpuTextureUsage);
}

declare class SubscriptTypegpuCommandBufferDescriptor {
  label: string;
  constructor(label: string);
}

declare class SubscriptTypegpuPassTimestampWrites {
  querySet: SubscriptTypegpuQuerySet;
  beginningOfPassWriteIndex: u32;
  endOfPassWriteIndex: u32;
  constructor(querySet: SubscriptTypegpuQuerySet, beginningOfPassWriteIndex: u32, endOfPassWriteIndex: u32);
}

declare class SubscriptTypegpuComputePassDescriptor {
  label: string;
  timestampWrites: SubscriptTypegpuPassTimestampWrites | null;
  constructor(label: string, timestampWrites: SubscriptTypegpuPassTimestampWrites | null);
}

declare class SubscriptTypegpuColor {
  r: f64;
  g: f64;
  b: f64;
  a: f64;
  constructor(r: f64, g: f64, b: f64, a: f64);
}

declare class SubscriptTypegpuRenderPassColorAttachment {
  view: SubscriptTypegpuTextureView | null;
  depthSlice: u32;
  resolveTarget: SubscriptTypegpuTextureView | null;
  loadOp: GPULoadOp;
  storeOp: GPUStoreOp;
  clearValue: SubscriptTypegpuColor;
  constructor(view: SubscriptTypegpuTextureView | null, depthSlice: u32, resolveTarget: SubscriptTypegpuTextureView | null, loadOp: GPULoadOp, storeOp: GPUStoreOp, clearValue: SubscriptTypegpuColor);
}

declare class SubscriptTypegpuRenderPassDepthStencilAttachment {
  view: SubscriptTypegpuTextureView;
  depthLoadOp: GPULoadOp;
  depthStoreOp: GPUStoreOp;
  depthClearValue: f32;
  depthReadOnly: boolean;
  stencilLoadOp: GPULoadOp;
  stencilStoreOp: GPUStoreOp;
  stencilClearValue: u32;
  stencilReadOnly: boolean;
  constructor(view: SubscriptTypegpuTextureView, depthLoadOp: GPULoadOp, depthStoreOp: GPUStoreOp, depthClearValue: f32, depthReadOnly: boolean, stencilLoadOp: GPULoadOp, stencilStoreOp: GPUStoreOp, stencilClearValue: u32, stencilReadOnly: boolean);
}

declare class SubscriptTypegpuRenderPassDescriptor {
  label: string;
  colorAttachments: SubscriptTypegpuRenderPassColorAttachment[];
  depthStencilAttachment: SubscriptTypegpuRenderPassDepthStencilAttachment | null;
  occlusionQuerySet: SubscriptTypegpuQuerySet | null;
  timestampWrites: SubscriptTypegpuPassTimestampWrites | null;
  constructor(label: string, colorAttachments: SubscriptTypegpuRenderPassColorAttachment[], depthStencilAttachment: SubscriptTypegpuRenderPassDepthStencilAttachment | null, occlusionQuerySet: SubscriptTypegpuQuerySet | null, timestampWrites: SubscriptTypegpuPassTimestampWrites | null);
}

declare class SubscriptTypegpuTexelCopyBufferInfo {
  layout: SubscriptTypegpuTexelCopyBufferLayout;
  buffer: SubscriptTypegpuBuffer;
  constructor(layout: SubscriptTypegpuTexelCopyBufferLayout, buffer: SubscriptTypegpuBuffer);
}

declare class SubscriptTypegpuRenderBundleDescriptor {
  label: string;
  constructor(label: string);
}

declare class SubscriptTypegpuDeviceDescriptor {
  label: string;
  requiredFeatures: GPUFeatureName[];
  requiredLimits: SubscriptTypegpuLimits | null;
  defaultQueue: SubscriptTypegpuQueueDescriptor;
  constructor(label: string, requiredFeatures: GPUFeatureName[], requiredLimits: SubscriptTypegpuLimits | null, defaultQueue: SubscriptTypegpuQueueDescriptor);
}

declare class SubscriptTypegpuShaderModuleDescriptor {
  label: string;
  code: string;
  constructor(label: string, code: string);
}

declare function subscript_typegpu_create_instance(): SubscriptTypegpuInstance;
declare function subscript_typegpu_instance_process_events(instance: SubscriptTypegpuInstance): void;
declare function subscript_typegpu_instance_release(instance: SubscriptTypegpuInstance): void;
declare function subscript_typegpu_get_instance_limits(out: SubscriptTypegpuInstanceLimits | null): i32;
declare function subscript_typegpu_has_instance_feature(feature: SubscriptTypegpuInstanceFeatureName): boolean;
declare function subscript_typegpu_instance_request_adapter(instance: SubscriptTypegpuInstance): SubscriptTypegpuFutureId;
declare function subscript_typegpu_future_status(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId): i32;
declare function subscript_typegpu_future_drop(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId): void;
declare function subscript_typegpu_request_adapter_take(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId): SubscriptTypegpuAdapter;
declare function subscript_typegpu_adapter_get_limits(adapter: SubscriptTypegpuAdapter, out: SubscriptTypegpuLimits | null): i32;
declare function subscript_typegpu_adapter_get_info(adapter: SubscriptTypegpuAdapter, out: SubscriptTypegpuAdapterInfo | null): boolean;
declare function subscript_typegpu_adapter_has_feature(adapter: SubscriptTypegpuAdapter, feature: GPUFeatureName): boolean;
declare function subscript_typegpu_adapter_request_device(instance: SubscriptTypegpuInstance, adapter: SubscriptTypegpuAdapter): SubscriptTypegpuFutureId;
declare function subscript_typegpu_adapter_request_device_with_descriptor(instance: SubscriptTypegpuInstance, adapter: SubscriptTypegpuAdapter, descriptor: SubscriptTypegpuDeviceDescriptor | null): SubscriptTypegpuFutureId;
declare function subscript_typegpu_request_device_take(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId): SubscriptTypegpuDevice;
declare function subscript_typegpu_device_get_queue(device: SubscriptTypegpuDevice): SubscriptTypegpuQueue;
declare function subscript_typegpu_device_destroy(device: SubscriptTypegpuDevice): void;
declare function subscript_typegpu_device_set_label(device: SubscriptTypegpuDevice, label: string): void;
declare function subscript_typegpu_device_push_error_scope(device: SubscriptTypegpuDevice, filter: GPUErrorFilter): void;
declare function subscript_typegpu_device_pop_error_scope(device: SubscriptTypegpuDevice): SubscriptTypegpuFutureId;
declare function subscript_typegpu_pop_error_scope_take(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId, out: SubscriptTypegpuErrorRecord | null): boolean;
declare function subscript_typegpu_device_next_uncaptured_error(device: SubscriptTypegpuDevice, out: SubscriptTypegpuErrorRecord | null): boolean;
declare function subscript_typegpu_device_lost_info(device: SubscriptTypegpuDevice, out: SubscriptTypegpuLostRecord | null): boolean;
declare function subscript_typegpu_device_get_limits(device: SubscriptTypegpuDevice, out: SubscriptTypegpuLimits | null): i32;
declare function subscript_typegpu_device_get_adapter_info(device: SubscriptTypegpuDevice, out: SubscriptTypegpuAdapterInfo | null): boolean;
declare function subscript_typegpu_device_has_feature(device: SubscriptTypegpuDevice, feature: GPUFeatureName): boolean;
declare function subscript_typegpu_device_create_buffer(device: SubscriptTypegpuDevice, descriptor: SubscriptTypegpuBufferDescriptor | null): SubscriptTypegpuBuffer;
declare function subscript_typegpu_device_create_texture(device: SubscriptTypegpuDevice, descriptor: SubscriptTypegpuTextureDescriptor | null): SubscriptTypegpuTexture;
declare function subscript_typegpu_device_create_sampler(device: SubscriptTypegpuDevice, descriptor: SubscriptTypegpuSamplerDescriptor | null): SubscriptTypegpuSampler;
declare function subscript_typegpu_device_create_bind_group_layout(device: SubscriptTypegpuDevice, descriptor: SubscriptTypegpuBindGroupLayoutDescriptor | null): SubscriptTypegpuBindGroupLayout;
declare function subscript_typegpu_device_create_bind_group(device: SubscriptTypegpuDevice, descriptor: SubscriptTypegpuBindGroupDescriptor | null): SubscriptTypegpuBindGroup;
declare function subscript_typegpu_device_create_pipeline_layout(device: SubscriptTypegpuDevice, descriptor: SubscriptTypegpuPipelineLayoutDescriptor | null): SubscriptTypegpuPipelineLayout;
declare function subscript_typegpu_device_create_shader_module(device: SubscriptTypegpuDevice, descriptor: SubscriptTypegpuShaderModuleDescriptor | null): SubscriptTypegpuShaderModule;
declare function subscript_typegpu_device_create_compute_pipeline(device: SubscriptTypegpuDevice, descriptor: SubscriptTypegpuComputePipelineDescriptor | null): SubscriptTypegpuComputePipeline;
declare function subscript_typegpu_device_create_compute_pipeline_async_begin(instance: SubscriptTypegpuInstance, device: SubscriptTypegpuDevice, descriptor: SubscriptTypegpuComputePipelineDescriptor | null): SubscriptTypegpuFutureId;
declare function subscript_typegpu_create_compute_pipeline_async_take(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId): SubscriptTypegpuComputePipeline;
declare function subscript_typegpu_device_create_render_pipeline(device: SubscriptTypegpuDevice, descriptor: SubscriptTypegpuRenderPipelineDescriptor | null): SubscriptTypegpuRenderPipeline;
declare function subscript_typegpu_device_create_render_pipeline_async_begin(instance: SubscriptTypegpuInstance, device: SubscriptTypegpuDevice, descriptor: SubscriptTypegpuRenderPipelineDescriptor | null): SubscriptTypegpuFutureId;
declare function subscript_typegpu_create_render_pipeline_async_take(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId): SubscriptTypegpuRenderPipeline;
declare function subscript_typegpu_device_create_command_encoder(device: SubscriptTypegpuDevice, descriptor: SubscriptTypegpuCommandEncoderDescriptor | null): SubscriptTypegpuCommandEncoder;
declare function subscript_typegpu_device_create_render_bundle_encoder(device: SubscriptTypegpuDevice, descriptor: SubscriptTypegpuRenderBundleEncoderDescriptor | null): SubscriptTypegpuRenderBundleEncoder;
declare function subscript_typegpu_device_create_query_set(device: SubscriptTypegpuDevice, descriptor: SubscriptTypegpuQuerySetDescriptor | null): SubscriptTypegpuQuerySet;
declare function subscript_typegpu_queue_submit(queue: SubscriptTypegpuQueue, commands: SubscriptTypegpuCommandBuffer[]): void;
declare function subscript_typegpu_queue_on_submitted_work_done(instance: SubscriptTypegpuInstance, queue: SubscriptTypegpuQueue): SubscriptTypegpuFutureId;
declare function subscript_typegpu_queue_write_buffer(queue: SubscriptTypegpuQueue, buffer: SubscriptTypegpuBuffer, bufferOffset: u64, data: u8[]): void;
declare function subscript_typegpu_queue_write_buffer_f32(queue: SubscriptTypegpuQueue, buffer: SubscriptTypegpuBuffer, bufferOffsetBytes: u64, data: f32[]): void;
declare function subscript_typegpu_queue_write_texture(queue: SubscriptTypegpuQueue, dst: SubscriptTypegpuTexelCopyTextureInfo | null, layout: SubscriptTypegpuTexelCopyBufferLayout | null, extent: SubscriptTypegpuExtent3D | null, data: u8[]): void;
declare function subscript_typegpu_queue_set_label(queue: SubscriptTypegpuQueue, label: string): void;
declare function subscript_typegpu_buffer_map_async(buffer: SubscriptTypegpuBuffer, mode: SubscriptTypegpuMapMode, offset: u64, size: u64): SubscriptTypegpuFutureId;
declare function subscript_typegpu_buffer_map_whole_async(buffer: SubscriptTypegpuBuffer, mode: SubscriptTypegpuMapMode): SubscriptTypegpuFutureId;
declare function subscript_typegpu_buffer_read_mapped_range(buffer: SubscriptTypegpuBuffer, offset: u64, out: u8[]): i32;
declare function subscript_typegpu_buffer_read_mapped_range_f32(buffer: SubscriptTypegpuBuffer, offsetBytes: u64, out: f32[]): i32;
declare function subscript_typegpu_buffer_write_mapped_range(buffer: SubscriptTypegpuBuffer, offset: u64, data: u8[]): i32;
declare function subscript_typegpu_buffer_set_label(buffer: SubscriptTypegpuBuffer, label: string): void;
declare function subscript_typegpu_buffer_get_usage(buffer: SubscriptTypegpuBuffer): SubscriptTypegpuBufferUsage;
declare function subscript_typegpu_buffer_get_size(buffer: SubscriptTypegpuBuffer): u64;
declare function subscript_typegpu_buffer_get_map_state(buffer: SubscriptTypegpuBuffer): GPUBufferMapState;
declare function subscript_typegpu_buffer_unmap(buffer: SubscriptTypegpuBuffer): void;
declare function subscript_typegpu_buffer_destroy(buffer: SubscriptTypegpuBuffer): void;
declare function subscript_typegpu_texture_create_view(texture: SubscriptTypegpuTexture, descriptor: SubscriptTypegpuTextureViewDescriptor | null): SubscriptTypegpuTextureView;
declare function subscript_typegpu_texture_set_label(texture: SubscriptTypegpuTexture, label: string): void;
declare function subscript_typegpu_texture_get_width(texture: SubscriptTypegpuTexture): u32;
declare function subscript_typegpu_texture_get_height(texture: SubscriptTypegpuTexture): u32;
declare function subscript_typegpu_texture_get_depth_or_array_layers(texture: SubscriptTypegpuTexture): u32;
declare function subscript_typegpu_texture_get_mip_level_count(texture: SubscriptTypegpuTexture): u32;
declare function subscript_typegpu_texture_get_sample_count(texture: SubscriptTypegpuTexture): u32;
declare function subscript_typegpu_texture_get_dimension(texture: SubscriptTypegpuTexture): GPUTextureDimension;
declare function subscript_typegpu_texture_get_texture_binding_view_dimension(texture: SubscriptTypegpuTexture): GPUTextureViewDimension;
declare function subscript_typegpu_texture_get_format(texture: SubscriptTypegpuTexture): GPUTextureFormat;
declare function subscript_typegpu_texture_get_usage(texture: SubscriptTypegpuTexture): SubscriptTypegpuTextureUsage;
declare function subscript_typegpu_texture_destroy(texture: SubscriptTypegpuTexture): void;
declare function subscript_typegpu_texture_view_set_label(textureView: SubscriptTypegpuTextureView, label: string): void;
declare function subscript_typegpu_sampler_set_label(sampler: SubscriptTypegpuSampler, label: string): void;
declare function subscript_typegpu_bind_group_layout_set_label(bindGroupLayout: SubscriptTypegpuBindGroupLayout, label: string): void;
declare function subscript_typegpu_bind_group_set_label(bindGroup: SubscriptTypegpuBindGroup, label: string): void;
declare function subscript_typegpu_pipeline_layout_set_label(pipelineLayout: SubscriptTypegpuPipelineLayout, label: string): void;
declare function subscript_typegpu_shader_module_set_label(shaderModule: SubscriptTypegpuShaderModule, label: string): void;
declare function subscript_typegpu_compute_pipeline_get_bind_group_layout(computePipeline: SubscriptTypegpuComputePipeline, groupIndex: u32): SubscriptTypegpuBindGroupLayout;
declare function subscript_typegpu_compute_pipeline_set_label(computePipeline: SubscriptTypegpuComputePipeline, label: string): void;
declare function subscript_typegpu_render_pipeline_get_bind_group_layout(renderPipeline: SubscriptTypegpuRenderPipeline, groupIndex: u32): SubscriptTypegpuBindGroupLayout;
declare function subscript_typegpu_render_pipeline_set_label(renderPipeline: SubscriptTypegpuRenderPipeline, label: string): void;
declare function subscript_typegpu_command_encoder_finish(commandEncoder: SubscriptTypegpuCommandEncoder, descriptor: SubscriptTypegpuCommandBufferDescriptor | null): SubscriptTypegpuCommandBuffer;
declare function subscript_typegpu_command_encoder_begin_compute_pass(commandEncoder: SubscriptTypegpuCommandEncoder, descriptor: SubscriptTypegpuComputePassDescriptor | null): SubscriptTypegpuComputePassEncoder;
declare function subscript_typegpu_command_encoder_begin_render_pass(commandEncoder: SubscriptTypegpuCommandEncoder, descriptor: SubscriptTypegpuRenderPassDescriptor | null): SubscriptTypegpuRenderPassEncoder;
declare function subscript_typegpu_command_encoder_copy_buffer_to_buffer(commandEncoder: SubscriptTypegpuCommandEncoder, source: SubscriptTypegpuBuffer, sourceOffset: u64, destination: SubscriptTypegpuBuffer, destinationOffset: u64, size: u64): void;
declare function subscript_typegpu_command_encoder_copy_buffer_to_texture(commandEncoder: SubscriptTypegpuCommandEncoder, source: SubscriptTypegpuTexelCopyBufferInfo | null, destination: SubscriptTypegpuTexelCopyTextureInfo | null, copySize: SubscriptTypegpuExtent3D | null): void;
declare function subscript_typegpu_command_encoder_copy_texture_to_buffer(commandEncoder: SubscriptTypegpuCommandEncoder, source: SubscriptTypegpuTexelCopyTextureInfo | null, destination: SubscriptTypegpuTexelCopyBufferInfo | null, copySize: SubscriptTypegpuExtent3D | null): void;
declare function subscript_typegpu_command_encoder_copy_texture_to_texture(commandEncoder: SubscriptTypegpuCommandEncoder, source: SubscriptTypegpuTexelCopyTextureInfo | null, destination: SubscriptTypegpuTexelCopyTextureInfo | null, copySize: SubscriptTypegpuExtent3D | null): void;
declare function subscript_typegpu_command_encoder_clear_buffer(commandEncoder: SubscriptTypegpuCommandEncoder, buffer: SubscriptTypegpuBuffer, offset: u64, size: u64): void;
declare function subscript_typegpu_command_encoder_resolve_query_set(commandEncoder: SubscriptTypegpuCommandEncoder, querySet: SubscriptTypegpuQuerySet, firstQuery: u32, queryCount: u32, destination: SubscriptTypegpuBuffer, destinationOffset: u64): void;
declare function subscript_typegpu_command_encoder_write_timestamp(commandEncoder: SubscriptTypegpuCommandEncoder, querySet: SubscriptTypegpuQuerySet, queryIndex: u32): void;
declare function subscript_typegpu_command_encoder_insert_debug_marker(commandEncoder: SubscriptTypegpuCommandEncoder, markerLabel: string): void;
declare function subscript_typegpu_command_encoder_push_debug_group(commandEncoder: SubscriptTypegpuCommandEncoder, groupLabel: string): void;
declare function subscript_typegpu_command_encoder_pop_debug_group(commandEncoder: SubscriptTypegpuCommandEncoder): void;
declare function subscript_typegpu_command_encoder_set_label(commandEncoder: SubscriptTypegpuCommandEncoder, label: string): void;
declare function subscript_typegpu_compute_pass_encoder_set_pipeline(computePassEncoder: SubscriptTypegpuComputePassEncoder, pipeline: SubscriptTypegpuComputePipeline): void;
declare function subscript_typegpu_compute_pass_encoder_set_bind_group(computePassEncoder: SubscriptTypegpuComputePassEncoder, groupIndex: u32, group: SubscriptTypegpuBindGroup | null, dynamicOffsets: u32[]): void;
declare function subscript_typegpu_compute_pass_encoder_dispatch_workgroups(computePassEncoder: SubscriptTypegpuComputePassEncoder, workgroupCountX: u32, workgroupCountY: u32, workgroupCountZ: u32): void;
declare function subscript_typegpu_compute_pass_encoder_dispatch_workgroups_indirect(computePassEncoder: SubscriptTypegpuComputePassEncoder, indirectBuffer: SubscriptTypegpuBuffer, indirectOffset: u64): void;
declare function subscript_typegpu_compute_pass_encoder_insert_debug_marker(computePassEncoder: SubscriptTypegpuComputePassEncoder, markerLabel: string): void;
declare function subscript_typegpu_compute_pass_encoder_push_debug_group(computePassEncoder: SubscriptTypegpuComputePassEncoder, groupLabel: string): void;
declare function subscript_typegpu_compute_pass_encoder_pop_debug_group(computePassEncoder: SubscriptTypegpuComputePassEncoder): void;
declare function subscript_typegpu_compute_pass_encoder_end(computePassEncoder: SubscriptTypegpuComputePassEncoder): void;
declare function subscript_typegpu_compute_pass_encoder_set_label(computePassEncoder: SubscriptTypegpuComputePassEncoder, label: string): void;
declare function subscript_typegpu_render_pass_encoder_set_pipeline(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, pipeline: SubscriptTypegpuRenderPipeline): void;
declare function subscript_typegpu_render_pass_encoder_set_bind_group(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, groupIndex: u32, group: SubscriptTypegpuBindGroup | null, dynamicOffsets: u32[]): void;
declare function subscript_typegpu_render_pass_encoder_set_vertex_buffer(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, slot: u32, buffer: SubscriptTypegpuBuffer | null, offset: u64, size: u64): void;
declare function subscript_typegpu_render_pass_encoder_set_index_buffer(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, buffer: SubscriptTypegpuBuffer, format: GPUIndexFormat, offset: u64, size: u64): void;
declare function subscript_typegpu_render_pass_encoder_draw(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32): void;
declare function subscript_typegpu_render_pass_encoder_draw_indexed(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, indexCount: u32, instanceCount: u32, firstIndex: u32, baseVertex: i32, firstInstance: u32): void;
declare function subscript_typegpu_render_pass_encoder_draw_indirect(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, indirectBuffer: SubscriptTypegpuBuffer, indirectOffset: u64): void;
declare function subscript_typegpu_render_pass_encoder_draw_indexed_indirect(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, indirectBuffer: SubscriptTypegpuBuffer, indirectOffset: u64): void;
declare function subscript_typegpu_render_pass_encoder_set_viewport(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, x: f32, y: f32, width: f32, height: f32, minDepth: f32, maxDepth: f32): void;
declare function subscript_typegpu_render_pass_encoder_set_scissor_rect(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, x: u32, y: u32, width: u32, height: u32): void;
declare function subscript_typegpu_render_pass_encoder_set_blend_constant(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, color: SubscriptTypegpuColor | null): void;
declare function subscript_typegpu_render_pass_encoder_set_stencil_reference(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, reference: u32): void;
declare function subscript_typegpu_render_pass_encoder_begin_occlusion_query(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, queryIndex: u32): void;
declare function subscript_typegpu_render_pass_encoder_end_occlusion_query(renderPassEncoder: SubscriptTypegpuRenderPassEncoder): void;
declare function subscript_typegpu_render_pass_encoder_execute_bundles(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, bundles: SubscriptTypegpuRenderBundle[]): void;
declare function subscript_typegpu_render_pass_encoder_insert_debug_marker(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, markerLabel: string): void;
declare function subscript_typegpu_render_pass_encoder_push_debug_group(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, groupLabel: string): void;
declare function subscript_typegpu_render_pass_encoder_pop_debug_group(renderPassEncoder: SubscriptTypegpuRenderPassEncoder): void;
declare function subscript_typegpu_render_pass_encoder_end(renderPassEncoder: SubscriptTypegpuRenderPassEncoder): void;
declare function subscript_typegpu_render_pass_encoder_set_label(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, label: string): void;
declare function subscript_typegpu_command_buffer_set_label(commandBuffer: SubscriptTypegpuCommandBuffer, label: string): void;
declare function subscript_typegpu_render_bundle_encoder_set_pipeline(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, pipeline: SubscriptTypegpuRenderPipeline): void;
declare function subscript_typegpu_render_bundle_encoder_set_bind_group(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, groupIndex: u32, group: SubscriptTypegpuBindGroup | null, dynamicOffsets: u32[]): void;
declare function subscript_typegpu_render_bundle_encoder_set_vertex_buffer(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, slot: u32, buffer: SubscriptTypegpuBuffer | null, offset: u64, size: u64): void;
declare function subscript_typegpu_render_bundle_encoder_set_index_buffer(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, buffer: SubscriptTypegpuBuffer, format: GPUIndexFormat, offset: u64, size: u64): void;
declare function subscript_typegpu_render_bundle_encoder_draw(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32): void;
declare function subscript_typegpu_render_bundle_encoder_draw_indexed(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, indexCount: u32, instanceCount: u32, firstIndex: u32, baseVertex: i32, firstInstance: u32): void;
declare function subscript_typegpu_render_bundle_encoder_draw_indirect(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, indirectBuffer: SubscriptTypegpuBuffer, indirectOffset: u64): void;
declare function subscript_typegpu_render_bundle_encoder_draw_indexed_indirect(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, indirectBuffer: SubscriptTypegpuBuffer, indirectOffset: u64): void;
declare function subscript_typegpu_render_bundle_encoder_insert_debug_marker(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, markerLabel: string): void;
declare function subscript_typegpu_render_bundle_encoder_push_debug_group(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, groupLabel: string): void;
declare function subscript_typegpu_render_bundle_encoder_pop_debug_group(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder): void;
declare function subscript_typegpu_render_bundle_encoder_finish(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, descriptor: SubscriptTypegpuRenderBundleDescriptor | null): SubscriptTypegpuRenderBundle;
declare function subscript_typegpu_render_bundle_encoder_set_label(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, label: string): void;
declare function subscript_typegpu_render_bundle_set_label(renderBundle: SubscriptTypegpuRenderBundle, label: string): void;
declare function subscript_typegpu_query_set_get_type(querySet: SubscriptTypegpuQuerySet): GPUQueryType;
declare function subscript_typegpu_query_set_get_count(querySet: SubscriptTypegpuQuerySet): u32;
declare function subscript_typegpu_query_set_destroy(querySet: SubscriptTypegpuQuerySet): void;
declare function subscript_typegpu_query_set_set_label(querySet: SubscriptTypegpuQuerySet, label: string): void;
declare function subscript_typegpu_query_set_release(querySet: SubscriptTypegpuQuerySet): void;
declare function subscript_typegpu_render_bundle_release(renderBundle: SubscriptTypegpuRenderBundle): void;
declare function subscript_typegpu_render_bundle_encoder_release(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder): void;
declare function subscript_typegpu_command_buffer_release(commandBuffer: SubscriptTypegpuCommandBuffer): void;
declare function subscript_typegpu_render_pass_encoder_release(renderPassEncoder: SubscriptTypegpuRenderPassEncoder): void;
declare function subscript_typegpu_compute_pass_encoder_release(computePassEncoder: SubscriptTypegpuComputePassEncoder): void;
declare function subscript_typegpu_command_encoder_release(commandEncoder: SubscriptTypegpuCommandEncoder): void;
declare function subscript_typegpu_render_pipeline_release(renderPipeline: SubscriptTypegpuRenderPipeline): void;
declare function subscript_typegpu_compute_pipeline_release(computePipeline: SubscriptTypegpuComputePipeline): void;
declare function subscript_typegpu_shader_module_release(shaderModule: SubscriptTypegpuShaderModule): void;
declare function subscript_typegpu_pipeline_layout_release(pipelineLayout: SubscriptTypegpuPipelineLayout): void;
declare function subscript_typegpu_bind_group_release(bindGroup: SubscriptTypegpuBindGroup): void;
declare function subscript_typegpu_bind_group_layout_release(bindGroupLayout: SubscriptTypegpuBindGroupLayout): void;
declare function subscript_typegpu_sampler_release(sampler: SubscriptTypegpuSampler): void;
declare function subscript_typegpu_texture_view_release(textureView: SubscriptTypegpuTextureView): void;
declare function subscript_typegpu_texture_release(texture: SubscriptTypegpuTexture): void;
declare function subscript_typegpu_buffer_release(buffer: SubscriptTypegpuBuffer): void;
declare function subscript_typegpu_queue_release(queue: SubscriptTypegpuQueue): void;
declare function subscript_typegpu_device_release(device: SubscriptTypegpuDevice): void;
declare function subscript_typegpu_adapter_release(adapter: SubscriptTypegpuAdapter): void;

type SubscriptTypegpuBufferUsage = u64;
declare const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_NONE = 0;
declare const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_MAP_READ = 1;
declare const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_MAP_WRITE = 2;
declare const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_COPY_SRC = 4;
declare const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_COPY_DST = 8;
declare const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_INDEX = 16;
declare const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_VERTEX = 32;
declare const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_UNIFORM = 64;
declare const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_STORAGE = 128;
declare const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_INDIRECT = 256;
declare const SUBSCRIPT_TYPEGPU_BUFFER_USAGE_QUERY_RESOLVE = 512;

type SubscriptTypegpuMapMode = u64;
declare const SUBSCRIPT_TYPEGPU_MAP_MODE_NONE = 0;
declare const SUBSCRIPT_TYPEGPU_MAP_MODE_READ = 1;
declare const SUBSCRIPT_TYPEGPU_MAP_MODE_WRITE = 2;

type SubscriptTypegpuTextureUsage = u64;
declare const SUBSCRIPT_TYPEGPU_TEXTURE_USAGE_NONE = 0;
declare const SUBSCRIPT_TYPEGPU_TEXTURE_USAGE_COPY_SRC = 1;
declare const SUBSCRIPT_TYPEGPU_TEXTURE_USAGE_COPY_DST = 2;
declare const SUBSCRIPT_TYPEGPU_TEXTURE_USAGE_TEXTURE_BINDING = 4;
declare const SUBSCRIPT_TYPEGPU_TEXTURE_USAGE_STORAGE_BINDING = 8;
declare const SUBSCRIPT_TYPEGPU_TEXTURE_USAGE_RENDER_ATTACHMENT = 16;
declare const SUBSCRIPT_TYPEGPU_TEXTURE_USAGE_TRANSIENT_ATTACHMENT = 32;

type SubscriptTypegpuShaderStage = u64;
declare const SUBSCRIPT_TYPEGPU_SHADER_STAGE_NONE = 0;
declare const SUBSCRIPT_TYPEGPU_SHADER_STAGE_VERTEX = 1;
declare const SUBSCRIPT_TYPEGPU_SHADER_STAGE_FRAGMENT = 2;
declare const SUBSCRIPT_TYPEGPU_SHADER_STAGE_COMPUTE = 4;

type SubscriptTypegpuColorWriteMask = u64;
declare const SUBSCRIPT_TYPEGPU_COLOR_WRITE_MASK_NONE = 0;
declare const SUBSCRIPT_TYPEGPU_COLOR_WRITE_MASK_RED = 1;
declare const SUBSCRIPT_TYPEGPU_COLOR_WRITE_MASK_GREEN = 2;
declare const SUBSCRIPT_TYPEGPU_COLOR_WRITE_MASK_BLUE = 4;
declare const SUBSCRIPT_TYPEGPU_COLOR_WRITE_MASK_ALPHA = 8;
declare const SUBSCRIPT_TYPEGPU_COLOR_WRITE_MASK_ALL = 15;

type SubscriptTypegpuFutureId = u64;
