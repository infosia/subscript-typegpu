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
    fn released_children_leave_the_live_instance_index_unchanged() {
        const CHILD: &str = "SUBSCRIPT_TYPEGPU_HANDLE_INDEX_TEST_CHILD";
        if std::env::var_os("SUBSCRIPT_TYPEGPU_BACKEND_LIB").is_none() {
            println!("pending: backend library — set SUBSCRIPT_TYPEGPU_BACKEND_LIB");
            return;
        }
        if std::env::var_os(CHILD).is_none() {
            let output = Command::new(std::env::current_exe().expect("test binary path"))
                .args([
                    "--exact",
                    "tests::released_children_leave_the_live_instance_index_unchanged",
                    "--nocapture",
                ])
                .env(CHILD, "1")
                .env_remove("SUBSCRIPT_TYPEGPU_BACKEND")
                .output()
                .expect("run handle index child");
            assert!(
                output.status.success(),
                "handle index child failed:\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }

        let instance = subscript_typegpu_create_instance();
        assert!(!instance.is_null());
        let adapter_future = subscript_typegpu_instance_request_adapter(instance);
        wait_for_success(instance, adapter_future);
        let adapter = subscript_typegpu_request_adapter_take(instance, adapter_future);
        assert!(!adapter.is_null());
        let device_future = subscript_typegpu_adapter_request_device_with_descriptor(
            instance,
            adapter,
            std::ptr::null(),
        );
        wait_for_success(instance, device_future);
        let device = subscript_typegpu_request_device_take(instance, device_future);
        assert!(!device.is_null());

        let baseline = runtime::handle_instance_count();
        let descriptor = SubscriptTypegpuBufferDescriptor {
            label: SubscriptTypegpuStringView {
                data: std::ptr::null(),
                length: 0,
            },
            usage: 0x8, // CopyDst.
            size: 16,
            mapped_at_creation: false,
        };
        for _ in 0..4 {
            let mut children = Vec::new();
            for _ in 0..32 {
                let buffer = subscript_typegpu_device_create_buffer(device, &descriptor);
                let encoder =
                    subscript_typegpu_device_create_command_encoder(device, std::ptr::null());
                assert!(!buffer.is_null());
                assert!(!encoder.is_null());
                assert_eq!(
                    runtime::instance_for_handle(buffer as usize),
                    instance as usize
                );
                assert_eq!(
                    runtime::instance_for_handle(encoder as usize),
                    instance as usize
                );
                children.push((buffer, encoder));
            }
            assert_eq!(runtime::handle_instance_count(), baseline + 64);
            for (buffer, encoder) in children {
                subscript_typegpu_buffer_release(buffer);
                subscript_typegpu_command_encoder_release(encoder);
            }
            assert_eq!(runtime::handle_instance_count(), baseline);
            assert_eq!(
                runtime::instance_for_handle(instance as usize),
                instance as usize
            );
            assert_eq!(
                runtime::instance_for_handle(device as usize),
                instance as usize
            );
        }

        let first = subscript_typegpu_device_get_queue(device);
        let second = subscript_typegpu_device_get_queue(device);
        assert!(!first.is_null());
        assert_eq!(first, second);
        assert_eq!(runtime::handle_instance_count(), baseline + 1);
        subscript_typegpu_queue_release(first);
        assert_eq!(
            runtime::instance_for_handle(second as usize),
            instance as usize
        );
        subscript_typegpu_queue_release(second);
        assert_eq!(runtime::instance_for_handle(second as usize), 0);
        assert_eq!(runtime::handle_instance_count(), baseline);

        subscript_typegpu_device_release(device);
        subscript_typegpu_adapter_release(adapter);
        assert_eq!(runtime::handle_instance_count(), 1);
        subscript_typegpu_instance_release(instance);
        assert_eq!(runtime::handle_instance_count(), 0);
    }

    fn wait_for_success(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId) {
        assert_ne!(future, 0);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            subscript_typegpu_instance_process_events(instance);
            let status = subscript_typegpu_future_status(instance, future);
            if status != 0 {
                assert_eq!(status, 1);
                return;
            }
            assert!(std::time::Instant::now() < deadline, "future timed out");
            std::thread::yield_now();
        }
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
                "subscript-typegpu: unknown SUBSCRIPT_TYPEGPU_BACKEND value `unknown-test-backend`; expected metal, vulkan, gles, d3d11, or d3d12"
            );
            return;
        }

        assert!(subscript_typegpu_create_instance().is_null());
    }
}
