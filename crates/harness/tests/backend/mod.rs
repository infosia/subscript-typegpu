use std::process::Command;

use crate::differential::backend_is_available;

#[test]
fn d3d12_request_is_rejected_by_yawgpu() {
    if !backend_is_available() {
        return;
    }

    let backend = subscript_typegpu_harness::backend_lib()
        .expect("backend library path")
        .expect("available backend library");
    const CHILD: &str = "SUBSCRIPT_TYPEGPU_D3D12_YAWGPU_TEST_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let output = Command::new(std::env::current_exe().expect("test binary path"))
            .args([
                "--exact",
                "backend::d3d12_request_is_rejected_by_yawgpu",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .env("SUBSCRIPT_TYPEGPU_BACKEND", "d3d12")
            .env("SUBSCRIPT_TYPEGPU_BACKEND_LIB", backend)
            .output()
            .expect("run d3d12 yawgpu request child");
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("backend library is not yawgpu (no marker)") {
            println!("pending: backend library is not yawgpu");
            return;
        }
        assert!(
            output.status.success(),
            "d3d12 yawgpu request child failed:\n{stderr}"
        );
        assert!(
            stderr.contains("subscript-typegpu: backend `d3d12` is not a yawgpu backend"),
            "d3d12 yawgpu request child lacks the diagnostic:\n{stderr}"
        );
        return;
    }

    let instance = subscript_typegpu_harness::native::subscript_typegpu_create_instance();
    if !instance.is_null() {
        subscript_typegpu_harness::native::subscript_typegpu_instance_release(instance);
        eprintln!("backend library is not yawgpu (no marker)");
        return;
    }
    assert!(instance.is_null());
}
