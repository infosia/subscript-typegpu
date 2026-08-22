//! Generator-owned uniform barrier placement checks.

use std::path::PathBuf;

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
fn non_uniform_barrier_names_statement_value_and_author() {
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
    let diagnostics =
        subscript_typegpu_gen::generate(&files).expect_err("non-uniform barrier must be rejected");
    assert_eq!(diagnostics.len(), 1, "one uniformity diagnostic");
    let diagnostic = &diagnostics[0];
    assert!(diagnostic.message.starts_with("K22:"));
    assert!(diagnostic.message.contains("barrier statement"));
    assert!(diagnostic.message.contains("builtin `ctx.localIndex`"));
    assert!(diagnostic.message.ends_with("(author)"));
}
