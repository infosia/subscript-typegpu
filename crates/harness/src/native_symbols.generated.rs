// GENERATED FILE — DO NOT EDIT.
//
// Facade exports emitted from the resolved generator plan.

#![allow(non_snake_case)]

use subscript_typegpu_facade as facade;
use subscript_typegpu_facade::*;

extern "C" fn coverage_0() -> SubscriptTypegpuInstance {
    super::coverage_hit(0);
    facade::subscript_typegpu_create_instance()
}

extern "C" fn coverage_1(instance: SubscriptTypegpuInstance) {
    super::coverage_hit(1);
    facade::subscript_typegpu_instance_process_events(instance)
}

extern "C" fn coverage_2(instance: SubscriptTypegpuInstance) {
    super::coverage_hit(2);
    facade::subscript_typegpu_instance_release(instance)
}

extern "C" fn coverage_3(
    instance: SubscriptTypegpuInstance,
) -> SubscriptTypegpuFutureId {
    super::coverage_hit(3);
    facade::subscript_typegpu_instance_request_adapter(instance)
}

extern "C" fn coverage_4(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId) -> i32 {
    super::coverage_hit(4);
    facade::subscript_typegpu_future_status(instance, future)
}

extern "C" fn coverage_5(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId) {
    super::coverage_hit(5);
    facade::subscript_typegpu_future_drop(instance, future)
}

extern "C" fn coverage_6(
    instance: SubscriptTypegpuInstance,
    future: SubscriptTypegpuFutureId,
) -> SubscriptTypegpuAdapter {
    super::coverage_hit(6);
    facade::subscript_typegpu_request_adapter_take(instance, future)
}

extern "C" fn coverage_7(adapter: SubscriptTypegpuAdapter, out: *mut SubscriptTypegpuLimits) -> i32 {
    super::coverage_hit(7);
    facade::subscript_typegpu_adapter_get_limits(adapter, out)
}

extern "C" fn coverage_8(
    adapter: SubscriptTypegpuAdapter,
    out: *mut SubscriptTypegpuAdapterInfo,
) -> bool {
    super::coverage_hit(8);
    facade::subscript_typegpu_adapter_get_info(adapter, out)
}

extern "C" fn coverage_9(adapter: SubscriptTypegpuAdapter, feature: i32) -> bool {
    super::coverage_hit(9);
    facade::subscript_typegpu_adapter_has_feature(adapter, feature)
}

extern "C" fn coverage_10(
    instance: SubscriptTypegpuInstance,
    adapter: SubscriptTypegpuAdapter,
    descriptor: *const SubscriptTypegpuDeviceDescriptor,
) -> SubscriptTypegpuFutureId {
    super::coverage_hit(10);
    facade::subscript_typegpu_adapter_request_device_with_descriptor(instance, adapter, descriptor)
}

extern "C" fn coverage_11(
    instance: SubscriptTypegpuInstance,
    future: SubscriptTypegpuFutureId,
) -> SubscriptTypegpuDevice {
    super::coverage_hit(11);
    facade::subscript_typegpu_request_device_take(instance, future)
}

extern "C" fn coverage_12(device: SubscriptTypegpuDevice) -> SubscriptTypegpuQueue {
    super::coverage_hit(12);
    facade::subscript_typegpu_device_get_queue(device)
}

extern "C" fn coverage_13(device: SubscriptTypegpuDevice) {
    super::coverage_hit(13);
    facade::subscript_typegpu_device_destroy(device)
}

extern "C" fn coverage_14(device: SubscriptTypegpuDevice, label: SubscriptTypegpuStringView) {
    super::coverage_hit(14);
    facade::subscript_typegpu_device_set_label(device, label)
}

extern "C" fn coverage_15(device: SubscriptTypegpuDevice, filter: i32) {
    super::coverage_hit(15);
    facade::subscript_typegpu_device_push_error_scope(device, filter)
}

extern "C" fn coverage_16(device: SubscriptTypegpuDevice) -> SubscriptTypegpuFutureId {
    super::coverage_hit(16);
    facade::subscript_typegpu_device_pop_error_scope(device)
}

extern "C" fn coverage_17(
    instance: SubscriptTypegpuInstance,
    future: SubscriptTypegpuFutureId,
    out: *mut SubscriptTypegpuErrorRecord,
) -> bool {
    super::coverage_hit(17);
    facade::subscript_typegpu_pop_error_scope_take(instance, future, out)
}

extern "C" fn coverage_18(
    device: SubscriptTypegpuDevice,
    out: *mut SubscriptTypegpuErrorRecord,
) -> bool {
    super::coverage_hit(18);
    facade::subscript_typegpu_device_next_uncaptured_error(device, out)
}

extern "C" fn coverage_19(
    device: SubscriptTypegpuDevice,
    out: *mut SubscriptTypegpuLostRecord,
) -> bool {
    super::coverage_hit(19);
    facade::subscript_typegpu_device_lost_info(device, out)
}

extern "C" fn coverage_20(device: SubscriptTypegpuDevice, out: *mut SubscriptTypegpuLimits) -> i32 {
    super::coverage_hit(20);
    facade::subscript_typegpu_device_get_limits(device, out)
}

extern "C" fn coverage_21(
    device: SubscriptTypegpuDevice,
    out: *mut SubscriptTypegpuAdapterInfo,
) -> bool {
    super::coverage_hit(21);
    facade::subscript_typegpu_device_get_adapter_info(device, out)
}

extern "C" fn coverage_22(device: SubscriptTypegpuDevice, feature: i32) -> bool {
    super::coverage_hit(22);
    facade::subscript_typegpu_device_has_feature(device, feature)
}

extern "C" fn coverage_23(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuBufferDescriptor,
) -> SubscriptTypegpuBuffer {
    super::coverage_hit(23);
    facade::subscript_typegpu_device_create_buffer(device, descriptor)
}

extern "C" fn coverage_24(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuTextureDescriptor,
) -> SubscriptTypegpuTexture {
    super::coverage_hit(24);
    facade::subscript_typegpu_device_create_texture(device, descriptor)
}

extern "C" fn coverage_25(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuSamplerDescriptor,
) -> SubscriptTypegpuSampler {
    super::coverage_hit(25);
    facade::subscript_typegpu_device_create_sampler(device, descriptor)
}

extern "C" fn coverage_26(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuBindGroupLayoutDescriptor,
) -> SubscriptTypegpuBindGroupLayout {
    super::coverage_hit(26);
    facade::subscript_typegpu_device_create_bind_group_layout(device, descriptor)
}

extern "C" fn coverage_27(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuBindGroupDescriptor,
) -> SubscriptTypegpuBindGroup {
    super::coverage_hit(27);
    facade::subscript_typegpu_device_create_bind_group(device, descriptor)
}

extern "C" fn coverage_28(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuPipelineLayoutDescriptor,
) -> SubscriptTypegpuPipelineLayout {
    super::coverage_hit(28);
    facade::subscript_typegpu_device_create_pipeline_layout(device, descriptor)
}

extern "C" fn coverage_29(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuShaderModuleDescriptor,
) -> SubscriptTypegpuShaderModule {
    super::coverage_hit(29);
    facade::subscript_typegpu_device_create_shader_module(device, descriptor)
}

extern "C" fn coverage_30(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuComputePipelineDescriptor,
) -> SubscriptTypegpuComputePipeline {
    super::coverage_hit(30);
    facade::subscript_typegpu_device_create_compute_pipeline(device, descriptor)
}

extern "C" fn coverage_31(
    instance: SubscriptTypegpuInstance,
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuComputePipelineDescriptor,
) -> SubscriptTypegpuFutureId {
    super::coverage_hit(31);
    facade::subscript_typegpu_device_create_compute_pipeline_async_begin(instance, device, descriptor)
}

extern "C" fn coverage_32(
    instance: SubscriptTypegpuInstance,
    future: SubscriptTypegpuFutureId,
) -> SubscriptTypegpuComputePipeline {
    super::coverage_hit(32);
    facade::subscript_typegpu_create_compute_pipeline_async_take(instance, future)
}

extern "C" fn coverage_33(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuRenderPipelineDescriptor,
) -> SubscriptTypegpuRenderPipeline {
    super::coverage_hit(33);
    facade::subscript_typegpu_device_create_render_pipeline(device, descriptor)
}

extern "C" fn coverage_34(
    instance: SubscriptTypegpuInstance,
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuRenderPipelineDescriptor,
) -> SubscriptTypegpuFutureId {
    super::coverage_hit(34);
    facade::subscript_typegpu_device_create_render_pipeline_async_begin(instance, device, descriptor)
}

extern "C" fn coverage_35(
    instance: SubscriptTypegpuInstance,
    future: SubscriptTypegpuFutureId,
) -> SubscriptTypegpuRenderPipeline {
    super::coverage_hit(35);
    facade::subscript_typegpu_create_render_pipeline_async_take(instance, future)
}

extern "C" fn coverage_36(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuCommandEncoderDescriptor,
) -> SubscriptTypegpuCommandEncoder {
    super::coverage_hit(36);
    facade::subscript_typegpu_device_create_command_encoder(device, descriptor)
}

extern "C" fn coverage_37(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuRenderBundleEncoderDescriptor,
) -> SubscriptTypegpuRenderBundleEncoder {
    super::coverage_hit(37);
    facade::subscript_typegpu_device_create_render_bundle_encoder(device, descriptor)
}

extern "C" fn coverage_38(
    device: SubscriptTypegpuDevice,
    descriptor: *const SubscriptTypegpuQuerySetDescriptor,
) -> SubscriptTypegpuQuerySet {
    super::coverage_hit(38);
    facade::subscript_typegpu_device_create_query_set(device, descriptor)
}

extern "C" fn coverage_39(queue: SubscriptTypegpuQueue, commands_count: usize, commands: *const SubscriptTypegpuCommandBuffer) {
    super::coverage_hit(39);
    facade::subscript_typegpu_queue_submit(queue, commands_count, commands)
}

extern "C" fn coverage_40(
    instance: SubscriptTypegpuInstance,
    queue: SubscriptTypegpuQueue,
) -> SubscriptTypegpuFutureId {
    super::coverage_hit(40);
    facade::subscript_typegpu_queue_on_submitted_work_done(instance, queue)
}

extern "C" fn coverage_41(
    queue: SubscriptTypegpuQueue,
    buffer: SubscriptTypegpuBuffer,
    bufferOffset: u64,
    dataCount: usize,
    data: *const u8,
) {
    super::coverage_hit(41);
    facade::subscript_typegpu_queue_write_buffer(queue, buffer, bufferOffset, dataCount, data)
}

extern "C" fn coverage_42(
    queue: SubscriptTypegpuQueue,
    buffer: SubscriptTypegpuBuffer,
    bufferOffsetBytes: u64,
    dataCount: usize,
    data: *const f32,
) {
    super::coverage_hit(42);
    facade::subscript_typegpu_queue_write_buffer_f32(queue, buffer, bufferOffsetBytes, dataCount, data)
}

