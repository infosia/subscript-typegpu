// program: a02-textures
// purpose: cover texture copies, mapped buffer access, query sets, and object labels
// exercises: EG4, B1-B4, E1-E6, F15, F20
// questions: none

import {
  gpu,
  GPUAdapter,
  GPUBuffer,
  GPUBufferUsage,
  GPUDevice,
  GPUMapMode,
  GPUQuerySet,
  GPUTexture,
  GPUTextureUsage,
} from "./webgpu";

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
    device.pushErrorScope("validation");
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print("pipeline:invalid");
      print("FAIL");
      return;
    }
    const queue = device.queue();
    using upload = device.createBuffer({
      label: "a02-upload",
      size: 256,
      usage: GPUBufferUsage.COPY_SRC + GPUBufferUsage.COPY_DST + GPUBufferUsage.MAP_WRITE,
      mappedAtCreation: true,
    });
    upload.label("a02-upload-label");
    upload.usage();
    upload.mapState();
    upload.writeMappedRange(0, [1, 2, 3, 4]);
    upload.unmap();
    queue.writeBufferF32(upload, 0, [1.0, 2.0, 3.0, 4.0]);

    using readback = device.createBuffer({
      label: "a02-readback",
      size: 256,
      usage: GPUBufferUsage.COPY_DST + GPUBufferUsage.MAP_READ,
    });
    using textureA: GPUTexture = device.createTexture({
      label: "a02-texture-a",
      size: { width: 1, height: 1, depthOrArrayLayers: 1 },
      mipLevelCount: 1,
      sampleCount: 1,
      dimension: "2d",
      format: "rgba8unorm",
      usage: GPUTextureUsage.COPY_DST + GPUTextureUsage.COPY_SRC + GPUTextureUsage.TEXTURE_BINDING,
    });
    using textureB: GPUTexture = device.createTexture({
      label: "a02-texture-b",
      size: { width: 1, height: 1, depthOrArrayLayers: 1 },
      mipLevelCount: 1,
      sampleCount: 1,
      dimension: "2d",
      format: "rgba8unorm",
      usage: GPUTextureUsage.COPY_DST + GPUTextureUsage.COPY_SRC,
    });
    textureA.label("a02-texture-label");
    textureA.width();
    textureA.height();
    textureA.depthOrArrayLayers();
    textureA.mipLevelCount();
    textureA.sampleCount();
    textureA.dimension();
    textureA.format();
    textureA.usage();
    using view = textureA.createView();
    view.label("a02-view");
    using sampler = device.createSampler({ minFilter: "nearest", magFilter: "nearest" });
    sampler.label("a02-sampler");

    using queries: GPUQuerySet = device.createQuerySet({
      label: "a02-queries",
      type: "occlusion",
      count: 2,
    });
    queries.label("a02-query-label");
    queries.type();
    queries.count();

    using encoder = device.createCommandEncoder({ label: "a02-encoder" });
    encoder.label("a02-encoder-label");
    encoder.pushDebugGroup("a02-copy");
    encoder.insertDebugMarker("a02-before-copy");
    encoder.clearBuffer(readback, 0, 256);
    encoder.copyBufferToTexture(
      { buffer: upload, offset: 0, bytesPerRow: 256, rowsPerImage: 1 },
      { texture: textureA, mipLevel: 0, origin: { x: 0, y: 0, z: 0 }, aspect: "all" },
      { width: 1, height: 1, depthOrArrayLayers: 1 },
    );
    encoder.copyTextureToTexture(
      { texture: textureA, mipLevel: 0, origin: { x: 0, y: 0, z: 0 }, aspect: "all" },
      { texture: textureB, mipLevel: 0, origin: { x: 0, y: 0, z: 0 }, aspect: "all" },
      { width: 1, height: 1, depthOrArrayLayers: 1 },
    );
    encoder.resolveQuerySet(queries, 0, 2, readback, 0);
    encoder.popDebugGroup();
    using command = encoder.finish({ label: "a02-command" });
    command.label("a02-command-label");
    queue.submit([command]);

    const readMapped: boolean = await readback.mapAsync(GPUMapMode.READ, 0, 256);
    if (readMapped) {
      readback.readMappedRangeF32(0, 4);
      readback.unmap();
    }
    queries.destroy();
    textureB.destroy();
    upload.destroy();
    readback.destroy();
    print("textures:covered");
    print("queries:covered");
  }
  gpu.dispose();
  print("PASS");
}
