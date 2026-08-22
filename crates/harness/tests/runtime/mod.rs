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
import { MutStorage, Storage, Uniform } from "./typegpu";
export function main(): void {
  const uniform: Uniform<u32> = new Uniform<u32>(7);
  const storage: Storage<u32> = new Storage<u32>([2, 3]);
  const mutable: MutStorage<u32> = new MutStorage<u32>([4, 5]);
  mutable.set(1, 9);
  print(`runtime=${uniform.get()},${storage.get(1)},${storage.length()},${mutable.get(1)},${mutable.length()}`);
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
    assert_eq!(output, b"runtime=7,3,2,9,2\n");
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