extern "C" fn coverage_43(
    queue: SubscriptTypegpuQueue,
    dst: *const SubscriptTypegpuTexelCopyTextureInfo,
    layout: *const SubscriptTypegpuTexelCopyBufferLayout,
    extent: *const SubscriptTypegpuExtent3D,
    dataCount: usize,
    data: *const u8,
) {
    super::coverage_hit(43);
    facade::subscript_typegpu_queue_write_texture(queue, dst, layout, extent, dataCount, data)
}

extern "C" fn coverage_44(queue: SubscriptTypegpuQueue, label: SubscriptTypegpuStringView) {
    super::coverage_hit(44);
    facade::subscript_typegpu_queue_set_label(queue, label)
}

extern "C" fn coverage_45(
    buffer: SubscriptTypegpuBuffer,
    mode: u64,
    offset: usize,
    size: usize,
) -> SubscriptTypegpuFutureId {
    super::coverage_hit(45);
    facade::subscript_typegpu_buffer_map_async(buffer, mode, offset, size)
}

extern "C" fn coverage_46(
    buffer: SubscriptTypegpuBuffer,
    offset: usize,
    outCount: usize,
    out: *mut u8,
) -> i32 {
    super::coverage_hit(46);
    facade::subscript_typegpu_buffer_read_mapped_range(buffer, offset, outCount, out)
}

extern "C" fn coverage_47(
    buffer: SubscriptTypegpuBuffer,
    offsetBytes: usize,
    outCount: usize,
    out: *mut f32,
) -> i32 {
    super::coverage_hit(47);
    facade::subscript_typegpu_buffer_read_mapped_range_f32(buffer, offsetBytes, outCount, out)
}

extern "C" fn coverage_48(
    buffer: SubscriptTypegpuBuffer,
    offset: usize,
    dataCount: usize,
    data: *const u8,
) -> i32 {
    super::coverage_hit(48);
    facade::subscript_typegpu_buffer_write_mapped_range(buffer, offset, dataCount, data)
}

extern "C" fn coverage_49(buffer: SubscriptTypegpuBuffer, label: SubscriptTypegpuStringView) {
    super::coverage_hit(49);
    facade::subscript_typegpu_buffer_set_label(buffer, label)
}

extern "C" fn coverage_50(buffer: SubscriptTypegpuBuffer) -> u64 {
    super::coverage_hit(50);
    facade::subscript_typegpu_buffer_get_usage(buffer)
}

extern "C" fn coverage_51(buffer: SubscriptTypegpuBuffer) -> u64 {
    super::coverage_hit(51);
    facade::subscript_typegpu_buffer_get_size(buffer)
}

extern "C" fn coverage_52(buffer: SubscriptTypegpuBuffer) -> i32 {
    super::coverage_hit(52);
    facade::subscript_typegpu_buffer_get_map_state(buffer)
}

extern "C" fn coverage_53(buffer: SubscriptTypegpuBuffer) {
    super::coverage_hit(53);
    facade::subscript_typegpu_buffer_unmap(buffer)
}

extern "C" fn coverage_54(buffer: SubscriptTypegpuBuffer) {
    super::coverage_hit(54);
    facade::subscript_typegpu_buffer_destroy(buffer)
}

extern "C" fn coverage_55(
    texture: SubscriptTypegpuTexture,
    descriptor: *const SubscriptTypegpuTextureViewDescriptor,
) -> SubscriptTypegpuTextureView {
    super::coverage_hit(55);
    facade::subscript_typegpu_texture_create_view(texture, descriptor)
}

extern "C" fn coverage_56(texture: SubscriptTypegpuTexture, label: SubscriptTypegpuStringView) {
    super::coverage_hit(56);
    facade::subscript_typegpu_texture_set_label(texture, label)
}

extern "C" fn coverage_57(texture: SubscriptTypegpuTexture) -> u32 {
    super::coverage_hit(57);
    facade::subscript_typegpu_texture_get_width(texture)
}

extern "C" fn coverage_58(texture: SubscriptTypegpuTexture) -> u32 {
    super::coverage_hit(58);
    facade::subscript_typegpu_texture_get_height(texture)
}

extern "C" fn coverage_59(texture: SubscriptTypegpuTexture) -> u32 {
    super::coverage_hit(59);
    facade::subscript_typegpu_texture_get_depth_or_array_layers(texture)
}

extern "C" fn coverage_60(texture: SubscriptTypegpuTexture) -> u32 {
    super::coverage_hit(60);
    facade::subscript_typegpu_texture_get_mip_level_count(texture)
}

extern "C" fn coverage_61(texture: SubscriptTypegpuTexture) -> u32 {
    super::coverage_hit(61);
    facade::subscript_typegpu_texture_get_sample_count(texture)
}

extern "C" fn coverage_62(texture: SubscriptTypegpuTexture) -> i32 {
    super::coverage_hit(62);
    facade::subscript_typegpu_texture_get_dimension(texture)
}

extern "C" fn coverage_63(texture: SubscriptTypegpuTexture) -> i32 {
    super::coverage_hit(63);
    facade::subscript_typegpu_texture_get_format(texture)
}

extern "C" fn coverage_64(texture: SubscriptTypegpuTexture) -> u64 {
    super::coverage_hit(64);
    facade::subscript_typegpu_texture_get_usage(texture)
}

extern "C" fn coverage_65(texture: SubscriptTypegpuTexture) {
    super::coverage_hit(65);
    facade::subscript_typegpu_texture_destroy(texture)
}

extern "C" fn coverage_66(textureView: SubscriptTypegpuTextureView, label: SubscriptTypegpuStringView) {
    super::coverage_hit(66);
    facade::subscript_typegpu_texture_view_set_label(textureView, label)
}

extern "C" fn coverage_67(sampler: SubscriptTypegpuSampler, label: SubscriptTypegpuStringView) {
    super::coverage_hit(67);
    facade::subscript_typegpu_sampler_set_label(sampler, label)
}

extern "C" fn coverage_68(bindGroupLayout: SubscriptTypegpuBindGroupLayout, label: SubscriptTypegpuStringView) {
    super::coverage_hit(68);
    facade::subscript_typegpu_bind_group_layout_set_label(bindGroupLayout, label)
}

extern "C" fn coverage_69(bindGroup: SubscriptTypegpuBindGroup, label: SubscriptTypegpuStringView) {
    super::coverage_hit(69);
    facade::subscript_typegpu_bind_group_set_label(bindGroup, label)
}

extern "C" fn coverage_70(pipelineLayout: SubscriptTypegpuPipelineLayout, label: SubscriptTypegpuStringView) {
    super::coverage_hit(70);
    facade::subscript_typegpu_pipeline_layout_set_label(pipelineLayout, label)
}

extern "C" fn coverage_71(shaderModule: SubscriptTypegpuShaderModule, label: SubscriptTypegpuStringView) {
    super::coverage_hit(71);
    facade::subscript_typegpu_shader_module_set_label(shaderModule, label)
}

extern "C" fn coverage_72(computePipeline: SubscriptTypegpuComputePipeline, groupIndex: u32) -> SubscriptTypegpuBindGroupLayout {
    super::coverage_hit(72);
    facade::subscript_typegpu_compute_pipeline_get_bind_group_layout(computePipeline, groupIndex)
}

extern "C" fn coverage_73(computePipeline: SubscriptTypegpuComputePipeline, label: SubscriptTypegpuStringView) {
    super::coverage_hit(73);
    facade::subscript_typegpu_compute_pipeline_set_label(computePipeline, label)
}

extern "C" fn coverage_74(renderPipeline: SubscriptTypegpuRenderPipeline, groupIndex: u32) -> SubscriptTypegpuBindGroupLayout {
    super::coverage_hit(74);
    facade::subscript_typegpu_render_pipeline_get_bind_group_layout(renderPipeline, groupIndex)
}

extern "C" fn coverage_75(renderPipeline: SubscriptTypegpuRenderPipeline, label: SubscriptTypegpuStringView) {
    super::coverage_hit(75);
    facade::subscript_typegpu_render_pipeline_set_label(renderPipeline, label)
}

extern "C" fn coverage_76(
    commandEncoder: SubscriptTypegpuCommandEncoder,
    descriptor: *const SubscriptTypegpuCommandBufferDescriptor,
) -> SubscriptTypegpuCommandBuffer {
    super::coverage_hit(76);
    facade::subscript_typegpu_command_encoder_finish(commandEncoder, descriptor)
}

extern "C" fn coverage_77(
    commandEncoder: SubscriptTypegpuCommandEncoder,
    descriptor: *const SubscriptTypegpuComputePassDescriptor,
) -> SubscriptTypegpuComputePassEncoder {
    super::coverage_hit(77);
    facade::subscript_typegpu_command_encoder_begin_compute_pass(commandEncoder, descriptor)
}

extern "C" fn coverage_78(
    commandEncoder: SubscriptTypegpuCommandEncoder,
    descriptor: *const SubscriptTypegpuRenderPassDescriptor,
) -> SubscriptTypegpuRenderPassEncoder {
    super::coverage_hit(78);
    facade::subscript_typegpu_command_encoder_begin_render_pass(commandEncoder, descriptor)
}

extern "C" fn coverage_79(commandEncoder: SubscriptTypegpuCommandEncoder, source: SubscriptTypegpuBuffer, sourceOffset: u64, destination: SubscriptTypegpuBuffer, destinationOffset: u64, size: u64) {
    super::coverage_hit(79);
    facade::subscript_typegpu_command_encoder_copy_buffer_to_buffer(commandEncoder, source, sourceOffset, destination, destinationOffset, size)
}

extern "C" fn coverage_80(commandEncoder: SubscriptTypegpuCommandEncoder, source: *const SubscriptTypegpuTexelCopyBufferInfo, destination: *const SubscriptTypegpuTexelCopyTextureInfo, copySize: *const SubscriptTypegpuExtent3D) {
    super::coverage_hit(80);
    facade::subscript_typegpu_command_encoder_copy_buffer_to_texture(commandEncoder, source, destination, copySize)
}

extern "C" fn coverage_81(commandEncoder: SubscriptTypegpuCommandEncoder, source: *const SubscriptTypegpuTexelCopyTextureInfo, destination: *const SubscriptTypegpuTexelCopyBufferInfo, copySize: *const SubscriptTypegpuExtent3D) {
    super::coverage_hit(81);
    facade::subscript_typegpu_command_encoder_copy_texture_to_buffer(commandEncoder, source, destination, copySize)
}

extern "C" fn coverage_82(commandEncoder: SubscriptTypegpuCommandEncoder, source: *const SubscriptTypegpuTexelCopyTextureInfo, destination: *const SubscriptTypegpuTexelCopyTextureInfo, copySize: *const SubscriptTypegpuExtent3D) {
    super::coverage_hit(82);
    facade::subscript_typegpu_command_encoder_copy_texture_to_texture(commandEncoder, source, destination, copySize)
}

extern "C" fn coverage_83(commandEncoder: SubscriptTypegpuCommandEncoder, buffer: SubscriptTypegpuBuffer, offset: u64, size: u64) {
    super::coverage_hit(83);
    facade::subscript_typegpu_command_encoder_clear_buffer(commandEncoder, buffer, offset, size)
}

extern "C" fn coverage_84(commandEncoder: SubscriptTypegpuCommandEncoder, querySet: SubscriptTypegpuQuerySet, firstQuery: u32, queryCount: u32, destination: SubscriptTypegpuBuffer, destinationOffset: u64) {
    super::coverage_hit(84);
    facade::subscript_typegpu_command_encoder_resolve_query_set(commandEncoder, querySet, firstQuery, queryCount, destination, destinationOffset)
}

