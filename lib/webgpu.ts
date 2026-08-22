// GENERATED FILE — DO NOT EDIT.
// The API layer is emitted from pinned GPUWeb IDL, the subscript-typegpu.h mirror, and API policy.
// Boundary handles and future polling are implementation details.
//
// API policy deviations:
// - *.dispose: J7 replaces JavaScript garbage-collection lifetime with explicit dispose()
// - gpu: the exported gpu constant replaces navigator.gpu because the DOM surface is excluded
// - GPUShaderModule.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPUComputePipeline.getBindGroupLayout: the facade names the WebIDL index parameter groupIndex; the generated API keeps the IDL name and joins it explicitly
// - GPUComputePipeline.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPURenderPipeline.getBindGroupLayout: the facade names the WebIDL index parameter groupIndex; the generated API keeps the IDL name and joins it explicitly
// - GPURenderPipeline.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPUProgrammableStage.entryPoint: JavaScript may omit entryPoint when exactly one matching shader entry can be inferred, while this API requires the entry-point name explicitly
// - GPUProgrammableStage.constants: WebIDL's map literal { constants: { gain: 2.0 } } is not expressible, so JavaScript users must supply an entry array { constants: [{ key: "gain", value: 2.0 }] }; unlike a record, the array can contain duplicate keys, so duplicates become backend validation errors instead of an unrepresentable state
// - GPUComputePipelineDescriptor.layout: IDL requires a layout handle or the string "auto"; this API makes layout optional and replaces "auto" with null, so omission/null selects automatic layout *(docs)* and an accidentally omitted layout produces a pipeline whose bind groups cannot be shared through an explicit GPUPipelineLayout
// - GPUVertexState.entryPoint: JavaScript may omit entryPoint when exactly one matching shader entry can be inferred, while this API requires the entry-point name explicitly
// - GPUVertexState.constants: WebIDL's pipeline-constant map is not expressible, so JavaScript users must supply key/value entry objects in an array; unlike a record, the array can contain duplicate keys, so duplicates become backend validation errors instead of an unrepresentable state
// - GPUVertexState.buffers: the IDL sequence permits null holes for unused vertex-buffer slots, while the facade aggregate array cannot carry null elements, so JavaScript users can provide only concrete GPUVertexBufferLayout entries
// - GPUDepthStencilState.depthWriteEnabled: JavaScript distinguishes an absent depthWriteEnabled from false, but Q33 has no absence-capable boolean member; callers must state true or false and the facade receives that explicit optional-bool value
// - GPUColorTargetState.blend: JavaScript expresses an absent blend member by omission; this API also accepts explicit null, and omission or null both disable blending
// - GPUFragmentState.entryPoint: JavaScript may omit entryPoint when exactly one matching shader entry can be inferred, while this API requires the entry-point name explicitly
// - GPUFragmentState.constants: WebIDL's pipeline-constant map is not expressible, so JavaScript users must supply key/value entry objects in an array; unlike a record, the array can contain duplicate keys, so duplicates become backend validation errors instead of an unrepresentable state
// - GPUFragmentState.targets: the IDL sequence permits null holes for disabled color targets, while the facade aggregate array cannot carry null elements, so JavaScript users can provide only concrete GPUColorTargetState entries
// - GPURenderPipelineDescriptor.layout: IDL requires a layout handle or the string "auto"; this API makes layout optional and replaces "auto" with null, so omission/null selects automatic layout *(docs)* and an accidentally omitted layout produces a pipeline whose bind groups cannot be shared through an explicit GPUPipelineLayout
// - GPURenderPipelineDescriptor.depthStencil: JavaScript omits depthStencil to disable depth/stencil state; this API also accepts explicit null, and omission or null both pass a null facade pointer
// - GPURenderPipelineDescriptor.fragment: JavaScript omits fragment for a no-color-output pipeline; this API also accepts explicit null, and omission or null both pass a null facade pointer
// - GPU.requestAdapter: the facade exports only subscript_typegpu_instance_request_adapter(instance) and no options-bearing entry point, so this API drops the IDL options argument and polls the facade future explicitly
// - GPUAdapter.requestDevice: GPUDeviceDescriptor is lowered through the facade's WithDescriptor route and requiredFeatures entries are passed without filtering, so an unsupported feature fails the request rather than being dropped; JavaScript rejects requestDevice with a DOMException on failure, while this API resolves null without an error reason or message
// - GPUDeviceDescriptor.requiredLimits: WebIDL accepts an open record<DOMString, GPUSize64>, while this API uses a fixed-key GPURequiredLimits descriptor for the 32 fields exposed by the facade (29 u32 and 3 u64), so arbitrary keys, the four compatibility-only limits, and values above u32 for narrowed fields are unavailable; public zero is reserved for omission and cannot request a real zero-valued limit, lowering u32 omission to WGPU_LIMIT_U32_UNDEFINED and leaving the three u64 fields for the facade's F13 zero-to-undefined rule
// - GPUDevice.@constructor: JavaScript exposes no GPUDevice constructor; static methods are unavailable, so this API keeps a public owning constructor that accepts private raw-handle fields, acquires and caches one queue reference through subscript_typegpu_device_get_queue, and is paired with hostOwnedGPUDevice, whose GPUHostOwnedDevice wraps a host-owned device, exposes the same creation methods but neither dispose nor destroy, and returns a new owned queue wrapper from each queue call
// - GPUDevice.queue: user-defined accessors are unavailable, so the IDL attribute is a cached zero-argument method
// - GPUBuffer.size: user-defined accessors are unavailable, so the IDL attribute is a zero-argument method
// - GPUBuffer.usage: user-defined accessors are unavailable, so the IDL attribute is a zero-argument method
// - GPUBuffer.mapState: user-defined accessors are unavailable, so the IDL string-enum attribute is a zero-argument method with generated reverse lowering
// - GPUBuffer.mapAsync: the facade accepts explicit offset and size values and exposes mapping completion as Promise<boolean> while encapsulating future polling
// - GPUBuffer.getMappedRange: ArrayBuffer and raw pointers are unavailable, so readMappedRange(offset, size) and writeMappedRange(offset, data) require explicit ranges; failed reads return an empty u8[] indistinguishable from zero-size success
// - GPUBuffer.readMappedRangeF32: the IDL spells this element type as Float32Array inside BufferSource, and subscript has no typed-array view; offset counts bytes and count counts elements
// - GPUQueue.writeBuffer: the facade accepts a complete byte array and drops IDL's dataOffset and size arguments
// - GPUQueue.writeBufferF32: the IDL spells this element type as Float32Array inside BufferSource, and subscript has no typed-array view; bufferOffset counts bytes and data length counts elements
// - GPUQueue.onSubmittedWorkDone: the facade exposes completion status as Promise<boolean> while encapsulating future polling
// - GPUAdapter.features: the IDL GPUSupportedFeatures setlike attribute becomes hasFeature(name); without a set type or facade enumeration, callers lose the stable set object, iteration, size, and features.has(name) spelling
// - GPUAdapter.limits: the readonly SameObject limits attribute becomes limits(), which returns a fresh GPUSupportedLimits record; a failed i32 facade fill returns null rather than a success-looking zero record
// - GPUAdapter.info: the readonly SameObject info attribute becomes info(), which returns a fresh GPUAdapterInfo record; a failed boolean facade fill returns null rather than a success-looking empty record
// - GPUDevice.features: the IDL GPUSupportedFeatures setlike attribute becomes hasFeature(name); without a set type or facade enumeration, callers lose the stable set object, iteration, size, and features.has(name) spelling
// - GPUDevice.limits: the readonly SameObject limits attribute becomes limits(), which returns a fresh GPUSupportedLimits record; a failed i32 facade fill returns null rather than a success-looking zero record
// - GPUDevice.adapterInfo: the readonly SameObject adapterInfo attribute becomes adapterInfo(), which returns a fresh GPUAdapterInfo record; a failed boolean facade fill returns null rather than a success-looking empty record
// - GPUDevice.createComputePipelineAsync: JavaScript rejects with a GPUPipelineError carrying reason and message, while this API resolves null and carries neither directly; callers must use a separate device error scope to obtain the backend error type and message
// - GPUDevice.createRenderPipelineAsync: JavaScript rejects with a GPUPipelineError carrying reason and message, while this API resolves null and carries neither directly; callers must use a separate device error scope to obtain the backend error type and message
// - GPUDevice.createCommandEncoder: JavaScript omits the empty descriptor on createCommandEncoder(); this API keeps a typed createCommandEncoder(descriptor) and provides createCommandEncoderDefault() for omission
// - GPUDevice.lost: JavaScript exposes a once-resolving device.lost promise attribute that can be awaited at setup; without promise attributes this API provides non-consuming deviceLostInfo(), so callers must poll at chosen points and may read the same record repeatedly; each call pumps the instance event loop and can complete unrelated futures
// - GPUDevice.popErrorScope: JavaScript rejects a failed pop and resolves a GPUError subclass or null; this API polls the F6 delivery separately, then converts a captured fill to flattened GPUError, so delivery-status failure, fill-take failure, and a successful NoError fill all resolve null and carry no rejection detail
// - GPUDevice.onuncapturederror: JavaScript handler registration becomes consuming nextUncapturedError() over the F11 FIFO; callers lose EventTarget delivery and preventDefault, errors are observed only when the program drains, and unlike deviceLostInfo() the drain does not pump the instance event loop
// - GPUDevice.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPUBuffer.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPUQueue.submit: the public method keeps IDL's commandBuffers name while explicitly joining the facade's R11-collapsed commands handle array; the array shape is unchanged
// - GPUQueue.writeTexture: the facade reorders the IDL byte source after its three lowered copy dictionaries and accepts u8[] instead of AllowSharedBufferSource
// - GPUQueue.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPUTexture.width: user-defined accessors are unavailable, so the IDL attribute is a zero-argument method
// - GPUTexture.height: user-defined accessors are unavailable, so the IDL attribute is a zero-argument method
// - GPUTexture.depthOrArrayLayers: user-defined accessors are unavailable, so the IDL attribute is a zero-argument method
// - GPUTexture.mipLevelCount: user-defined accessors are unavailable, so the IDL attribute is a zero-argument method
// - GPUTexture.sampleCount: user-defined accessors are unavailable, so the IDL attribute is a zero-argument method
// - GPUTexture.dimension: user-defined accessors are unavailable, so the IDL string-enum attribute is a zero-argument method with generated reverse lowering
// - GPUTexture.format: user-defined accessors are unavailable, so the IDL string-enum attribute is a zero-argument method with generated reverse lowering
// - GPUTexture.usage: user-defined accessors are unavailable, so the IDL attribute is a zero-argument method
// - GPUTexture.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPUTextureView.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPUSampler.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPUTextureViewDescriptor.mipLevelCount: WebIDL permits omission, while this API requires an explicit mipLevelCount instead of exposing the facade's undefined-count sentinel as a descriptor default, so callers lose backend-derived remaining-level selection
// - GPUTextureViewDescriptor.arrayLayerCount: WebIDL permits omission, while this API requires an explicit arrayLayerCount instead of exposing the facade's undefined-count sentinel as a descriptor default, so callers lose backend-derived remaining-layer selection
// - GPUTexelCopyBufferLayout.bytesPerRow: WebIDL permits omission, while this API requires an explicit bytesPerRow instead of exposing the facade's undefined-stride sentinel as a descriptor default, so callers lose the omission form where WebGPU permits inferred layout
// - GPUTexelCopyBufferLayout.rowsPerImage: WebIDL permits omission, while this API requires an explicit rowsPerImage instead of exposing the facade's undefined-stride sentinel as a descriptor default, so callers lose the omission form where WebGPU permits inferred layout
// - GPUBindGroupLayout.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPUBindGroup.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPUPipelineLayout.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPUBindGroupLayoutEntry.binding: the facade's bindingArraySize field is fixed to zero because JavaScript bind-group-layout entries have no binding-array member in the selected IDL
// - GPUBindGroupLayoutEntry.buffer: the optional JavaScript buffer layout becomes GPUBufferBindingLayout | null; null is lowered to the facade BindingNotUsed sentinel, while setting more than one layout kind remains a backend validation error rather than a type error
// - GPUBindGroupLayoutEntry.sampler: the optional JavaScript sampler layout becomes GPUSamplerBindingLayout | null; null is lowered to the facade BindingNotUsed sentinel, while setting more than one layout kind remains a backend validation error rather than a type error
// - GPUBindGroupLayoutEntry.texture: the optional JavaScript texture layout becomes GPUTextureBindingLayout | null; null is lowered to the facade BindingNotUsed sentinel, while setting more than one layout kind remains a backend validation error rather than a type error
// - GPUBindGroupLayoutEntry.storageTexture: the optional JavaScript storage-texture layout becomes GPUStorageTextureBindingLayout | null; null is lowered to the facade BindingNotUsed sentinel, while setting more than one layout kind remains a backend validation error rather than a type error
// - GPUBindGroupEntry.resource: the JavaScript API's one resource union becomes nullable buffer, sampler, and textureView members plus buffer offset and size; GPUTexture shorthand and GPUExternalTexture are unavailable, and setting none or more than one handle is a backend validation error rather than a type error
// - GPUBufferBinding.size: WebIDL omission means the rest of the buffer; the public default 0 lowers through the facade's 0-to-WGPU_WHOLE_SIZE rule, and zero is never a valid binding size, so no valid JavaScript size is conflated and this is not an R16 case
// - GPUPipelineLayoutDescriptor.bindGroupLayouts: the IDL accepts null holes in the bind-group-layout sequence, but the facade handle-element array cannot carry null elements, so the public array accepts only concrete GPUBindGroupLayout wrappers
// - GPURenderBundleEncoderDescriptor.colorFormats: JavaScript permits null holes for unused color attachments, while the facade enum array cannot carry null elements, so callers can provide only concrete GPUTextureFormat values
// - GPUComputePassTimestampWrites.beginningOfPassWriteIndex: IDL omission is represented by the pinned WGPU_QUERY_SET_INDEX_UNDEFINED u32 sentinel; the public default preserves omission without exposing an additional value
// - GPUComputePassTimestampWrites.endOfPassWriteIndex: IDL omission is represented by the pinned WGPU_QUERY_SET_INDEX_UNDEFINED u32 sentinel; the public default preserves omission without exposing an additional value
// - GPURenderPassTimestampWrites.beginningOfPassWriteIndex: IDL omission is represented by the pinned WGPU_QUERY_SET_INDEX_UNDEFINED u32 sentinel; the shared pass-timestamp descriptor preserves omission
// - GPURenderPassTimestampWrites.endOfPassWriteIndex: IDL omission is represented by the pinned WGPU_QUERY_SET_INDEX_UNDEFINED u32 sentinel; the shared pass-timestamp descriptor preserves omission
// - GPUComputePassDescriptor.timestampWrites: IDL omission disables pass timestamp writes; this API also accepts explicit null, and omission or null both pass a null facade pointer
// - GPURenderPassColorAttachment.view: IDL accepts a GPUTexture or GPUTextureView, while the facade accepts only a texture-view handle; callers lose the GPUTexture shorthand and must provide a GPUTextureView, which remains required
// - GPURenderPassColorAttachment.depthSlice: IDL omission is represented by the pinned WGPU_DEPTH_SLICE_UNDEFINED u32 sentinel, preserving the absence required by non-3d attachments
// - GPURenderPassColorAttachment.resolveTarget: IDL accepts a GPUTexture or GPUTextureView when present, while the facade accepts only a texture-view handle; callers lose the GPUTexture shorthand, and omission or null disables resolve
// - GPURenderPassColorAttachment.clearValue: GPUColor is a dictionary-or-sequence union; callers must use { r, g, b, a } and lose the [r, g, b, a] sequence spelling, while omission keeps the IDL zero-color default
// - GPURenderPassDepthStencilAttachment.view: IDL accepts a GPUTexture or GPUTextureView, while the facade accepts only a texture-view handle; callers lose the GPUTexture shorthand and must provide a GPUTextureView
// - GPURenderPassDepthStencilAttachment.depthClearValue: IDL absence uses a NaN sentinel that cannot cross the public API, so callers must state depthClearValue even when the selected load operation ignores it
// - GPURenderPassDescriptor.colorAttachments: IDL permits null holes for unused color attachments, while the facade aggregate array cannot carry null elements, so callers can provide only concrete GPURenderPassColorAttachment entries
// - GPURenderPassDescriptor.depthStencilAttachment: IDL omission disables the depth/stencil attachment; this API also accepts explicit null, and omission or null both pass a null facade pointer
// - GPURenderPassDescriptor.occlusionQuerySet: IDL omission disables occlusion queries; this API also accepts explicit null, and omission or null both pass a null query-set handle
// - GPURenderPassDescriptor.timestampWrites: IDL omission disables render-pass timestamp writes; this API also accepts explicit null, and omission or null both pass a null facade pointer
// - GPUTexelCopyBufferInfo.bytesPerRow: WebIDL permits omission, while this API requires an explicit bytesPerRow instead of exposing the facade's undefined-stride sentinel as a descriptor default, so callers lose the omission form where WebGPU permits inferred layout
// - GPUTexelCopyBufferInfo.rowsPerImage: WebIDL permits omission, while this API requires an explicit rowsPerImage instead of exposing the facade's undefined-stride sentinel as a descriptor default, so callers lose the omission form where WebGPU permits inferred layout
// - GPUCommandEncoder.copyBufferToBuffer: this API keeps the explicit-offset overload and drops JavaScript's source/destination convenience overload; size is required because the facade does not expose the whole-size sentinel
// - GPUCommandEncoder.beginComputePass: JavaScript omits the empty compute-pass descriptor; this API keeps a typed beginComputePass(descriptor) and provides beginComputePassDefault() for omission
// - GPUCommandEncoder.clearBuffer: JavaScript may omit offset and size to clear the rest of the buffer; the facade exposes no whole-size sentinel, so this API requires both an explicit offset and an explicit size
// - GPUCommandEncoder.finish: JavaScript omits the empty command-buffer descriptor; this API keeps finish(descriptor) and provides finishDefault() for omission
// - GPUCommandEncoder.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPUComputePassEncoder.setBindGroup: this API keeps setBindGroup(index, bindGroup, dynamicOffsets = []) and drops the Uint32Array subrange overload with dynamicOffsetsDataStart/dynamicOffsetsDataLength; callers lose zero-copy subrange selection
// - GPUComputePassEncoder.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPURenderPassEncoder.setBindGroup: this API keeps setBindGroup(index, bindGroup, dynamicOffsets = []) and drops the Uint32Array subrange overload with dynamicOffsetsDataStart/dynamicOffsetsDataLength; callers lose zero-copy subrange selection
// - GPURenderPassEncoder.setIndexBuffer: JavaScript may omit offset and size to bind the rest of the index buffer; the facade exposes no whole-size sentinel, so this API requires both, while retaining the IDL indexFormat name
// - GPURenderPassEncoder.setVertexBuffer: JavaScript may omit offset and size to bind the rest of the vertex buffer; the facade exposes no whole-size sentinel, so this API requires both; a null buffer still unbinds the slot
// - GPURenderPassEncoder.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPUCommandBuffer.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPURenderBundleEncoder.finish: JavaScript omits the empty render-bundle descriptor; this API keeps finish(descriptor) and provides finishDefault() for omission
// - GPURenderBundleEncoder.setBindGroup: this API keeps setBindGroup(index, bindGroup, dynamicOffsets = []) and drops the Uint32Array subrange overload with dynamicOffsetsDataStart/dynamicOffsetsDataLength; callers lose zero-copy subrange selection
// - GPURenderBundleEncoder.setIndexBuffer: JavaScript may omit offset and size to bind the rest of the index buffer; the facade exposes no whole-size sentinel, so this API requires both, while retaining the IDL indexFormat name
// - GPURenderBundleEncoder.setVertexBuffer: JavaScript may omit offset and size to bind the rest of the vertex buffer; the facade exposes no whole-size sentinel, so this API requires both; a null buffer still unbinds the slot
// - GPURenderBundleEncoder.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPURenderBundle.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPUQuerySet.type: user-defined accessors are unavailable, so the IDL type attribute is a zero-argument method
// - GPUQuerySet.count: user-defined accessors are unavailable, so the IDL count attribute is a zero-argument method
// - GPUQuerySet.label: the readable/writable JavaScript label becomes a write-only label(value) method because user-defined accessors and a facade label getter are unavailable
// - GPUError.message: GPUValidationError, GPUOutOfMemoryError, and GPUInternalError are flattened into GPUError(type, message); callers retain a string type but lose subclass constructors and err instanceof GPUValidationError-style checks
// - GPUBufferUsage: static fields and user-defined namespaces are unavailable, so WebIDL namespaces become numeric singleton objects
// - GPUMapMode: static fields and user-defined namespaces are unavailable, so WebIDL namespaces become numeric singleton objects
// - GPUTextureUsage: static fields and user-defined namespaces are unavailable, so WebIDL namespaces become numeric singleton objects
// - GPUShaderStage: static fields and user-defined namespaces are unavailable, so WebIDL namespaces become numeric singleton objects
// - GPUColorWrite: static fields and user-defined namespaces are unavailable, so WebIDL namespaces become numeric singleton objects
// - GPUExtent3D: the WebIDL dictionary-or-sequence union is represented by its dictionary branch because subscript cannot express that union
// - GPUOrigin3D: the WebIDL dictionary-or-sequence union is represented by its dictionary branch because subscript cannot express that union
// - GPUColor: GPUColor is a dictionary-or-sequence union; this API keeps the dictionary branch, so JavaScript's [r, g, b, a] sequence spelling is unavailable
// - GPUComputePassTimestampWrites: the facade uses one pass-timestamp aggregate for the structurally identical compute and render IDL dictionaries
// - GPUProgrammableStage: the facade names the shared programmable-stage aggregate SubscriptTypegpuComputeState even though the same public dictionary shape seeds compute, vertex, and fragment state
// - GPURenderPassTimestampWrites: the structurally identical render timestamp dictionary shares GPUPassTimestampWrites with compute because the facade exposes one SubscriptTypegpuPassTimestampWrites aggregate
// - GPUBufferBinding: GPUBufferBinding remains a public IDL descriptor, but the flattened GPUBindingResource facade stores its fields directly in SubscriptTypegpuBindGroupEntry, so there is no standalone boundary aggregate
// - GPUTextureFormat."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUTextureDimension."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUTextureViewDimension."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUTextureAspect."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUAddressMode."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUFilterMode."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUMipmapFilterMode."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUCompareFunction."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUBufferBindingType."binding-not-used": the generated CEnum alias exposes boundary-only string member "binding-not-used" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUBufferBindingType."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 1; the pinned WebIDL enum does not declare it
// - GPUSamplerBindingType."binding-not-used": the generated CEnum alias exposes boundary-only string member "binding-not-used" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUSamplerBindingType."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 1; the pinned WebIDL enum does not declare it
// - GPUTextureSampleType."binding-not-used": the generated CEnum alias exposes boundary-only string member "binding-not-used" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUTextureSampleType."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 1; the pinned WebIDL enum does not declare it
// - GPUStorageTextureAccess."binding-not-used": the generated CEnum alias exposes boundary-only string member "binding-not-used" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUStorageTextureAccess."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 1; the pinned WebIDL enum does not declare it
// - GPUVertexStepMode."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUPrimitiveTopology."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUIndexFormat."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUFrontFace."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUCullMode."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUBlendFactor."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUBlendOperation."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUStencilOperation."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPULoadOp."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUStoreOp."undefined": the generated CEnum alias exposes boundary-only string member "undefined" for facade wire value 0; the pinned WebIDL enum does not declare it
// - GPUDeviceLostReason."callback-cancelled": the generated CEnum alias exposes boundary-only string member "callback-cancelled" for facade wire value 3; the pinned WebIDL enum does not declare it
// - GPUDeviceLostReason."failed-creation": the generated CEnum alias exposes boundary-only string member "failed-creation" for facade wire value 4; the pinned WebIDL enum does not declare it
// - GPUError: F11 materializes the facade-owned type and message fill as an immutable API result; GPUErrorType is a project-defined discriminator rather than a pinned-IDL enum, GPUError gains a public constructor the IDL does not declare, and NoError becomes null rather than a GPUError
// - GPUDeviceLostInfo: F11 materializes the facade-owned reason and message fill as an immutable API result
// - GPUSupportedLimits: the pinned IDL exposes 36 readonly limit attributes while the facade fill carries 32; this API returns those 32 as an immutable result class with a public constructor, preserves all three u64 fields, and excludes the four stage-specific storage-limit attributes the facade does not carry
// - GPUAdapterInfo: the pinned IDL exposes seven readonly adapter-info attributes while the facade fill carries only the four facade-public strings; this API returns those strings as an immutable result class with a public constructor, and does not invent public IDL surface for the facade-only backend type, adapter type, vendor ID, or device ID
// - GPUValidationError: the public GPUError type field preserves validation identity, but JavaScript subclass construction and err instanceof GPUValidationError are unavailable
// - GPUOutOfMemoryError: the public GPUError type field preserves out-of-memory identity, but JavaScript subclass construction and err instanceof GPUOutOfMemoryError are unavailable
// - GPUInternalError: the public GPUError type field preserves internal identity, but JavaScript subclass construction and err instanceof GPUInternalError are unavailable

