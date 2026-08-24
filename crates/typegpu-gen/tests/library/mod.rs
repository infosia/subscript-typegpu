use subscript_compiler::SourceFile;

use crate::support;

fn validate(wgsl: &str) {
    let module = naga::front::wgsl::parse_str(wgsl)
        .unwrap_or_else(|error| panic!("WGSL parse failed:\n{}", error.emit_to_string(wgsl)));
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .unwrap_or_else(|error| panic!("WGSL validation failed: {error:?}\n{wgsl}"));
}

#[test]
fn every_library_method_has_the_sc6_body() {
    let root = support::root();
    let files = [SourceFile::new(
        "typegpu-types.ts",
        support::read(&root.join("lib/typegpu-types.ts")),
    )];
    let module = subscript_compiler::check_program(&files).expect("check type library");
    assert_eq!(
        module
            .classes
            .iter()
            .filter(|class| class.pos.file == "typegpu-types.ts")
            .count(),
        23
    );
    for class in module
        .classes
        .iter()
        .filter(|class| class.pos.file == "typegpu-types.ts")
    {
        assert!(
            class.methods.iter().all(|method| !method.body.is_empty()),
            "{} has an empty method body",
            class.name
        );
    }
    for factory in [
        "v3fFrom2",
        "v4fFrom2",
        "v4fFrom3",
        "v2fSplat",
        "v3fSplat",
        "v4fSplat",
        "v3iFrom2",
        "v4iFrom2",
        "v4iFrom3",
        "v2iSplat",
        "v3iSplat",
        "v4iSplat",
        "v3uFrom2",
        "v4uFrom2",
        "v4uFrom3",
        "v2uSplat",
        "v3uSplat",
        "v4uSplat",
        "mat2x2fIdentity",
        "mat3x3fIdentity",
        "mat4x4fIdentity",
    ] {
        let function = module
            .functions
            .iter()
            .find(|function| function.name == factory)
            .unwrap_or_else(|| panic!("missing {factory}"));
        assert!(!function.body.is_empty(), "{factory} has an empty body");
    }
}

#[test]
fn sdf_library_helpers_emit_and_validate() {
    let mut files = support::program_files(&support::root().join("programs/b01-layout.ts"));
    files.pop();
    files.push(SourceFile::new(
        "sdf-library-test.ts",
        r#"
import { Vec2f, Vec3f } from "./typegpu-types";
import { sdSphere, sdBox, sdBoxFrame, sdPlane, sdLine, opUnion, opSmoothUnion } from "./typegpu-sdf";
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { output!: MutStorage<f32>; }
function sdfKernel(res: Layout, ctx: ComputeInvocation): void {
  let value: f32 = sdSphere(new Vec3f(1.0, 2.0, 3.0), 1.0);
  value += sdBox(new Vec3f(1.0, 2.0, 3.0), new Vec3f(0.5, 0.5, 0.5));
  value += sdBoxFrame(new Vec3f(1.0, 2.0, 3.0), new Vec3f(0.5, 0.5, 0.5), 0.1);
  value += sdPlane(new Vec3f(1.0, 2.0, 3.0), new Vec3f(0.0, 1.0, 0.0), 0.25);
  value += sdLine(new Vec2f(1.0, 2.0), new Vec2f(0.0, 0.0), new Vec2f(2.0, 0.0));
  value += opUnion(1.0, 2.0);
  value += opSmoothUnion(1.0, 2.0, 0.5);
  res.output.set(0, value);
}
export const sdf: ComputePipelineSpec = computePipeline<Layout>(sdfKernel, { name: "sdf", workgroupSize: [1, 1, 1] });
"#,
    ));
    let generated = subscript_typegpu_gen::generate(&files)
        .unwrap_or_else(|diagnostics| panic!("generate SDF library kernel: {diagnostics:?}"));
    let wgsl = &generated.pipelines[0].1;
    for helper in [
        "sdSphere",
        "sdBox",
        "sdBoxFrame",
        "sdPlane",
        "sdLine",
        "opUnion",
        "opSmoothUnion",
    ] {
        assert!(
            wgsl.contains(&format!("fn {helper}(")),
            "missing {helper}:\n{wgsl}"
        );
    }
    validate(wgsl);
}
