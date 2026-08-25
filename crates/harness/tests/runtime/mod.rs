use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use subscript_typegpu_harness::native::{
    SubscriptTypegpuBindGroupLayout, SubscriptTypegpuBindGroupLayoutDescriptor,
    SubscriptTypegpuBuffer, SubscriptTypegpuBufferDescriptor, SubscriptTypegpuComputePipeline,
    SubscriptTypegpuComputePipelineDescriptor, SubscriptTypegpuDevice, SubscriptTypegpuInstance,
    SubscriptTypegpuPipelineLayout, SubscriptTypegpuPipelineLayoutDescriptor,
    SubscriptTypegpuQueue, SubscriptTypegpuShaderModule, SubscriptTypegpuShaderModuleDescriptor,
};

static TEST_QUEUE_RELEASED: AtomicBool = AtomicBool::new(false);
static TEST_QUEUE_WRITES: AtomicU32 = AtomicU32::new(0);

extern "C" fn test_instance() -> SubscriptTypegpuInstance {
    1_usize as SubscriptTypegpuInstance
}

extern "C" fn test_device() -> SubscriptTypegpuDevice {
    2_usize as SubscriptTypegpuDevice
}

extern "C" fn test_queue_writes() -> u32 {
    TEST_QUEUE_WRITES.load(Ordering::SeqCst)
}

extern "C" fn test_device_get_queue(_: SubscriptTypegpuDevice) -> SubscriptTypegpuQueue {
    3_usize as SubscriptTypegpuQueue
}

extern "C" fn test_device_create_buffer(
    _: SubscriptTypegpuDevice,
    _: *const SubscriptTypegpuBufferDescriptor,
) -> SubscriptTypegpuBuffer {
    4_usize as SubscriptTypegpuBuffer
}

extern "C" fn test_device_create_bind_group_layout(
    _: SubscriptTypegpuDevice,
    _: *const SubscriptTypegpuBindGroupLayoutDescriptor,
) -> SubscriptTypegpuBindGroupLayout {
    5_usize as SubscriptTypegpuBindGroupLayout
}

extern "C" fn test_device_create_pipeline_layout(
    _: SubscriptTypegpuDevice,
    _: *const SubscriptTypegpuPipelineLayoutDescriptor,
) -> SubscriptTypegpuPipelineLayout {
    6_usize as SubscriptTypegpuPipelineLayout
}

extern "C" fn test_device_create_shader_module(
    _: SubscriptTypegpuDevice,
    _: *const SubscriptTypegpuShaderModuleDescriptor,
) -> SubscriptTypegpuShaderModule {
    7_usize as SubscriptTypegpuShaderModule
}

extern "C" fn test_device_create_compute_pipeline(
    _: SubscriptTypegpuDevice,
    _: *const SubscriptTypegpuComputePipelineDescriptor,
) -> SubscriptTypegpuComputePipeline {
    8_usize as SubscriptTypegpuComputePipeline
}