export type GPUErrorType = "validation" | "out-of-memory" | "internal" | "unknown";

export class GPUError {
  readonly type: GPUErrorType;
  readonly message: string;

  constructor(type: GPUErrorType, message: string) {
    this.type = type;
    this.message = message;
  }
}

function fromSubscriptTypegpuErrorRecord(value: SubscriptTypegpuErrorRecord): GPUError | null {
  switch (value.type) {
    case SubscriptTypegpuErrorType.SUBSCRIPT_TYPEGPU_ERROR_TYPE_VALIDATION:
      return new GPUError(
        "validation",
        value.message,
      );
    case SubscriptTypegpuErrorType.SUBSCRIPT_TYPEGPU_ERROR_TYPE_OUT_OF_MEMORY:
      return new GPUError(
        "out-of-memory",
        value.message,
      );
    case SubscriptTypegpuErrorType.SUBSCRIPT_TYPEGPU_ERROR_TYPE_INTERNAL:
      return new GPUError(
        "internal",
        value.message,
      );
    case SubscriptTypegpuErrorType.SUBSCRIPT_TYPEGPU_ERROR_TYPE_UNKNOWN:
      return new GPUError(
        "unknown",
        value.message,
      );
    case SubscriptTypegpuErrorType.SUBSCRIPT_TYPEGPU_ERROR_TYPE_NO_ERROR:
      return null;
  }
  unreachable();
}

export class GPUDeviceLostInfo {
  readonly reason: GPUDeviceLostReason;
  readonly message: string;

  constructor(reason: GPUDeviceLostReason, message: string) {
    this.reason = reason;
    this.message = message;
  }
}

function fromSubscriptTypegpuLostRecord(value: SubscriptTypegpuLostRecord): GPUDeviceLostInfo {
  return new GPUDeviceLostInfo(
    value.reason,
    value.message,
  );
}

export class GPUSupportedLimits {
  readonly maxTextureDimension1D: u32;
  readonly maxTextureDimension2D: u32;
  readonly maxTextureDimension3D: u32;
  readonly maxTextureArrayLayers: u32;
  readonly maxBindGroups: u32;
  readonly maxBindGroupsPlusVertexBuffers: u32;
  readonly maxBindingsPerBindGroup: u32;
  readonly maxDynamicUniformBuffersPerPipelineLayout: u32;
  readonly maxDynamicStorageBuffersPerPipelineLayout: u32;
  readonly maxSampledTexturesPerShaderStage: u32;
  readonly maxSamplersPerShaderStage: u32;
  readonly maxStorageBuffersPerShaderStage: u32;
  readonly maxStorageTexturesPerShaderStage: u32;
  readonly maxUniformBuffersPerShaderStage: u32;
  readonly maxUniformBufferBindingSize: u64;
  readonly maxStorageBufferBindingSize: u64;
  readonly minUniformBufferOffsetAlignment: u32;
  readonly minStorageBufferOffsetAlignment: u32;
  readonly maxVertexBuffers: u32;
  readonly maxBufferSize: u64;
  readonly maxVertexAttributes: u32;
  readonly maxVertexBufferArrayStride: u32;
  readonly maxInterStageShaderVariables: u32;
  readonly maxColorAttachments: u32;
  readonly maxColorAttachmentBytesPerSample: u32;
  readonly maxComputeWorkgroupStorageSize: u32;
  readonly maxComputeInvocationsPerWorkgroup: u32;
  readonly maxComputeWorkgroupSizeX: u32;
  readonly maxComputeWorkgroupSizeY: u32;
  readonly maxComputeWorkgroupSizeZ: u32;
  readonly maxComputeWorkgroupsPerDimension: u32;
  readonly maxImmediateSize: u32;

  constructor(maxTextureDimension1D: u32, maxTextureDimension2D: u32, maxTextureDimension3D: u32, maxTextureArrayLayers: u32, maxBindGroups: u32, maxBindGroupsPlusVertexBuffers: u32, maxBindingsPerBindGroup: u32, maxDynamicUniformBuffersPerPipelineLayout: u32, maxDynamicStorageBuffersPerPipelineLayout: u32, maxSampledTexturesPerShaderStage: u32, maxSamplersPerShaderStage: u32, maxStorageBuffersPerShaderStage: u32, maxStorageTexturesPerShaderStage: u32, maxUniformBuffersPerShaderStage: u32, maxUniformBufferBindingSize: u64, maxStorageBufferBindingSize: u64, minUniformBufferOffsetAlignment: u32, minStorageBufferOffsetAlignment: u32, maxVertexBuffers: u32, maxBufferSize: u64, maxVertexAttributes: u32, maxVertexBufferArrayStride: u32, maxInterStageShaderVariables: u32, maxColorAttachments: u32, maxColorAttachmentBytesPerSample: u32, maxComputeWorkgroupStorageSize: u32, maxComputeInvocationsPerWorkgroup: u32, maxComputeWorkgroupSizeX: u32, maxComputeWorkgroupSizeY: u32, maxComputeWorkgroupSizeZ: u32, maxComputeWorkgroupsPerDimension: u32, maxImmediateSize: u32) {
    this.maxTextureDimension1D = maxTextureDimension1D;
    this.maxTextureDimension2D = maxTextureDimension2D;
    this.maxTextureDimension3D = maxTextureDimension3D;
    this.maxTextureArrayLayers = maxTextureArrayLayers;
    this.maxBindGroups = maxBindGroups;
    this.maxBindGroupsPlusVertexBuffers = maxBindGroupsPlusVertexBuffers;
    this.maxBindingsPerBindGroup = maxBindingsPerBindGroup;
    this.maxDynamicUniformBuffersPerPipelineLayout = maxDynamicUniformBuffersPerPipelineLayout;
    this.maxDynamicStorageBuffersPerPipelineLayout = maxDynamicStorageBuffersPerPipelineLayout;
    this.maxSampledTexturesPerShaderStage = maxSampledTexturesPerShaderStage;
    this.maxSamplersPerShaderStage = maxSamplersPerShaderStage;
    this.maxStorageBuffersPerShaderStage = maxStorageBuffersPerShaderStage;
    this.maxStorageTexturesPerShaderStage = maxStorageTexturesPerShaderStage;
    this.maxUniformBuffersPerShaderStage = maxUniformBuffersPerShaderStage;
    this.maxUniformBufferBindingSize = maxUniformBufferBindingSize;
    this.maxStorageBufferBindingSize = maxStorageBufferBindingSize;
    this.minUniformBufferOffsetAlignment = minUniformBufferOffsetAlignment;
    this.minStorageBufferOffsetAlignment = minStorageBufferOffsetAlignment;
    this.maxVertexBuffers = maxVertexBuffers;
    this.maxBufferSize = maxBufferSize;
    this.maxVertexAttributes = maxVertexAttributes;
    this.maxVertexBufferArrayStride = maxVertexBufferArrayStride;
    this.maxInterStageShaderVariables = maxInterStageShaderVariables;
    this.maxColorAttachments = maxColorAttachments;
    this.maxColorAttachmentBytesPerSample = maxColorAttachmentBytesPerSample;
    this.maxComputeWorkgroupStorageSize = maxComputeWorkgroupStorageSize;
    this.maxComputeInvocationsPerWorkgroup = maxComputeInvocationsPerWorkgroup;
    this.maxComputeWorkgroupSizeX = maxComputeWorkgroupSizeX;
    this.maxComputeWorkgroupSizeY = maxComputeWorkgroupSizeY;
    this.maxComputeWorkgroupSizeZ = maxComputeWorkgroupSizeZ;
    this.maxComputeWorkgroupsPerDimension = maxComputeWorkgroupsPerDimension;
    this.maxImmediateSize = maxImmediateSize;
  }
}

function fromSubscriptTypegpuLimits(value: SubscriptTypegpuLimits): GPUSupportedLimits {
  return new GPUSupportedLimits(
    value.maxTextureDimension1D,
    value.maxTextureDimension2D,
    value.maxTextureDimension3D,
    value.maxTextureArrayLayers,
    value.maxBindGroups,
    value.maxBindGroupsPlusVertexBuffers,
    value.maxBindingsPerBindGroup,
    value.maxDynamicUniformBuffersPerPipelineLayout,
    value.maxDynamicStorageBuffersPerPipelineLayout,
    value.maxSampledTexturesPerShaderStage,
    value.maxSamplersPerShaderStage,
    value.maxStorageBuffersPerShaderStage,
    value.maxStorageTexturesPerShaderStage,
    value.maxUniformBuffersPerShaderStage,
    value.maxUniformBufferBindingSize,
    value.maxStorageBufferBindingSize,
    value.minUniformBufferOffsetAlignment,
    value.minStorageBufferOffsetAlignment,
    value.maxVertexBuffers,
    value.maxBufferSize,
    value.maxVertexAttributes,
    value.maxVertexBufferArrayStride,
    value.maxInterStageShaderVariables,
    value.maxColorAttachments,
    value.maxColorAttachmentBytesPerSample,
    value.maxComputeWorkgroupStorageSize,
    value.maxComputeInvocationsPerWorkgroup,
    value.maxComputeWorkgroupSizeX,
    value.maxComputeWorkgroupSizeY,
    value.maxComputeWorkgroupSizeZ,
    value.maxComputeWorkgroupsPerDimension,
    value.maxImmediateSize,
  );
}

export class GPUAdapterInfo {
  readonly vendor: string;
  readonly architecture: string;
  readonly device: string;
  readonly description: string;

  constructor(vendor: string, architecture: string, device: string, description: string) {
    this.vendor = vendor;
    this.architecture = architecture;
    this.device = device;
    this.description = description;
  }
}

function fromSubscriptTypegpuAdapterInfo(value: SubscriptTypegpuAdapterInfo): GPUAdapterInfo {
  return new GPUAdapterInfo(
    value.vendor,
    value.architecture,
    value.device,
    value.description,
  );
}

@Descriptor
export class GPUPipelineConstantEntry {
  key!: string;
  value!: f64;
}

function defaultRequiredLimitU32(value: u32 = 0): u32 {
  return value;
}

function defaultRequiredLimitU64(value: u64 = 0): u64 {
  return value;
}

function toRequiredLimitU32(value: u32): u32 {
  if (value === 0) {
    return 4294967295;
  }
  return value;
}

