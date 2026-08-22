//! R19 optional aggregate lowerings stay as one conditional constructor call.

use std::path::Path;

use crate::support;

fn repo_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repo root")
        .to_path_buf()
        .join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

fn generate_api() -> Option<subscript_typegpu_webgpu_gen::GeneratedApi> {
    let gpuweb = subscript_typegpu_webgpu_gen::GPUWEB_IDL_INPUTS
        .iter()
        .map(|relative| repo_file(relative))
        .collect::<Vec<_>>()
        .join("\n");
    let mirror = support::base_mirror()?;
    Some(
        subscript_typegpu_webgpu_gen::generate_api(
            &gpuweb,
            mirror,
            &repo_file("crates/webgpu-gen/policy.toml"),
        )
        .expect("the committed API policy joins the pinned IDL and mirror"),
    )
}

fn converter_body<'a>(source: &'a str, converter: &str) -> &'a str {
    source
        .split(&format!("function {converter}("))
        .nth(1)
        .and_then(|tail| tail.split("\n}\n\n").next())
        .unwrap_or_else(|| panic!("R19 optional aggregate converter `{converter}` was not emitted"))
}

#[test]
fn optional_aggregates_use_one_conditional_boundary_constructor_call() {
    let Some(generated) = generate_api() else {
        return;
    };
    let cases = [
        (
            "toSubscriptTypegpuColorTargetState",
            "SubscriptTypegpuColorTargetState",
            &["nullableBlend !== null ? toSubscriptTypegpuBlendState(nullableBlend) : null"][..],
        ),
        (
            "toSubscriptTypegpuRenderPipelineDescriptor",
            "SubscriptTypegpuRenderPipelineDescriptor",
            &[
                "nullableDepthStencil !== null ? toSubscriptTypegpuDepthStencilState(nullableDepthStencil) : null",
                "nullableFragment !== null ? toSubscriptTypegpuFragmentState(nullableFragment) : null",
            ][..],
        ),
        (
            "toSubscriptTypegpuComputePassDescriptor",
            "SubscriptTypegpuComputePassDescriptor",
            &["nullableTimestampWrites !== null ? toSubscriptTypegpuPassTimestampWrites(nullableTimestampWrites) : null"][..],
        ),
        (
            "toSubscriptTypegpuRenderPassDescriptor",
            "SubscriptTypegpuRenderPassDescriptor",
            &[
                "nullableDepthStencilAttachment !== null ? toSubscriptTypegpuRenderPassDepthStencilAttachment(nullableDepthStencilAttachment) : null",
                "nullableTimestampWrites !== null ? toSubscriptTypegpuPassTimestampWrites(nullableTimestampWrites) : null",
            ][..],
        ),
        (
            "toSubscriptTypegpuDeviceDescriptor",
            "SubscriptTypegpuDeviceDescriptor",
            &["isGPURequiredLimitsEmpty(nullableRequiredLimits) ? null : toSubscriptTypegpuLimits(nullableRequiredLimits)"][..],
        ),
    ];

    for (converter, boundary, conditionals) in cases {
        let body = converter_body(&generated.source, converter);
        let constructor = format!("new {boundary}(");
        let calls = body.matches(&constructor).count();
        assert_eq!(
            calls, 1,
            "R19 optional aggregate converter `{converter}` must emit exactly one `{constructor}` call, found {calls}; 2^n branch emission is forbidden"
        );
        for conditional in conditionals {
            let occurrences = body.matches(conditional).count();
            assert_eq!(
                occurrences, 1,
                "R19 optional aggregate converter `{converter}` must emit `{conditional}` exactly once as a constructor argument, found {occurrences}"
            );
        }
        assert!(
            !body.contains("\n  if ("),
            "R19 optional aggregate converter `{converter}` emitted a statement branch; optional aggregates require conditional constructor arguments"
        );
    }
}
