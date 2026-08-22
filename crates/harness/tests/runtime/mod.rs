use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("harness crate is below repository root")
        .to_path_buf()
}

#[test]
fn wrappers_execute_their_real_host_bodies() {
    let directory =
        std::env::temp_dir().join(format!("subscript-typegpu-runtime-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("create runtime test directory");
    let program = directory.join("runtime-host.ts");
    std::fs::write(&program, r#"
import { MutStorage, PrivateVar, Rgba8unorm, Sampler, Storage, StorageTexture2d, Texture2d, Uniform, WorkgroupArray, WorkgroupVar, privateVar, storageBarrier, workgroupArray, workgroupBarrier, workgroupVar } from "./typegpu";
import { AtomicI32, AtomicU32, Vec2f, Vec2i, Vec4f } from "./typegpu-types";
export function main(): void {
  const uniform: Uniform<u32> = new Uniform<u32>(7);
  const storage: Storage<u32> = new Storage<u32>([2, 3]);
  const mutable: MutStorage<u32> = new MutStorage<u32>([4, 5]);
  mutable.set(1, 9);
  print(`runtime=${uniform.get()},${storage.get(1)},${storage.length()},${mutable.get(1)},${mutable.length()}`);
  const privateValue: PrivateVar<u32> = privateVar<u32>(6);
  privateValue.set(privateValue.get() + 1);
  const workgroupValue: WorkgroupVar<u32> = workgroupVar<u32>();
  workgroupValue.set(8);
  const workgroupValues: WorkgroupArray<u32> = workgroupArray<u32>(2);
  workgroupValues.set(0, 9);
  workgroupValues.set(1, 10);
  workgroupBarrier();
  storageBarrier();
  print(`variables=${privateValue.get()},${workgroupValue.get()},${workgroupValues.get(0)},${workgroupValues.get(1)},${workgroupValues.length()}`);
  const unsigned: AtomicU32 = new AtomicU32(10);
  const signed: AtomicI32 = new AtomicI32(-2);
  print(`atomic=${unsigned.add(5)},${unsigned.sub(3)},${unsigned.min(20)},${unsigned.max(20)},${unsigned.exchange(7)},${unsigned.load()}`);
  signed.store(4);
  print(`signed=${signed.add(-6)},${signed.min(-3)},${signed.max(9)},${signed.exchange(1)},${signed.load()}`);
  const texturePixels: Vec4f[] = [new Vec4f(1.0, 0.0, 0.0, 1.0), new Vec4f(0.0, 1.0, 0.0, 1.0)];
  const texture = new Texture2d<f32>(texturePixels, 2, 1);
  const sampler = new Sampler("nearest");
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
        b"runtime=7,3,2,9,2\nvariables=7,8,9,10,2\natomic=10,15,12,12,20,7\nsigned=4,-2,-3,9,1\ntexture=2,1,0,1,1,0,1,nearest=true\n"
    );
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
import { ComputeInvocation, ComputePipelineSpec, simulateCompute, simulateCompute2, simulateCompute3, simulateCompute4 } from "./typegpu";
class State {
  values: u32[];
  constructor() { this.values = []; }
}
function one(a: State, ctx: ComputeInvocation): void { a.values.push(ctx.globalId.x + ctx.globalId.y * 10 + ctx.globalId.z * 100); }
function two(a: State, b: State, ctx: ComputeInvocation): void { a.values.push(ctx.localIndex); b.values.push(ctx.localIndex); }
function three(a: State, b: State, c: State, ctx: ComputeInvocation): void { a.values.push(ctx.localIndex); b.values.push(ctx.localIndex); c.values.push(ctx.localIndex); }
function four(a: State, b: State, c: State, d: State, ctx: ComputeInvocation): void { a.values.push(ctx.localIndex); b.values.push(ctx.localIndex); c.values.push(ctx.localIndex); d.values.push(ctx.localIndex); }
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
    assert_eq!(result.stdout, b"order=0,111,2,113\ncounts=64,48,32,16\n");
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