@Descriptor
export class GPURequiredLimits {
  maxTextureDimension1D?: u32 = 0;
  maxTextureDimension2D?: u32 = 0;
  maxTextureDimension3D?: u32 = 0;
  maxTextureArrayLayers?: u32 = 0;
  maxBindGroups?: u32 = 0;
  maxBindGroupsPlusVertexBuffers?: u32 = 0;
  maxBindingsPerBindGroup?: u32 = 0;
  maxDynamicUniformBuffersPerPipelineLayout?: u32 = 0;
  maxDynamicStorageBuffersPerPipelineLayout?: u32 = 0;
  maxSampledTexturesPerShaderStage?: u32 = 0;
  maxSamplersPerShaderStage?: u32 = 0;
  maxStorageBuffersPerShaderStage?: u32 = 0;
  maxStorageTexturesPerShaderStage?: u32 = 0;
  maxUniformBuffersPerShaderStage?: u32 = 0;
  maxUniformBufferBindingSize?: u64 = 0;
  maxStorageBufferBindingSize?: u64 = 0;
  minUniformBufferOffsetAlignment?: u32 = 0;
  minStorageBufferOffsetAlignment?: u32 = 0;
  maxVertexBuffers?: u32 = 0;
  maxBufferSize?: u64 = 0;
  maxVertexAttributes?: u32 = 0;
  maxVertexBufferArrayStride?: u32 = 0;
  maxInterStageShaderVariables?: u32 = 0;
  maxColorAttachments?: u32 = 0;
  maxColorAttachmentBytesPerSample?: u32 = 0;
  maxComputeWorkgroupStorageSize?: u32 = 0;
  maxComputeInvocationsPerWorkgroup?: u32 = 0;
  maxComputeWorkgroupSizeX?: u32 = 0;
  maxComputeWorkgroupSizeY?: u32 = 0;
  maxComputeWorkgroupSizeZ?: u32 = 0;
  maxComputeWorkgroupsPerDimension?: u32 = 0;
  maxImmediateSize?: u32 = 0;
}

function toSubscriptTypegpuLimits(value: GPURequiredLimits): SubscriptTypegpuLimits {
  return new SubscriptTypegpuLimits(
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxTextureDimension1D)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxTextureDimension2D)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxTextureDimension3D)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxTextureArrayLayers)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxBindGroups)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxBindGroupsPlusVertexBuffers)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxBindingsPerBindGroup)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxDynamicUniformBuffersPerPipelineLayout)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxDynamicStorageBuffersPerPipelineLayout)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxSampledTexturesPerShaderStage)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxSamplersPerShaderStage)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxStorageBuffersPerShaderStage)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxStorageTexturesPerShaderStage)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxUniformBuffersPerShaderStage)),
    defaultRequiredLimitU64(value.maxUniformBufferBindingSize),
    defaultRequiredLimitU64(value.maxStorageBufferBindingSize),
    toRequiredLimitU32(defaultRequiredLimitU32(value.minUniformBufferOffsetAlignment)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.minStorageBufferOffsetAlignment)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxVertexBuffers)),
    defaultRequiredLimitU64(value.maxBufferSize),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxVertexAttributes)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxVertexBufferArrayStride)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxInterStageShaderVariables)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxColorAttachments)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxColorAttachmentBytesPerSample)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxComputeWorkgroupStorageSize)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxComputeInvocationsPerWorkgroup)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxComputeWorkgroupSizeX)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxComputeWorkgroupSizeY)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxComputeWorkgroupSizeZ)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxComputeWorkgroupsPerDimension)),
    toRequiredLimitU32(defaultRequiredLimitU32(value.maxImmediateSize)),
  );
}

function isGPURequiredLimitsEmpty(value: GPURequiredLimits): boolean {
  return defaultRequiredLimitU32(value.maxTextureDimension1D) === 0
    && defaultRequiredLimitU32(value.maxTextureDimension2D) === 0
    && defaultRequiredLimitU32(value.maxTextureDimension3D) === 0
    && defaultRequiredLimitU32(value.maxTextureArrayLayers) === 0
    && defaultRequiredLimitU32(value.maxBindGroups) === 0
    && defaultRequiredLimitU32(value.maxBindGroupsPlusVertexBuffers) === 0
    && defaultRequiredLimitU32(value.maxBindingsPerBindGroup) === 0
    && defaultRequiredLimitU32(value.maxDynamicUniformBuffersPerPipelineLayout) === 0
    && defaultRequiredLimitU32(value.maxDynamicStorageBuffersPerPipelineLayout) === 0
    && defaultRequiredLimitU32(value.maxSampledTexturesPerShaderStage) === 0
    && defaultRequiredLimitU32(value.maxSamplersPerShaderStage) === 0
    && defaultRequiredLimitU32(value.maxStorageBuffersPerShaderStage) === 0
    && defaultRequiredLimitU32(value.maxStorageTexturesPerShaderStage) === 0
    && defaultRequiredLimitU32(value.maxUniformBuffersPerShaderStage) === 0
    && defaultRequiredLimitU64(value.maxUniformBufferBindingSize) === 0
    && defaultRequiredLimitU64(value.maxStorageBufferBindingSize) === 0
    && defaultRequiredLimitU32(value.minUniformBufferOffsetAlignment) === 0
    && defaultRequiredLimitU32(value.minStorageBufferOffsetAlignment) === 0
    && defaultRequiredLimitU32(value.maxVertexBuffers) === 0
    && defaultRequiredLimitU64(value.maxBufferSize) === 0
    && defaultRequiredLimitU32(value.maxVertexAttributes) === 0
    && defaultRequiredLimitU32(value.maxVertexBufferArrayStride) === 0
    && defaultRequiredLimitU32(value.maxInterStageShaderVariables) === 0
    && defaultRequiredLimitU32(value.maxColorAttachments) === 0
    && defaultRequiredLimitU32(value.maxColorAttachmentBytesPerSample) === 0
    && defaultRequiredLimitU32(value.maxComputeWorkgroupStorageSize) === 0
    && defaultRequiredLimitU32(value.maxComputeInvocationsPerWorkgroup) === 0
    && defaultRequiredLimitU32(value.maxComputeWorkgroupSizeX) === 0
    && defaultRequiredLimitU32(value.maxComputeWorkgroupSizeY) === 0
    && defaultRequiredLimitU32(value.maxComputeWorkgroupSizeZ) === 0
    && defaultRequiredLimitU32(value.maxComputeWorkgroupsPerDimension) === 0
    && defaultRequiredLimitU32(value.maxImmediateSize) === 0;
}

@Descriptor
export class GPUBufferDescriptor {
  label?: string = "";
  size!: u64;
  usage!: u64;
  mappedAtCreation?: boolean = false;
}

@Descriptor
export class GPUTextureDescriptor {
  label?: string = "";
  size!: GPUExtent3D;
  mipLevelCount?: u32 = 1;
  sampleCount?: u32 = 1;
  dimension?: GPUTextureDimension = "2d";
  format!: GPUTextureFormat;
  usage!: u64;
  viewFormats?: GPUTextureFormat[] = [];
}

@Descriptor
export class GPUTextureViewDescriptor {
  label?: string = "";
  format?: GPUTextureFormat;
  dimension?: GPUTextureViewDimension;
  usage?: u64 = 0;
  aspect?: GPUTextureAspect = "all";
  baseMipLevel?: u32 = 0;
  mipLevelCount!: u32;
  baseArrayLayer?: u32 = 0;
  arrayLayerCount!: u32;
}

@Descriptor
export class GPUSamplerDescriptor {
  label?: string = "";
  addressModeU?: GPUAddressMode = "clamp-to-edge";
  addressModeV?: GPUAddressMode = "clamp-to-edge";
  addressModeW?: GPUAddressMode = "clamp-to-edge";
  magFilter?: GPUFilterMode = "nearest";
  minFilter?: GPUFilterMode = "nearest";
  mipmapFilter?: GPUMipmapFilterMode = "nearest";
  lodMinClamp?: f32 = 0;
  lodMaxClamp?: f32 = 32;
  compare?: GPUCompareFunction;
  maxAnisotropy?: u16 = 1;
}

@Descriptor
export class GPUExtent3D {
  width!: u32;
  height?: u32 = 1;
  depthOrArrayLayers?: u32 = 1;
}

@Descriptor
export class GPUOrigin3D {
  x?: u32 = 0;
  y?: u32 = 0;
  z?: u32 = 0;
}

@Descriptor
export class GPUTexelCopyTextureInfo {
  texture!: GPUTexture;
  mipLevel?: u32 = 0;
  origin?: GPUOrigin3D = {};
  aspect?: GPUTextureAspect = "all";
}

@Descriptor
export class GPUTexelCopyBufferLayout {
  offset?: u64 = 0;
  bytesPerRow!: u32;
  rowsPerImage!: u32;
}

@Descriptor
export class GPUBufferBindingLayout {
  type?: GPUBufferBindingType = "uniform";
  hasDynamicOffset?: boolean = false;
  minBindingSize?: u64 = 0;
}

@Descriptor
export class GPUSamplerBindingLayout {
  type?: GPUSamplerBindingType = "filtering";
}

@Descriptor
export class GPUTextureBindingLayout {
  sampleType?: GPUTextureSampleType = "float";
  viewDimension?: GPUTextureViewDimension = "2d";
  multisampled?: boolean = false;
}

@Descriptor
export class GPUStorageTextureBindingLayout {
  access?: GPUStorageTextureAccess = "write-only";
  format!: GPUTextureFormat;
  viewDimension?: GPUTextureViewDimension = "2d";
}

@Descriptor
export class GPUBindGroupLayoutEntry {
  binding!: u32;
  visibility!: u64;
  buffer?: GPUBufferBindingLayout | null = null;
  sampler?: GPUSamplerBindingLayout | null = null;
  texture?: GPUTextureBindingLayout | null = null;
  storageTexture?: GPUStorageTextureBindingLayout | null = null;
}

@Descriptor
export class GPUBindGroupLayoutDescriptor {
  label?: string = "";
  entries!: GPUBindGroupLayoutEntry[];
}

@Descriptor
export class GPUBindGroupEntry {
  binding!: u32;
  buffer?: GPUBuffer | null = null;
  offset?: u64 = 0;
  size?: u64 = 0;
  sampler?: GPUSampler | null = null;
  textureView?: GPUTextureView | null = null;
}

@Descriptor
export class GPUBindGroupDescriptor {
  label?: string = "";
  layout!: GPUBindGroupLayout;
  entries!: GPUBindGroupEntry[];
}

@Descriptor
export class GPUBufferBinding {
  buffer!: GPUBuffer;
  offset?: u64 = 0;
  size?: u64 = 0;
}

@Descriptor
export class GPUPipelineLayoutDescriptor {
  label?: string = "";
  bindGroupLayouts!: GPUBindGroupLayout[];
  immediateSize?: u32 = 0;
}

@Descriptor
export class GPUShaderModuleDescriptor {
  label?: string = "";
  code!: string;
}

@Descriptor
export class GPUProgrammableStage {
  module!: GPUShaderModule;
  entryPoint!: string;
  constants?: GPUPipelineConstantEntry[] = [];
}

@Descriptor
export class GPUComputePipelineDescriptor {
  label?: string = "";
  layout?: GPUPipelineLayout | null = null;
  compute!: GPUProgrammableStage;
}

@Descriptor
export class GPUVertexAttribute {
  format!: GPUVertexFormat;
  offset!: u64;
  shaderLocation!: u32;
}

@Descriptor
export class GPUVertexBufferLayout {
  arrayStride!: u64;
  stepMode?: GPUVertexStepMode = "vertex";
  attributes!: GPUVertexAttribute[];
}

@Descriptor
export class GPUVertexState {
  module!: GPUShaderModule;
  entryPoint!: string;
  constants?: GPUPipelineConstantEntry[] = [];
  buffers?: GPUVertexBufferLayout[] = [];
}

@Descriptor
export class GPUPrimitiveState {
  topology?: GPUPrimitiveTopology = "triangle-list";
  stripIndexFormat?: GPUIndexFormat;
  frontFace?: GPUFrontFace = "ccw";
  cullMode?: GPUCullMode = "none";
  unclippedDepth?: boolean = false;
}

@Descriptor
export class GPUStencilFaceState {
  compare?: GPUCompareFunction = "always";
  failOp?: GPUStencilOperation = "keep";
  depthFailOp?: GPUStencilOperation = "keep";
  passOp?: GPUStencilOperation = "keep";
}

@Descriptor
export class GPUDepthStencilState {
  format!: GPUTextureFormat;
  depthWriteEnabled!: boolean;
  depthCompare?: GPUCompareFunction;
  stencilFront?: GPUStencilFaceState = {};
  stencilBack?: GPUStencilFaceState = {};
  stencilReadMask?: u32 = 0xFFFFFFFF;
  stencilWriteMask?: u32 = 0xFFFFFFFF;
  depthBias?: i32 = 0;
  depthBiasSlopeScale?: f32 = 0;
  depthBiasClamp?: f32 = 0;
}

@Descriptor
export class GPUMultisampleState {
  count?: u32 = 1;
  mask?: u32 = 0xFFFFFFFF;
  alphaToCoverageEnabled?: boolean = false;
}

@Descriptor
export class GPUBlendComponent {
  operation?: GPUBlendOperation = "add";
  srcFactor?: GPUBlendFactor = "one";
  dstFactor?: GPUBlendFactor = "zero";
}

@Descriptor
export class GPUBlendState {
  color!: GPUBlendComponent;
  alpha!: GPUBlendComponent;
}

@Descriptor
export class GPUColorTargetState {
  format!: GPUTextureFormat;
  blend?: GPUBlendState | null = null;
  writeMask?: u64 = 0xF;
}

@Descriptor
export class GPUFragmentState {
  module!: GPUShaderModule;
  entryPoint!: string;
  constants?: GPUPipelineConstantEntry[] = [];
  targets!: GPUColorTargetState[];
}

@Descriptor
export class GPURenderPipelineDescriptor {
  label?: string = "";
  layout?: GPUPipelineLayout | null = null;
  vertex!: GPUVertexState;
  primitive?: GPUPrimitiveState = {};
  depthStencil?: GPUDepthStencilState | null = null;
  multisample?: GPUMultisampleState = {};
  fragment?: GPUFragmentState | null = null;
}

@Descriptor
export class GPUCommandEncoderDescriptor {
  label?: string = "";
}

@Descriptor
export class GPURenderBundleEncoderDescriptor {
  label?: string = "";
  colorFormats!: GPUTextureFormat[];
  depthStencilFormat?: GPUTextureFormat;
  sampleCount?: u32 = 1;
  depthReadOnly?: boolean = false;
  stencilReadOnly?: boolean = false;
}

@Descriptor
export class GPUQuerySetDescriptor {
  label?: string = "";
  type!: GPUQueryType;
  count!: u32;
}

@Descriptor
export class GPUCommandBufferDescriptor {
  label?: string = "";
}

@Descriptor
export class GPUPassTimestampWrites {
  querySet!: GPUQuerySet;
  beginningOfPassWriteIndex?: u32 = 4294967295;
  endOfPassWriteIndex?: u32 = 4294967295;
}

@Descriptor
export class GPUComputePassDescriptor {
  label?: string = "";
  timestampWrites?: GPUPassTimestampWrites | null = null;
}

@Descriptor
export class GPUColor {
  r!: f64;
  g!: f64;
  b!: f64;
  a!: f64;
}

@Descriptor
export class GPURenderPassColorAttachment {
  view!: GPUTextureView;
  depthSlice?: u32 = 4294967295;
  resolveTarget?: GPUTextureView | null = null;
  clearValue?: GPUColor = { r: 0, g: 0, b: 0, a: 0 };
  loadOp!: GPULoadOp;
  storeOp!: GPUStoreOp;
}

@Descriptor
export class GPURenderPassDepthStencilAttachment {
  view!: GPUTextureView;
  depthClearValue!: f32;
  depthLoadOp?: GPULoadOp;
  depthStoreOp?: GPUStoreOp;
  depthReadOnly?: boolean = false;
  stencilClearValue?: u32 = 0;
  stencilLoadOp?: GPULoadOp;
  stencilStoreOp?: GPUStoreOp;
  stencilReadOnly?: boolean = false;
}

@Descriptor
export class GPURenderPassDescriptor {
  label?: string = "";
  colorAttachments!: GPURenderPassColorAttachment[];
  depthStencilAttachment?: GPURenderPassDepthStencilAttachment | null = null;
  occlusionQuerySet?: GPUQuerySet | null = null;
  timestampWrites?: GPUPassTimestampWrites | null = null;
}

@Descriptor
export class GPUTexelCopyBufferInfo {
  offset?: u64 = 0;
  bytesPerRow!: u32;
  rowsPerImage!: u32;
  buffer!: GPUBuffer;
}

@Descriptor
export class GPURenderBundleDescriptor {
  label?: string = "";
}

@Descriptor
export class GPUQueueDescriptor {
  label?: string = "";
}

@Descriptor
export class GPUDeviceDescriptor {
  label?: string = "";
  requiredFeatures?: GPUFeatureName[] = [];
  requiredLimits?: GPURequiredLimits = {};
  defaultQueue?: GPUQueueDescriptor = {};
}

function toOptionalSubscriptTypegpuBufferBindingLayout(value: GPUBufferBindingLayout | null): SubscriptTypegpuBufferBindingLayout {
  if (value === null) {
    return new SubscriptTypegpuBufferBindingLayout("binding-not-used", false, 0);
  }
  return toSubscriptTypegpuBufferBindingLayout(value);
}

function toOptionalSubscriptTypegpuSamplerBindingLayout(value: GPUSamplerBindingLayout | null): SubscriptTypegpuSamplerBindingLayout {
  if (value === null) {
    return new SubscriptTypegpuSamplerBindingLayout("binding-not-used");
  }
  return toSubscriptTypegpuSamplerBindingLayout(value);
}

