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
  and reads them after. `simulateComputeThreads` mirrors
  `dispatchThreads`: it takes thread counts and rounds up by the
  workgroup size, so a host run and a GPU dispatch take the same
  counts.
- **CL2 — Single-threaded, so no barrier.** Rev 1, 2026-08-23. A
  kernel that reaches `workgroupBarrier`, `storageBarrier`, a
  workgroup variable, an atomic, or a private variable it writes
  runs invocations in sequence on the host, which is not the GPU's
  per-invocation state. The generator emits `<name>_HOST_RUNNABLE:
  boolean` from the call graph: false when any of those is reached
  (a private variable the kernel only reads is fine, because the
  host instance holds its initializer). `simulateCompute` takes the
  constant as its last argument and traps with `CL2`, the method,
  and the pipeline name when it is false. The harness enforces the
  pairing: the `simulate` module walks each `a`, `b`, and `x`
  program's HIR, and every `simulateCompute*` call must pass a
  `FuncRef` kernel and the `<name>_HOST_RUNNABLE` global of the
  pipeline declaration that names that kernel. Any other argument
  fails the gate with the program and the call. A later revision
  can phase invocations at barriers.
- **CL3 — Same numbers.** The host and the GPU compute in `f32`.
  The live programs `x01`–`x04` and `x09` compare the GPU with a
  host formula today; under CL they compare the GPU with
  `simulateCompute` over the same kernel, so the kernel body itself
  is the oracle and the formula leaves the program.
- **CL4 — The lane is a gate module.** Rev 1. Every `b` program
  whose pipeline is host-runnable calls `simulateCompute` itself
  after its Noop dispatch and prints one `host:` line per pipeline
  with the host result by value (T2: a host-computed result). The
  golden pins the value on both tiers. The harness `simulate`
  module requires the line for every host-runnable pipeline, reads
  host-runnability from `Generated` (never from the support
  module's text), and performs the CL2 pairing check. The lane is a
  module of the harness executable, never a new executable.

- **CL5 — The host texture bodies model the bound sampler.** A
  program that runs `simulateCompute` over a texture kernel binds
  the same sampler kind on the GPU and on the host. The host
  `Sampler` carries its filter mode, `sampleLevel` and `sample`
  implement `nearest` and trap with `TX3` for any other mode. A
  host golden never models a resource the pipeline does not bind.
- **CL6 — The analysis has a red per branch.** Emitter tests assert
  `<name>_HOST_RUNNABLE` false for a kernel that reaches only a
  barrier, only an atomic on a storage schema, only a written
  private variable, and only a workgroup variable, each without the
  others, and true for a kernel that only reads a private variable.

## Exit

`x01`–`x04` and `x09` use `simulateCompute` as their oracle and
print `PASS` on Metal. One `b` program's host golden is committed
and compared on both tiers. `CL2` has a fixture that reaches the
trap through the generator's constant. CL6's five emitter tests
exist. The lane is a gate module, not a new executable. Budgets
hold.