extern "C" fn coverage_85(commandEncoder: SubscriptTypegpuCommandEncoder, markerLabel: SubscriptTypegpuStringView) {
    super::coverage_hit(85);
    facade::subscript_typegpu_command_encoder_insert_debug_marker(commandEncoder, markerLabel)
}

extern "C" fn coverage_86(commandEncoder: SubscriptTypegpuCommandEncoder, groupLabel: SubscriptTypegpuStringView) {
    super::coverage_hit(86);
    facade::subscript_typegpu_command_encoder_push_debug_group(commandEncoder, groupLabel)
}

extern "C" fn coverage_87(commandEncoder: SubscriptTypegpuCommandEncoder) {
    super::coverage_hit(87);
    facade::subscript_typegpu_command_encoder_pop_debug_group(commandEncoder)
}

extern "C" fn coverage_88(commandEncoder: SubscriptTypegpuCommandEncoder, label: SubscriptTypegpuStringView) {
    super::coverage_hit(88);
    facade::subscript_typegpu_command_encoder_set_label(commandEncoder, label)
}

extern "C" fn coverage_89(computePassEncoder: SubscriptTypegpuComputePassEncoder, pipeline: SubscriptTypegpuComputePipeline) {
    super::coverage_hit(89);
    facade::subscript_typegpu_compute_pass_encoder_set_pipeline(computePassEncoder, pipeline)
}

extern "C" fn coverage_90(computePassEncoder: SubscriptTypegpuComputePassEncoder, groupIndex: u32, group: SubscriptTypegpuBindGroup, dynamicOffsets_count: usize, dynamicOffsets: *const u32) {
    super::coverage_hit(90);
    facade::subscript_typegpu_compute_pass_encoder_set_bind_group(computePassEncoder, groupIndex, group, dynamicOffsets_count, dynamicOffsets)
}

extern "C" fn coverage_91(computePassEncoder: SubscriptTypegpuComputePassEncoder, workgroupCountX: u32, workgroupCountY: u32, workgroupCountZ: u32) {
    super::coverage_hit(91);
    facade::subscript_typegpu_compute_pass_encoder_dispatch_workgroups(computePassEncoder, workgroupCountX, workgroupCountY, workgroupCountZ)
}

extern "C" fn coverage_92(computePassEncoder: SubscriptTypegpuComputePassEncoder, indirectBuffer: SubscriptTypegpuBuffer, indirectOffset: u64) {
    super::coverage_hit(92);
    facade::subscript_typegpu_compute_pass_encoder_dispatch_workgroups_indirect(computePassEncoder, indirectBuffer, indirectOffset)
}

extern "C" fn coverage_93(computePassEncoder: SubscriptTypegpuComputePassEncoder, markerLabel: SubscriptTypegpuStringView) {
    super::coverage_hit(93);
    facade::subscript_typegpu_compute_pass_encoder_insert_debug_marker(computePassEncoder, markerLabel)
}

extern "C" fn coverage_94(computePassEncoder: SubscriptTypegpuComputePassEncoder, groupLabel: SubscriptTypegpuStringView) {
    super::coverage_hit(94);
    facade::subscript_typegpu_compute_pass_encoder_push_debug_group(computePassEncoder, groupLabel)
}

extern "C" fn coverage_95(computePassEncoder: SubscriptTypegpuComputePassEncoder) {
    super::coverage_hit(95);
    facade::subscript_typegpu_compute_pass_encoder_pop_debug_group(computePassEncoder)
}

extern "C" fn coverage_96(computePassEncoder: SubscriptTypegpuComputePassEncoder) {
    super::coverage_hit(96);
    facade::subscript_typegpu_compute_pass_encoder_end(computePassEncoder)
}

extern "C" fn coverage_97(computePassEncoder: SubscriptTypegpuComputePassEncoder, label: SubscriptTypegpuStringView) {
    super::coverage_hit(97);
    facade::subscript_typegpu_compute_pass_encoder_set_label(computePassEncoder, label)
}

extern "C" fn coverage_98(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, pipeline: SubscriptTypegpuRenderPipeline) {
    super::coverage_hit(98);
    facade::subscript_typegpu_render_pass_encoder_set_pipeline(renderPassEncoder, pipeline)
}

extern "C" fn coverage_99(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, groupIndex: u32, group: SubscriptTypegpuBindGroup, dynamicOffsets_count: usize, dynamicOffsets: *const u32) {
    super::coverage_hit(99);
    facade::subscript_typegpu_render_pass_encoder_set_bind_group(renderPassEncoder, groupIndex, group, dynamicOffsets_count, dynamicOffsets)
}

extern "C" fn coverage_100(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, slot: u32, buffer: SubscriptTypegpuBuffer, offset: u64, size: u64) {
    super::coverage_hit(100);
    facade::subscript_typegpu_render_pass_encoder_set_vertex_buffer(renderPassEncoder, slot, buffer, offset, size)
}

extern "C" fn coverage_101(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, buffer: SubscriptTypegpuBuffer, format: i32, offset: u64, size: u64) {
    super::coverage_hit(101);
    facade::subscript_typegpu_render_pass_encoder_set_index_buffer(renderPassEncoder, buffer, format, offset, size)
}

extern "C" fn coverage_102(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32) {
    super::coverage_hit(102);
    facade::subscript_typegpu_render_pass_encoder_draw(renderPassEncoder, vertexCount, instanceCount, firstVertex, firstInstance)
}

extern "C" fn coverage_103(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, indexCount: u32, instanceCount: u32, firstIndex: u32, baseVertex: i32, firstInstance: u32) {
    super::coverage_hit(103);
    facade::subscript_typegpu_render_pass_encoder_draw_indexed(renderPassEncoder, indexCount, instanceCount, firstIndex, baseVertex, firstInstance)
}

extern "C" fn coverage_104(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, indirectBuffer: SubscriptTypegpuBuffer, indirectOffset: u64) {
    super::coverage_hit(104);
    facade::subscript_typegpu_render_pass_encoder_draw_indirect(renderPassEncoder, indirectBuffer, indirectOffset)
}

extern "C" fn coverage_105(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, indirectBuffer: SubscriptTypegpuBuffer, indirectOffset: u64) {
    super::coverage_hit(105);
    facade::subscript_typegpu_render_pass_encoder_draw_indexed_indirect(renderPassEncoder, indirectBuffer, indirectOffset)
}

extern "C" fn coverage_106(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, x: f32, y: f32, width: f32, height: f32, minDepth: f32, maxDepth: f32) {
    super::coverage_hit(106);
    facade::subscript_typegpu_render_pass_encoder_set_viewport(renderPassEncoder, x, y, width, height, minDepth, maxDepth)
}

extern "C" fn coverage_107(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, x: u32, y: u32, width: u32, height: u32) {
    super::coverage_hit(107);
    facade::subscript_typegpu_render_pass_encoder_set_scissor_rect(renderPassEncoder, x, y, width, height)
}

extern "C" fn coverage_108(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, color: *const SubscriptTypegpuColor) {
    super::coverage_hit(108);
    facade::subscript_typegpu_render_pass_encoder_set_blend_constant(renderPassEncoder, color)
}

extern "C" fn coverage_109(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, reference: u32) {
    super::coverage_hit(109);
    facade::subscript_typegpu_render_pass_encoder_set_stencil_reference(renderPassEncoder, reference)
}

extern "C" fn coverage_110(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, queryIndex: u32) {
    super::coverage_hit(110);
    facade::subscript_typegpu_render_pass_encoder_begin_occlusion_query(renderPassEncoder, queryIndex)
}

extern "C" fn coverage_111(renderPassEncoder: SubscriptTypegpuRenderPassEncoder) {
    super::coverage_hit(111);
    facade::subscript_typegpu_render_pass_encoder_end_occlusion_query(renderPassEncoder)
}

extern "C" fn coverage_112(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, bundles_count: usize, bundles: *const SubscriptTypegpuRenderBundle) {
    super::coverage_hit(112);
    facade::subscript_typegpu_render_pass_encoder_execute_bundles(renderPassEncoder, bundles_count, bundles)
}

extern "C" fn coverage_113(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, markerLabel: SubscriptTypegpuStringView) {
    super::coverage_hit(113);
    facade::subscript_typegpu_render_pass_encoder_insert_debug_marker(renderPassEncoder, markerLabel)
}

extern "C" fn coverage_114(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, groupLabel: SubscriptTypegpuStringView) {
    super::coverage_hit(114);
    facade::subscript_typegpu_render_pass_encoder_push_debug_group(renderPassEncoder, groupLabel)
}

extern "C" fn coverage_115(renderPassEncoder: SubscriptTypegpuRenderPassEncoder) {
    super::coverage_hit(115);
    facade::subscript_typegpu_render_pass_encoder_pop_debug_group(renderPassEncoder)
}

extern "C" fn coverage_116(renderPassEncoder: SubscriptTypegpuRenderPassEncoder) {
    super::coverage_hit(116);
    facade::subscript_typegpu_render_pass_encoder_end(renderPassEncoder)
}

extern "C" fn coverage_117(renderPassEncoder: SubscriptTypegpuRenderPassEncoder, label: SubscriptTypegpuStringView) {
    super::coverage_hit(117);
    facade::subscript_typegpu_render_pass_encoder_set_label(renderPassEncoder, label)
}

extern "C" fn coverage_118(commandBuffer: SubscriptTypegpuCommandBuffer, label: SubscriptTypegpuStringView) {
    super::coverage_hit(118);
    facade::subscript_typegpu_command_buffer_set_label(commandBuffer, label)
}

extern "C" fn coverage_119(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, pipeline: SubscriptTypegpuRenderPipeline) {
    super::coverage_hit(119);
    facade::subscript_typegpu_render_bundle_encoder_set_pipeline(renderBundleEncoder, pipeline)
}

extern "C" fn coverage_120(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, groupIndex: u32, group: SubscriptTypegpuBindGroup, dynamicOffsets_count: usize, dynamicOffsets: *const u32) {
    super::coverage_hit(120);
    facade::subscript_typegpu_render_bundle_encoder_set_bind_group(renderBundleEncoder, groupIndex, group, dynamicOffsets_count, dynamicOffsets)
}

extern "C" fn coverage_121(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, slot: u32, buffer: SubscriptTypegpuBuffer, offset: u64, size: u64) {
    super::coverage_hit(121);
    facade::subscript_typegpu_render_bundle_encoder_set_vertex_buffer(renderBundleEncoder, slot, buffer, offset, size)
}

extern "C" fn coverage_122(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, buffer: SubscriptTypegpuBuffer, format: i32, offset: u64, size: u64) {
    super::coverage_hit(122);
    facade::subscript_typegpu_render_bundle_encoder_set_index_buffer(renderBundleEncoder, buffer, format, offset, size)
}

extern "C" fn coverage_123(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32) {
    super::coverage_hit(123);
    facade::subscript_typegpu_render_bundle_encoder_draw(renderBundleEncoder, vertexCount, instanceCount, firstVertex, firstInstance)
}

