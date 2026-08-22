fn run(source: &str) -> Vec<u8> {
    let directory =
        std::env::temp_dir().join(format!("subscript-typegpu-library-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("create library test directory");
    let program = directory.join("library-host.ts");
    std::fs::write(&program, source).expect("write library test program");
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_subscript-typegpu-harness"))
        .arg("dev")
        .arg(&program)
        .output()
        .expect("spawn library test program");
    assert!(
        result.status.success(),
        "library test program: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let output = result.stdout;
    std::fs::remove_file(&program).expect("remove library test program");
    std::fs::remove_dir(&directory).expect("remove library test directory");
    output
}

#[test]
fn every_k11_free_function_matches_its_host_definition() {
    let output = run(r#"
import { clamp, fract, mix, sign, smoothstep, step } from "./typegpu-types";
export function main(): void {
  print(`clamp=${clamp(2.0, 0.0, 1.0)}`);
  print(`mix=${mix(0.0, 10.0, 0.5)}`);
  print(`step=${step(1.0, 2.0)}`);
  print(`smoothstep=${smoothstep(0.0, 1.0, 0.5)}`);
  print(`fract=${fract(1.25)}`);
  print(`sign=${sign(-2.0)}`);
}
"#);
    assert_eq!(
        output,
        b"clamp=1\nmix=5\nstep=1\nsmoothstep=0.5\nfract=0.25\nsign=-1\n"
    );
}
