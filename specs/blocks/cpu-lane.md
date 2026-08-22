# Block: the CPU lane (CL-rules)

P7 contract. Rev 0, 2026-08-23. The owner decided on 2026-08-23 to
run P7.

## What the lane is

- **CL1 — A kernel runs on the host through its own body.** Every
  library method a kernel calls has a real host body (D7), every
  binding wrapper stores its values on the host (PI5), and every
  builtin is a field of `ComputeInvocation`. `simulateCompute<L>(kernel,
  res: L, spec: ComputePipelineSpec, workgroups: FixedArray<u32, 3>)`
  is a library generic function with a real body: it loops over
  workgroups and local invocations in row-major order, builds one
  `ComputeInvocation` per invocation, and calls `kernel(res, ctx)`.
  The author fills `res`'s wrappers with host data before the call
  and reads them after.
- **CL2 — Single-threaded, so no barrier.** A kernel that reaches
  `workgroupBarrier`, `storageBarrier`, a workgroup variable, or an
  atomic runs invocations in sequence, which is not the GPU's
  lockstep. `simulateCompute` traps with `CL2` when the kernel's
  call graph reaches one of those (the generator emits a
  `<name>_HOST_RUNNABLE: boolean` constant and the library checks it
  through the spec). A later revision can phase invocations at
  barriers.
- **CL3 — Same numbers.** The host and the GPU compute in `f32`.
  The live programs `x01`–`x04` and `x09` compare the GPU with a
  host formula today; under CL they compare the GPU with
  `simulateCompute` over the same kernel, so the kernel body itself
  is the oracle and the formula leaves the program.
- **CL4 — The lane is a gate module.** The harness runs
  `simulateCompute` for every `b` program's pipelines on the dev
  tier (Noop executes no shader, so the host run is the only
  numeric check the gate lane has) and compares the result with a
  committed host golden per pipeline. A live run compares the GPU
  with the same host run.

## Exit

`x01`–`x04` and `x09` use `simulateCompute` as their oracle and
print `PASS` on Metal. One `b` program's host golden is committed
and compared on both tiers. `CL2` has a fixture. Budgets hold.