extern "C" fn coverage_124(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, indexCount: u32, instanceCount: u32, firstIndex: u32, baseVertex: i32, firstInstance: u32) {
    super::coverage_hit(124);
    facade::subscript_typegpu_render_bundle_encoder_draw_indexed(renderBundleEncoder, indexCount, instanceCount, firstIndex, baseVertex, firstInstance)
}

extern "C" fn coverage_125(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, indirectBuffer: SubscriptTypegpuBuffer, indirectOffset: u64) {
    super::coverage_hit(125);
    facade::subscript_typegpu_render_bundle_encoder_draw_indirect(renderBundleEncoder, indirectBuffer, indirectOffset)
}

extern "C" fn coverage_126(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, indirectBuffer: SubscriptTypegpuBuffer, indirectOffset: u64) {
    super::coverage_hit(126);
    facade::subscript_typegpu_render_bundle_encoder_draw_indexed_indirect(renderBundleEncoder, indirectBuffer, indirectOffset)
}

extern "C" fn coverage_127(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, markerLabel: SubscriptTypegpuStringView) {
    super::coverage_hit(127);
    facade::subscript_typegpu_render_bundle_encoder_insert_debug_marker(renderBundleEncoder, markerLabel)
}

extern "C" fn coverage_128(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, groupLabel: SubscriptTypegpuStringView) {
    super::coverage_hit(128);
    facade::subscript_typegpu_render_bundle_encoder_push_debug_group(renderBundleEncoder, groupLabel)
}

extern "C" fn coverage_129(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder) {
    super::coverage_hit(129);
    facade::subscript_typegpu_render_bundle_encoder_pop_debug_group(renderBundleEncoder)
}

extern "C" fn coverage_130(
    renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder,
    descriptor: *const SubscriptTypegpuRenderBundleDescriptor,
) -> SubscriptTypegpuRenderBundle {
    super::coverage_hit(130);
    facade::subscript_typegpu_render_bundle_encoder_finish(renderBundleEncoder, descriptor)
}

extern "C" fn coverage_131(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder, label: SubscriptTypegpuStringView) {
    super::coverage_hit(131);
    facade::subscript_typegpu_render_bundle_encoder_set_label(renderBundleEncoder, label)
}

extern "C" fn coverage_132(renderBundle: SubscriptTypegpuRenderBundle, label: SubscriptTypegpuStringView) {
    super::coverage_hit(132);
    facade::subscript_typegpu_render_bundle_set_label(renderBundle, label)
}

extern "C" fn coverage_133(querySet: SubscriptTypegpuQuerySet) -> i32 {
    super::coverage_hit(133);
    facade::subscript_typegpu_query_set_get_type(querySet)
}

extern "C" fn coverage_134(querySet: SubscriptTypegpuQuerySet) -> u32 {
    super::coverage_hit(134);
    facade::subscript_typegpu_query_set_get_count(querySet)
}

extern "C" fn coverage_135(querySet: SubscriptTypegpuQuerySet) {
    super::coverage_hit(135);
    facade::subscript_typegpu_query_set_destroy(querySet)
}

extern "C" fn coverage_136(querySet: SubscriptTypegpuQuerySet, label: SubscriptTypegpuStringView) {
    super::coverage_hit(136);
    facade::subscript_typegpu_query_set_set_label(querySet, label)
}

extern "C" fn coverage_137(querySet: SubscriptTypegpuQuerySet) {
    super::coverage_hit(137);
    facade::subscript_typegpu_query_set_release(querySet)
}

extern "C" fn coverage_138(renderBundle: SubscriptTypegpuRenderBundle) {
    super::coverage_hit(138);
    facade::subscript_typegpu_render_bundle_release(renderBundle)
}

extern "C" fn coverage_139(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder) {
    super::coverage_hit(139);
    facade::subscript_typegpu_render_bundle_encoder_release(renderBundleEncoder)
}

extern "C" fn coverage_140(commandBuffer: SubscriptTypegpuCommandBuffer) {
    super::coverage_hit(140);
    facade::subscript_typegpu_command_buffer_release(commandBuffer)
}

extern "C" fn coverage_141(renderPassEncoder: SubscriptTypegpuRenderPassEncoder) {
    super::coverage_hit(141);
    facade::subscript_typegpu_render_pass_encoder_release(renderPassEncoder)
}

extern "C" fn coverage_142(computePassEncoder: SubscriptTypegpuComputePassEncoder) {
    super::coverage_hit(142);
    facade::subscript_typegpu_compute_pass_encoder_release(computePassEncoder)
}

extern "C" fn coverage_143(commandEncoder: SubscriptTypegpuCommandEncoder) {
    super::coverage_hit(143);
    facade::subscript_typegpu_command_encoder_release(commandEncoder)
}

extern "C" fn coverage_144(renderPipeline: SubscriptTypegpuRenderPipeline) {
    super::coverage_hit(144);
    facade::subscript_typegpu_render_pipeline_release(renderPipeline)
}

extern "C" fn coverage_145(computePipeline: SubscriptTypegpuComputePipeline) {
    super::coverage_hit(145);
    facade::subscript_typegpu_compute_pipeline_release(computePipeline)
}

extern "C" fn coverage_146(shaderModule: SubscriptTypegpuShaderModule) {
    super::coverage_hit(146);
    facade::subscript_typegpu_shader_module_release(shaderModule)
}

extern "C" fn coverage_147(pipelineLayout: SubscriptTypegpuPipelineLayout) {
    super::coverage_hit(147);
    facade::subscript_typegpu_pipeline_layout_release(pipelineLayout)
}

extern "C" fn coverage_148(bindGroup: SubscriptTypegpuBindGroup) {
    super::coverage_hit(148);
    facade::subscript_typegpu_bind_group_release(bindGroup)
}

extern "C" fn coverage_149(bindGroupLayout: SubscriptTypegpuBindGroupLayout) {
    super::coverage_hit(149);
    facade::subscript_typegpu_bind_group_layout_release(bindGroupLayout)
}

extern "C" fn coverage_150(sampler: SubscriptTypegpuSampler) {
    super::coverage_hit(150);
    facade::subscript_typegpu_sampler_release(sampler)
}

extern "C" fn coverage_151(textureView: SubscriptTypegpuTextureView) {
    super::coverage_hit(151);
    facade::subscript_typegpu_texture_view_release(textureView)
}

extern "C" fn coverage_152(texture: SubscriptTypegpuTexture) {
    super::coverage_hit(152);
    facade::subscript_typegpu_texture_release(texture)
}

extern "C" fn coverage_153(buffer: SubscriptTypegpuBuffer) {
    super::coverage_hit(153);
    facade::subscript_typegpu_buffer_release(buffer)
}

extern "C" fn coverage_154(queue: SubscriptTypegpuQueue) {
    super::coverage_hit(154);
    facade::subscript_typegpu_queue_release(queue)
}

extern "C" fn coverage_155(device: SubscriptTypegpuDevice) {
    super::coverage_hit(155);
    facade::subscript_typegpu_device_release(device)
}

extern "C" fn coverage_156(adapter: SubscriptTypegpuAdapter) {
    super::coverage_hit(156);
    facade::subscript_typegpu_adapter_release(adapter)
}

