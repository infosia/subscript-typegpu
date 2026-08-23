# backend-request — one variable over Dawn and yawgpu

Goal: `SUBSCRIPT_TYPEGPU_BACKEND=vulkan` selects the Vulkan adapter on
both yawgpu and Dawn. The script-visible surface is unchanged.

## Measured constraints (2026-08-23, Windows 11, NVIDIA RTX 5060 Ti)

- Dawn rejects the yawgpu instance chain: `Unexpected chained struct
  of type SType::1879048193` and a null instance. The L4 Rev 1 claim
  that an unknown `sType` is ignored was a docs claim and was wrong.
- Dawn honors `WGPURequestAdapterOptions.backendType`. The
  webgpu-native-cts binary returned the named backend for `vulkan`
  and `d3d12`.
- yawgpu read only `featureLevel` from the adapter options. Its
  backend is fixed at instance creation by the vendor chain, and a
  surface binds to that backend at creation. A lazy backend at
  request time is not possible there.
- yawgpu exports `yawgpuDeviceCreateExternalTexture` without a
  feature flag. Dawn does not export it. The facade uses it as the
  yawgpu marker.
- Dawn on Windows loads `vulkan-1.dll` and `d3dcompiler_47.dll` from
  the library directory only, never from `System32`. A missing one
  fails with `DynamicLib.Open: ... Windows Error: 87`.

## Design

`specs/blocks/facade.md`: L4 Rev 2 (marker probe, chain only to
yawgpu), L13 Rev 1 (five values, loud rejection of `d3d11`/`d3d12` on
yawgpu), L15 (the request carries `backendType`, no post-request
check).

Escalated to yawgpu: honor `backendType` at
`wgpuInstanceRequestAdapter`. Landed as rule IB5 at
https://github.com/infosia/yawgpu commit `13ac0b4`. A mismatch fails
with `WGPURequestAdapterStatus_Unavailable` and a message.

## Evidence

Implementation at `8f99796`. `tools/regen.sh` after the change: zero
diff. Gate `tools/gate.sh --require-backend` on windows-msvc with the
yawgpu release library at `13ac0b4`: green, 244 passed, 1 ignored,
191 s. The new red: `backend::d3d12_request_is_rejected_by_yawgpu`
asserts the null instance and the diagnostic against the yawgpu
library.

Live lane `tools/live.sh`, `SUBSCRIPT_TYPEGPU_BACKEND=vulkan`:

- Before the x17 fix: x01–x16 pass, x17 red on both implementations,
  identical bytes.
- After the x17 fix: x01–x18 PASS on yawgpu Vulkan (54.64 s) and on
  Dawn Vulkan (55.10 s). Gate after the fix: green, 247 passed, 1
  ignored, 192 s. These numbers are not reference-machine
  measurements.

## Defect found by the Vulkan lane

`x17-live-indirect` printed `FAIL pixel 1,2` on yawgpu Vulkan, Dawn
Vulkan, and Dawn D3D12, and passed on Metal. Measured bytes: `got
64,127,191,255 want 64,128,191,255`. The fragment green constant
`0.5` times 255 is the exact tie `127.5`, and the float-to-unorm
rounding of a tie is implementation-defined. The rule is RN14 Rev 1.
The fix changes the constant to `0.6` (153 on both vendors).
