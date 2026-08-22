//! The generated WebGPU facade.

#![warn(missing_docs)]

#[rustfmt::skip]
pub mod generated;
mod runtime;

pub use generated::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receiverless_instance_queries_return_zero_without_a_table() {
        let previous = std::env::var_os("SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        std::env::remove_var("SUBSCRIPT_TYPEGPU_BACKEND_LIB");
        // SAFETY: every limits field admits its all-zero value.
        let mut limits: SubscriptTypegpuInstanceLimits = unsafe { std::mem::zeroed() };
        assert_eq!(subscript_typegpu_get_instance_limits(&mut limits), 0);
        assert!(!subscript_typegpu_has_instance_feature(0));
        if let Some(value) = previous {
            std::env::set_var("SUBSCRIPT_TYPEGPU_BACKEND_LIB", value);
        }
    }
}