pub fn facade_export_names() -> &'static [&'static str] {
    &[
        "subscript_typegpu_create_instance",
        "subscript_typegpu_instance_process_events",
        "subscript_typegpu_instance_release",
        "subscript_typegpu_instance_request_adapter",
        "subscript_typegpu_future_status",
        "subscript_typegpu_future_drop",
        "subscript_typegpu_request_adapter_take",
        "subscript_typegpu_adapter_get_limits",
        "subscript_typegpu_adapter_get_info",
        "subscript_typegpu_adapter_has_feature",
        "subscript_typegpu_adapter_request_device_with_descriptor",
        "subscript_typegpu_request_device_take",
        "subscript_typegpu_device_get_queue",
        "subscript_typegpu_device_destroy",
        "subscript_typegpu_device_set_label",
        "subscript_typegpu_device_push_error_scope",
        "subscript_typegpu_device_pop_error_scope",
        "subscript_typegpu_pop_error_scope_take",
        "subscript_typegpu_device_next_uncaptured_error",
        "subscript_typegpu_device_lost_info",
        "subscript_typegpu_device_get_limits",
        "subscript_typegpu_device_get_adapter_info",
        "subscript_typegpu_device_has_feature",
        "subscript_typegpu_device_create_buffer",
        "subscript_typegpu_device_create_texture",
        "subscript_typegpu_device_create_sampler",
        "subscript_typegpu_device_create_bind_group_layout",
        "subscript_typegpu_device_create_bind_group",
        "subscript_typegpu_device_create_pipeline_layout",
        "subscript_typegpu_device_create_shader_module",
        "subscript_typegpu_device_create_compute_pipeline",
        "subscript_typegpu_device_create_compute_pipeline_async_begin",
        "subscript_typegpu_create_compute_pipeline_async_take",
        "subscript_typegpu_device_create_render_pipeline",
        "subscript_typegpu_device_create_render_pipeline_async_begin",
        "subscript_typegpu_create_render_pipeline_async_take",
        "subscript_typegpu_device_create_command_encoder",
        "subscript_typegpu_device_create_render_bundle_encoder",
        "subscript_typegpu_device_create_query_set",
        "subscript_typegpu_queue_submit",
        "subscript_typegpu_queue_on_submitted_work_done",
        "subscript_typegpu_queue_write_buffer",
        "subscript_typegpu_queue_write_buffer_f32",
        "subscript_typegpu_queue_write_texture",
        "subscript_typegpu_queue_set_label",
        "subscript_typegpu_buffer_map_async",
        "subscript_typegpu_buffer_read_mapped_range",
        "subscript_typegpu_buffer_read_mapped_range_f32",
        "subscript_typegpu_buffer_write_mapped_range",
        "subscript_typegpu_buffer_set_label",
        "subscript_typegpu_buffer_get_usage",
        "subscript_typegpu_buffer_get_size",
        "subscript_typegpu_buffer_get_map_state",
        "subscript_typegpu_buffer_unmap",
        "subscript_typegpu_buffer_destroy",
        "subscript_typegpu_texture_create_view",
        "subscript_typegpu_texture_set_label",
        "subscript_typegpu_texture_get_width",
        "subscript_typegpu_texture_get_height",
        "subscript_typegpu_texture_get_depth_or_array_layers",
        "subscript_typegpu_texture_get_mip_level_count",
        "subscript_typegpu_texture_get_sample_count",
        "subscript_typegpu_texture_get_dimension",
        "subscript_typegpu_texture_get_format",
        "subscript_typegpu_texture_get_usage",
        "subscript_typegpu_texture_destroy",
        "subscript_typegpu_texture_view_set_label",
        "subscript_typegpu_sampler_set_label",
        "subscript_typegpu_bind_group_layout_set_label",
        "subscript_typegpu_bind_group_set_label",
        "subscript_typegpu_pipeline_layout_set_label",
        "subscript_typegpu_shader_module_set_label",
        "subscript_typegpu_compute_pipeline_get_bind_group_layout",
        "subscript_typegpu_compute_pipeline_set_label",
        "subscript_typegpu_render_pipeline_get_bind_group_layout",
        "subscript_typegpu_render_pipeline_set_label",
        "subscript_typegpu_command_encoder_finish",
        "subscript_typegpu_command_encoder_begin_compute_pass",
        "subscript_typegpu_command_encoder_begin_render_pass",
        "subscript_typegpu_command_encoder_copy_buffer_to_buffer",
        "subscript_typegpu_command_encoder_copy_buffer_to_texture",
        "subscript_typegpu_command_encoder_copy_texture_to_buffer",
        "subscript_typegpu_command_encoder_copy_texture_to_texture",
        "subscript_typegpu_command_encoder_clear_buffer",
        "subscript_typegpu_command_encoder_resolve_query_set",
        "subscript_typegpu_command_encoder_insert_debug_marker",
        "subscript_typegpu_command_encoder_push_debug_group",
        "subscript_typegpu_command_encoder_pop_debug_group",
        "subscript_typegpu_command_encoder_set_label",
        "subscript_typegpu_compute_pass_encoder_set_pipeline",
        "subscript_typegpu_compute_pass_encoder_set_bind_group",
        "subscript_typegpu_compute_pass_encoder_dispatch_workgroups",
        "subscript_typegpu_compute_pass_encoder_dispatch_workgroups_indirect",
        "subscript_typegpu_compute_pass_encoder_insert_debug_marker",
        "subscript_typegpu_compute_pass_encoder_push_debug_group",
        "subscript_typegpu_compute_pass_encoder_pop_debug_group",
        "subscript_typegpu_compute_pass_encoder_end",
        "subscript_typegpu_compute_pass_encoder_set_label",
        "subscript_typegpu_render_pass_encoder_set_pipeline",
        "subscript_typegpu_render_pass_encoder_set_bind_group",
        "subscript_typegpu_render_pass_encoder_set_vertex_buffer",
        "subscript_typegpu_render_pass_encoder_set_index_buffer",
        "subscript_typegpu_render_pass_encoder_draw",
        "subscript_typegpu_render_pass_encoder_draw_indexed",
        "subscript_typegpu_render_pass_encoder_draw_indirect",
        "subscript_typegpu_render_pass_encoder_draw_indexed_indirect",
        "subscript_typegpu_render_pass_encoder_set_viewport",
        "subscript_typegpu_render_pass_encoder_set_scissor_rect",
        "subscript_typegpu_render_pass_encoder_set_blend_constant",
        "subscript_typegpu_render_pass_encoder_set_stencil_reference",
        "subscript_typegpu_render_pass_encoder_begin_occlusion_query",
        "subscript_typegpu_render_pass_encoder_end_occlusion_query",
        "subscript_typegpu_render_pass_encoder_execute_bundles",
        "subscript_typegpu_render_pass_encoder_insert_debug_marker",
        "subscript_typegpu_render_pass_encoder_push_debug_group",
        "subscript_typegpu_render_pass_encoder_pop_debug_group",
        "subscript_typegpu_render_pass_encoder_end",
        "subscript_typegpu_render_pass_encoder_set_label",
        "subscript_typegpu_command_buffer_set_label",
        "subscript_typegpu_render_bundle_encoder_set_pipeline",
        "subscript_typegpu_render_bundle_encoder_set_bind_group",
        "subscript_typegpu_render_bundle_encoder_set_vertex_buffer",
        "subscript_typegpu_render_bundle_encoder_set_index_buffer",
        "subscript_typegpu_render_bundle_encoder_draw",
        "subscript_typegpu_render_bundle_encoder_draw_indexed",
        "subscript_typegpu_render_bundle_encoder_draw_indirect",
        "subscript_typegpu_render_bundle_encoder_draw_indexed_indirect",
        "subscript_typegpu_render_bundle_encoder_insert_debug_marker",
        "subscript_typegpu_render_bundle_encoder_push_debug_group",
        "subscript_typegpu_render_bundle_encoder_pop_debug_group",
        "subscript_typegpu_render_bundle_encoder_finish",
        "subscript_typegpu_render_bundle_encoder_set_label",
        "subscript_typegpu_render_bundle_set_label",
        "subscript_typegpu_query_set_get_type",
        "subscript_typegpu_query_set_get_count",
        "subscript_typegpu_query_set_destroy",
        "subscript_typegpu_query_set_set_label",
        "subscript_typegpu_query_set_release",
        "subscript_typegpu_render_bundle_release",
        "subscript_typegpu_render_bundle_encoder_release",
        "subscript_typegpu_command_buffer_release",
        "subscript_typegpu_render_pass_encoder_release",
        "subscript_typegpu_compute_pass_encoder_release",
        "subscript_typegpu_command_encoder_release",
        "subscript_typegpu_render_pipeline_release",
        "subscript_typegpu_compute_pipeline_release",
        "subscript_typegpu_shader_module_release",
        "subscript_typegpu_pipeline_layout_release",
        "subscript_typegpu_bind_group_release",
        "subscript_typegpu_bind_group_layout_release",
        "subscript_typegpu_sampler_release",
        "subscript_typegpu_texture_view_release",
        "subscript_typegpu_texture_release",
        "subscript_typegpu_buffer_release",
        "subscript_typegpu_queue_release",
        "subscript_typegpu_device_release",
        "subscript_typegpu_adapter_release",
    ]
}

