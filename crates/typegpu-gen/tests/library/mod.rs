use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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

fn dependency_rlib(directory: &Path, crate_name: &str) -> PathBuf {
    let prefix = format!("lib{crate_name}-");
    std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&prefix) && name.ends_with(".rlib"))
        })
        .max_by_key(|path| {
            path.metadata()
                .and_then(|metadata| metadata.modified())
                .ok()
        })
        .unwrap_or_else(|| panic!("missing {crate_name} rlib in {}", directory.display()))
}

fn native_search_paths(dependencies: &Path) -> Vec<PathBuf> {
    let Some(profile) = dependencies.parent() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(profile.join("build")) else {
        return Vec::new();
    };
    let mut paths = Vec::new();
    for output in entries
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("output"))
    {
        let Ok(contents) = std::fs::read_to_string(output) else {
            continue;
        };
        paths.extend(contents.lines().filter_map(|line| {
            line.strip_prefix("cargo:rustc-link-search=native=")
                .map(PathBuf::from)
        }));
    }
    paths.sort();
    paths
}

fn run_host_fixture(runner_body: &str, fixture: &Path) -> Output {
    let dependencies = std::env::current_exe()
        .expect("current test executable")
        .parent()
        .expect("test executable has a dependency directory")
        .to_path_buf();
    let harness = dependency_rlib(&dependencies, "subscript_typegpu_harness");
    let fixture_name = fixture
        .file_stem()
        .and_then(|name| name.to_str())
        .expect("fixture has a UTF-8 file stem");
    let scratch = std::env::temp_dir().join(format!(
        "subscript-typegpu-host-{fixture_name}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&scratch)
        .unwrap_or_else(|error| panic!("create {}: {error}", scratch.display()));
    let source_path = scratch.join("runner.rs");
    let binary_path = scratch.join(if cfg!(windows) {
        "runner.exe"
    } else {
        "runner"
    });
    let mut source = std::fs::File::create(&source_path)
        .unwrap_or_else(|error| panic!("create {}: {error}", source_path.display()));
    let source_text = format!(
        r#"
use std::io::Write;
use std::path::Path;
use subscript_typegpu_harness::{{ReloadSession, facade_library, program_files, run_on_compiler_stack}};

fn main() {{
    let fixture = std::env::args_os().nth(1).expect("fixture argument");
    let output = run_on_compiler_stack(move || {{
        let files = program_files(Path::new(&fixture)).expect("load fixture");
        let libraries = [facade_library()];
        let mut session = ReloadSession::new_with_native_libraries(&files, &libraries)
            .expect("compile fixture");
{runner_body}
        session.take_output()
    }});
    std::io::stdout().write_all(&output).expect("write output");
}}
"#
    );
    source
        .write_all(source_text.as_bytes())
        .expect("write runner source");
    let mut command = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
    command
        .arg("--edition=2021")
        .arg(&source_path)
        .arg("-L")
        .arg(format!("dependency={}", dependencies.display()))
        .arg("--extern")
        .arg(format!("subscript_typegpu_harness={}", harness.display()))
        .arg("-o")
        .arg(&binary_path);
    for path in native_search_paths(&dependencies) {
        command.arg("-L").arg(format!("native={}", path.display()));
    }
    let compile = command.output().expect("compile host runner");
    assert!(
        compile.status.success(),
        "compile host runner:\n{}",
        String::from_utf8_lossy(&compile.stderr),
    );
    let run = Command::new(&binary_path)
        .arg(fixture)
        .output()
        .expect("run host fixture");
    std::fs::remove_dir_all(&scratch)
        .unwrap_or_else(|error| panic!("remove {}: {error}", scratch.display()));
    run
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
        "vec3fFrom2",
        "vec4fFrom2",
        "vec4fFrom3",
        "vec2fSplat",
        "vec3fSplat",
        "vec4fSplat",
        "vec3iFrom2",
        "vec4iFrom2",
        "vec4iFrom3",
        "vec2iSplat",
        "vec3iSplat",
        "vec4iSplat",
        "vec3uFrom2",
        "vec4uFrom2",
        "vec4uFrom3",
        "vec2uSplat",
        "vec3uSplat",
        "vec4uSplat",
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
import { sdDisk, sdBox2d, sdSphere, sdBox, sdBoxFrame, sdPlane, sdLine, opUnion, opSmoothUnion } from "./typegpu-sdf";
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { output!: MutStorage<f32>; }
function sdfKernel(res: Layout, ctx: ComputeInvocation): void {
  let value: f32 = sdDisk(new Vec2f(1.0, 2.0), new Vec2f(0.5, 0.5), 0.25);
  value += sdBox2d(new Vec2f(1.0, 2.0), new Vec2f(0.5, 0.5), new Vec2f(0.25, 0.75));
  value += sdSphere(new Vec3f(1.0, 2.0, 3.0), 1.0);
  value += sdBox(new Vec3f(1.0, 2.0, 3.0), new Vec3f(0.5, 0.5, 0.5));
  value += sdBoxFrame(new Vec3f(1.0, 2.0, 3.0), new Vec3f(0.5, 0.5, 0.5), 0.1);
  value += sdPlane(new Vec3f(1.0, 2.0, 3.0), new Vec3f(0.0, 1.0, 0.0), 0.25);
  value += sdLine(new Vec2f(1.0, 2.0), new Vec2f(0.0, 0.0), new Vec2f(2.0, 0.0));
  value += opUnion(1.0, 2.0);
  value += opSmoothUnion(1.0, 2.0, 0.5);
  res.output[0] = value;
}
export const sdf: ComputePipelineSpec = computePipeline<Layout>(sdfKernel, { name: "sdf", workgroupSize: [1, 1, 1] });
"#,
    ));
    let generated = subscript_typegpu_gen::generate(&files)
        .unwrap_or_else(|diagnostics| panic!("generate SDF library kernel: {diagnostics:?}"));
    let wgsl = &generated.pipelines[0].1;
    for helper in [
        "sdDisk",
        "sdBox2d",
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

#[test]
fn radiance_cascade_library_helpers_emit_and_validate() {
    let mut files = support::program_files(&support::root().join("programs/b01-layout.ts"));
    files.pop();
    files.push(SourceFile::new(
        "radiance-cascade-library-test.ts",
        r#"
import { Vec2f, Vec2u, Vec4f } from "./typegpu-types";
import { cascadeRaysStored, cascadeProbesAt, cascadeIntervalStart, cascadeIntervalEnd, cascadeRayAngle, cascadeMergeUv, radianceGatherUv } from "./typegpu-radiance-cascades";
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { output!: MutStorage<Vec4f>; }
function cascadeKernel(res: Layout, ctx: ComputeInvocation): void {
  const layer: u32 = ctx.globalId.x;
  const stored: u32 = cascadeRaysStored(layer);
  const probes: u32 = cascadeProbesAt(256, layer);
  const start: f32 = cascadeIntervalStart(0.01, layer);
  const end: f32 = cascadeIntervalEnd(0.01, layer);
  const angle: f32 = cascadeRayAngle(new Vec2u(1, 0), stored * 2);
  const merged: Vec2f = cascadeMergeUv(new Vec2u(1, 0), probes, new Vec2f(0.5, 0.5), 512.0);
  const gathered: Vec2f = radianceGatherUv(layer % 4, merged, 256.0, 512.0);
  res.output[layer] = new Vec4f(gathered.x, gathered.y, angle + start, end);
}
export const cascade: ComputePipelineSpec = computePipeline<Layout>(cascadeKernel, { name: "cascade", workgroupSize: [1, 1, 1] });
"#,
    ));
    let generated = subscript_typegpu_gen::generate(&files).unwrap_or_else(|diagnostics| {
        panic!("generate radiance cascade library kernel: {diagnostics:?}")
    });
    let wgsl = &generated.pipelines[0].1;
    for helper in [
        "cascadePow2",
        "cascadeRaysStored",
        "cascadeProbesAt",
        "cascadeIntervalStart",
        "cascadeIntervalEnd",
        "cascadeRayAngle",
        "cascadeMergeUv",
        "radianceGatherUv",
    ] {
        assert!(
            wgsl.contains(&format!("fn {helper}(")),
            "missing {helper}:\n{wgsl}"
        );
    }
    validate(wgsl);
}

#[test]
fn radiance_cascade_host_helpers_return_committed_dimensions_and_sides() {
    let fixture = support::root().join("crates/typegpu-gen/tests/library/radiance-cascade-host.ts");
    let run = run_host_fixture(
        r#"        session.call_main().expect("run fixture");
        while session.async_pending() != 0 {
            session.async_step().expect("step fixture");
        }"#,
        &fixture,
    );
    let stdout = String::from_utf8(run.stdout).expect("host output is UTF-8");
    assert!(run.status.success(), "host runner failed:\n{stdout}");
    for expected in [
        "cascadeDimensions 512 probes=256 dim=512 count=6",
        "cascadeDimensions 128 probes=64 dim=128 count=5",
        "cascadeWriteSide side A when count - 1 - layer is even",
    ] {
        assert!(
            stdout.contains(expected),
            "host output lacks `{expected}`:\n{stdout}"
        );
    }
    assert!(
        !stdout.contains("FAIL"),
        "host output contains FAIL:\n{stdout}"
    );
    println!("observed host:\n{}", stdout.trim_end());
}

#[test]
fn noise_library_helpers_emit_and_validate() {
    let mut files = support::program_files(&support::root().join("programs/b01-layout.ts"));
    files.pop();
    files.push(SourceFile::new(
        "noise-library-test.ts",
        r#"
import { Vec3f } from "./typegpu-types";
import { RandomF32, perlin3d, randF32, randSeed } from "./typegpu-noise";
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { output!: MutStorage<f32>; }
function noiseKernel(res: Layout, ctx: ComputeInvocation): void {
  const seed: u32 = randSeed(ctx.globalId.x + 1);
  const sample: RandomF32 = randF32(seed);
  const noise: f32 = perlin3d(new Vec3f(0.25, 0.5, 0.75));
  res.output[0] = sample.value + noise + ((sample.state & 255) as f32);
}
export const noise: ComputePipelineSpec = computePipeline<Layout>(noiseKernel, { name: "noise", workgroupSize: [1, 1, 1] });
"#,
    ));
    let generated = subscript_typegpu_gen::generate(&files)
        .unwrap_or_else(|diagnostics| panic!("generate noise library kernel: {diagnostics:?}"));
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "struct RandomF32 {",
        "fn xorU32(",
        "fn randSeed(",
        "fn randF32(",
        "fn perlin3d(",
        "var next = xorU32(state, state * 8192u);",
        "next = xorU32(next, next / 131072u);",
        "next = xorU32(next, next * 32u);",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}`:\n{wgsl}");
    }
    validate(wgsl);
}

#[test]
fn color_library_helpers_emit_and_validate() {
    let mut files = support::program_files(&support::root().join("programs/b01-layout.ts"));
    files.pop();
    files.push(SourceFile::new(
        "color-library-test.ts",
        r#"
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
import { Vec3f } from "./typegpu-types";
import { hsvToRgb, linearRgbToOklab, linearToSrgb, oklabGamutClipAdaptiveL05, oklabToLinearRgb, oklabToRgb, rgbToHsv, rgbToOklab, srgbToLinear } from "./typegpu-color";
class Layout { output!: MutStorage<Vec3f>; }
function colorKernel(res: Layout, ctx: ComputeInvocation): void {
  const input = new Vec3f(0.25, 0.5, 0.75);
  res.output[0] = linearToSrgb(input);
  res.output[1] = srgbToLinear(input);
  res.output[2] = hsvToRgb(input);
  res.output[3] = rgbToHsv(input);
  res.output[4] = linearRgbToOklab(input);
  res.output[5] = oklabToLinearRgb(input);
  res.output[6] = rgbToOklab(input);
  res.output[7] = oklabToRgb(input);
  res.output[8] = oklabGamutClipAdaptiveL05(input, 0.2);
}
export const color: ComputePipelineSpec = computePipeline<Layout>(colorKernel, { name: "color", workgroupSize: [1, 1, 1] });
"#,
    ));
    let generated = subscript_typegpu_gen::generate(&files)
        .unwrap_or_else(|diagnostics| panic!("generate color library kernel: {diagnostics:?}"));
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "fn linearToSrgb(",
        "fn srgbToLinear(",
        "fn hsvToRgb(",
        "fn rgbToHsv(",
        "fn linearRgbToOklab(",
        "fn oklabToLinearRgb(",
        "fn rgbToOklab(",
        "fn oklabToRgb(",
        "fn oklabGamutClipAdaptiveL05(",
        "fn computeMaxSaturation(",
        "fn findCusp(",
        "fn findGamutIntersection(",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}`:\n{wgsl}");
    }
    validate(wgsl);
}

