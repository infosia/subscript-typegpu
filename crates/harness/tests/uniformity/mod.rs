//! Naga owns the uniform-control-flow check for author barriers.

use std::path::PathBuf;

use naga::valid::{Capabilities, UniformityRequirements, ValidationFlags, Validator};
use subscript_compiler::SourceFile;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("harness crate is below the repository root")
        .to_path_buf()
}

fn read(name: &str) -> String {
    std::fs::read_to_string(root().join("lib").join(name))
        .unwrap_or_else(|error| panic!("read {name}: {error}"))
}

#[test]
fn non_uniform_barrier_names_the_kernel_and_author() {
    let files = vec![
        SourceFile::ambient(
            "subscript-typegpu.generated.d.ts",
            read("subscript-typegpu.generated.d.ts"),
        ),
        SourceFile::ambient(
            "wire-enum-aliases.generated.d.ts",
            read("wire-enum-aliases.generated.d.ts"),
        ),
        SourceFile::new("webgpu.ts", read("webgpu.ts")),
        SourceFile::new("typegpu-types.ts", read("typegpu-types.ts")),
        SourceFile::new("typegpu.ts", read("typegpu.ts")),
        SourceFile::new(
            "uniformity-test.ts",
            r#"
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage, workgroupBarrier } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
function nonUniform(res: Layout, ctx: ComputeInvocation): void {
  if (ctx.localIndex === 0) { workgroupBarrier(); }
  res.output[ctx.globalId.x] = new Item(ctx.localIndex);
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(nonUniform, { workgroupSize: [4, 1, 1] });
"#,
        ),
    ];
    let generated = subscript_typegpu_gen::generate(&files).expect("generate non-uniform barrier");
    let wgsl = &generated.pipelines[0].1;
    let module = naga::front::wgsl::parse_str(wgsl).expect("parse non-uniform barrier WGSL");
    let info = Validator::new(ValidationFlags::all(), Capabilities::empty())
        .validate(&module)
        .expect("naga must analyze the non-uniform barrier module");
    let requirements = info.get_entry_point(0).uniformity.requirements;
    assert!(
        requirements.contains(UniformityRequirements::WORK_GROUP_BARRIER),
        "naga must diagnose the entry point as requiring uniform control flow"
    );
    let report =
        format!("K22 (author): kernel `nonUniform`: naga uniformity diagnostic: {requirements:?}");
    assert!(report.contains("K22 (author): kernel `nonUniform`"));
    assert!(
        report.to_ascii_lowercase().contains("uniform"),
        "naga report lacks its uniformity diagnostic:\n{report}"
    );
}