pub fn facade_symbols() -> Vec<(String, *const u8)> {
    vec![
        ("subscript_typegpu_create_instance".to_owned(), facade::subscript_typegpu_create_instance as *const u8),
        ("subscript_typegpu_instance_process_events".to_owned(), facade::subscript_typegpu_instance_process_events as *const u8),
        ("subscript_typegpu_instance_release".to_owned(), facade::subscript_typegpu_instance_release as *const u8),
        ("subscript_typegpu_instance_request_adapter".to_owned(), facade::subscript_typegpu_instance_request_adapter as *const u8),
        ("subscript_typegpu_future_status".to_owned(), facade::subscript_typegpu_future_status as *const u8),
        ("subscript_typegpu_future_drop".to_owned(), facade::subscript_typegpu_future_drop as *const u8),
        ("subscript_typegpu_request_adapter_take".to_owned(), facade::subscript_typegpu_request_adapter_take as *const u8),
        ("subscript_typegpu_adapter_get_limits".to_owned(), facade::subscript_typegpu_adapter_get_limits as *const u8),
        ("subscript_typegpu_adapter_get_info".to_owned(), facade::subscript_typegpu_adapter_get_info as *const u8),
        ("subscript_typegpu_adapter_has_feature".to_owned(), facade::subscript_typegpu_adapter_has_feature as *const u8),
        ("subscript_typegpu_adapter_request_device_with_descriptor".to_owned(), facade::subscript_typegpu_adapter_request_device_with_descriptor as *const u8),
        ("subscript_typegpu_request_device_take".to_owned(), facade::subscript_typegpu_request_device_take as *const u8),
        ("subscript_typegpu_device_get_queue".to_owned(), facade::subscript_typegpu_device_get_queue as *const u8),
        ("subscript_typegpu_device_destroy".to_owned(), facade::subscript_typegpu_device_destroy as *const u8),
        ("subscript_typegpu_device_set_label".to_owned(), facade::subscript_typegpu_device_set_label as *const u8),
        ("subscript_typegpu_device_push_error_scope".to_owned(), facade::subscript_typegpu_device_push_error_scope as *const u8),
        ("subscript_typegpu_device_pop_error_scope".to_owned(), facade::subscript_typegpu_device_pop_error_scope as *const u8),
        ("subscript_typegpu_pop_error_scope_take".to_owned(), facade::subscript_typegpu_pop_error_scope_take as *const u8),
        ("subscript_typegpu_device_next_uncaptured_error".to_owned(), facade::subscript_typegpu_device_next_uncaptured_error as *const u8),
        ("subscript_typegpu_device_lost_info".to_owned(), facade::subscript_typegpu_device_lost_info as *const u8),
        ("subscript_typegpu_device_get_limits".to_owned(), facade::subscript_typegpu_device_get_limits as *const u8),
        ("subscript_typegpu_device_get_adapter_info".to_owned(), facade::subscript_typegpu_device_get_adapter_info as *const u8),
        ("subscript_typegpu_device_has_feature".to_owned(), facade::subscript_typegpu_device_has_feature as *const u8),
        ("subscript_typegpu_device_create_buffer".to_owned(), facade::subscript_typegpu_device_create_buffer as *const u8),
        ("subscript_typegpu_device_create_texture".to_owned(), facade::subscript_typegpu_device_create_texture as *const u8),
        ("subscript_typegpu_device_create_sampler".to_owned(), facade::subscript_typegpu_device_create_sampler as *const u8),
        ("subscript_typegpu_device_create_bind_group_layout".to_owned(), facade::subscript_typegpu_device_create_bind_group_layout as *const u8),
        ("subscript_typegpu_device_create_bind_group".to_owned(), facade::subscript_typegpu_device_create_bind_group as *const u8),
        ("subscript_typegpu_device_create_pipeline_layout".to_owned(), facade::subscript_typegpu_device_create_pipeline_layout as *const u8),
        ("subscript_typegpu_device_create_shader_module".to_owned(), facade::subscript_typegpu_device_create_shader_module as *const u8),
        ("subscript_typegpu_device_create_compute_pipeline".to_owned(), facade::subscript_typegpu_device_create_compute_pipeline as *const u8),
        ("subscript_typegpu_device_create_compute_pipeline_async_begin".to_owned(), facade::subscript_typegpu_device_create_compute_pipeline_async_begin as *const u8),
        ("subscript_typegpu_create_compute_pipeline_async_take".to_owned(), facade::subscript_typegpu_create_compute_pipeline_async_take as *const u8),
        ("subscript_typegpu_device_create_render_pipeline".to_owned(), facade::subscript_typegpu_device_create_render_pipeline as *const u8),
        ("subscript_typegpu_device_create_render_pipeline_async_begin".to_owned(), facade::subscript_typegpu_device_create_render_pipeline_async_begin as *const u8),
        ("subscript_typegpu_create_render_pipeline_async_take".to_owned(), facade::subscript_typegpu_create_render_pipeline_async_take as *const u8),
        ("subscript_typegpu_device_create_command_encoder".to_owned(), facade::subscript_typegpu_device_create_command_encoder as *const u8),
        ("subscript_typegpu_device_create_render_bundle_encoder".to_owned(), facade::subscript_typegpu_device_create_render_bundle_encoder as *const u8),
        ("subscript_typegpu_device_create_query_set".to_owned(), facade::subscript_typegpu_device_create_query_set as *const u8),
        ("subscript_typegpu_queue_submit".to_owned(), facade::subscript_typegpu_queue_submit as *const u8),
        ("subscript_typegpu_queue_on_submitted_work_done".to_owned(), facade::subscript_typegpu_queue_on_submitted_work_done as *const u8),
        ("subscript_typegpu_queue_write_buffer".to_owned(), facade::subscript_typegpu_queue_write_buffer as *const u8),
        ("subscript_typegpu_queue_write_buffer_f32".to_owned(), facade::subscript_typegpu_queue_write_buffer_f32 as *const u8),
        ("subscript_typegpu_queue_write_texture".to_owned(), facade::subscript_typegpu_queue_write_texture as *const u8),
        ("subscript_typegpu_queue_set_label".to_owned(), facade::subscript_typegpu_queue_set_label as *const u8),
        ("subscript_typegpu_buffer_map_async".to_owned(), facade::subscript_typegpu_buffer_map_async as *const u8),
        ("subscript_typegpu_buffer_read_mapped_range".to_owned(), facade::subscript_typegpu_buffer_read_mapped_range as *const u8),
        ("subscript_typegpu_buffer_read_mapped_range_f32".to_owned(), facade::subscript_typegpu_buffer_read_mapped_range_f32 as *const u8),
        ("subscript_typegpu_buffer_write_mapped_range".to_owned(), facade::subscript_typegpu_buffer_write_mapped_range as *const u8),
        ("subscript_typegpu_buffer_set_label".to_owned(), facade::subscript_typegpu_buffer_set_label as *const u8),
        ("subscript_typegpu_buffer_get_usage".to_owned(), facade::subscript_typegpu_buffer_get_usage as *const u8),
        ("subscript_typegpu_buffer_get_size".to_owned(), facade::subscript_typegpu_buffer_get_size as *const u8),
        ("subscript_typegpu_buffer_get_map_state".to_owned(), facade::subscript_typegpu_buffer_get_map_state as *const u8),
        ("subscript_typegpu_buffer_unmap".to_owned(), facade::subscript_typegpu_buffer_unmap as *const u8),
        ("subscript_typegpu_buffer_destroy".to_owned(), facade::subscript_typegpu_buffer_destroy as *const u8),
        ("subscript_typegpu_texture_create_view".to_owned(), facade::subscript_typegpu_texture_create_view as *const u8),
        ("subscript_typegpu_texture_set_label".to_owned(), facade::subscript_typegpu_texture_set_label as *const u8),
        ("subscript_typegpu_texture_get_width".to_owned(), facade::subscript_typegpu_texture_get_width as *const u8),
        ("subscript_typegpu_texture_get_height".to_owned(), facade::subscript_typegpu_texture_get_height as *const u8),
        ("subscript_typegpu_texture_get_depth_or_array_layers".to_owned(), facade::subscript_typegpu_texture_get_depth_or_array_layers as *const u8),
        ("subscript_typegpu_texture_get_mip_level_count".to_owned(), facade::subscript_typegpu_texture_get_mip_level_count as *const u8),
        ("subscript_typegpu_texture_get_sample_count".to_owned(), facade::subscript_typegpu_texture_get_sample_count as *const u8),
        ("subscript_typegpu_texture_get_dimension".to_owned(), facade::subscript_typegpu_texture_get_dimension as *const u8),
        ("subscript_typegpu_texture_get_format".to_owned(), facade::subscript_typegpu_texture_get_format as *const u8),
        ("subscript_typegpu_texture_get_usage".to_owned(), facade::subscript_typegpu_texture_get_usage as *const u8),
        ("subscript_typegpu_texture_destroy".to_owned(), facade::subscript_typegpu_texture_destroy as *const u8),
        ("subscript_typegpu_texture_view_set_label".to_owned(), facade::subscript_typegpu_texture_view_set_label as *const u8),
        ("subscript_typegpu_sampler_set_label".to_owned(), facade::subscript_typegpu_sampler_set_label as *const u8),
        ("subscript_typegpu_bind_group_layout_set_label".to_owned(), facade::subscript_typegpu_bind_group_layout_set_label as *const u8),
        ("subscript_typegpu_bind_group_set_label".to_owned(), facade::subscript_typegpu_bind_group_set_label as *const u8),
        ("subscript_typegpu_pipeline_layout_set_label".to_owned(), facade::subscript_typegpu_pipeline_layout_set_label as *const u8),
        ("subscript_typegpu_shader_module_set_label".to_owned(), facade::subscript_typegpu_shader_module_set_label as *const u8),
        ("subscript_typegpu_compute_pipeline_get_bind_group_layout".to_owned(), facade::subscript_typegpu_compute_pipeline_get_bind_group_layout as *const u8),
        ("subscript_typegpu_compute_pipeline_set_label".to_owned(), facade::subscript_typegpu_compute_pipeline_set_label as *const u8),
        ("subscript_typegpu_render_pipeline_get_bind_group_layout".to_owned(), facade::subscript_typegpu_render_pipeline_get_bind_group_layout as *const u8),
        ("subscript_typegpu_render_pipeline_set_label".to_owned(), facade::subscript_typegpu_render_pipeline_set_label as *const u8),
        ("subscript_typegpu_command_encoder_finish".to_owned(), facade::subscript_typegpu_command_encoder_finish as *const u8),
        ("subscript_typegpu_command_encoder_begin_compute_pass".to_owned(), facade::subscript_typegpu_command_encoder_begin_compute_pass as *const u8),
        ("subscript_typegpu_command_encoder_begin_render_pass".to_owned(), facade::subscript_typegpu_command_encoder_begin_render_pass as *const u8),
        ("subscript_typegpu_command_encoder_copy_buffer_to_buffer".to_owned(), facade::subscript_typegpu_command_encoder_copy_buffer_to_buffer as *const u8),
        ("subscript_typegpu_command_encoder_copy_buffer_to_texture".to_owned(), facade::subscript_typegpu_command_encoder_copy_buffer_to_texture as *const u8),
        ("subscript_typegpu_command_encoder_copy_texture_to_buffer".to_owned(), facade::subscript_typegpu_command_encoder_copy_texture_to_buffer as *const u8),
        ("subscript_typegpu_command_encoder_copy_texture_to_texture".to_owned(), facade::subscript_typegpu_command_encoder_copy_texture_to_texture as *const u8),
        ("subscript_typegpu_command_encoder_clear_buffer".to_owned(), facade::subscript_typegpu_command_encoder_clear_buffer as *const u8),
        ("subscript_typegpu_command_encoder_resolve_query_set".to_owned(), facade::subscript_typegpu_command_encoder_resolve_query_set as *const u8),
        ("subscript_typegpu_command_encoder_insert_debug_marker".to_owned(), facade::subscript_typegpu_command_encoder_insert_debug_marker as *const u8),
        ("subscript_typegpu_command_encoder_push_debug_group".to_owned(), facade::subscript_typegpu_command_encoder_push_debug_group as *const u8),
        ("subscript_typegpu_command_encoder_pop_debug_group".to_owned(), facade::subscript_typegpu_command_encoder_pop_debug_group as *const u8),
        ("subscript_typegpu_command_encoder_set_label".to_owned(), facade::subscript_typegpu_command_encoder_set_label as *const u8),
        ("subscript_typegpu_compute_pass_encoder_set_pipeline".to_owned(), facade::subscript_typegpu_compute_pass_encoder_set_pipeline as *const u8),
        ("subscript_typegpu_compute_pass_encoder_set_bind_group".to_owned(), facade::subscript_typegpu_compute_pass_encoder_set_bind_group as *const u8),
        ("subscript_typegpu_compute_pass_encoder_dispatch_workgroups".to_owned(), facade::subscript_typegpu_compute_pass_encoder_dispatch_workgroups as *const u8),
        ("subscript_typegpu_compute_pass_encoder_dispatch_workgroups_indirect".to_owned(), facade::subscript_typegpu_compute_pass_encoder_dispatch_workgroups_indirect as *const u8),
        ("subscript_typegpu_compute_pass_encoder_insert_debug_marker".to_owned(), facade::subscript_typegpu_compute_pass_encoder_insert_debug_marker as *const u8),
        ("subscript_typegpu_compute_pass_encoder_push_debug_group".to_owned(), facade::subscript_typegpu_compute_pass_encoder_push_debug_group as *const u8),
        ("subscript_typegpu_compute_pass_encoder_pop_debug_group".to_owned(), facade::subscript_typegpu_compute_pass_encoder_pop_debug_group as *const u8),
        ("subscript_typegpu_compute_pass_encoder_end".to_owned(), facade::subscript_typegpu_compute_pass_encoder_end as *const u8),
        ("subscript_typegpu_compute_pass_encoder_set_label".to_owned(), facade::subscript_typegpu_compute_pass_encoder_set_label as *const u8),
        ("subscript_typegpu_render_pass_encoder_set_pipeline".to_owned(), facade::subscript_typegpu_render_pass_encoder_set_pipeline as *const u8),
        ("subscript_typegpu_render_pass_encoder_set_bind_group".to_owned(), facade::subscript_typegpu_render_pass_encoder_set_bind_group as *const u8),
        ("subscript_typegpu_render_pass_encoder_set_vertex_buffer".to_owned(), facade::subscript_typegpu_render_pass_encoder_set_vertex_buffer as *const u8),
        ("subscript_typegpu_render_pass_encoder_set_index_buffer".to_owned(), facade::subscript_typegpu_render_pass_encoder_set_index_buffer as *const u8),
        ("subscript_typegpu_render_pass_encoder_draw".to_owned(), facade::subscript_typegpu_render_pass_encoder_draw as *const u8),
        ("subscript_typegpu_render_pass_encoder_draw_indexed".to_owned(), facade::subscript_typegpu_render_pass_encoder_draw_indexed as *const u8),
        ("subscript_typegpu_render_pass_encoder_draw_indirect".to_owned(), facade::subscript_typegpu_render_pass_encoder_draw_indirect as *const u8),
        ("subscript_typegpu_render_pass_encoder_draw_indexed_indirect".to_owned(), facade::subscript_typegpu_render_pass_encoder_draw_indexed_indirect as *const u8),
        ("subscript_typegpu_render_pass_encoder_set_viewport".to_owned(), facade::subscript_typegpu_render_pass_encoder_set_viewport as *const u8),
        ("subscript_typegpu_render_pass_encoder_set_scissor_rect".to_owned(), facade::subscript_typegpu_render_pass_encoder_set_scissor_rect as *const u8),
        ("subscript_typegpu_render_pass_encoder_set_blend_constant".to_owned(), facade::subscript_typegpu_render_pass_encoder_set_blend_constant as *const u8),
        ("subscript_typegpu_render_pass_encoder_set_stencil_reference".to_owned(), facade::subscript_typegpu_render_pass_encoder_set_stencil_reference as *const u8),
        ("subscript_typegpu_render_pass_encoder_begin_occlusion_query".to_owned(), facade::subscript_typegpu_render_pass_encoder_begin_occlusion_query as *const u8),
        ("subscript_typegpu_render_pass_encoder_end_occlusion_query".to_owned(), facade::subscript_typegpu_render_pass_encoder_end_occlusion_query as *const u8),
        ("subscript_typegpu_render_pass_encoder_execute_bundles".to_owned(), facade::subscript_typegpu_render_pass_encoder_execute_bundles as *const u8),
        ("subscript_typegpu_render_pass_encoder_insert_debug_marker".to_owned(), facade::subscript_typegpu_render_pass_encoder_insert_debug_marker as *const u8),
        ("subscript_typegpu_render_pass_encoder_push_debug_group".to_owned(), facade::subscript_typegpu_render_pass_encoder_push_debug_group as *const u8),
        ("subscript_typegpu_render_pass_encoder_pop_debug_group".to_owned(), facade::subscript_typegpu_render_pass_encoder_pop_debug_group as *const u8),
        ("subscript_typegpu_render_pass_encoder_end".to_owned(), facade::subscript_typegpu_render_pass_encoder_end as *const u8),
        ("subscript_typegpu_render_pass_encoder_set_label".to_owned(), facade::subscript_typegpu_render_pass_encoder_set_label as *const u8),
        ("subscript_typegpu_command_buffer_set_label".to_owned(), facade::subscript_typegpu_command_buffer_set_label as *const u8),
        ("subscript_typegpu_render_bundle_encoder_set_pipeline".to_owned(), facade::subscript_typegpu_render_bundle_encoder_set_pipeline as *const u8),
        ("subscript_typegpu_render_bundle_encoder_set_bind_group".to_owned(), facade::subscript_typegpu_render_bundle_encoder_set_bind_group as *const u8),
        ("subscript_typegpu_render_bundle_encoder_set_vertex_buffer".to_owned(), facade::subscript_typegpu_render_bundle_encoder_set_vertex_buffer as *const u8),
        ("subscript_typegpu_render_bundle_encoder_set_index_buffer".to_owned(), facade::subscript_typegpu_render_bundle_encoder_set_index_buffer as *const u8),
        ("subscript_typegpu_render_bundle_encoder_draw".to_owned(), facade::subscript_typegpu_render_bundle_encoder_draw as *const u8),
        ("subscript_typegpu_render_bundle_encoder_draw_indexed".to_owned(), facade::subscript_typegpu_render_bundle_encoder_draw_indexed as *const u8),
        ("subscript_typegpu_render_bundle_encoder_draw_indirect".to_owned(), facade::subscript_typegpu_render_bundle_encoder_draw_indirect as *const u8),
        ("subscript_typegpu_render_bundle_encoder_draw_indexed_indirect".to_owned(), facade::subscript_typegpu_render_bundle_encoder_draw_indexed_indirect as *const u8),
        ("subscript_typegpu_render_bundle_encoder_insert_debug_marker".to_owned(), facade::subscript_typegpu_render_bundle_encoder_insert_debug_marker as *const u8),
        ("subscript_typegpu_render_bundle_encoder_push_debug_group".to_owned(), facade::subscript_typegpu_render_bundle_encoder_push_debug_group as *const u8),
        ("subscript_typegpu_render_bundle_encoder_pop_debug_group".to_owned(), facade::subscript_typegpu_render_bundle_encoder_pop_debug_group as *const u8),
        ("subscript_typegpu_render_bundle_encoder_finish".to_owned(), facade::subscript_typegpu_render_bundle_encoder_finish as *const u8),
        ("subscript_typegpu_render_bundle_encoder_set_label".to_owned(), facade::subscript_typegpu_render_bundle_encoder_set_label as *const u8),
        ("subscript_typegpu_render_bundle_set_label".to_owned(), facade::subscript_typegpu_render_bundle_set_label as *const u8),
        ("subscript_typegpu_query_set_get_type".to_owned(), facade::subscript_typegpu_query_set_get_type as *const u8),
        ("subscript_typegpu_query_set_get_count".to_owned(), facade::subscript_typegpu_query_set_get_count as *const u8),
        ("subscript_typegpu_query_set_destroy".to_owned(), facade::subscript_typegpu_query_set_destroy as *const u8),
        ("subscript_typegpu_query_set_set_label".to_owned(), facade::subscript_typegpu_query_set_set_label as *const u8),
        ("subscript_typegpu_query_set_release".to_owned(), facade::subscript_typegpu_query_set_release as *const u8),
        ("subscript_typegpu_render_bundle_release".to_owned(), facade::subscript_typegpu_render_bundle_release as *const u8),
        ("subscript_typegpu_render_bundle_encoder_release".to_owned(), facade::subscript_typegpu_render_bundle_encoder_release as *const u8),
        ("subscript_typegpu_command_buffer_release".to_owned(), facade::subscript_typegpu_command_buffer_release as *const u8),
        ("subscript_typegpu_render_pass_encoder_release".to_owned(), facade::subscript_typegpu_render_pass_encoder_release as *const u8),
        ("subscript_typegpu_compute_pass_encoder_release".to_owned(), facade::subscript_typegpu_compute_pass_encoder_release as *const u8),
        ("subscript_typegpu_command_encoder_release".to_owned(), facade::subscript_typegpu_command_encoder_release as *const u8),
        ("subscript_typegpu_render_pipeline_release".to_owned(), facade::subscript_typegpu_render_pipeline_release as *const u8),
        ("subscript_typegpu_compute_pipeline_release".to_owned(), facade::subscript_typegpu_compute_pipeline_release as *const u8),
        ("subscript_typegpu_shader_module_release".to_owned(), facade::subscript_typegpu_shader_module_release as *const u8),
        ("subscript_typegpu_pipeline_layout_release".to_owned(), facade::subscript_typegpu_pipeline_layout_release as *const u8),
        ("subscript_typegpu_bind_group_release".to_owned(), facade::subscript_typegpu_bind_group_release as *const u8),
        ("subscript_typegpu_bind_group_layout_release".to_owned(), facade::subscript_typegpu_bind_group_layout_release as *const u8),
        ("subscript_typegpu_sampler_release".to_owned(), facade::subscript_typegpu_sampler_release as *const u8),
        ("subscript_typegpu_texture_view_release".to_owned(), facade::subscript_typegpu_texture_view_release as *const u8),
        ("subscript_typegpu_texture_release".to_owned(), facade::subscript_typegpu_texture_release as *const u8),
        ("subscript_typegpu_buffer_release".to_owned(), facade::subscript_typegpu_buffer_release as *const u8),
        ("subscript_typegpu_queue_release".to_owned(), facade::subscript_typegpu_queue_release as *const u8),
        ("subscript_typegpu_device_release".to_owned(), facade::subscript_typegpu_device_release as *const u8),
        ("subscript_typegpu_adapter_release".to_owned(), facade::subscript_typegpu_adapter_release as *const u8),
    ]
}

