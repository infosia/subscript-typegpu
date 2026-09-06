//! The generated WebGPU facade.

// T22: library code holds no panic site. An export that cannot return an error clamps or ignores
// the value and never unwinds. A test module allows the lints again.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable,
    clippy::todo,
    clippy::unimplemented,
    clippy::indexing_slicing
)]
#![deny(missing_docs)]

#[rustfmt::skip]
pub mod generated;
mod runtime;
#[rustfmt::skip]
pub mod surface;

pub use generated::*;

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented,
        clippy::indexing_slicing
    )]

    use super::*;
    use std::process::Command;

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
                "subscript-typegpu: unknown SUBSCRIPT_TYPEGPU_BACKEND value `unknown-test-backend`; expected metal, vulkan, gles, d3d11, or d3d12"
            );
            return;
        }

        assert!(subscript_typegpu_create_instance().is_null());
    }
}
