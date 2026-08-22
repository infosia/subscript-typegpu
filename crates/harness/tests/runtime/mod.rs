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
