import { GPUHostOwnedDevice } from "./webgpu";

function main(instance: SubscriptTypegpuInstance, device: SubscriptTypegpuDevice): void {
  const wrapper = new GPUHostOwnedDevice(instance, device);
}
