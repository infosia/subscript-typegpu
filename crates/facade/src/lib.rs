//! The generated WebGPU facade.

#![warn(missing_docs)]

#[rustfmt::skip]
pub mod generated;
mod runtime;

pub use generated::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn receiverless_instance_queries_return_zero_without_a_table() {
        const CHILD: &str = "SUBSCRIPT_TYPEGPU_RECEIVERLESS_TEST_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let output = Command::new(std::env::current_exe().expect("test binary path"))
                .args([
                    "--exact",
                    "tests::receiverless_instance_queries_return_zero_without_a_table",
                    "--nocapture",
                ])
                .env(CHILD, "1")
                .env_remove("SUBSCRIPT_TYPEGPU_BACKEND_LIB")
                .output()
                .expect("run receiverless instance query child");
            assert!(
                output.status.success(),
                "receiverless instance query child failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }

        // SAFETY: every limits field admits its all-zero value.
        let mut limits: SubscriptTypegpuInstanceLimits = unsafe { std::mem::zeroed() };
        assert_eq!(subscript_typegpu_get_instance_limits(&mut limits), 0);
        assert!(!subscript_typegpu_has_instance_feature(0));
    }

    #[test]
    fn unknown_backend_request_is_loud_and_returns_null() {
        const CHILD: &str = "SUBSCRIPT_TYPEGPU_UNKNOWN_BACKEND_TEST_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let output = Command::new(std::env::current_exe().expect("test binary path"))
                .args([
                    "--exact",
                    "tests::unknown_backend_request_is_loud_and_returns_null",
                    "--nocapture",
                ])
                .env(CHILD, "1")
                .env("SUBSCRIPT_TYPEGPU_BACKEND", "unknown-test-backend")
                .env_remove("SUBSCRIPT_TYPEGPU_BACKEND_LIB")
                .output()
                .expect("run unknown backend request child");
            assert!(
                output.status.success(),
                "unknown backend request child failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                String::from_utf8_lossy(&output.stderr).trim(),
                "subscript-typegpu: unknown SUBSCRIPT_TYPEGPU_BACKEND value `unknown-test-backend`; expected metal, vulkan, or gles"
            );
            return;
        }

        assert!(subscript_typegpu_create_instance().is_null());
    }
}
