use crate::support;

#[test]
fn typed_and_positional_bind_group_paths_share_layout_and_binding_order() {
    let program = support::root().join("programs/b02-vecadd.ts");
    let generated = subscript_typegpu_gen::generate(&support::program_files(&program))
        .expect("generate b02 typed resources");
    let source = &generated.support_module;
    for expected in [
        "@Descriptor\nexport class VecAddLayoutResources",
        "a!: GPUBuffer;",
        "b!: GPUBuffer;",
        "out!: GPUBuffer;",
        "export function createVecAddLayoutResources(",
        "export function createVecAddBindGroup0<D extends GPUDeviceCreation>(",
        "using layout = pipeline.bindGroupLayout(0);",
        "return createBindGroup(device, layout, vecAdd_LAYOUT0, [",
    ] {
        assert!(
            source.contains(expected),
            "missing `{expected}` in:\n{source}"
        );
    }

    let typed_order = [
        "bufferResource(resources.a)",
        "bufferResource(resources.b)",
        "bufferResource(resources.out)",
    ];
    let mut previous = 0;
    for expression in typed_order {
        let offset = source[previous..]
            .find(expression)
            .unwrap_or_else(|| panic!("typed factory lacks `{expression}`:\n{source}"))
            + previous;
        previous = offset + expression.len();
    }
    for binding in 0..3 {
        assert!(
            source.contains(&format!("{{ binding: {binding},")),
            "positional layout lacks binding {binding}:\n{source}"
        );
    }
}

#[test]
fn creation_helpers_accept_each_declared_device_class() {
    use subscript_compiler::SourceFile;

    let source = r#"
import { GPUDevice, GPUHostOwnedDevice } from "./webgpu";
import { Buffer, createBuffer, createComputePipeline, createRenderPipeline, createBindGroup, createTimestampPair } from "./typegpu";
import { UiRenderer, UiPipelineFacts } from "./typegpu-ui";
function owned(device: GPUDevice, facts: UiPipelineFacts): void {
  using buffer: Buffer<u32> = createBuffer(device, 4, 1, 8, "owned");
  using compute = createComputePipeline(device, "", "main", [], [1, 1, 1]);
  using render = createRenderPipeline(device, "", "vertex", "fragment", [], [], facts.spec);
  using layout = compute.bindGroupLayout(0);
  using group = createBindGroup(device, layout, { entries: [] }, []);
  const timestamps = createTimestampPair(device);
  timestamps?.dispose();
  using renderer = UiRenderer.create(device, facts, 1);
}
function host(device: GPUHostOwnedDevice, facts: UiPipelineFacts): void {
  using buffer: Buffer<u32> = createBuffer(device, 4, 1, 8, "host");
  using compute = createComputePipeline(device, "", "main", [], [1, 1, 1]);
  using render = createRenderPipeline(device, "", "vertex", "fragment", [], [], facts.spec);
  using layout = compute.bindGroupLayout(0);
  using group = createBindGroup(device, layout, { entries: [] }, []);
  const timestamps = createTimestampPair(device);
  timestamps?.dispose();
  using renderer = UiRenderer.create(device, facts, 1);
}
export function main(): void {}
"#;
    let files = support::source_files(SourceFile::entry("device-creation.ts", source));
    subscript_compiler::check_program(&files).unwrap_or_else(|diagnostics| {
        panic!(
            "{}",
            subscript_compiler::render_diagnostics(&files, &diagnostics)
        )
    });
}

#[test]
fn a_class_without_the_implements_clause_fails_the_device_constraint() {
    use subscript_compiler::SourceFile;

    let source = r#"
import { createBuffer } from "./typegpu";
class Device {}
export function main(): void {
  const buffer = createBuffer<u32, Device>(new Device(), 4, 1, 8, "invalid");
}
"#;
    let files = support::source_files(SourceFile::entry("device-constraint-red.ts", source));
    let diagnostics =
        subscript_compiler::check_program(&files).expect_err("undeclared constraint must fail");
    let text = subscript_compiler::render_diagnostics(&files, &diagnostics);
    eprintln!("{text}");
    assert_eq!(diagnostics.len(), 1, "{text}");
    assert!(
        text.contains("does not declare `implements GPUDeviceCreation`"),
        "{text}"
    );
}

#[test]
fn generated_bind_group_factories_accept_each_device_class() {
    use subscript_compiler::SourceFile;

    let program = support::root().join("programs/b02-vecadd.ts");
    let generated = subscript_typegpu_gen::generate(&support::program_files(&program))
        .expect("generate typed resources");
    let render = subscript_typegpu_gen::generate(&support::program_files(
        &support::root().join("programs/b08-render-bindings.ts"),
    ))
    .expect("generate render resources");
    let source = r#"
import { GPUDevice, GPUHostOwnedDevice, GPUBuffer } from "./webgpu";
import { ComputePipeline, RenderPipeline } from "./typegpu";
import { createShiftedBindGroup0 } from "./b08-render-bindings.typegpu";
import { createVecAddBindGroup0 } from "./b02-vecadd.typegpu";
function owned(device: GPUDevice, pipeline: ComputePipeline, buffer: GPUBuffer, render: RenderPipeline): void {
  using rendered = createShiftedBindGroup0(device, render, { params: buffer, tint: buffer });
  using group = createVecAddBindGroup0(device, pipeline, { a: buffer, b: buffer, out: buffer });
}
function host(device: GPUHostOwnedDevice, pipeline: ComputePipeline, buffer: GPUBuffer, render: RenderPipeline): void {
  using rendered = createShiftedBindGroup0(device, render, { params: buffer, tint: buffer });
  using group = createVecAddBindGroup0(device, pipeline, { a: buffer, b: buffer, out: buffer });
}
export function main(): void {}
"#;
    let mut files = support::source_files(SourceFile::entry("device-factories.ts", source));
    let entry = files.pop().expect("entry file");
    files.push(SourceFile::new(
        "b02-vecadd.typegpu.ts",
        generated.support_module,
    ));
    files.push(SourceFile::new(
        "b08-render-bindings.typegpu.ts",
        render.support_module,
    ));
    files.push(entry);
    subscript_compiler::check_program(&files).unwrap_or_else(|diagnostics| {
        panic!(
            "{}",
            subscript_compiler::render_diagnostics(&files, &diagnostics)
        )
    });
}

#[test]
fn device_constraint_accepts_defaulted_methods_and_contextual_descriptors() {
    use subscript_compiler::SourceFile;

    let source = r#"
import { GPUDevice, GPUHostOwnedDevice, GPUDeviceCreation, GPUSampler } from "./webgpu";
function sampler<D extends GPUDeviceCreation>(device: D): GPUSampler {
  using defaultSampler = device.createSampler(null);
  return device.createSampler({ minFilter: "nearest", magFilter: "nearest" });
}
function owned(device: GPUDevice): void {
  using directDefault = device.createSampler();
  using constrained = sampler(device);
}
function host(device: GPUHostOwnedDevice): void {
  using directDefault = device.createSampler();
  using constrained = sampler(device);
}
export function main(): void {}
"#;
    let files = support::source_files(SourceFile::entry("device-signatures.ts", source));
    subscript_compiler::check_program(&files).unwrap_or_else(|diagnostics| {
        panic!(
            "{}",
            subscript_compiler::render_diagnostics(&files, &diagnostics)
        )
    });
}