pub fn facade_counting_symbols() -> Vec<(String, *const u8)> {
    vec![
        ("subscript_typegpu_create_instance".to_owned(), coverage_0 as *const u8),
        ("subscript_typegpu_instance_process_events".to_owned(), coverage_1 as *const u8),
        ("subscript_typegpu_instance_release".to_owned(), coverage_2 as *const u8),
        ("subscript_typegpu_instance_request_adapter".to_owned(), coverage_3 as *const u8),
        ("subscript_typegpu_future_status".to_owned(), coverage_4 as *const u8),
        ("subscript_typegpu_future_drop".to_owned(), coverage_5 as *const u8),
        ("subscript_typegpu_request_adapter_take".to_owned(), coverage_6 as *const u8),
        ("subscript_typegpu_adapter_get_limits".to_owned(), coverage_7 as *const u8),
        ("subscript_typegpu_adapter_get_info".to_owned(), coverage_8 as *const u8),
        ("subscript_typegpu_adapter_has_feature".to_owned(), coverage_9 as *const u8),
        ("subscript_typegpu_adapter_request_device_with_descriptor".to_owned(), coverage_10 as *const u8),
        ("subscript_typegpu_request_device_take".to_owned(), coverage_11 as *const u8),
        ("subscript_typegpu_device_get_queue".to_owned(), coverage_12 as *const u8),
        ("subscript_typegpu_device_destroy".to_owned(), coverage_13 as *const u8),
        ("subscript_typegpu_device_set_label".to_owned(), coverage_14 as *const u8),
        ("subscript_typegpu_device_push_error_scope".to_owned(), coverage_15 as *const u8),
        ("subscript_typegpu_device_pop_error_scope".to_owned(), coverage_16 as *const u8),
        ("subscript_typegpu_pop_error_scope_take".to_owned(), coverage_17 as *const u8),
        ("subscript_typegpu_device_next_uncaptured_error".to_owned(), coverage_18 as *const u8),
        ("subscript_typegpu_device_lost_info".to_owned(), coverage_19 as *const u8),
        ("subscript_typegpu_device_get_limits".to_owned(), coverage_20 as *const u8),
        ("subscript_typegpu_device_get_adapter_info".to_owned(), coverage_21 as *const u8),
        ("subscript_typegpu_device_has_feature".to_owned(), coverage_22 as *const u8),
        ("subscript_typegpu_device_create_buffer".to_owned(), coverage_23 as *const u8),
        ("subscript_typegpu_device_create_texture".to_owned(), coverage_24 as *const u8),
        ("subscript_typegpu_device_create_sampler".to_owned(), coverage_25 as *const u8),
        ("subscript_typegpu_device_create_bind_group_layout".to_owned(), coverage_26 as *const u8),
        ("subscript_typegpu_device_create_bind_group".to_owned(), coverage_27 as *const u8),
        ("subscript_typegpu_device_create_pipeline_layout".to_owned(), coverage_28 as *const u8),
        ("subscript_typegpu_device_create_shader_module".to_owned(), coverage_29 as *const u8),
        ("subscript_typegpu_device_create_compute_pipeline".to_owned(), coverage_30 as *const u8),
        ("subscript_typegpu_device_create_compute_pipeline_async_begin".to_owned(), coverage_31 as *const u8),
        ("subscript_typegpu_create_compute_pipeline_async_take".to_owned(), coverage_32 as *const u8),
        ("subscript_typegpu_device_create_render_pipeline".to_owned(), coverage_33 as *const u8),
        ("subscript_typegpu_device_create_render_pipeline_async_begin".to_owned(), coverage_34 as *const u8),
        ("subscript_typegpu_create_render_pipeline_async_take".to_owned(), coverage_35 as *const u8),
        ("subscript_typegpu_device_create_command_encoder".to_owned(), coverage_36 as *const u8),
        ("subscript_typegpu_device_create_render_bundle_encoder".to_owned(), coverage_37 as *const u8),
        ("subscript_typegpu_device_create_query_set".to_owned(), coverage_38 as *const u8),
        ("subscript_typegpu_queue_submit".to_owned(), coverage_39 as *const u8),
        ("subscript_typegpu_queue_on_submitted_work_done".to_owned(), coverage_40 as *const u8),
        ("subscript_typegpu_queue_write_buffer".to_owned(), coverage_41 as *const u8),
        ("subscript_typegpu_queue_write_buffer_f32".to_owned(), coverage_42 as *const u8),
        ("subscript_typegpu_queue_write_texture".to_owned(), coverage_43 as *const u8),
        ("subscript_typegpu_queue_set_label".to_owned(), coverage_44 as *const u8),
        ("subscript_typegpu_buffer_map_async".to_owned(), coverage_45 as *const u8),
        ("subscript_typegpu_buffer_read_mapped_range".to_owned(), coverage_46 as *const u8),
        ("subscript_typegpu_buffer_read_mapped_range_f32".to_owned(), coverage_47 as *const u8),
        ("subscript_typegpu_buffer_write_mapped_range".to_owned(), coverage_48 as *const u8),
        ("subscript_typegpu_buffer_set_label".to_owned(), coverage_49 as *const u8),
        ("subscript_typegpu_buffer_get_usage".to_owned(), coverage_50 as *const u8),
        ("subscript_typegpu_buffer_get_size".to_owned(), coverage_51 as *const u8),
        ("subscript_typegpu_buffer_get_map_state".to_owned(), coverage_52 as *const u8),
        ("subscript_typegpu_buffer_unmap".to_owned(), coverage_53 as *const u8),
        ("subscript_typegpu_buffer_destroy".to_owned(), coverage_54 as *const u8),
        ("subscript_typegpu_texture_create_view".to_owned(), coverage_55 as *const u8),
        ("subscript_typegpu_texture_set_label".to_owned(), coverage_56 as *const u8),
        ("subscript_typegpu_texture_get_width".to_owned(), coverage_57 as *const u8),
        ("subscript_typegpu_texture_get_height".to_owned(), coverage_58 as *const u8),
        ("subscript_typegpu_texture_get_depth_or_array_layers".to_owned(), coverage_59 as *const u8),
        ("subscript_typegpu_texture_get_mip_level_count".to_owned(), coverage_60 as *const u8),
        ("subscript_typegpu_texture_get_sample_count".to_owned(), coverage_61 as *const u8),
        ("subscript_typegpu_texture_get_dimension".to_owned(), coverage_62 as *const u8),
        ("subscript_typegpu_texture_get_format".to_owned(), coverage_63 as *const u8),
        ("subscript_typegpu_texture_get_usage".to_owned(), coverage_64 as *const u8),
        ("subscript_typegpu_texture_destroy".to_owned(), coverage_65 as *const u8),
        ("subscript_typegpu_texture_view_set_label".to_owned(), coverage_66 as *const u8),
        ("subscript_typegpu_sampler_set_label".to_owned(), coverage_67 as *const u8),
        ("subscript_typegpu_bind_group_layout_set_label".to_owned(), coverage_68 as *const u8),
        ("subscript_typegpu_bind_group_set_label".to_owned(), coverage_69 as *const u8),
        ("subscript_typegpu_pipeline_layout_set_label".to_owned(), coverage_70 as *const u8),
        ("subscript_typegpu_shader_module_set_label".to_owned(), coverage_71 as *const u8),
        ("subscript_typegpu_compute_pipeline_get_bind_group_layout".to_owned(), coverage_72 as *const u8),
        ("subscript_typegpu_compute_pipeline_set_label".to_owned(), coverage_73 as *const u8),
        ("subscript_typegpu_render_pipeline_get_bind_group_layout".to_owned(), coverage_74 as *const u8),
        ("subscript_typegpu_render_pipeline_set_label".to_owned(), coverage_75 as *const u8),
        ("subscript_typegpu_command_encoder_finish".to_owned(), coverage_76 as *const u8),
        ("subscript_typegpu_command_encoder_begin_compute_pass".to_owned(), coverage_77 as *const u8),
        ("subscript_typegpu_command_encoder_begin_render_pass".to_owned(), coverage_78 as *const u8),
        ("subscript_typegpu_command_encoder_copy_buffer_to_buffer".to_owned(), coverage_79 as *const u8),
        ("subscript_typegpu_command_encoder_copy_buffer_to_texture".to_owned(), coverage_80 as *const u8),
        ("subscript_typegpu_command_encoder_copy_texture_to_buffer".to_owned(), coverage_81 as *const u8),
        ("subscript_typegpu_command_encoder_copy_texture_to_texture".to_owned(), coverage_82 as *const u8),
        ("subscript_typegpu_command_encoder_clear_buffer".to_owned(), coverage_83 as *const u8),
        ("subscript_typegpu_command_encoder_resolve_query_set".to_owned(), coverage_84 as *const u8),
        ("subscript_typegpu_command_encoder_insert_debug_marker".to_owned(), coverage_85 as *const u8),
        ("subscript_typegpu_command_encoder_push_debug_group".to_owned(), coverage_86 as *const u8),
        ("subscript_typegpu_command_encoder_pop_debug_group".to_owned(), coverage_87 as *const u8),
        ("subscript_typegpu_command_encoder_set_label".to_owned(), coverage_88 as *const u8),
        ("subscript_typegpu_compute_pass_encoder_set_pipeline".to_owned(), coverage_89 as *const u8),
        ("subscript_typegpu_compute_pass_encoder_set_bind_group".to_owned(), coverage_90 as *const u8),
        ("subscript_typegpu_compute_pass_encoder_dispatch_workgroups".to_owned(), coverage_91 as *const u8),
        ("subscript_typegpu_compute_pass_encoder_dispatch_workgroups_indirect".to_owned(), coverage_92 as *const u8),
        ("subscript_typegpu_compute_pass_encoder_insert_debug_marker".to_owned(), coverage_93 as *const u8),
        ("subscript_typegpu_compute_pass_encoder_push_debug_group".to_owned(), coverage_94 as *const u8),
        ("subscript_typegpu_compute_pass_encoder_pop_debug_group".to_owned(), coverage_95 as *const u8),
        ("subscript_typegpu_compute_pass_encoder_end".to_owned(), coverage_96 as *const u8),
        ("subscript_typegpu_compute_pass_encoder_set_label".to_owned(), coverage_97 as *const u8),
        ("subscript_typegpu_render_pass_encoder_set_pipeline".to_owned(), coverage_98 as *const u8),
        ("subscript_typegpu_render_pass_encoder_set_bind_group".to_owned(), coverage_99 as *const u8),
        ("subscript_typegpu_render_pass_encoder_set_vertex_buffer".to_owned(), coverage_100 as *const u8),
        ("subscript_typegpu_render_pass_encoder_set_index_buffer".to_owned(), coverage_101 as *const u8),
        ("subscript_typegpu_render_pass_encoder_draw".to_owned(), coverage_102 as *const u8),
        ("subscript_typegpu_render_pass_encoder_draw_indexed".to_owned(), coverage_103 as *const u8),
        ("subscript_typegpu_render_pass_encoder_draw_indirect".to_owned(), coverage_104 as *const u8),
        ("subscript_typegpu_render_pass_encoder_draw_indexed_indirect".to_owned(), coverage_105 as *const u8),
        ("subscript_typegpu_render_pass_encoder_set_viewport".to_owned(), coverage_106 as *const u8),
        ("subscript_typegpu_render_pass_encoder_set_scissor_rect".to_owned(), coverage_107 as *const u8),
        ("subscript_typegpu_render_pass_encoder_set_blend_constant".to_owned(), coverage_108 as *const u8),
        ("subscript_typegpu_render_pass_encoder_set_stencil_reference".to_owned(), coverage_109 as *const u8),
        ("subscript_typegpu_render_pass_encoder_begin_occlusion_query".to_owned(), coverage_110 as *const u8),
        ("subscript_typegpu_render_pass_encoder_end_occlusion_query".to_owned(), coverage_111 as *const u8),
        ("subscript_typegpu_render_pass_encoder_execute_bundles".to_owned(), coverage_112 as *const u8),
        ("subscript_typegpu_render_pass_encoder_insert_debug_marker".to_owned(), coverage_113 as *const u8),
        ("subscript_typegpu_render_pass_encoder_push_debug_group".to_owned(), coverage_114 as *const u8),
        ("subscript_typegpu_render_pass_encoder_pop_debug_group".to_owned(), coverage_115 as *const u8),
        ("subscript_typegpu_render_pass_encoder_end".to_owned(), coverage_116 as *const u8),
        ("subscript_typegpu_render_pass_encoder_set_label".to_owned(), coverage_117 as *const u8),
        ("subscript_typegpu_command_buffer_set_label".to_owned(), coverage_118 as *const u8),
        ("subscript_typegpu_render_bundle_encoder_set_pipeline".to_owned(), coverage_119 as *const u8),
        ("subscript_typegpu_render_bundle_encoder_set_bind_group".to_owned(), coverage_120 as *const u8),
        ("subscript_typegpu_render_bundle_encoder_set_vertex_buffer".to_owned(), coverage_121 as *const u8),
        ("subscript_typegpu_render_bundle_encoder_set_index_buffer".to_owned(), coverage_122 as *const u8),
        ("subscript_typegpu_render_bundle_encoder_draw".to_owned(), coverage_123 as *const u8),
        ("subscript_typegpu_render_bundle_encoder_draw_indexed".to_owned(), coverage_124 as *const u8),
        ("subscript_typegpu_render_bundle_encoder_draw_indirect".to_owned(), coverage_125 as *const u8),
        ("subscript_typegpu_render_bundle_encoder_draw_indexed_indirect".to_owned(), coverage_126 as *const u8),
        ("subscript_typegpu_render_bundle_encoder_insert_debug_marker".to_owned(), coverage_127 as *const u8),
        ("subscript_typegpu_render_bundle_encoder_push_debug_group".to_owned(), coverage_128 as *const u8),
        ("subscript_typegpu_render_bundle_encoder_pop_debug_group".to_owned(), coverage_129 as *const u8),
        ("subscript_typegpu_render_bundle_encoder_finish".to_owned(), coverage_130 as *const u8),
        ("subscript_typegpu_render_bundle_encoder_set_label".to_owned(), coverage_131 as *const u8),
        ("subscript_typegpu_render_bundle_set_label".to_owned(), coverage_132 as *const u8),
        ("subscript_typegpu_query_set_get_type".to_owned(), coverage_133 as *const u8),
        ("subscript_typegpu_query_set_get_count".to_owned(), coverage_134 as *const u8),
        ("subscript_typegpu_query_set_destroy".to_owned(), coverage_135 as *const u8),
        ("subscript_typegpu_query_set_set_label".to_owned(), coverage_136 as *const u8),
        ("subscript_typegpu_query_set_release".to_owned(), coverage_137 as *const u8),
        ("subscript_typegpu_render_bundle_release".to_owned(), coverage_138 as *const u8),
        ("subscript_typegpu_render_bundle_encoder_release".to_owned(), coverage_139 as *const u8),
        ("subscript_typegpu_command_buffer_release".to_owned(), coverage_140 as *const u8),
        ("subscript_typegpu_render_pass_encoder_release".to_owned(), coverage_141 as *const u8),
        ("subscript_typegpu_compute_pass_encoder_release".to_owned(), coverage_142 as *const u8),
        ("subscript_typegpu_command_encoder_release".to_owned(), coverage_143 as *const u8),
        ("subscript_typegpu_render_pipeline_release".to_owned(), coverage_144 as *const u8),
        ("subscript_typegpu_compute_pipeline_release".to_owned(), coverage_145 as *const u8),
        ("subscript_typegpu_shader_module_release".to_owned(), coverage_146 as *const u8),
        ("subscript_typegpu_pipeline_layout_release".to_owned(), coverage_147 as *const u8),
        ("subscript_typegpu_bind_group_release".to_owned(), coverage_148 as *const u8),
        ("subscript_typegpu_bind_group_layout_release".to_owned(), coverage_149 as *const u8),
        ("subscript_typegpu_sampler_release".to_owned(), coverage_150 as *const u8),
        ("subscript_typegpu_texture_view_release".to_owned(), coverage_151 as *const u8),
        ("subscript_typegpu_texture_release".to_owned(), coverage_152 as *const u8),
        ("subscript_typegpu_buffer_release".to_owned(), coverage_153 as *const u8),
        ("subscript_typegpu_queue_release".to_owned(), coverage_154 as *const u8),
        ("subscript_typegpu_device_release".to_owned(), coverage_155 as *const u8),
        ("subscript_typegpu_adapter_release".to_owned(), coverage_156 as *const u8),
    ]
}