function toOptionalSubscriptTypegpuTextureBindingLayout(value: GPUTextureBindingLayout | null): SubscriptTypegpuTextureBindingLayout {
  if (value === null) {
    return new SubscriptTypegpuTextureBindingLayout("binding-not-used", "undefined", false);
  }
  return toSubscriptTypegpuTextureBindingLayout(value);
}

function toOptionalSubscriptTypegpuStorageTextureBindingLayout(value: GPUStorageTextureBindingLayout | null): SubscriptTypegpuStorageTextureBindingLayout {
  if (value === null) {
    return new SubscriptTypegpuStorageTextureBindingLayout("binding-not-used", "undefined", "undefined");
  }
  return toSubscriptTypegpuStorageTextureBindingLayout(value);
}

function toSubscriptTypegpuBindGroupLayoutEntryArray(values: GPUBindGroupLayoutEntry[]): SubscriptTypegpuBindGroupLayoutEntry[] {
  const lowered: SubscriptTypegpuBindGroupLayoutEntry[] = [];
  let index: i32 = 0;
  while (index < values.length) {
    lowered.push(toSubscriptTypegpuBindGroupLayoutEntry(values[index]));
    index = index + 1;
  }
  return lowered;
}

function toNullableSubscriptTypegpuBuffer(value: GPUBuffer | null): SubscriptTypegpuBuffer | null {
  if (value === null) {
    return null;
  }
  return value.buffer;
}

function toNullableSubscriptTypegpuSampler(value: GPUSampler | null): SubscriptTypegpuSampler | null {
  if (value === null) {
    return null;
  }
  return value.sampler;
}

function toNullableSubscriptTypegpuTextureView(value: GPUTextureView | null): SubscriptTypegpuTextureView | null {
  if (value === null) {
    return null;
  }
  return value.textureView;
}

function toSubscriptTypegpuBindGroupEntryArray(values: GPUBindGroupEntry[]): SubscriptTypegpuBindGroupEntry[] {
  const lowered: SubscriptTypegpuBindGroupEntry[] = [];
  let index: i32 = 0;
  while (index < values.length) {
    lowered.push(toSubscriptTypegpuBindGroupEntry(values[index]));
    index = index + 1;
  }
  return lowered;
}

function toSubscriptTypegpuBindGroupLayoutArray(values: GPUBindGroupLayout[]): SubscriptTypegpuBindGroupLayout[] {
  const lowered: SubscriptTypegpuBindGroupLayout[] = [];
  let index: i32 = 0;
  while (index < values.length) {
    lowered.push(values[index].bindGroupLayout);
    index = index + 1;
  }
  return lowered;
}

function toSubscriptTypegpuConstantEntry(value: GPUPipelineConstantEntry): SubscriptTypegpuConstantEntry {
  return new SubscriptTypegpuConstantEntry(value.key, value.value);
}

function toSubscriptTypegpuConstantEntryArray(values: GPUPipelineConstantEntry[]): SubscriptTypegpuConstantEntry[] {
  const lowered: SubscriptTypegpuConstantEntry[] = [];
  let index: i32 = 0;
  while (index < values.length) {
    lowered.push(toSubscriptTypegpuConstantEntry(values[index]));
    index = index + 1;
  }
  return lowered;
}

function toNullableSubscriptTypegpuPipelineLayout(value: GPUPipelineLayout | null): SubscriptTypegpuPipelineLayout | null {
  if (value === null) {
    return null;
  }
  return value.pipelineLayout;
}

function toSubscriptTypegpuVertexAttributeArray(values: GPUVertexAttribute[]): SubscriptTypegpuVertexAttribute[] {
  const lowered: SubscriptTypegpuVertexAttribute[] = [];
  let index: i32 = 0;
  while (index < values.length) {
    lowered.push(toSubscriptTypegpuVertexAttribute(values[index]));
    index = index + 1;
  }
  return lowered;
}

function toSubscriptTypegpuVertexBufferLayoutArray(values: GPUVertexBufferLayout[]): SubscriptTypegpuVertexBufferLayout[] {
  const lowered: SubscriptTypegpuVertexBufferLayout[] = [];
  let index: i32 = 0;
  while (index < values.length) {
    lowered.push(toSubscriptTypegpuVertexBufferLayout(values[index]));
    index = index + 1;
  }
  return lowered;
}

function toSubscriptTypegpuOptionalBool(value: boolean): SubscriptTypegpuOptionalBool {
  if (value) {
    return SubscriptTypegpuOptionalBool.SUBSCRIPT_TYPEGPU_OPTIONAL_BOOL_TRUE;
  }
  return SubscriptTypegpuOptionalBool.SUBSCRIPT_TYPEGPU_OPTIONAL_BOOL_FALSE;
}

function toSubscriptTypegpuColorTargetStateArray(values: GPUColorTargetState[]): SubscriptTypegpuColorTargetState[] {
  const lowered: SubscriptTypegpuColorTargetState[] = [];
  let index: i32 = 0;
  while (index < values.length) {
    lowered.push(toSubscriptTypegpuColorTargetState(values[index]));
    index = index + 1;
  }
  return lowered;
}

function toSubscriptTypegpuRenderPassColorAttachmentArray(values: GPURenderPassColorAttachment[]): SubscriptTypegpuRenderPassColorAttachment[] {
  const lowered: SubscriptTypegpuRenderPassColorAttachment[] = [];
  let index: i32 = 0;
  while (index < values.length) {
    lowered.push(toSubscriptTypegpuRenderPassColorAttachment(values[index]));
    index = index + 1;
  }
  return lowered;
}

function toNullableSubscriptTypegpuQuerySet(value: GPUQuerySet | null): SubscriptTypegpuQuerySet | null {
  if (value === null) {
    return null;
  }
  return value.querySet;
}

function toSubscriptTypegpuCommandBufferArray(values: GPUCommandBuffer[]): SubscriptTypegpuCommandBuffer[] {
  const lowered: SubscriptTypegpuCommandBuffer[] = [];
  let index: i32 = 0;
  while (index < values.length) {
    lowered.push(values[index].commandBuffer);
    index = index + 1;
  }
  return lowered;
}

function toNullableSubscriptTypegpuBindGroup(value: GPUBindGroup | null): SubscriptTypegpuBindGroup | null {
  if (value === null) {
    return null;
  }
  return value.bindGroup;
}

function toSubscriptTypegpuRenderBundleArray(values: GPURenderBundle[]): SubscriptTypegpuRenderBundle[] {
  const lowered: SubscriptTypegpuRenderBundle[] = [];
  let index: i32 = 0;
  while (index < values.length) {
    lowered.push(values[index].renderBundle);
    index = index + 1;
  }
  return lowered;
}

function toSubscriptTypegpuBufferDescriptor(value: GPUBufferDescriptor): SubscriptTypegpuBufferDescriptor {
  return new SubscriptTypegpuBufferDescriptor(
    defaultLabel(value.label),
    value.usage,
    value.size,
    defaultMappedAtCreation(value.mappedAtCreation),
  );
}

function toSubscriptTypegpuTextureDescriptor(value: GPUTextureDescriptor): SubscriptTypegpuTextureDescriptor {
  return new SubscriptTypegpuTextureDescriptor(
    defaultLabel(value.label),
    value.usage,
    defaultDimension(value.dimension),
    toSubscriptTypegpuExtent3D(value.size),
    value.format,
    defaultMipLevelCount(value.mipLevelCount),
    defaultSampleCount(value.sampleCount),
    defaultViewFormats(value.viewFormats),
  );
}

function resolveGPUTextureFormatForGPUTextureViewDescriptorFormat(value: GPUTextureViewDescriptor): GPUTextureFormat {
  if (value.format !== undefined) {
    return value.format;
  }
  return "undefined";
}

function resolveGPUTextureViewDimensionForGPUTextureViewDescriptorDimension(value: GPUTextureViewDescriptor): GPUTextureViewDimension {
  if (value.dimension !== undefined) {
    return value.dimension;
  }
  return "undefined";
}

function toSubscriptTypegpuTextureViewDescriptor(value: GPUTextureViewDescriptor): SubscriptTypegpuTextureViewDescriptor {
  return new SubscriptTypegpuTextureViewDescriptor(
    defaultLabel(value.label),
    resolveGPUTextureFormatForGPUTextureViewDescriptorFormat(value),
    resolveGPUTextureViewDimensionForGPUTextureViewDescriptorDimension(value),
    defaultBaseMipLevel(value.baseMipLevel),
    value.mipLevelCount,
    defaultBaseArrayLayer(value.baseArrayLayer),
    value.arrayLayerCount,
    defaultAspect(value.aspect),
    defaultUsage(value.usage),
  );
}

function resolveGPUCompareFunctionForGPUSamplerDescriptorCompare(value: GPUSamplerDescriptor): GPUCompareFunction {
  if (value.compare !== undefined) {
    return value.compare;
  }
  return "undefined";
}

function toSubscriptTypegpuSamplerDescriptor(value: GPUSamplerDescriptor): SubscriptTypegpuSamplerDescriptor {
  return new SubscriptTypegpuSamplerDescriptor(
    defaultLabel(value.label),
    defaultAddressModeU(value.addressModeU),
    defaultAddressModeV(value.addressModeV),
    defaultAddressModeW(value.addressModeW),
    defaultMagFilter(value.magFilter),
    defaultMinFilter(value.minFilter),
    defaultMipmapFilter(value.mipmapFilter),
    defaultLodMinClamp(value.lodMinClamp),
    defaultLodMaxClamp(value.lodMaxClamp),
    resolveGPUCompareFunctionForGPUSamplerDescriptorCompare(value),
    defaultMaxAnisotropy(value.maxAnisotropy),
  );
}

function toSubscriptTypegpuExtent3D(value: GPUExtent3D): SubscriptTypegpuExtent3D {
  return new SubscriptTypegpuExtent3D(
    value.width,
    defaultHeight(value.height),
    defaultDepthOrArrayLayers(value.depthOrArrayLayers),
  );
}

function toSubscriptTypegpuOrigin3D(value: GPUOrigin3D): SubscriptTypegpuOrigin3D {
  return new SubscriptTypegpuOrigin3D(
    defaultX(value.x),
    defaultY(value.y),
    defaultZ(value.z),
  );
}

function toSubscriptTypegpuTexelCopyTextureInfo(value: GPUTexelCopyTextureInfo): SubscriptTypegpuTexelCopyTextureInfo {
  return new SubscriptTypegpuTexelCopyTextureInfo(
    value.texture.texture,
    defaultMipLevel(value.mipLevel),
    toSubscriptTypegpuOrigin3D(defaultOrigin(value.origin)),
    defaultAspect(value.aspect),
  );
}

function toSubscriptTypegpuTexelCopyBufferLayout(value: GPUTexelCopyBufferLayout): SubscriptTypegpuTexelCopyBufferLayout {
  return new SubscriptTypegpuTexelCopyBufferLayout(
    defaultOffset(value.offset),
    value.bytesPerRow,
    value.rowsPerImage,
  );
}

function toSubscriptTypegpuBufferBindingLayout(value: GPUBufferBindingLayout): SubscriptTypegpuBufferBindingLayout {
  return new SubscriptTypegpuBufferBindingLayout(
    defaultType(value.type),
    defaultHasDynamicOffset(value.hasDynamicOffset),
    defaultMinBindingSize(value.minBindingSize),
  );
}

function toSubscriptTypegpuSamplerBindingLayout(value: GPUSamplerBindingLayout): SubscriptTypegpuSamplerBindingLayout {
  return new SubscriptTypegpuSamplerBindingLayout(
    defaultSamplerBindingType(value.type),
  );
}

function toSubscriptTypegpuTextureBindingLayout(value: GPUTextureBindingLayout): SubscriptTypegpuTextureBindingLayout {
  return new SubscriptTypegpuTextureBindingLayout(
    defaultSampleType(value.sampleType),
    defaultViewDimension(value.viewDimension),
    defaultMultisampled(value.multisampled),
  );
}

function toSubscriptTypegpuStorageTextureBindingLayout(value: GPUStorageTextureBindingLayout): SubscriptTypegpuStorageTextureBindingLayout {
  return new SubscriptTypegpuStorageTextureBindingLayout(
    defaultAccess(value.access),
    value.format,
    defaultViewDimension(value.viewDimension),
  );
}

function toSubscriptTypegpuBindGroupLayoutEntry(value: GPUBindGroupLayoutEntry): SubscriptTypegpuBindGroupLayoutEntry {
  return new SubscriptTypegpuBindGroupLayoutEntry(
    value.binding,
    value.visibility,
    0,
    toOptionalSubscriptTypegpuBufferBindingLayout(defaultBuffer(value.buffer)),
    toOptionalSubscriptTypegpuSamplerBindingLayout(defaultSampler(value.sampler)),
    toOptionalSubscriptTypegpuTextureBindingLayout(defaultTexture(value.texture)),
    toOptionalSubscriptTypegpuStorageTextureBindingLayout(defaultStorageTexture(value.storageTexture)),
  );
}

function toSubscriptTypegpuBindGroupLayoutDescriptor(value: GPUBindGroupLayoutDescriptor): SubscriptTypegpuBindGroupLayoutDescriptor {
  return new SubscriptTypegpuBindGroupLayoutDescriptor(
    defaultLabel(value.label),
    toSubscriptTypegpuBindGroupLayoutEntryArray(value.entries),
  );
}

function toSubscriptTypegpuBindGroupEntry(value: GPUBindGroupEntry): SubscriptTypegpuBindGroupEntry {
  return new SubscriptTypegpuBindGroupEntry(
    value.binding,
    toNullableSubscriptTypegpuBuffer(defaultBindGroupResourceBuffer(value.buffer)),
    defaultOffset(value.offset),
    defaultSize(value.size),
    toNullableSubscriptTypegpuSampler(defaultBindGroupResourceSampler(value.sampler)),
    toNullableSubscriptTypegpuTextureView(defaultTextureView(value.textureView)),
  );
}

function toSubscriptTypegpuBindGroupDescriptor(value: GPUBindGroupDescriptor): SubscriptTypegpuBindGroupDescriptor {
  return new SubscriptTypegpuBindGroupDescriptor(
    defaultLabel(value.label),
    value.layout.bindGroupLayout,
    toSubscriptTypegpuBindGroupEntryArray(value.entries),
  );
}

function toSubscriptTypegpuPipelineLayoutDescriptor(value: GPUPipelineLayoutDescriptor): SubscriptTypegpuPipelineLayoutDescriptor {
  return new SubscriptTypegpuPipelineLayoutDescriptor(
    defaultLabel(value.label),
    toSubscriptTypegpuBindGroupLayoutArray(value.bindGroupLayouts),
    defaultImmediateSize(value.immediateSize),
  );
}

function toSubscriptTypegpuShaderModuleDescriptor(value: GPUShaderModuleDescriptor): SubscriptTypegpuShaderModuleDescriptor {
  return new SubscriptTypegpuShaderModuleDescriptor(
    defaultLabel(value.label),
    value.code,
  );
}

function toSubscriptTypegpuComputeState(value: GPUProgrammableStage): SubscriptTypegpuComputeState {
  return new SubscriptTypegpuComputeState(
    value.module.shaderModule,
    value.entryPoint,
    toSubscriptTypegpuConstantEntryArray(defaultConstants(value.constants)),
  );
}

function toSubscriptTypegpuComputePipelineDescriptor(value: GPUComputePipelineDescriptor): SubscriptTypegpuComputePipelineDescriptor {
  return new SubscriptTypegpuComputePipelineDescriptor(
    defaultLabel(value.label),
    toNullableSubscriptTypegpuPipelineLayout(defaultLayout(value.layout)),
    toSubscriptTypegpuComputeState(value.compute),
  );
}

function toSubscriptTypegpuVertexAttribute(value: GPUVertexAttribute): SubscriptTypegpuVertexAttribute {
  return new SubscriptTypegpuVertexAttribute(
    value.format,
    value.offset,
    value.shaderLocation,
  );
}

function toSubscriptTypegpuVertexBufferLayout(value: GPUVertexBufferLayout): SubscriptTypegpuVertexBufferLayout {
  return new SubscriptTypegpuVertexBufferLayout(
    defaultStepMode(value.stepMode),
    value.arrayStride,
    toSubscriptTypegpuVertexAttributeArray(value.attributes),
  );
}

function toSubscriptTypegpuVertexState(value: GPUVertexState): SubscriptTypegpuVertexState {
  return new SubscriptTypegpuVertexState(
    value.module.shaderModule,
    value.entryPoint,
    toSubscriptTypegpuConstantEntryArray(defaultConstants(value.constants)),
    toSubscriptTypegpuVertexBufferLayoutArray(defaultBuffers(value.buffers)),
  );
}

function resolveGPUIndexFormatForGPUPrimitiveStateStripIndexFormat(value: GPUPrimitiveState): GPUIndexFormat {
  if (value.stripIndexFormat !== undefined) {
    return value.stripIndexFormat;
  }
  return "undefined";
}

function toSubscriptTypegpuPrimitiveState(value: GPUPrimitiveState): SubscriptTypegpuPrimitiveState {
  return new SubscriptTypegpuPrimitiveState(
    defaultTopology(value.topology),
    resolveGPUIndexFormatForGPUPrimitiveStateStripIndexFormat(value),
    defaultFrontFace(value.frontFace),
    defaultCullMode(value.cullMode),
    defaultUnclippedDepth(value.unclippedDepth),
  );
}

function toSubscriptTypegpuStencilFaceState(value: GPUStencilFaceState): SubscriptTypegpuStencilFaceState {
  return new SubscriptTypegpuStencilFaceState(
    defaultCompare(value.compare),
    defaultFailOp(value.failOp),
    defaultDepthFailOp(value.depthFailOp),
    defaultPassOp(value.passOp),
  );
}

