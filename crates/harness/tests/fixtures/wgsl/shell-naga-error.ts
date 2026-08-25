import { ComputeInvocation, ComputePipelineSpec, computePipeline, MutStorage, WgslShellSpec, wgslShell } from "./typegpu";

function badShell(value: u32): u32 { return value; }
const shell: WgslShellSpec = wgslShell<(value: u32) => u32>(badShell, {
  body: "return missing_shell_name + value;",
});
class ShellLayout { output!: MutStorage<u32>; }
function shellKernel(res: ShellLayout, ctx: ComputeInvocation): void { res.output[0] = badShell(1); }
export const shellNaga: ComputePipelineSpec = computePipeline<ShellLayout>(shellKernel, { name: "shellNaga", workgroupSize: [1,1,1] });
