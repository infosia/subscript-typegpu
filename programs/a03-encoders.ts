// program: a03-encoders
// purpose: cover asynchronous pipelines, render extras, bundles, indirect calls, and debug labels
// exercises: EG4, PL2-PL4, E1, E4
// questions: none

import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUComputePipeline,
  GPUDevice,
  GPURenderPipeline,
  GPUShaderModule,
  GPUTextureUsage,
} from "./webgpu";

const SHADER: string = `
@compute @workgroup_size(1) fn computeMain() {}
@vertex fn vertexMain(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
  let x = f32(i32(index) - 1);
  let y = select(-1.0, 1.0, index == 2u);
  return vec4f(x, y, 0.0, 1.0);
}
@fragment fn fragmentMain() -> @location(0) vec4f {
  return vec4f(0.25, 0.5, 0.75, 1.0);
}
`;

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) {
    print("FAIL adapter");
    gpu.dispose();
    return;
  }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) {
    print("FAIL device");
    adapterResult.dispose();
    gpu.dispose();
    return;
  }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const queue = device.queue();
    device.pushErrorScope("validation");
    using shader: GPUShaderModule = device.createShaderModule({ label: "a03-shader", code: SHADER });
    shader.label("a03-shader-label");
    using bindGroupLayout = device.createBindGroupLayout({ label: "a03-layout", entries: [] });
    bindGroupLayout.label("a03-layout-label");
    using pipelineLayout = device.createPipelineLayout({
      label: "a03-pipeline-layout",
      bindGroupLayouts: [bindGroupLayout],
    });
    pipelineLayout.label("a03-pipeline-layout-label");
    using bindGroup = device.createBindGroup({
      label: "a03-bind-group",
      layout: bindGroupLayout,
      entries: [],
    });
    bindGroup.label("a03-bind-group-label");

    using compute: GPUComputePipeline = device.createComputePipeline({
      label: "a03-compute",
      layout: pipelineLayout,
      compute: { module: shader, entryPoint: "computeMain" },
    });
    compute.label("a03-compute-label");
    const computeAsync: GPUComputePipeline | null = await device.createComputePipelineAsync({
      label: "a03-compute-async",
      layout: pipelineLayout,
      compute: { module: shader, entryPoint: "computeMain" },
    });
    if (computeAsync !== null) {
      computeAsync.label("a03-compute-async-label");
      computeAsync.dispose();
    }

    using render: GPURenderPipeline = device.createRenderPipeline({
      label: "a03-render",
      layout: pipelineLayout,
      vertex: { module: shader, entryPoint: "vertexMain", buffers: [] },
      primitive: { topology: "triangle-list" },
      multisample: { count: 1 },
      fragment: {
        module: shader,
        entryPoint: "fragmentMain",
        targets: [{
          format: "rgba8unorm",
          blend: {
            color: { operation: "add", srcFactor: "one", dstFactor: "zero" },
            alpha: { operation: "add", srcFactor: "one", dstFactor: "zero" },
          },
        }],
      },
    });
    render.label("a03-render-label");
    const renderAsync: GPURenderPipeline | null = await device.createRenderPipelineAsync({
      label: "a03-render-async",
      layout: pipelineLayout,
      vertex: { module: shader, entryPoint: "vertexMain", buffers: [] },
      primitive: { topology: "triangle-list" },
      multisample: { count: 1 },
      fragment: {
        module: shader,
        entryPoint: "fragmentMain",
        targets: [{ format: "rgba8unorm" }],
      },
    });
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print("pipeline:invalid");
      print("FAIL");
      return;
    }
    if (renderAsync !== null) {
      renderAsync.label("a03-render-async-label");
      renderAsync.dispose();
    }

    using indirect = device.createBuffer({
      label: "a03-indirect",
      size: 64,
      usage: GPUBufferUsage.INDIRECT + GPUBufferUsage.COPY_DST,
    });
    queue.writeBuffer(indirect, 0, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    using vertex = device.createBuffer({
      label: "a03-vertex",
      size: 64,
      usage: GPUBufferUsage.VERTEX,
    });
    using index = device.createBuffer({
      label: "a03-index",
      size: 64,
      usage: GPUBufferUsage.INDEX,
    });
    using target = device.createTexture({
      label: "a03-target",
      size: { width: 4, height: 4, depthOrArrayLayers: 1 },
      mipLevelCount: 1,
      sampleCount: 1,
      dimension: "2d",
      format: "rgba8unorm",
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });
    using targetView = target.createView();
    using occlusion = device.createQuerySet({ label: "a03-occlusion", type: "occlusion", count: 2 });

    using bundleEncoder = device.createRenderBundleEncoder({
      label: "a03-bundle-encoder",
      colorFormats: ["rgba8unorm"],
      sampleCount: 1,
    });
    bundleEncoder.label("a03-bundle-encoder-label");
    bundleEncoder.pushDebugGroup("a03-bundle");
    bundleEncoder.insertDebugMarker("a03-bundle-marker");
    bundleEncoder.setPipeline(render);
    bundleEncoder.setBindGroup(0, bindGroup);
    bundleEncoder.setVertexBuffer(0, vertex, 0, 64);
    bundleEncoder.setIndexBuffer(index, "uint16", 0, 64);
    bundleEncoder.draw(3);
    bundleEncoder.drawIndexed(3);
    bundleEncoder.drawIndirect(indirect, 0);
    bundleEncoder.drawIndexedIndirect(indirect, 0);
    bundleEncoder.popDebugGroup();
    using bundle = bundleEncoder.finish({ label: "a03-bundle" });
    bundle.label("a03-bundle-label");

    using encoder = device.createCommandEncoder({ label: "a03-encoder" });
    using computePass = encoder.beginComputePass({ label: "a03-compute-pass" });
    computePass.label("a03-compute-pass-label");
    computePass.pushDebugGroup("a03-compute");
    computePass.insertDebugMarker("a03-compute-marker");
    computePass.setPipeline(compute);
    computePass.setBindGroup(0, bindGroup);
    computePass.dispatchWorkgroupsIndirect(indirect, 0);
    computePass.popDebugGroup();
    computePass.end();

    using renderPass = encoder.beginRenderPass({
      label: "a03-render-pass",
      colorAttachments: [{
        view: targetView,
        clearValue: { r: 0, g: 0, b: 0, a: 1 },
        loadOp: "clear",
        storeOp: "store",
      }],
      occlusionQuerySet: occlusion,
    });
    renderPass.label("a03-render-pass-label");
    renderPass.pushDebugGroup("a03-render");
    renderPass.insertDebugMarker("a03-render-marker");
    renderPass.setViewport(0.0, 0.0, 4.0, 4.0, 0.0, 1.0);
    renderPass.setScissorRect(0, 0, 4, 4);
    renderPass.setBlendConstant({ r: 0, g: 0, b: 0, a: 0 });
    renderPass.setStencilReference(0);
    renderPass.setPipeline(render);
    renderPass.setBindGroup(0, bindGroup);
    renderPass.setVertexBuffer(0, vertex, 0, 64);
    renderPass.setIndexBuffer(index, "uint16", 0, 64);
    renderPass.beginOcclusionQuery(0);
    renderPass.drawIndirect(indirect, 0);
    renderPass.drawIndexedIndirect(indirect, 0);
    renderPass.endOcclusionQuery();
    renderPass.executeBundles([bundle]);
    renderPass.popDebugGroup();
    renderPass.end();
    using command = encoder.finish({ label: "a03-command" });
    queue.submit([command]);
    print("pipelines:covered");
    print("render:covered");
  }
  gpu.dispose();
  print("PASS");
}