function resolveGPUCompareFunctionForGPUDepthStencilStateDepthCompare(value: GPUDepthStencilState): GPUCompareFunction {
  if (value.depthCompare !== undefined) {
    return value.depthCompare;
  }
  return "undefined";
}

function toSubscriptTypegpuDepthStencilState(value: GPUDepthStencilState): SubscriptTypegpuDepthStencilState {
  return new SubscriptTypegpuDepthStencilState(
    value.format,
    toSubscriptTypegpuOptionalBool(value.depthWriteEnabled),
    resolveGPUCompareFunctionForGPUDepthStencilStateDepthCompare(value),
    toSubscriptTypegpuStencilFaceState(defaultStencilFront(value.stencilFront)),
    toSubscriptTypegpuStencilFaceState(defaultStencilBack(value.stencilBack)),
    defaultStencilReadMask(value.stencilReadMask),
    defaultStencilWriteMask(value.stencilWriteMask),
    defaultDepthBias(value.depthBias),
    defaultDepthBiasSlopeScale(value.depthBiasSlopeScale),
    defaultDepthBiasClamp(value.depthBiasClamp),
  );
}

function toSubscriptTypegpuMultisampleState(value: GPUMultisampleState): SubscriptTypegpuMultisampleState {
  return new SubscriptTypegpuMultisampleState(
    defaultCount(value.count),
    defaultMask(value.mask),
    defaultAlphaToCoverageEnabled(value.alphaToCoverageEnabled),
  );
}

function toSubscriptTypegpuBlendComponent(value: GPUBlendComponent): SubscriptTypegpuBlendComponent {
  return new SubscriptTypegpuBlendComponent(
    defaultOperation(value.operation),
    defaultSrcFactor(value.srcFactor),
    defaultDstFactor(value.dstFactor),
  );
}

function toSubscriptTypegpuBlendState(value: GPUBlendState): SubscriptTypegpuBlendState {
  return new SubscriptTypegpuBlendState(
    toSubscriptTypegpuBlendComponent(value.color),
    toSubscriptTypegpuBlendComponent(value.alpha),
  );
}

function toSubscriptTypegpuColorTargetState(value: GPUColorTargetState): SubscriptTypegpuColorTargetState {
  const nullableBlend: GPUBlendState | null = defaultBlend(value.blend);
  return new SubscriptTypegpuColorTargetState(
    value.format,
    nullableBlend !== null ? toSubscriptTypegpuBlendState(nullableBlend) : null,
    defaultWriteMask(value.writeMask),
  );
}

function toSubscriptTypegpuFragmentState(value: GPUFragmentState): SubscriptTypegpuFragmentState {
  return new SubscriptTypegpuFragmentState(
    value.module.shaderModule,
    value.entryPoint,
    toSubscriptTypegpuConstantEntryArray(defaultConstants(value.constants)),
    toSubscriptTypegpuColorTargetStateArray(value.targets),
  );
}

function toSubscriptTypegpuRenderPipelineDescriptor(value: GPURenderPipelineDescriptor): SubscriptTypegpuRenderPipelineDescriptor {
  const nullableDepthStencil: GPUDepthStencilState | null = defaultDepthStencil(value.depthStencil);
  const nullableFragment: GPUFragmentState | null = defaultFragment(value.fragment);
  return new SubscriptTypegpuRenderPipelineDescriptor(
    defaultLabel(value.label),
    toNullableSubscriptTypegpuPipelineLayout(defaultLayout(value.layout)),
    toSubscriptTypegpuVertexState(value.vertex),
    toSubscriptTypegpuPrimitiveState(defaultPrimitive(value.primitive)),
    nullableDepthStencil !== null ? toSubscriptTypegpuDepthStencilState(nullableDepthStencil) : null,
    toSubscriptTypegpuMultisampleState(defaultMultisample(value.multisample)),
    nullableFragment !== null ? toSubscriptTypegpuFragmentState(nullableFragment) : null,
  );
}

function toSubscriptTypegpuCommandEncoderDescriptor(value: GPUCommandEncoderDescriptor): SubscriptTypegpuCommandEncoderDescriptor {
  return new SubscriptTypegpuCommandEncoderDescriptor(
    defaultLabel(value.label),
  );
}

function resolveGPUTextureFormatForGPURenderBundleEncoderDescriptorDepthStencilFormat(value: GPURenderBundleEncoderDescriptor): GPUTextureFormat {
  if (value.depthStencilFormat !== undefined) {
    return value.depthStencilFormat;
  }
  return "undefined";
}

function toSubscriptTypegpuRenderBundleEncoderDescriptor(value: GPURenderBundleEncoderDescriptor): SubscriptTypegpuRenderBundleEncoderDescriptor {
  return new SubscriptTypegpuRenderBundleEncoderDescriptor(
    defaultLabel(value.label),
    value.colorFormats,
    resolveGPUTextureFormatForGPURenderBundleEncoderDescriptorDepthStencilFormat(value),
    defaultSampleCount(value.sampleCount),
    defaultDepthReadOnly(value.depthReadOnly),
    defaultStencilReadOnly(value.stencilReadOnly),
  );
}

function toSubscriptTypegpuQuerySetDescriptor(value: GPUQuerySetDescriptor): SubscriptTypegpuQuerySetDescriptor {
  return new SubscriptTypegpuQuerySetDescriptor(
    defaultLabel(value.label),
    value.type,
    value.count,
  );
}

function toSubscriptTypegpuCommandBufferDescriptor(value: GPUCommandBufferDescriptor): SubscriptTypegpuCommandBufferDescriptor {
  return new SubscriptTypegpuCommandBufferDescriptor(
    defaultLabel(value.label),
  );
}

function toSubscriptTypegpuPassTimestampWrites(value: GPUPassTimestampWrites): SubscriptTypegpuPassTimestampWrites {
  return new SubscriptTypegpuPassTimestampWrites(
    value.querySet.querySet,
    defaultBeginningOfPassWriteIndex(value.beginningOfPassWriteIndex),
    defaultEndOfPassWriteIndex(value.endOfPassWriteIndex),
  );
}

function toSubscriptTypegpuComputePassDescriptor(value: GPUComputePassDescriptor): SubscriptTypegpuComputePassDescriptor {
  const nullableTimestampWrites: GPUPassTimestampWrites | null = defaultTimestampWrites(value.timestampWrites);
  return new SubscriptTypegpuComputePassDescriptor(
    defaultLabel(value.label),
    nullableTimestampWrites !== null ? toSubscriptTypegpuPassTimestampWrites(nullableTimestampWrites) : null,
  );
}

function toSubscriptTypegpuColor(value: GPUColor): SubscriptTypegpuColor {
  return new SubscriptTypegpuColor(
    value.r,
    value.g,
    value.b,
    value.a,
  );
}

function toSubscriptTypegpuRenderPassColorAttachment(value: GPURenderPassColorAttachment): SubscriptTypegpuRenderPassColorAttachment {
  return new SubscriptTypegpuRenderPassColorAttachment(
    value.view.textureView,
    defaultDepthSlice(value.depthSlice),
    toNullableSubscriptTypegpuTextureView(defaultResolveTarget(value.resolveTarget)),
    value.loadOp,
    value.storeOp,
    toSubscriptTypegpuColor(defaultClearValue(value.clearValue)),
  );
}

function resolveGPULoadOpForGPURenderPassDepthStencilAttachmentDepthLoadOp(value: GPURenderPassDepthStencilAttachment): GPULoadOp {
  if (value.depthLoadOp !== undefined) {
    return value.depthLoadOp;
  }
  return "undefined";
}

function resolveGPUStoreOpForGPURenderPassDepthStencilAttachmentDepthStoreOp(value: GPURenderPassDepthStencilAttachment): GPUStoreOp {
  if (value.depthStoreOp !== undefined) {
    return value.depthStoreOp;
  }
  return "undefined";
}

function resolveGPULoadOpForGPURenderPassDepthStencilAttachmentStencilLoadOp(value: GPURenderPassDepthStencilAttachment): GPULoadOp {
  if (value.stencilLoadOp !== undefined) {
    return value.stencilLoadOp;
  }
  return "undefined";
}

function resolveGPUStoreOpForGPURenderPassDepthStencilAttachmentStencilStoreOp(value: GPURenderPassDepthStencilAttachment): GPUStoreOp {
  if (value.stencilStoreOp !== undefined) {
    return value.stencilStoreOp;
  }
  return "undefined";
}

function toSubscriptTypegpuRenderPassDepthStencilAttachment(value: GPURenderPassDepthStencilAttachment): SubscriptTypegpuRenderPassDepthStencilAttachment {
  return new SubscriptTypegpuRenderPassDepthStencilAttachment(
    value.view.textureView,
    resolveGPULoadOpForGPURenderPassDepthStencilAttachmentDepthLoadOp(value),
    resolveGPUStoreOpForGPURenderPassDepthStencilAttachmentDepthStoreOp(value),
    value.depthClearValue,
    defaultDepthReadOnly(value.depthReadOnly),
    resolveGPULoadOpForGPURenderPassDepthStencilAttachmentStencilLoadOp(value),
    resolveGPUStoreOpForGPURenderPassDepthStencilAttachmentStencilStoreOp(value),
    defaultStencilClearValue(value.stencilClearValue),
    defaultStencilReadOnly(value.stencilReadOnly),
  );
}

function toSubscriptTypegpuRenderPassDescriptor(value: GPURenderPassDescriptor): SubscriptTypegpuRenderPassDescriptor {
  const nullableDepthStencilAttachment: GPURenderPassDepthStencilAttachment | null = defaultDepthStencilAttachment(value.depthStencilAttachment);
  const nullableTimestampWrites: GPUPassTimestampWrites | null = defaultTimestampWrites(value.timestampWrites);
  return new SubscriptTypegpuRenderPassDescriptor(
    defaultLabel(value.label),
    toSubscriptTypegpuRenderPassColorAttachmentArray(value.colorAttachments),
    nullableDepthStencilAttachment !== null ? toSubscriptTypegpuRenderPassDepthStencilAttachment(nullableDepthStencilAttachment) : null,
    toNullableSubscriptTypegpuQuerySet(defaultOcclusionQuerySet(value.occlusionQuerySet)),
    nullableTimestampWrites !== null ? toSubscriptTypegpuPassTimestampWrites(nullableTimestampWrites) : null,
  );
}

function toSubscriptTypegpuTexelCopyBufferInfo(value: GPUTexelCopyBufferInfo): SubscriptTypegpuTexelCopyBufferInfo {
  return new SubscriptTypegpuTexelCopyBufferInfo(
    new SubscriptTypegpuTexelCopyBufferLayout(
      defaultOffset(value.offset),
      value.bytesPerRow,
      value.rowsPerImage,
    ),
    value.buffer.buffer,
  );
}

function toSubscriptTypegpuRenderBundleDescriptor(value: GPURenderBundleDescriptor): SubscriptTypegpuRenderBundleDescriptor {
  return new SubscriptTypegpuRenderBundleDescriptor(
    defaultLabel(value.label),
  );
}

function toSubscriptTypegpuQueueDescriptor(value: GPUQueueDescriptor): SubscriptTypegpuQueueDescriptor {
  return new SubscriptTypegpuQueueDescriptor(
    defaultLabel(value.label),
  );
}

function toSubscriptTypegpuDeviceDescriptor(value: GPUDeviceDescriptor): SubscriptTypegpuDeviceDescriptor {
  const nullableRequiredLimits: GPURequiredLimits = defaultRequiredLimits(value.requiredLimits);
  return new SubscriptTypegpuDeviceDescriptor(
    defaultLabel(value.label),
    defaultRequiredFeatures(value.requiredFeatures),
    isGPURequiredLimitsEmpty(nullableRequiredLimits) ? null : toSubscriptTypegpuLimits(nullableRequiredLimits),
    toSubscriptTypegpuQueueDescriptor(defaultDefaultQueue(value.defaultQueue)),
  );
}

class GPUBufferUsageNamespace {
  MAP_READ: u64;
  MAP_WRITE: u64;
  COPY_SRC: u64;
  COPY_DST: u64;
  INDEX: u64;
  VERTEX: u64;
  UNIFORM: u64;
  STORAGE: u64;
  INDIRECT: u64;
  QUERY_RESOLVE: u64;

  constructor() {
    this.MAP_READ = 0x0001;
    this.MAP_WRITE = 0x0002;
    this.COPY_SRC = 0x0004;
    this.COPY_DST = 0x0008;
    this.INDEX = 0x0010;
    this.VERTEX = 0x0020;
    this.UNIFORM = 0x0040;
    this.STORAGE = 0x0080;
    this.INDIRECT = 0x0100;
    this.QUERY_RESOLVE = 0x0200;
  }
}

export const GPUBufferUsage: GPUBufferUsageNamespace = new GPUBufferUsageNamespace();

class GPUMapModeNamespace {
  READ: u64;
  WRITE: u64;

  constructor() {
    this.READ = 0x0001;
    this.WRITE = 0x0002;
  }
}

export const GPUMapMode: GPUMapModeNamespace = new GPUMapModeNamespace();

class GPUTextureUsageNamespace {
  COPY_SRC: u64;
  COPY_DST: u64;
  TEXTURE_BINDING: u64;
  STORAGE_BINDING: u64;
  RENDER_ATTACHMENT: u64;
  TRANSIENT_ATTACHMENT: u64;

  constructor() {
    this.COPY_SRC = 0x0001;
    this.COPY_DST = 0x0002;
    this.TEXTURE_BINDING = 0x0004;
    this.STORAGE_BINDING = 0x0008;
    this.RENDER_ATTACHMENT = 0x0010;
    this.TRANSIENT_ATTACHMENT = 0x0020;
  }
}

export const GPUTextureUsage: GPUTextureUsageNamespace = new GPUTextureUsageNamespace();

class GPUShaderStageNamespace {
  VERTEX: u64;
  FRAGMENT: u64;
  COMPUTE: u64;

  constructor() {
    this.VERTEX = 0x0001;
    this.FRAGMENT = 0x0002;
    this.COMPUTE = 0x0004;
  }
}

export const GPUShaderStage: GPUShaderStageNamespace = new GPUShaderStageNamespace();

class GPUColorWriteNamespace {
  RED: u64;
  GREEN: u64;
  BLUE: u64;
  ALPHA: u64;
  ALL: u64;

  constructor() {
    this.RED = 0x0001;
    this.GREEN = 0x0002;
    this.BLUE = 0x0004;
    this.ALPHA = 0x0008;
    this.ALL = 0x000F;
  }
}

export const GPUColorWrite: GPUColorWriteNamespace = new GPUColorWriteNamespace();

// TypeScript reads Q33 optional fields as `T | undefined`, while subscript
// applies descriptor defaults before reads; default parameters bridge both.
function defaultLabel(value: string = ""): string {
  return value;
}

function defaultMappedAtCreation(value: boolean = false): boolean {
  return value;
}

function defaultMipLevelCount(value: u32 = 1): u32 {
  return value;
}

function defaultSampleCount(value: u32 = 1): u32 {
  return value;
}

function defaultDimension(value: GPUTextureDimension = "2d"): GPUTextureDimension {
  return value;
}

function defaultViewFormats(value: GPUTextureFormat[] = []): GPUTextureFormat[] {
  return value;
}

function defaultUsage(value: u64 = 0): u64 {
  return value;
}

function defaultAspect(value: GPUTextureAspect = "all"): GPUTextureAspect {
  return value;
}

function defaultBaseMipLevel(value: u32 = 0): u32 {
  return value;
}

function defaultBaseArrayLayer(value: u32 = 0): u32 {
  return value;
}

function defaultAddressModeU(value: GPUAddressMode = "clamp-to-edge"): GPUAddressMode {
  return value;
}

function defaultAddressModeV(value: GPUAddressMode = "clamp-to-edge"): GPUAddressMode {
  return value;
}

function defaultAddressModeW(value: GPUAddressMode = "clamp-to-edge"): GPUAddressMode {
  return value;
}

function defaultMagFilter(value: GPUFilterMode = "nearest"): GPUFilterMode {
  return value;
}

function defaultMinFilter(value: GPUFilterMode = "nearest"): GPUFilterMode {
  return value;
}

function defaultMipmapFilter(value: GPUMipmapFilterMode = "nearest"): GPUMipmapFilterMode {
  return value;
}

function defaultLodMinClamp(value: f32 = 0): f32 {
  return value;
}

function defaultLodMaxClamp(value: f32 = 32): f32 {
  return value;
}

function defaultMaxAnisotropy(value: u16 = 1): u16 {
  return value;
}

function defaultHeight(value: u32 = 1): u32 {
  return value;
}

function defaultDepthOrArrayLayers(value: u32 = 1): u32 {
  return value;
}

function defaultX(value: u32 = 0): u32 {
  return value;
}

function defaultY(value: u32 = 0): u32 {
  return value;
}

function defaultZ(value: u32 = 0): u32 {
  return value;
}

function defaultMipLevel(value: u32 = 0): u32 {
  return value;
}

function defaultOrigin(value: GPUOrigin3D = {}): GPUOrigin3D {
  return value;
}

function defaultOffset(value: u64 = 0): u64 {
  return value;
}

function defaultType(value: GPUBufferBindingType = "uniform"): GPUBufferBindingType {
  return value;
}

function defaultHasDynamicOffset(value: boolean = false): boolean {
  return value;
}

function defaultMinBindingSize(value: u64 = 0): u64 {
  return value;
}

function defaultSamplerBindingType(value: GPUSamplerBindingType = "filtering"): GPUSamplerBindingType {
  return value;
}

function defaultSampleType(value: GPUTextureSampleType = "float"): GPUTextureSampleType {
  return value;
}