#[test]
fn sort_library_kernels_emit_and_validate_from_imports() {
    let mut files = support::program_files(&support::root().join("programs/b01-layout.ts"));
    files.pop();
    files.push(SourceFile::new(
        "sort-library-test.ts",
        r#"
import { ComputePipelineSpec, computePipeline } from "./typegpu";
import { BitonicSortResources, PrefixScanApplyResources, PrefixScanBlockResources, bitonicSortStep, prefixScanApplyF32, prefixScanBlockF32 } from "./typegpu-sort";
export const bitonic: ComputePipelineSpec = computePipeline<BitonicSortResources>(bitonicSortStep, { name: "bitonic", workgroupSize: [256, 1, 1] });
export const scanBlock: ComputePipelineSpec = computePipeline<PrefixScanBlockResources>(prefixScanBlockF32, { name: "scanBlock", workgroupSize: [256, 1, 1] });
export const scanApply: ComputePipelineSpec = computePipeline<PrefixScanApplyResources>(prefixScanApplyF32, { name: "scanApply", workgroupSize: [256, 1, 1] });
"#,
    ));
    let generated = subscript_typegpu_gen::generate(&files)
        .unwrap_or_else(|diagnostics| panic!("generate imported sort kernels: {diagnostics:?}"));
    assert_eq!(generated.pipelines.len(), 3);
    let pipeline = |name: &str| {
        generated
            .pipelines
            .iter()
            .find(|(pipeline, _)| pipeline == name)
            .map(|(_, wgsl)| wgsl.as_str())
            .unwrap_or_else(|| panic!("missing generated pipeline {name}"))
    };
    let bitonic = pipeline("bitonic");
    assert!(bitonic.contains("fn bitonicSortStride("));
    assert!(bitonic.contains("fn bitonicSortStep("));
    assert!(bitonic.contains("let below = thread & (stride - 1u);"));
    let block = pipeline("scanBlock");
    assert!(block.contains("var<workgroup> prefixScanShared: array<f32, 256u>;"));
    assert!(block.contains("fn prefixScanBlockF32("));
    let apply = pipeline("scanApply");
    assert!(apply.contains("fn prefixScanApplyF32("));
    for (_, wgsl) in &generated.pipelines {
        validate(wgsl);
    }
}

#[test]
fn bitonic_non_power_of_two_length_has_the_named_red_trap() {
    let fixture = support::root().join("crates/typegpu-gen/tests/library/bitonic-sort-trap.ts");
    let run = run_host_fixture(
        r#"        let mut trapped = session.call_main().is_err();
        while !trapped && session.async_pending() != 0 {
            trapped = session.async_step().is_err();
        }
        if !trapped {
            std::io::stdout()
                .write_all(&session.take_output())
                .expect("write output");
            panic!("fixture unexpectedly passed");
        }"#,
        &fixture,
    );
    let stdout = String::from_utf8(run.stdout).expect("trap output is UTF-8");
    assert!(run.status.success(), "trap runner failed:\n{stdout}");
    assert!(
        stdout.contains("SORT1 bitonicSortPassCount length=3 is not a power of two (author)"),
        "trap output lacks SORT1 message:\n{stdout}",
    );
    println!("observed red: {}", stdout.trim_end());
}