extern "C" fn test_queue_write_buffer(
    _: SubscriptTypegpuQueue,
    _: SubscriptTypegpuBuffer,
    _: u64,
    _: usize,
    _: *const u8,
) {
    if !TEST_QUEUE_RELEASED.load(Ordering::SeqCst) {
        TEST_QUEUE_WRITES.fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn test_queue_release(_: SubscriptTypegpuQueue) {
    TEST_QUEUE_RELEASED.store(true, Ordering::SeqCst);
}

extern "C" fn test_buffer_release(_: SubscriptTypegpuBuffer) {}

extern "C" fn test_bind_group_layout_release(_: SubscriptTypegpuBindGroupLayout) {}

extern "C" fn test_pipeline_layout_release(_: SubscriptTypegpuPipelineLayout) {}

extern "C" fn test_shader_module_release(_: SubscriptTypegpuShaderModule) {}

extern "C" fn test_compute_pipeline_release(_: SubscriptTypegpuComputePipeline) {}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("harness crate is below repository root")
        .to_path_buf()
}

fn replace_symbol(symbols: &mut [(String, *const u8)], name: &str, address: *const u8) {
    let (_, current) = symbols
        .iter_mut()
        .find(|(candidate, _)| candidate == name)
        .unwrap_or_else(|| panic!("facade symbol table lacks {name}"));
    *current = address;
}

#[test]
fn owned_device_queue_survives_guarded_pipeline_disposal() {
    TEST_QUEUE_RELEASED.store(false, Ordering::SeqCst);
    TEST_QUEUE_WRITES.store(0, Ordering::SeqCst);
    let directory = std::env::temp_dir().join(format!(
        "subscript-typegpu-queue-ownership-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).expect("create queue ownership test directory");
    let program = directory.join("queue-ownership.ts");
    let source = r#"
import { BindGroupLayoutSpec, createComputePipeline } from "./typegpu";
import { GPUBufferUsage, GPUDevice, GPUShaderStage } from "./webgpu";
export function main(): void {
  const device = new GPUDevice(test_instance(), test_device());
  const target = device.createBuffer({ size: 4, usage: GPUBufferUsage.COPY_DST });
  const guardLayout: BindGroupLayoutSpec = {
    entries: [{ binding: 0, visibility: GPUShaderStage.COMPUTE, kind: "guard", minBindingSize: 16 }],
  };
  const pipeline = createComputePipeline(
    device,
    "@compute @workgroup_size(1) fn main() {}",
    "main",
    [guardLayout],
    [1, 1, 1],
  );
  pipeline.dispose();
  device.queue.writeBuffer(target, 0, [1, 0, 0, 0]);
  print(`writes=${test_queue_writes()}`);
}
"#;
    std::fs::write(&program, "export function main(): void {}\n")
        .expect("write queue ownership test placeholder");
    let mut files =
        subscript_typegpu_harness::program_files(&program).expect("load queue ownership program");
    files
        .last_mut()
        .expect("queue ownership program file")
        .source = source.to_owned();
    files.push(subscript_compiler::SourceFile::ambient(
        "queue-ownership.d.ts",
        "// @subscript-c-header include=\"queue-ownership.h\"\n\
         declare function test_instance(): SubscriptTypegpuInstance;\n\
         declare function test_device(): SubscriptTypegpuDevice;\n\
         declare function test_queue_writes(): u32;\n",
    ));
    let mut symbols = subscript_typegpu_harness::native_symbols_generated::facade_symbols();
    for (name, address) in [
        (
            "subscript_typegpu_create_instance",
            test_instance as *const u8,
        ),
        (
            "subscript_typegpu_device_get_queue",
            test_device_get_queue as *const u8,
        ),
        (
            "subscript_typegpu_device_create_buffer",
            test_device_create_buffer as *const u8,
        ),
        (
            "subscript_typegpu_device_create_bind_group_layout",
            test_device_create_bind_group_layout as *const u8,
        ),
        (
            "subscript_typegpu_device_create_pipeline_layout",
            test_device_create_pipeline_layout as *const u8,
        ),
        (
            "subscript_typegpu_device_create_shader_module",
            test_device_create_shader_module as *const u8,
        ),
        (
            "subscript_typegpu_device_create_compute_pipeline",
            test_device_create_compute_pipeline as *const u8,
        ),
        (
            "subscript_typegpu_queue_write_buffer",
            test_queue_write_buffer as *const u8,
        ),
        (
            "subscript_typegpu_queue_release",
            test_queue_release as *const u8,
        ),
        (
            "subscript_typegpu_buffer_release",
            test_buffer_release as *const u8,
        ),
        (
            "subscript_typegpu_bind_group_layout_release",
            test_bind_group_layout_release as *const u8,
        ),
        (
            "subscript_typegpu_pipeline_layout_release",
            test_pipeline_layout_release as *const u8,
        ),
        (
            "subscript_typegpu_shader_module_release",
            test_shader_module_release as *const u8,
        ),
        (
            "subscript_typegpu_compute_pipeline_release",
            test_compute_pipeline_release as *const u8,
        ),
    ] {
        replace_symbol(&mut symbols, name, address);
    }
    symbols.extend([
        ("test_instance".to_owned(), test_instance as *const u8),
        ("test_device".to_owned(), test_device as *const u8),
        (
            "test_queue_writes".to_owned(),
            test_queue_writes as *const u8,
        ),
    ]);
    // SAFETY: each function has static lifetime and matches its ambient C signature.
    let library = unsafe { subscript_codegen::NativeLibrary::new(Vec::new(), Vec::new(), symbols) };
    let mut session =
        subscript_codegen::ReloadSession::new_with_native_libraries(&files, &[library])
            .expect("compile queue ownership program");
    session.call_main().expect("run queue ownership program");
    let output = String::from_utf8(session.take_output()).expect("UTF-8 queue ownership output");
    std::fs::remove_file(&program).expect("remove queue ownership test program");
    std::fs::remove_dir(&directory).expect("remove queue ownership test directory");
    assert_eq!(
        output, "writes=1\n",
        "queue write did not reach the live buffer: {output}"
    );
}

#[test]
fn wrappers_execute_their_real_host_bodies() {
    let directory =
        std::env::temp_dir().join(format!("subscript-typegpu-runtime-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("create runtime test directory");
    let program = directory.join("runtime-host.ts");
    std::fs::write(&program, r#"
import { MutStorage, PrivateVar, Rgba8unorm, Sampler, Storage, StorageTexture2d, Texture2d, Uniform, WorkgroupArray, WorkgroupVar, privateVar, samplerFromDescriptor, storageBarrier, workgroupArray, workgroupBarrier, workgroupVar } from "./typegpu";
import { GPUSamplerDescriptor } from "./webgpu";
import { AtomicI32, AtomicU32, Vec2f, Vec2i, Vec4f } from "./typegpu-types";
export function main(): void {
  const uniform: Uniform<u32> = new Uniform<u32>(7);
  const storage: Storage<u32> = new Storage<u32>([2, 3]);
  const mutable: MutStorage<u32> = new MutStorage<u32>([4, 5]);
  mutable[0] = storage[0];
  mutable.set(1, 9);
  print(`runtime=${uniform.$},${storage.get(1)},${storage.length()},${mutable.get(1)},${mutable.length()},${mutable[0]}`);
  const privateValue: PrivateVar<u32> = privateVar<u32>(6);
  privateValue.$ = privateValue.$ + 1;
  const workgroupValue: WorkgroupVar<u32> = workgroupVar<u32>();
  workgroupValue.$ = 8;
  const workgroupValues: WorkgroupArray<u32> = workgroupArray<u32>(2);
  workgroupValues.set(0, 9);
  workgroupValues.set(1, 10);
  workgroupValues[0] = 11;
  const indexedWorkgroupValue: u32 = workgroupValues[0];
  workgroupBarrier();
  storageBarrier();
  print(`variables=${privateValue.$},${workgroupValue.$},${workgroupValues.get(0)},${workgroupValues.get(1)},${workgroupValues.length()},${indexedWorkgroupValue}`);
  const unsigned: AtomicU32 = new AtomicU32(10);
  const signed: AtomicI32 = new AtomicI32(-2);
  print(`atomic=${unsigned.add(5)},${unsigned.sub(3)},${unsigned.min(20)},${unsigned.max(20)},${unsigned.exchange(7)},${unsigned.load()}`);
  signed.store(4);
  print(`signed=${signed.add(-6)},${signed.min(-3)},${signed.max(9)},${signed.exchange(1)},${signed.load()}`);
  const texturePixels: Vec4f[] = [new Vec4f(1.0, 0.0, 0.0, 1.0), new Vec4f(0.0, 1.0, 0.0, 1.0)];
  const texture = new Texture2d<f32>(texturePixels, 2, 1);
  const samplerDescriptor: GPUSamplerDescriptor = { minFilter: "nearest", magFilter: "nearest" };
  const sampler = samplerFromDescriptor(samplerDescriptor);
  const dimensions = texture.dimensions();
  const loaded = texture.load(new Vec2i(1, 0), 0);
  const sampled = texture.sample(sampler, new Vec2f(0.25, 0.5));
  const storagePixels: Vec4f[] = [new Vec4f(0.0, 0.0, 0.0, 0.0)];
  const storage = new StorageTexture2d<Rgba8unorm>(storagePixels, 1, 1);
  storage.store(new Vec2i(0, 0), sampled);
  print(`texture=${dimensions.x},${dimensions.y},${loaded.x},${loaded.y},${sampled.x},${sampled.y},${storagePixels[0].x},nearest=${sampler.isNearest()}`);
}

"#).expect("write runtime test program");
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_subscript-typegpu-harness"))
        .arg("dev")
        .arg(&program)
        .output()
        .expect("spawn runtime test program");
    assert!(
        result.status.success(),
        "runtime test program: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let output = result.stdout;
    std::fs::remove_file(&program).expect("remove runtime test program");
    std::fs::remove_dir(&directory).expect("remove runtime test directory");
    assert_eq!(
        output,
        b"runtime=7,3,2,9,2,2\nvariables=7,8,11,10,2,11\natomic=10,15,12,12,20,7\nsigned=4,-2,-3,9,1\ntexture=2,1,0,1,1,0,1,nearest=true\n"
    );
}

#[test]
fn unsupported_host_texture_sampling_traps_at_each_method() {
    let directory = std::env::temp_dir().join(format!(
        "subscript-typegpu-sampler-trap-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).expect("create sampler trap directory");
    let program = directory.join("sampler-trap.ts");
    for (filters, call, expected) in [
        (
            "linear",
            "texture.sampleLevel(sampler, new Vec2f(0.5, 0.5), 0.0)",
            "TX3 sampleLevel filterMode is not nearest",
        ),
        (
            "linear",
            "texture.sample(sampler, new Vec2f(0.5, 0.5))",
            "TX3 sample filterMode is not nearest",
        ),
        (
            "nearest",
            "texture.load(new Vec2i(0, 0), 1)",
            "TX3 load level=1 is not supported",
        ),
        (
            "nearest",
            "texture.sampleLevel(sampler, new Vec2f(0.5, 0.5), 1.0)",
            "TX3 sampleLevel level=1 is not supported",
        ),
    ] {
        let source = r#"
import { Texture2d, samplerFromDescriptor } from "./typegpu";
import { Vec2f, Vec2i, Vec4f } from "./typegpu-types";
import { GPUSamplerDescriptor } from "./webgpu";
export function main(): void {
  const descriptor: GPUSamplerDescriptor = { minFilter: "$FILTERS", magFilter: "$FILTERS" };
  const sampler = samplerFromDescriptor(descriptor);
  const texture = new Texture2d<f32>([new Vec4f(1.0, 0.0, 0.0, 1.0)], 1, 1);
  $CALL;
}
"#
        .replace("$FILTERS", filters)
        .replace("$CALL", call);
        std::fs::write(&program, source).expect("write sampler trap program");
        let files =
            subscript_typegpu_harness::program_files(&program).expect("load sampler trap program");
        let libraries = [subscript_typegpu_harness::facade_library()];
        let mut session =
            subscript_codegen::ReloadSession::new_with_native_libraries(&files, &libraries)
                .expect("compile sampler trap program");
        assert!(session.call_main().is_err(), "{call} did not trap");
        let output = String::from_utf8(session.take_output()).expect("UTF-8 sampler trap output");
        assert!(
            output.contains(expected),
            "{call} lacks `{expected}`:\n{output}"
        );
    }
    std::fs::remove_file(&program).expect("remove sampler trap program");
    std::fs::remove_dir(&directory).expect("remove sampler trap directory");
}

#[test]
fn simulate_compute_forms_run_every_invocation_in_row_major_order() {
    let directory =
        std::env::temp_dir().join(format!("subscript-typegpu-simulate-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("create simulate test directory");
    let program = directory.join("simulate-host.ts");
    std::fs::write(
        &program,
        r#"
import { ComputeInvocation, ComputePipelineSpec, simulateCompute, simulateCompute2, simulateCompute3, simulateCompute4, simulateComputeThreads } from "./typegpu";
class State {
  values: u32[];
  constructor() { this.values = []; }
}
class BuiltinState {
  count: u32;
  failures: u32;
  constructor() { this.count = 0; this.failures = 0; }
}
function one(a: State, ctx: ComputeInvocation): void { a.values.push(ctx.globalId.x + ctx.globalId.y * 10 + ctx.globalId.z * 100); }
function two(a: State, b: State, ctx: ComputeInvocation): void { a.values.push(ctx.localIndex); b.values.push(ctx.localIndex); }
function three(a: State, b: State, c: State, ctx: ComputeInvocation): void { a.values.push(ctx.localIndex); b.values.push(ctx.localIndex); c.values.push(ctx.localIndex); }
function four(a: State, b: State, c: State, d: State, ctx: ComputeInvocation): void { a.values.push(ctx.localIndex); b.values.push(ctx.localIndex); c.values.push(ctx.localIndex); d.values.push(ctx.localIndex); }
function builtins(state: BuiltinState, ctx: ComputeInvocation): void {
  const expectedIndex: u32 = (ctx.localId.z * 2 + ctx.localId.y) * 4 + ctx.localId.x;
  if (ctx.localIndex !== expectedIndex) state.failures += 1;
  if (ctx.workgroupId.x !== ctx.globalId.x / 4) state.failures += 1;
  if (ctx.workgroupId.y !== ctx.globalId.y / 2) state.failures += 1;
  if (ctx.workgroupId.z !== ctx.globalId.z / 2) state.failures += 1;
  if (ctx.localId.x !== ctx.globalId.x % 4) state.failures += 1;
  if (ctx.localId.y !== ctx.globalId.y % 2) state.failures += 1;
  if (ctx.localId.z !== ctx.globalId.z % 2) state.failures += 1;
  if (ctx.numWorkgroups.x !== 2 || ctx.numWorkgroups.y !== 2 || ctx.numWorkgroups.z !== 1) state.failures += 1;
  state.count += 1;
}
export function main(): void {
  const spec: ComputePipelineSpec = { name: "runtime", workgroupSize: [2, 2, 2] };
  const a = new State();
  const b = new State();
  const c = new State();
  const d = new State();
  simulateCompute<State>(one, a, spec, [2, 1, 1], true);
  print(`order=${a.values[0]},${a.values[7]},${a.values[8]},${a.values[15]}`);
  simulateCompute2<State, State>(two, a, b, spec, [2, 1, 1], true);
  simulateCompute3<State, State, State>(three, a, b, c, spec, [2, 1, 1], true);
  simulateCompute4<State, State, State, State>(four, a, b, c, d, spec, [2, 1, 1], true);
  print(`counts=${a.values.length},${b.values.length},${c.values.length},${d.values.length}`);
  const builtinSpec: ComputePipelineSpec = { name: "builtins", workgroupSize: [4, 2, 2] };
  const builtinState = new BuiltinState();
  simulateComputeThreads<BuiltinState>(builtins, builtinState, builtinSpec, 8, 4, 2, true);
  print(`builtins=${builtinState.count},${builtinState.failures}`);
}
"#,
    )
    .expect("write simulate test program");
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_subscript-typegpu-harness"))
        .arg("dev")
        .arg(&program)
        .output()
        .expect("spawn simulate test program");
    assert!(
        result.status.success(),
        "simulate test program: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    std::fs::remove_file(&program).expect("remove simulate test program");
    std::fs::remove_dir(&directory).expect("remove simulate test directory");
    assert_eq!(
        result.stdout,
        b"order=0,111,2,113\ncounts=64,48,32,16\nbuiltins=64,0\n"
    );
}

#[test]
fn dispatch_threads_rounds_all_three_axes() {
    let source =
        std::fs::read_to_string(root().join("lib/typegpu.ts")).expect("read TypeGPU runtime");
    for (axis, name) in [(0, "x"), (1, "y"), (2, "z")] {
        assert!(
            source.contains(&format!(
                "({name} + this.workgroup[{axis}] - 1) / this.workgroup[{axis}]"
            )),
            "dispatchThreads lacks workgroup axis {axis}"
        );
    }
    assert!(
        source.contains(
            "\"ComputePipeline.dispatchTimed\",\n      x * this.workgroup[0],\n      y * this.workgroup[1],\n      z * this.workgroup[2],"
        ),
        "dispatchTimed does not write three-axis guard bounds",
    );
}

#[test]
fn timestamp_pair_has_explicit_query_resolution() {
    let source =
        std::fs::read_to_string(root().join("lib/typegpu.ts")).expect("read TypeGPU runtime");
    for required in [
        "device.hasFeature(\"timestamp-query\")",
        "type: \"timestamp\"",
        "count: 2",
        "GPUBufferUsage.QUERY_RESOLVE + GPUBufferUsage.COPY_SRC",
        "beginningOfPassWriteIndex: 0",
        "endOfPassWriteIndex: 1",
        "resolve(encoder: GPUCommandEncoder): void",
        "encoder.resolveQuerySet(this.queries, 0, 2, this.resolved, 0)",
        "encoder.copyBufferToBuffer(this.resolved, 0, readback.handle(), 0, 16)",
    ] {
        assert!(
            source.contains(required),
            "timestamp runtime lacks `{required}`"
        );
    }
}

#[test]
fn render_declaration_helpers_execute_their_real_host_bodies() {
    let directory = std::env::temp_dir().join(format!(
        "subscript-typegpu-render-runtime-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).expect("create render runtime test directory");
    let program = directory.join("render-runtime-host.ts");
    std::fs::write(
        &program,
        r#"
import { FragmentInvocation, renderPipeline, renderPipelineInstanced, renderPipelineL, RenderPipelineSpec, VertexInvocation } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
@CStruct class Vertex { position: Vec2f; constructor(position: Vec2f) { this.position = position; } }
@CStruct class Instance { offset: Vec2f; constructor(offset: Vec2f) { this.offset = offset; } }
@CStruct class Varyings { position: Vec4f; constructor(position: Vec4f) { this.position = position; } }
class Layout {}
function vert(value: Vertex, ctx: VertexInvocation): Varyings { return new Varyings(new Vec4f(0.0, 0.0, 0.0, 1.0)); }
function vertL(res: Layout, value: Vertex, ctx: VertexInvocation): Varyings { return vert(value, ctx); }
function vertI(value: Vertex, instance: Instance, ctx: VertexInvocation): Varyings { return vert(value, ctx); }
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
function fragL(res: Layout, input: Varyings, ctx: FragmentInvocation): Vec4f { return input.position; }
export function main(): void {
  const plain: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, { format: "rgba8unorm" });
  const layout: RenderPipelineSpec = renderPipelineL<Layout, Vertex, Varyings>(vertL, fragL, { format: "bgra8unorm", topology: "line-list" });
  const instanced: RenderPipelineSpec = renderPipelineInstanced<Vertex, Instance, Varyings>(vertI, frag, { format: "rgba16float", cullMode: "back", frontFace: "cw" });
  print(`${plain.format},${plain.topology},${plain.cullMode},${plain.frontFace}`);
  print(`${layout.format},${layout.topology}`);
  print(`${instanced.format},${instanced.cullMode},${instanced.frontFace}`);
}
"#,
    )
    .expect("write render runtime test program");
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_subscript-typegpu-harness"))
        .arg("dev")
        .arg(&program)
        .output()
        .expect("spawn render runtime test program");
    assert!(
        result.status.success(),
        "render runtime test program: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    std::fs::remove_file(&program).expect("remove render runtime test program");
    std::fs::remove_dir(&directory).expect("remove render runtime test directory");
    assert_eq!(
        result.stdout,
        b"rgba8unorm,triangle-list,none,ccw\nbgra8unorm,line-list\nrgba16float,back,cw\n"
    );
}