function defaultViewDimension(value: GPUTextureViewDimension = "2d"): GPUTextureViewDimension {
  return value;
}

function defaultMultisampled(value: boolean = false): boolean {
  return value;
}

function defaultAccess(value: GPUStorageTextureAccess = "write-only"): GPUStorageTextureAccess {
  return value;
}

function defaultBuffer(value: GPUBufferBindingLayout | null = null): GPUBufferBindingLayout | null {
  return value;
}

function defaultSampler(value: GPUSamplerBindingLayout | null = null): GPUSamplerBindingLayout | null {
  return value;
}

function defaultTexture(value: GPUTextureBindingLayout | null = null): GPUTextureBindingLayout | null {
  return value;
}

function defaultStorageTexture(value: GPUStorageTextureBindingLayout | null = null): GPUStorageTextureBindingLayout | null {
  return value;
}

function defaultBindGroupResourceBuffer(value: GPUBuffer | null = null): GPUBuffer | null {
  return value;
}

function defaultSize(value: u64 = 0): u64 {
  return value;
}

function defaultBindGroupResourceSampler(value: GPUSampler | null = null): GPUSampler | null {
  return value;
}

function defaultTextureView(value: GPUTextureView | null = null): GPUTextureView | null {
  return value;
}

function defaultImmediateSize(value: u32 = 0): u32 {
  return value;
}

function defaultConstants(value: GPUPipelineConstantEntry[] = []): GPUPipelineConstantEntry[] {
  return value;
}

function defaultLayout(value: GPUPipelineLayout | null = null): GPUPipelineLayout | null {
  return value;
}

function defaultStepMode(value: GPUVertexStepMode = "vertex"): GPUVertexStepMode {
  return value;
}

function defaultBuffers(value: GPUVertexBufferLayout[] = []): GPUVertexBufferLayout[] {
  return value;
}

function defaultTopology(value: GPUPrimitiveTopology = "triangle-list"): GPUPrimitiveTopology {
  return value;
}

function defaultFrontFace(value: GPUFrontFace = "ccw"): GPUFrontFace {
  return value;
}

function defaultCullMode(value: GPUCullMode = "none"): GPUCullMode {
  return value;
}

function defaultUnclippedDepth(value: boolean = false): boolean {
  return value;
}

function defaultCompare(value: GPUCompareFunction = "always"): GPUCompareFunction {
  return value;
}

function defaultFailOp(value: GPUStencilOperation = "keep"): GPUStencilOperation {
  return value;
}

function defaultDepthFailOp(value: GPUStencilOperation = "keep"): GPUStencilOperation {
  return value;
}

function defaultPassOp(value: GPUStencilOperation = "keep"): GPUStencilOperation {
  return value;
}

function defaultStencilFront(value: GPUStencilFaceState = {}): GPUStencilFaceState {
  return value;
}

function defaultStencilBack(value: GPUStencilFaceState = {}): GPUStencilFaceState {
  return value;
}

function defaultStencilReadMask(value: u32 = 0xFFFFFFFF): u32 {
  return value;
}

function defaultStencilWriteMask(value: u32 = 0xFFFFFFFF): u32 {
  return value;
}

function defaultDepthBias(value: i32 = 0): i32 {
  return value;
}

function defaultDepthBiasSlopeScale(value: f32 = 0): f32 {
  return value;
}

function defaultDepthBiasClamp(value: f32 = 0): f32 {
  return value;
}

function defaultCount(value: u32 = 1): u32 {
  return value;
}

function defaultMask(value: u32 = 0xFFFFFFFF): u32 {
  return value;
}

function defaultAlphaToCoverageEnabled(value: boolean = false): boolean {
  return value;
}

function defaultOperation(value: GPUBlendOperation = "add"): GPUBlendOperation {
  return value;
}

function defaultSrcFactor(value: GPUBlendFactor = "one"): GPUBlendFactor {
  return value;
}

function defaultDstFactor(value: GPUBlendFactor = "zero"): GPUBlendFactor {
  return value;
}

function defaultBlend(value: GPUBlendState | null = null): GPUBlendState | null {
  return value;
}

function defaultWriteMask(value: u64 = 0xF): u64 {
  return value;
}

function defaultPrimitive(value: GPUPrimitiveState = {}): GPUPrimitiveState {
  return value;
}

function defaultDepthStencil(value: GPUDepthStencilState | null = null): GPUDepthStencilState | null {
  return value;
}

function defaultMultisample(value: GPUMultisampleState = {}): GPUMultisampleState {
  return value;
}

function defaultFragment(value: GPUFragmentState | null = null): GPUFragmentState | null {
  return value;
}

function defaultDepthReadOnly(value: boolean = false): boolean {
  return value;
}

function defaultStencilReadOnly(value: boolean = false): boolean {
  return value;
}

function defaultBeginningOfPassWriteIndex(value: u32 = 4294967295): u32 {
  return value;
}

function defaultEndOfPassWriteIndex(value: u32 = 4294967295): u32 {
  return value;
}

function defaultTimestampWrites(value: GPUPassTimestampWrites | null = null): GPUPassTimestampWrites | null {
  return value;
}

function defaultDepthSlice(value: u32 = 4294967295): u32 {
  return value;
}

function defaultResolveTarget(value: GPUTextureView | null = null): GPUTextureView | null {
  return value;
}

function defaultClearValue(value: GPUColor = { r: 0, g: 0, b: 0, a: 0 }): GPUColor {
  return value;
}

function defaultStencilClearValue(value: u32 = 0): u32 {
  return value;
}

function defaultDepthStencilAttachment(value: GPURenderPassDepthStencilAttachment | null = null): GPURenderPassDepthStencilAttachment | null {
  return value;
}

function defaultOcclusionQuerySet(value: GPUQuerySet | null = null): GPUQuerySet | null {
  return value;
}

function defaultRequiredFeatures(value: GPUFeatureName[] = []): GPUFeatureName[] {
  return value;
}

function defaultRequiredLimits(value: GPURequiredLimits = {}): GPURequiredLimits {
  return value;
}

function defaultDefaultQueue(value: GPUQueueDescriptor = {}): GPUQueueDescriptor {
  return value;
}

export class GPU {
  instance: SubscriptTypegpuInstance;

  constructor(instance: SubscriptTypegpuInstance) {
    this.instance = instance;
  }

  async requestAdapter(): Promise<GPUAdapter | null> {
    const future: SubscriptTypegpuFutureId = subscript_typegpu_instance_request_adapter(this.instance);
    let status: i32 = subscript_typegpu_future_status(this.instance, future);
    while (status === 0) {
      subscript_typegpu_instance_process_events(this.instance);
      status = subscript_typegpu_future_status(this.instance, future);
      await Context.suspend();
    }
    if (status !== 1) {
      subscript_typegpu_future_drop(this.instance, future);
      return null;
    }
    return new GPUAdapter(this.instance, subscript_typegpu_request_adapter_take(this.instance, future));
  }

  dispose(): void {
    subscript_typegpu_instance_release(this.instance);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPUAdapter {
  instance: SubscriptTypegpuInstance;
  adapter: SubscriptTypegpuAdapter;

  constructor(instance: SubscriptTypegpuInstance, adapter: SubscriptTypegpuAdapter) {
    this.instance = instance;
    this.adapter = adapter;
  }

  hasFeature(name: GPUFeatureName): boolean {
    return subscript_typegpu_adapter_has_feature(this.adapter, name);
  }

  limits(): GPUSupportedLimits | null {
    const record: SubscriptTypegpuLimits = new SubscriptTypegpuLimits(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    if (subscript_typegpu_adapter_get_limits(this.adapter, record) !== 1) {
      return null;
    }
    return fromSubscriptTypegpuLimits(record);
  }

  info(): GPUAdapterInfo | null {
    const record: SubscriptTypegpuAdapterInfo = new SubscriptTypegpuAdapterInfo("", "", "", "", SubscriptTypegpuBackendType.SUBSCRIPT_TYPEGPU_BACKEND_TYPE_UNDEFINED, SubscriptTypegpuAdapterType.SUBSCRIPT_TYPEGPU_ADAPTER_TYPE_DISCRETE_GPU, 0, 0);
    if (!subscript_typegpu_adapter_get_info(this.adapter, record)) {
      return null;
    }
    return fromSubscriptTypegpuAdapterInfo(record);
  }

  async requestDevice(descriptor: GPUDeviceDescriptor = {}): Promise<GPUDevice | null> {
    const future: SubscriptTypegpuFutureId = subscript_typegpu_adapter_request_device_with_descriptor(this.instance, this.adapter, toSubscriptTypegpuDeviceDescriptor(descriptor));
    let status: i32 = subscript_typegpu_future_status(this.instance, future);
    while (status === 0) {
      subscript_typegpu_instance_process_events(this.instance);
      status = subscript_typegpu_future_status(this.instance, future);
      await Context.suspend();
    }
    if (status !== 1) {
      subscript_typegpu_future_drop(this.instance, future);
      return null;
    }
    return new GPUDevice(this.instance, subscript_typegpu_request_device_take(this.instance, future));
  }

  dispose(): void {
    subscript_typegpu_adapter_release(this.adapter);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPUDevice {
  private instance: SubscriptTypegpuInstance;
  private device: SubscriptTypegpuDevice;
  queueValue: GPUQueue;

  constructor(instance: SubscriptTypegpuInstance, device: SubscriptTypegpuDevice) {
    this.instance = instance;
    this.device = device;
    this.queueValue = new GPUQueue(this.instance, subscript_typegpu_device_get_queue(this.device));
  }

  hasFeature(name: GPUFeatureName): boolean {
    return subscript_typegpu_device_has_feature(this.device, name);
  }

  limits(): GPUSupportedLimits | null {
    const record: SubscriptTypegpuLimits = new SubscriptTypegpuLimits(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    if (subscript_typegpu_device_get_limits(this.device, record) !== 1) {
      return null;
    }
    return fromSubscriptTypegpuLimits(record);
  }

  adapterInfo(): GPUAdapterInfo | null {
    const record: SubscriptTypegpuAdapterInfo = new SubscriptTypegpuAdapterInfo("", "", "", "", SubscriptTypegpuBackendType.SUBSCRIPT_TYPEGPU_BACKEND_TYPE_UNDEFINED, SubscriptTypegpuAdapterType.SUBSCRIPT_TYPEGPU_ADAPTER_TYPE_DISCRETE_GPU, 0, 0);
    if (!subscript_typegpu_device_get_adapter_info(this.device, record)) {
      return null;
    }
    return fromSubscriptTypegpuAdapterInfo(record);
  }

  queue(): GPUQueue {
    return this.queueValue;
  }

  destroy(): void {
    subscript_typegpu_device_destroy(this.device);
  }

  createBuffer(descriptor: GPUBufferDescriptor): GPUBuffer {
    return new GPUBuffer(this.instance, subscript_typegpu_device_create_buffer(this.device, toSubscriptTypegpuBufferDescriptor(descriptor)));
  }

  createTexture(descriptor: GPUTextureDescriptor): GPUTexture {
    return new GPUTexture(subscript_typegpu_device_create_texture(this.device, toSubscriptTypegpuTextureDescriptor(descriptor)));
  }

  createSampler(descriptor: GPUSamplerDescriptor | null = null): GPUSampler {
    if (descriptor === null) {
      return new GPUSampler(subscript_typegpu_device_create_sampler(this.device, null));
    }
    return new GPUSampler(subscript_typegpu_device_create_sampler(this.device, toSubscriptTypegpuSamplerDescriptor(descriptor)));
  }

  createBindGroupLayout(descriptor: GPUBindGroupLayoutDescriptor): GPUBindGroupLayout {
    return new GPUBindGroupLayout(subscript_typegpu_device_create_bind_group_layout(this.device, toSubscriptTypegpuBindGroupLayoutDescriptor(descriptor)));
  }

  createPipelineLayout(descriptor: GPUPipelineLayoutDescriptor): GPUPipelineLayout {
    return new GPUPipelineLayout(subscript_typegpu_device_create_pipeline_layout(this.device, toSubscriptTypegpuPipelineLayoutDescriptor(descriptor)));
  }

  createBindGroup(descriptor: GPUBindGroupDescriptor): GPUBindGroup {
    return new GPUBindGroup(subscript_typegpu_device_create_bind_group(this.device, toSubscriptTypegpuBindGroupDescriptor(descriptor)));
  }

  createShaderModule(descriptor: GPUShaderModuleDescriptor): GPUShaderModule {
    return new GPUShaderModule(subscript_typegpu_device_create_shader_module(this.device, toSubscriptTypegpuShaderModuleDescriptor(descriptor)));
  }

  createComputePipeline(descriptor: GPUComputePipelineDescriptor): GPUComputePipeline {
    return new GPUComputePipeline(subscript_typegpu_device_create_compute_pipeline(this.device, toSubscriptTypegpuComputePipelineDescriptor(descriptor)));
  }

  createRenderPipeline(descriptor: GPURenderPipelineDescriptor): GPURenderPipeline {
    return new GPURenderPipeline(subscript_typegpu_device_create_render_pipeline(this.device, toSubscriptTypegpuRenderPipelineDescriptor(descriptor)));
  }

  async createComputePipelineAsync(descriptor: GPUComputePipelineDescriptor): Promise<GPUComputePipeline | null> {
    const future: SubscriptTypegpuFutureId = subscript_typegpu_device_create_compute_pipeline_async_begin(this.instance, this.device, toSubscriptTypegpuComputePipelineDescriptor(descriptor));
    let status: i32 = subscript_typegpu_future_status(this.instance, future);
    while (status === 0) {
      subscript_typegpu_instance_process_events(this.instance);
      status = subscript_typegpu_future_status(this.instance, future);
      await Context.suspend();
    }
    if (status !== 1) {
      subscript_typegpu_future_drop(this.instance, future);
      return null;
    }
    return new GPUComputePipeline(subscript_typegpu_create_compute_pipeline_async_take(this.instance, future));
  }

  async createRenderPipelineAsync(descriptor: GPURenderPipelineDescriptor): Promise<GPURenderPipeline | null> {
    const future: SubscriptTypegpuFutureId = subscript_typegpu_device_create_render_pipeline_async_begin(this.instance, this.device, toSubscriptTypegpuRenderPipelineDescriptor(descriptor));
    let status: i32 = subscript_typegpu_future_status(this.instance, future);
    while (status === 0) {
      subscript_typegpu_instance_process_events(this.instance);
      status = subscript_typegpu_future_status(this.instance, future);
      await Context.suspend();
    }
    if (status !== 1) {
      subscript_typegpu_future_drop(this.instance, future);
      return null;
    }
    return new GPURenderPipeline(subscript_typegpu_create_render_pipeline_async_take(this.instance, future));
  }

  createCommandEncoder(descriptor: GPUCommandEncoderDescriptor | null = null): GPUCommandEncoder {
    if (descriptor === null) {
      return new GPUCommandEncoder(subscript_typegpu_device_create_command_encoder(this.device, null));
    }
    return new GPUCommandEncoder(subscript_typegpu_device_create_command_encoder(this.device, toSubscriptTypegpuCommandEncoderDescriptor(descriptor)));
  }

  createCommandEncoderDefault(): GPUCommandEncoder {
    return new GPUCommandEncoder(subscript_typegpu_device_create_command_encoder(this.device, null));
  }

  createRenderBundleEncoder(descriptor: GPURenderBundleEncoderDescriptor): GPURenderBundleEncoder {
    return new GPURenderBundleEncoder(subscript_typegpu_device_create_render_bundle_encoder(this.device, toSubscriptTypegpuRenderBundleEncoderDescriptor(descriptor)));
  }

  createQuerySet(descriptor: GPUQuerySetDescriptor): GPUQuerySet {
    return new GPUQuerySet(subscript_typegpu_device_create_query_set(this.device, toSubscriptTypegpuQuerySetDescriptor(descriptor)));
  }

  deviceLostInfo(): GPUDeviceLostInfo | null {
    subscript_typegpu_instance_process_events(this.instance);
    const record: SubscriptTypegpuLostRecord = new SubscriptTypegpuLostRecord("unknown", "");
    if (!subscript_typegpu_device_lost_info(this.device, record)) {
      return null;
    }
    return fromSubscriptTypegpuLostRecord(record);
  }

  pushErrorScope(filter: GPUErrorFilter): void {
    subscript_typegpu_device_push_error_scope(this.device, filter);
  }

  async popErrorScope(): Promise<GPUError | null> {
    const future: SubscriptTypegpuFutureId = subscript_typegpu_device_pop_error_scope(this.device);
    let status: i32 = subscript_typegpu_future_status(this.instance, future);
    while (status === 0) {
      subscript_typegpu_instance_process_events(this.instance);
      status = subscript_typegpu_future_status(this.instance, future);
      await Context.suspend();
    }
    if (status !== 1) {
      subscript_typegpu_future_drop(this.instance, future);
      return null;
    }
    const record: SubscriptTypegpuErrorRecord = new SubscriptTypegpuErrorRecord(SubscriptTypegpuErrorType.SUBSCRIPT_TYPEGPU_ERROR_TYPE_NO_ERROR, "");
    if (!subscript_typegpu_pop_error_scope_take(this.instance, future, record)) {
      subscript_typegpu_future_drop(this.instance, future);
      return null;
    }
    return fromSubscriptTypegpuErrorRecord(record);
  }

  nextUncapturedError(): GPUError | null {
    const record: SubscriptTypegpuErrorRecord = new SubscriptTypegpuErrorRecord(SubscriptTypegpuErrorType.SUBSCRIPT_TYPEGPU_ERROR_TYPE_NO_ERROR, "");
    if (!subscript_typegpu_device_next_uncaptured_error(this.device, record)) {
      return null;
    }
    return fromSubscriptTypegpuErrorRecord(record);
  }

  label(value: string): void {
    subscript_typegpu_device_set_label(this.device, value);
  }

  dispose(): void {
    this.queueValue.dispose();
    subscript_typegpu_device_release(this.device);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPUHostOwnedDevice {
  private instance: SubscriptTypegpuInstance;
  private device: SubscriptTypegpuDevice;

  constructor(instance: SubscriptTypegpuInstance, device: SubscriptTypegpuDevice) {
    this.instance = instance;
    this.device = device;
  }

  hasFeature(name: GPUFeatureName): boolean {
    return subscript_typegpu_device_has_feature(this.device, name);
  }

  limits(): GPUSupportedLimits | null {
    const record: SubscriptTypegpuLimits = new SubscriptTypegpuLimits(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    if (subscript_typegpu_device_get_limits(this.device, record) !== 1) {
      return null;
    }
    return fromSubscriptTypegpuLimits(record);
  }

  adapterInfo(): GPUAdapterInfo | null {
    const record: SubscriptTypegpuAdapterInfo = new SubscriptTypegpuAdapterInfo("", "", "", "", SubscriptTypegpuBackendType.SUBSCRIPT_TYPEGPU_BACKEND_TYPE_UNDEFINED, SubscriptTypegpuAdapterType.SUBSCRIPT_TYPEGPU_ADAPTER_TYPE_DISCRETE_GPU, 0, 0);
    if (!subscript_typegpu_device_get_adapter_info(this.device, record)) {
      return null;
    }
    return fromSubscriptTypegpuAdapterInfo(record);
  }

  queue(): GPUQueue {
    return new GPUQueue(this.instance, subscript_typegpu_device_get_queue(this.device));
  }

  createBuffer(descriptor: GPUBufferDescriptor): GPUBuffer {
    return new GPUBuffer(this.instance, subscript_typegpu_device_create_buffer(this.device, toSubscriptTypegpuBufferDescriptor(descriptor)));
  }

  createTexture(descriptor: GPUTextureDescriptor): GPUTexture {
    return new GPUTexture(subscript_typegpu_device_create_texture(this.device, toSubscriptTypegpuTextureDescriptor(descriptor)));
  }

  createSampler(descriptor: GPUSamplerDescriptor | null = null): GPUSampler {
    if (descriptor === null) {
      return new GPUSampler(subscript_typegpu_device_create_sampler(this.device, null));
    }
    return new GPUSampler(subscript_typegpu_device_create_sampler(this.device, toSubscriptTypegpuSamplerDescriptor(descriptor)));
  }

  createBindGroupLayout(descriptor: GPUBindGroupLayoutDescriptor): GPUBindGroupLayout {
    return new GPUBindGroupLayout(subscript_typegpu_device_create_bind_group_layout(this.device, toSubscriptTypegpuBindGroupLayoutDescriptor(descriptor)));
  }

  createPipelineLayout(descriptor: GPUPipelineLayoutDescriptor): GPUPipelineLayout {
    return new GPUPipelineLayout(subscript_typegpu_device_create_pipeline_layout(this.device, toSubscriptTypegpuPipelineLayoutDescriptor(descriptor)));
  }

  createBindGroup(descriptor: GPUBindGroupDescriptor): GPUBindGroup {
    return new GPUBindGroup(subscript_typegpu_device_create_bind_group(this.device, toSubscriptTypegpuBindGroupDescriptor(descriptor)));
  }

  createShaderModule(descriptor: GPUShaderModuleDescriptor): GPUShaderModule {
    return new GPUShaderModule(subscript_typegpu_device_create_shader_module(this.device, toSubscriptTypegpuShaderModuleDescriptor(descriptor)));
  }

  createComputePipeline(descriptor: GPUComputePipelineDescriptor): GPUComputePipeline {
    return new GPUComputePipeline(subscript_typegpu_device_create_compute_pipeline(this.device, toSubscriptTypegpuComputePipelineDescriptor(descriptor)));
  }

  createRenderPipeline(descriptor: GPURenderPipelineDescriptor): GPURenderPipeline {
    return new GPURenderPipeline(subscript_typegpu_device_create_render_pipeline(this.device, toSubscriptTypegpuRenderPipelineDescriptor(descriptor)));
  }

  async createComputePipelineAsync(descriptor: GPUComputePipelineDescriptor): Promise<GPUComputePipeline | null> {
    const future: SubscriptTypegpuFutureId = subscript_typegpu_device_create_compute_pipeline_async_begin(this.instance, this.device, toSubscriptTypegpuComputePipelineDescriptor(descriptor));
    let status: i32 = subscript_typegpu_future_status(this.instance, future);
    while (status === 0) {
      subscript_typegpu_instance_process_events(this.instance);
      status = subscript_typegpu_future_status(this.instance, future);
      await Context.suspend();
    }
    if (status !== 1) {
      subscript_typegpu_future_drop(this.instance, future);
      return null;
    }
    return new GPUComputePipeline(subscript_typegpu_create_compute_pipeline_async_take(this.instance, future));
  }

  async createRenderPipelineAsync(descriptor: GPURenderPipelineDescriptor): Promise<GPURenderPipeline | null> {
    const future: SubscriptTypegpuFutureId = subscript_typegpu_device_create_render_pipeline_async_begin(this.instance, this.device, toSubscriptTypegpuRenderPipelineDescriptor(descriptor));
    let status: i32 = subscript_typegpu_future_status(this.instance, future);
    while (status === 0) {
      subscript_typegpu_instance_process_events(this.instance);
      status = subscript_typegpu_future_status(this.instance, future);
      await Context.suspend();
    }
    if (status !== 1) {
      subscript_typegpu_future_drop(this.instance, future);
      return null;
    }
    return new GPURenderPipeline(subscript_typegpu_create_render_pipeline_async_take(this.instance, future));
  }

  createCommandEncoder(descriptor: GPUCommandEncoderDescriptor | null = null): GPUCommandEncoder {
    if (descriptor === null) {
      return new GPUCommandEncoder(subscript_typegpu_device_create_command_encoder(this.device, null));
    }
    return new GPUCommandEncoder(subscript_typegpu_device_create_command_encoder(this.device, toSubscriptTypegpuCommandEncoderDescriptor(descriptor)));
  }

  createCommandEncoderDefault(): GPUCommandEncoder {
    return new GPUCommandEncoder(subscript_typegpu_device_create_command_encoder(this.device, null));
  }

  createRenderBundleEncoder(descriptor: GPURenderBundleEncoderDescriptor): GPURenderBundleEncoder {
    return new GPURenderBundleEncoder(subscript_typegpu_device_create_render_bundle_encoder(this.device, toSubscriptTypegpuRenderBundleEncoderDescriptor(descriptor)));
  }

  createQuerySet(descriptor: GPUQuerySetDescriptor): GPUQuerySet {
    return new GPUQuerySet(subscript_typegpu_device_create_query_set(this.device, toSubscriptTypegpuQuerySetDescriptor(descriptor)));
  }

  deviceLostInfo(): GPUDeviceLostInfo | null {
    subscript_typegpu_instance_process_events(this.instance);
    const record: SubscriptTypegpuLostRecord = new SubscriptTypegpuLostRecord("unknown", "");
    if (!subscript_typegpu_device_lost_info(this.device, record)) {
      return null;
    }
    return fromSubscriptTypegpuLostRecord(record);
  }

  pushErrorScope(filter: GPUErrorFilter): void {
    subscript_typegpu_device_push_error_scope(this.device, filter);
  }

  async popErrorScope(): Promise<GPUError | null> {
    const future: SubscriptTypegpuFutureId = subscript_typegpu_device_pop_error_scope(this.device);
    let status: i32 = subscript_typegpu_future_status(this.instance, future);
    while (status === 0) {
      subscript_typegpu_instance_process_events(this.instance);
      status = subscript_typegpu_future_status(this.instance, future);
      await Context.suspend();
    }
    if (status !== 1) {
      subscript_typegpu_future_drop(this.instance, future);
      return null;
    }
    const record: SubscriptTypegpuErrorRecord = new SubscriptTypegpuErrorRecord(SubscriptTypegpuErrorType.SUBSCRIPT_TYPEGPU_ERROR_TYPE_NO_ERROR, "");
    if (!subscript_typegpu_pop_error_scope_take(this.instance, future, record)) {
      subscript_typegpu_future_drop(this.instance, future);
      return null;
    }
    return fromSubscriptTypegpuErrorRecord(record);
  }

  nextUncapturedError(): GPUError | null {
    const record: SubscriptTypegpuErrorRecord = new SubscriptTypegpuErrorRecord(SubscriptTypegpuErrorType.SUBSCRIPT_TYPEGPU_ERROR_TYPE_NO_ERROR, "");
    if (!subscript_typegpu_device_next_uncaptured_error(this.device, record)) {
      return null;
    }
    return fromSubscriptTypegpuErrorRecord(record);
  }

  label(value: string): void {
    subscript_typegpu_device_set_label(this.device, value);
  }

}

export function hostOwnedGPUDevice(instance: SubscriptTypegpuInstance, device: SubscriptTypegpuDevice): GPUHostOwnedDevice {
  return new GPUHostOwnedDevice(instance, device);
}

export class GPUBuffer {
  instance: SubscriptTypegpuInstance;
  buffer: SubscriptTypegpuBuffer;

  constructor(instance: SubscriptTypegpuInstance, buffer: SubscriptTypegpuBuffer) {
    this.instance = instance;
    this.buffer = buffer;
  }

  size(): u64 {
    return subscript_typegpu_buffer_get_size(this.buffer);
  }

  usage(): u64 {
    return subscript_typegpu_buffer_get_usage(this.buffer);
  }

  mapState(): GPUBufferMapState {
    return subscript_typegpu_buffer_get_map_state(this.buffer);
  }

  async mapAsync(mode: u64, offset: u64 = 0, size: u64): Promise<boolean> {
    const future: SubscriptTypegpuFutureId = subscript_typegpu_buffer_map_async(this.buffer, mode, offset, size);
    let status: i32 = subscript_typegpu_future_status(this.instance, future);
    while (status === 0) {
      subscript_typegpu_instance_process_events(this.instance);
      status = subscript_typegpu_future_status(this.instance, future);
      await Context.suspend();
    }
    subscript_typegpu_future_drop(this.instance, future);
    return status === 1;
  }

  readMappedRange(offset: u64, size: u64): u8[] {
    const bytes: u8[] = [];
    let index: u64 = 0;
    while (index < size) {
      bytes.push(0);
      index = index + 1;
    }
    if (subscript_typegpu_buffer_read_mapped_range(this.buffer, offset, bytes) !== 1) {
      return [];
    }
    return bytes;
  }

  writeMappedRange(offset: u64, data: u8[]): boolean {
    return subscript_typegpu_buffer_write_mapped_range(this.buffer, offset, data) === 1;
  }

  // offset counts bytes; count counts f32 elements.
  readMappedRangeF32(offset: u64, count: u64): f32[] {
    const values: f32[] = [];
    let index: u64 = 0;
    while (index < count) {
      values.push(0);
      index = index + 1;
    }
    if (subscript_typegpu_buffer_read_mapped_range_f32(this.buffer, offset, values) !== 1) {
      return [];
    }
    return values;
  }

  unmap(): void {
    subscript_typegpu_buffer_unmap(this.buffer);
  }

  destroy(): void {
    subscript_typegpu_buffer_destroy(this.buffer);
  }

  label(value: string): void {
    subscript_typegpu_buffer_set_label(this.buffer, value);
  }

  dispose(): void {
    subscript_typegpu_buffer_release(this.buffer);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPUQueue {
  instance: SubscriptTypegpuInstance;
  queue: SubscriptTypegpuQueue;
  private disposed: boolean;

  constructor(instance: SubscriptTypegpuInstance, queue: SubscriptTypegpuQueue) {
    this.instance = instance;
    this.queue = queue;
    this.disposed = false;
  }

  submit(commandBuffers: GPUCommandBuffer[]): void {
    subscript_typegpu_queue_submit(this.queue, toSubscriptTypegpuCommandBufferArray(commandBuffers));
  }

  async onSubmittedWorkDone(): Promise<boolean> {
    const future: SubscriptTypegpuFutureId = subscript_typegpu_queue_on_submitted_work_done(this.instance, this.queue);
    let status: i32 = subscript_typegpu_future_status(this.instance, future);
    while (status === 0) {
      subscript_typegpu_instance_process_events(this.instance);
      status = subscript_typegpu_future_status(this.instance, future);
      await Context.suspend();
    }
    subscript_typegpu_future_drop(this.instance, future);
    return status === 1;
  }

  writeBuffer(buffer: GPUBuffer, bufferOffset: u64, data: u8[]): void {
    subscript_typegpu_queue_write_buffer(this.queue, buffer.buffer, bufferOffset, data);
  }

  // bufferOffset counts bytes; data length counts f32 elements.
  writeBufferF32(buffer: GPUBuffer, bufferOffset: u64, data: f32[]): void {
    subscript_typegpu_queue_write_buffer_f32(this.queue, buffer.buffer, bufferOffset, data);
  }

  writeTexture(destination: GPUTexelCopyTextureInfo, data: u8[], dataLayout: GPUTexelCopyBufferLayout, size: GPUExtent3D): void {
    subscript_typegpu_queue_write_texture(this.queue, toSubscriptTypegpuTexelCopyTextureInfo(destination), toSubscriptTypegpuTexelCopyBufferLayout(dataLayout), toSubscriptTypegpuExtent3D(size), data);
  }

  label(value: string): void {
    subscript_typegpu_queue_set_label(this.queue, value);
  }

  dispose(): void {
    if (!this.disposed) {
      subscript_typegpu_queue_release(this.queue);
      this.disposed = true;
    }
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPUTexture {
  texture: SubscriptTypegpuTexture;

  constructor(texture: SubscriptTypegpuTexture) {
    this.texture = texture;
  }

  createView(descriptor: GPUTextureViewDescriptor | null = null): GPUTextureView {
    if (descriptor === null) {
      return new GPUTextureView(subscript_typegpu_texture_create_view(this.texture, null));
    }
    return new GPUTextureView(subscript_typegpu_texture_create_view(this.texture, toSubscriptTypegpuTextureViewDescriptor(descriptor)));
  }

  destroy(): void {
    subscript_typegpu_texture_destroy(this.texture);
  }

  width(): u32 {
    return subscript_typegpu_texture_get_width(this.texture);
  }

  height(): u32 {
    return subscript_typegpu_texture_get_height(this.texture);
  }

  depthOrArrayLayers(): u32 {
    return subscript_typegpu_texture_get_depth_or_array_layers(this.texture);
  }

  mipLevelCount(): u32 {
    return subscript_typegpu_texture_get_mip_level_count(this.texture);
  }

  sampleCount(): u32 {
    return subscript_typegpu_texture_get_sample_count(this.texture);
  }

  dimension(): GPUTextureDimension {
    return subscript_typegpu_texture_get_dimension(this.texture);
  }

  format(): GPUTextureFormat {
    return subscript_typegpu_texture_get_format(this.texture);
  }

  usage(): u64 {
    return subscript_typegpu_texture_get_usage(this.texture);
  }

  label(value: string): void {
    subscript_typegpu_texture_set_label(this.texture, value);
  }

  dispose(): void {
    subscript_typegpu_texture_release(this.texture);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPUTextureView {
  textureView: SubscriptTypegpuTextureView;

  constructor(textureView: SubscriptTypegpuTextureView) {
    this.textureView = textureView;
  }

  label(value: string): void {
    subscript_typegpu_texture_view_set_label(this.textureView, value);
  }

  dispose(): void {
    subscript_typegpu_texture_view_release(this.textureView);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPUSampler {
  sampler: SubscriptTypegpuSampler;

  constructor(sampler: SubscriptTypegpuSampler) {
    this.sampler = sampler;
  }

  label(value: string): void {
    subscript_typegpu_sampler_set_label(this.sampler, value);
  }

  dispose(): void {
    subscript_typegpu_sampler_release(this.sampler);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPUBindGroupLayout {
  bindGroupLayout: SubscriptTypegpuBindGroupLayout;

  constructor(bindGroupLayout: SubscriptTypegpuBindGroupLayout) {
    this.bindGroupLayout = bindGroupLayout;
  }

  label(value: string): void {
    subscript_typegpu_bind_group_layout_set_label(this.bindGroupLayout, value);
  }

  dispose(): void {
    subscript_typegpu_bind_group_layout_release(this.bindGroupLayout);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPUBindGroup {
  bindGroup: SubscriptTypegpuBindGroup;

  constructor(bindGroup: SubscriptTypegpuBindGroup) {
    this.bindGroup = bindGroup;
  }

  label(value: string): void {
    subscript_typegpu_bind_group_set_label(this.bindGroup, value);
  }

  dispose(): void {
    subscript_typegpu_bind_group_release(this.bindGroup);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPUPipelineLayout {
  pipelineLayout: SubscriptTypegpuPipelineLayout;

  constructor(pipelineLayout: SubscriptTypegpuPipelineLayout) {
    this.pipelineLayout = pipelineLayout;
  }

  label(value: string): void {
    subscript_typegpu_pipeline_layout_set_label(this.pipelineLayout, value);
  }

  dispose(): void {
    subscript_typegpu_pipeline_layout_release(this.pipelineLayout);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPUShaderModule {
  shaderModule: SubscriptTypegpuShaderModule;

  constructor(shaderModule: SubscriptTypegpuShaderModule) {
    this.shaderModule = shaderModule;
  }

  label(value: string): void {
    subscript_typegpu_shader_module_set_label(this.shaderModule, value);
  }

  dispose(): void {
    subscript_typegpu_shader_module_release(this.shaderModule);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPUComputePipeline {
  computePipeline: SubscriptTypegpuComputePipeline;

  constructor(computePipeline: SubscriptTypegpuComputePipeline) {
    this.computePipeline = computePipeline;
  }

  label(value: string): void {
    subscript_typegpu_compute_pipeline_set_label(this.computePipeline, value);
  }

  getBindGroupLayout(index: u32): GPUBindGroupLayout {
    return new GPUBindGroupLayout(subscript_typegpu_compute_pipeline_get_bind_group_layout(this.computePipeline, index));
  }

  dispose(): void {
    subscript_typegpu_compute_pipeline_release(this.computePipeline);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPURenderPipeline {
  renderPipeline: SubscriptTypegpuRenderPipeline;

  constructor(renderPipeline: SubscriptTypegpuRenderPipeline) {
    this.renderPipeline = renderPipeline;
  }

  label(value: string): void {
    subscript_typegpu_render_pipeline_set_label(this.renderPipeline, value);
  }

  getBindGroupLayout(index: u32): GPUBindGroupLayout {
    return new GPUBindGroupLayout(subscript_typegpu_render_pipeline_get_bind_group_layout(this.renderPipeline, index));
  }

  dispose(): void {
    subscript_typegpu_render_pipeline_release(this.renderPipeline);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPUCommandEncoder {
  commandEncoder: SubscriptTypegpuCommandEncoder;

  constructor(commandEncoder: SubscriptTypegpuCommandEncoder) {
    this.commandEncoder = commandEncoder;
  }

  beginRenderPass(descriptor: GPURenderPassDescriptor): GPURenderPassEncoder {
    return new GPURenderPassEncoder(subscript_typegpu_command_encoder_begin_render_pass(this.commandEncoder, toSubscriptTypegpuRenderPassDescriptor(descriptor)));
  }

  beginComputePass(descriptor: GPUComputePassDescriptor | null = null): GPUComputePassEncoder {
    if (descriptor === null) {
      return new GPUComputePassEncoder(subscript_typegpu_command_encoder_begin_compute_pass(this.commandEncoder, null));
    }
    return new GPUComputePassEncoder(subscript_typegpu_command_encoder_begin_compute_pass(this.commandEncoder, toSubscriptTypegpuComputePassDescriptor(descriptor)));
  }

  beginComputePassDefault(): GPUComputePassEncoder {
    return new GPUComputePassEncoder(subscript_typegpu_command_encoder_begin_compute_pass(this.commandEncoder, null));
  }

  copyBufferToBuffer(source: GPUBuffer, sourceOffset: u64, destination: GPUBuffer, destinationOffset: u64, size: u64): void {
    subscript_typegpu_command_encoder_copy_buffer_to_buffer(this.commandEncoder, source.buffer, sourceOffset, destination.buffer, destinationOffset, size);
  }

  copyBufferToTexture(source: GPUTexelCopyBufferInfo, destination: GPUTexelCopyTextureInfo, copySize: GPUExtent3D): void {
    subscript_typegpu_command_encoder_copy_buffer_to_texture(this.commandEncoder, toSubscriptTypegpuTexelCopyBufferInfo(source), toSubscriptTypegpuTexelCopyTextureInfo(destination), toSubscriptTypegpuExtent3D(copySize));
  }

  copyTextureToBuffer(source: GPUTexelCopyTextureInfo, destination: GPUTexelCopyBufferInfo, copySize: GPUExtent3D): void {
    subscript_typegpu_command_encoder_copy_texture_to_buffer(this.commandEncoder, toSubscriptTypegpuTexelCopyTextureInfo(source), toSubscriptTypegpuTexelCopyBufferInfo(destination), toSubscriptTypegpuExtent3D(copySize));
  }

  copyTextureToTexture(source: GPUTexelCopyTextureInfo, destination: GPUTexelCopyTextureInfo, copySize: GPUExtent3D): void {
    subscript_typegpu_command_encoder_copy_texture_to_texture(this.commandEncoder, toSubscriptTypegpuTexelCopyTextureInfo(source), toSubscriptTypegpuTexelCopyTextureInfo(destination), toSubscriptTypegpuExtent3D(copySize));
  }

  clearBuffer(buffer: GPUBuffer, offset: u64, size: u64): void {
    subscript_typegpu_command_encoder_clear_buffer(this.commandEncoder, buffer.buffer, offset, size);
  }

  resolveQuerySet(querySet: GPUQuerySet, firstQuery: u32, queryCount: u32, destination: GPUBuffer, destinationOffset: u64): void {
    subscript_typegpu_command_encoder_resolve_query_set(this.commandEncoder, querySet.querySet, firstQuery, queryCount, destination.buffer, destinationOffset);
  }

  finish(descriptor: GPUCommandBufferDescriptor | null = null): GPUCommandBuffer {
    if (descriptor === null) {
      return new GPUCommandBuffer(subscript_typegpu_command_encoder_finish(this.commandEncoder, null));
    }
    return new GPUCommandBuffer(subscript_typegpu_command_encoder_finish(this.commandEncoder, toSubscriptTypegpuCommandBufferDescriptor(descriptor)));
  }

  finishDefault(): GPUCommandBuffer {
    return new GPUCommandBuffer(subscript_typegpu_command_encoder_finish(this.commandEncoder, null));
  }

  label(value: string): void {
    subscript_typegpu_command_encoder_set_label(this.commandEncoder, value);
  }

  pushDebugGroup(groupLabel: string): void {
    subscript_typegpu_command_encoder_push_debug_group(this.commandEncoder, groupLabel);
  }

  popDebugGroup(): void {
    subscript_typegpu_command_encoder_pop_debug_group(this.commandEncoder);
  }

  insertDebugMarker(markerLabel: string): void {
    subscript_typegpu_command_encoder_insert_debug_marker(this.commandEncoder, markerLabel);
  }

  dispose(): void {
    subscript_typegpu_command_encoder_release(this.commandEncoder);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPUComputePassEncoder {
  computePassEncoder: SubscriptTypegpuComputePassEncoder;

  constructor(computePassEncoder: SubscriptTypegpuComputePassEncoder) {
    this.computePassEncoder = computePassEncoder;
  }

  setPipeline(pipeline: GPUComputePipeline): void {
    subscript_typegpu_compute_pass_encoder_set_pipeline(this.computePassEncoder, pipeline.computePipeline);
  }

  dispatchWorkgroups(workgroupCountX: u32, workgroupCountY: u32 = 1, workgroupCountZ: u32 = 1): void {
    subscript_typegpu_compute_pass_encoder_dispatch_workgroups(this.computePassEncoder, workgroupCountX, workgroupCountY, workgroupCountZ);
  }

  dispatchWorkgroupsIndirect(indirectBuffer: GPUBuffer, indirectOffset: u64): void {
    subscript_typegpu_compute_pass_encoder_dispatch_workgroups_indirect(this.computePassEncoder, indirectBuffer.buffer, indirectOffset);
  }

  end(): void {
    subscript_typegpu_compute_pass_encoder_end(this.computePassEncoder);
  }

  label(value: string): void {
    subscript_typegpu_compute_pass_encoder_set_label(this.computePassEncoder, value);
  }

  pushDebugGroup(groupLabel: string): void {
    subscript_typegpu_compute_pass_encoder_push_debug_group(this.computePassEncoder, groupLabel);
  }

  popDebugGroup(): void {
    subscript_typegpu_compute_pass_encoder_pop_debug_group(this.computePassEncoder);
  }

  insertDebugMarker(markerLabel: string): void {
    subscript_typegpu_compute_pass_encoder_insert_debug_marker(this.computePassEncoder, markerLabel);
  }

  setBindGroup(index: u32, bindGroup: GPUBindGroup | null, dynamicOffsets: u32[] = []): void {
    subscript_typegpu_compute_pass_encoder_set_bind_group(this.computePassEncoder, index, toNullableSubscriptTypegpuBindGroup(bindGroup), dynamicOffsets);
  }

  dispose(): void {
    subscript_typegpu_compute_pass_encoder_release(this.computePassEncoder);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPURenderPassEncoder {
  renderPassEncoder: SubscriptTypegpuRenderPassEncoder;

  constructor(renderPassEncoder: SubscriptTypegpuRenderPassEncoder) {
    this.renderPassEncoder = renderPassEncoder;
  }

  setViewport(x: f32, y: f32, width: f32, height: f32, minDepth: f32, maxDepth: f32): void {
    subscript_typegpu_render_pass_encoder_set_viewport(this.renderPassEncoder, x, y, width, height, minDepth, maxDepth);
  }

  setScissorRect(x: u32, y: u32, width: u32, height: u32): void {
    subscript_typegpu_render_pass_encoder_set_scissor_rect(this.renderPassEncoder, x, y, width, height);
  }

  setBlendConstant(color: GPUColor): void {
    subscript_typegpu_render_pass_encoder_set_blend_constant(this.renderPassEncoder, toSubscriptTypegpuColor(color));
  }

  setStencilReference(reference: u32): void {
    subscript_typegpu_render_pass_encoder_set_stencil_reference(this.renderPassEncoder, reference);
  }

  beginOcclusionQuery(queryIndex: u32): void {
    subscript_typegpu_render_pass_encoder_begin_occlusion_query(this.renderPassEncoder, queryIndex);
  }

  endOcclusionQuery(): void {
    subscript_typegpu_render_pass_encoder_end_occlusion_query(this.renderPassEncoder);
  }

  executeBundles(bundles: GPURenderBundle[]): void {
    subscript_typegpu_render_pass_encoder_execute_bundles(this.renderPassEncoder, toSubscriptTypegpuRenderBundleArray(bundles));
  }

  end(): void {
    subscript_typegpu_render_pass_encoder_end(this.renderPassEncoder);
  }

  label(value: string): void {
    subscript_typegpu_render_pass_encoder_set_label(this.renderPassEncoder, value);
  }

  pushDebugGroup(groupLabel: string): void {
    subscript_typegpu_render_pass_encoder_push_debug_group(this.renderPassEncoder, groupLabel);
  }

  popDebugGroup(): void {
    subscript_typegpu_render_pass_encoder_pop_debug_group(this.renderPassEncoder);
  }

  insertDebugMarker(markerLabel: string): void {
    subscript_typegpu_render_pass_encoder_insert_debug_marker(this.renderPassEncoder, markerLabel);
  }

  setBindGroup(index: u32, bindGroup: GPUBindGroup | null, dynamicOffsets: u32[] = []): void {
    subscript_typegpu_render_pass_encoder_set_bind_group(this.renderPassEncoder, index, toNullableSubscriptTypegpuBindGroup(bindGroup), dynamicOffsets);
  }

  setPipeline(pipeline: GPURenderPipeline): void {
    subscript_typegpu_render_pass_encoder_set_pipeline(this.renderPassEncoder, pipeline.renderPipeline);
  }

  setIndexBuffer(buffer: GPUBuffer, indexFormat: GPUIndexFormat, offset: u64, size: u64): void {
    subscript_typegpu_render_pass_encoder_set_index_buffer(this.renderPassEncoder, buffer.buffer, indexFormat, offset, size);
  }

  setVertexBuffer(slot: u32, buffer: GPUBuffer | null, offset: u64, size: u64): void {
    subscript_typegpu_render_pass_encoder_set_vertex_buffer(this.renderPassEncoder, slot, toNullableSubscriptTypegpuBuffer(buffer), offset, size);
  }

  draw(vertexCount: u32, instanceCount: u32 = 1, firstVertex: u32 = 0, firstInstance: u32 = 0): void {
    subscript_typegpu_render_pass_encoder_draw(this.renderPassEncoder, vertexCount, instanceCount, firstVertex, firstInstance);
  }

  drawIndexed(indexCount: u32, instanceCount: u32 = 1, firstIndex: u32 = 0, baseVertex: i32 = 0, firstInstance: u32 = 0): void {
    subscript_typegpu_render_pass_encoder_draw_indexed(this.renderPassEncoder, indexCount, instanceCount, firstIndex, baseVertex, firstInstance);
  }

  drawIndirect(indirectBuffer: GPUBuffer, indirectOffset: u64): void {
    subscript_typegpu_render_pass_encoder_draw_indirect(this.renderPassEncoder, indirectBuffer.buffer, indirectOffset);
  }

  drawIndexedIndirect(indirectBuffer: GPUBuffer, indirectOffset: u64): void {
    subscript_typegpu_render_pass_encoder_draw_indexed_indirect(this.renderPassEncoder, indirectBuffer.buffer, indirectOffset);
  }

  dispose(): void {
    subscript_typegpu_render_pass_encoder_release(this.renderPassEncoder);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPUCommandBuffer {
  commandBuffer: SubscriptTypegpuCommandBuffer;

  constructor(commandBuffer: SubscriptTypegpuCommandBuffer) {
    this.commandBuffer = commandBuffer;
  }

  label(value: string): void {
    subscript_typegpu_command_buffer_set_label(this.commandBuffer, value);
  }

  dispose(): void {
    subscript_typegpu_command_buffer_release(this.commandBuffer);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPURenderBundleEncoder {
  renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder;

  constructor(renderBundleEncoder: SubscriptTypegpuRenderBundleEncoder) {
    this.renderBundleEncoder = renderBundleEncoder;
  }

  finish(descriptor: GPURenderBundleDescriptor | null = null): GPURenderBundle {
    if (descriptor === null) {
      return new GPURenderBundle(subscript_typegpu_render_bundle_encoder_finish(this.renderBundleEncoder, null));
    }
    return new GPURenderBundle(subscript_typegpu_render_bundle_encoder_finish(this.renderBundleEncoder, toSubscriptTypegpuRenderBundleDescriptor(descriptor)));
  }

  finishDefault(): GPURenderBundle {
    return new GPURenderBundle(subscript_typegpu_render_bundle_encoder_finish(this.renderBundleEncoder, null));
  }

  label(value: string): void {
    subscript_typegpu_render_bundle_encoder_set_label(this.renderBundleEncoder, value);
  }

  pushDebugGroup(groupLabel: string): void {
    subscript_typegpu_render_bundle_encoder_push_debug_group(this.renderBundleEncoder, groupLabel);
  }

  popDebugGroup(): void {
    subscript_typegpu_render_bundle_encoder_pop_debug_group(this.renderBundleEncoder);
  }

  insertDebugMarker(markerLabel: string): void {
    subscript_typegpu_render_bundle_encoder_insert_debug_marker(this.renderBundleEncoder, markerLabel);
  }

  setBindGroup(index: u32, bindGroup: GPUBindGroup | null, dynamicOffsets: u32[] = []): void {
    subscript_typegpu_render_bundle_encoder_set_bind_group(this.renderBundleEncoder, index, toNullableSubscriptTypegpuBindGroup(bindGroup), dynamicOffsets);
  }

  setPipeline(pipeline: GPURenderPipeline): void {
    subscript_typegpu_render_bundle_encoder_set_pipeline(this.renderBundleEncoder, pipeline.renderPipeline);
  }

  setIndexBuffer(buffer: GPUBuffer, indexFormat: GPUIndexFormat, offset: u64, size: u64): void {
    subscript_typegpu_render_bundle_encoder_set_index_buffer(this.renderBundleEncoder, buffer.buffer, indexFormat, offset, size);
  }

  setVertexBuffer(slot: u32, buffer: GPUBuffer | null, offset: u64, size: u64): void {
    subscript_typegpu_render_bundle_encoder_set_vertex_buffer(this.renderBundleEncoder, slot, toNullableSubscriptTypegpuBuffer(buffer), offset, size);
  }

  draw(vertexCount: u32, instanceCount: u32 = 1, firstVertex: u32 = 0, firstInstance: u32 = 0): void {
    subscript_typegpu_render_bundle_encoder_draw(this.renderBundleEncoder, vertexCount, instanceCount, firstVertex, firstInstance);
  }

  drawIndexed(indexCount: u32, instanceCount: u32 = 1, firstIndex: u32 = 0, baseVertex: i32 = 0, firstInstance: u32 = 0): void {
    subscript_typegpu_render_bundle_encoder_draw_indexed(this.renderBundleEncoder, indexCount, instanceCount, firstIndex, baseVertex, firstInstance);
  }

  drawIndirect(indirectBuffer: GPUBuffer, indirectOffset: u64): void {
    subscript_typegpu_render_bundle_encoder_draw_indirect(this.renderBundleEncoder, indirectBuffer.buffer, indirectOffset);
  }

  drawIndexedIndirect(indirectBuffer: GPUBuffer, indirectOffset: u64): void {
    subscript_typegpu_render_bundle_encoder_draw_indexed_indirect(this.renderBundleEncoder, indirectBuffer.buffer, indirectOffset);
  }

  dispose(): void {
    subscript_typegpu_render_bundle_encoder_release(this.renderBundleEncoder);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPURenderBundle {
  renderBundle: SubscriptTypegpuRenderBundle;

  constructor(renderBundle: SubscriptTypegpuRenderBundle) {
    this.renderBundle = renderBundle;
  }

  label(value: string): void {
    subscript_typegpu_render_bundle_set_label(this.renderBundle, value);
  }

  dispose(): void {
    subscript_typegpu_render_bundle_release(this.renderBundle);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export class GPUQuerySet {
  querySet: SubscriptTypegpuQuerySet;

  constructor(querySet: SubscriptTypegpuQuerySet) {
    this.querySet = querySet;
  }

  destroy(): void {
    subscript_typegpu_query_set_destroy(this.querySet);
  }

  type(): GPUQueryType {
    return subscript_typegpu_query_set_get_type(this.querySet);
  }

  count(): u32 {
    return subscript_typegpu_query_set_get_count(this.querySet);
  }

  label(value: string): void {
    subscript_typegpu_query_set_set_label(this.querySet, value);
  }

  dispose(): void {
    subscript_typegpu_query_set_release(this.querySet);
  }

  [Symbol.dispose](): void {
    this.dispose();
  }
}

export const gpu: GPU = new GPU(subscript_typegpu_create_instance());
