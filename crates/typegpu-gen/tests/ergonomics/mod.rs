use crate::support;

#[test]
fn typed_and_positional_bind_group_paths_share_layout_and_binding_order() {
    let program = support::root().join("programs/b02-vecadd.ts");
    let generated = subscript_typegpu_gen::generate(&support::program_files(&program))
        .expect("generate b02 typed resources");
    let source = &generated.support_module;
    for expected in [
        "@Descriptor\nexport class VecAddLayoutResources",
        "a!: GPUBuffer;",
        "b!: GPUBuffer;",
        "out!: GPUBuffer;",
        "export function createVecAddLayoutResources(",
        "export function createVecAddBindGroup0(",
        "using layout = pipeline.bindGroupLayout(0);",
        "return createBindGroup(device, layout, vecAdd_LAYOUT0, [",
    ] {
        assert!(
            source.contains(expected),
            "missing `{expected}` in:\n{source}"
        );
    }

    let typed_order = [
        "bufferResource(resources.a)",
        "bufferResource(resources.b)",
        "bufferResource(resources.out)",
    ];
    let mut previous = 0;
    for expression in typed_order {
        let offset = source[previous..]
            .find(expression)
            .unwrap_or_else(|| panic!("typed factory lacks `{expression}`:\n{source}"))
            + previous;
        previous = offset + expression.len();
    }
    for binding in 0..3 {
        assert!(
            source.contains(&format!("{{ binding: {binding},")),
            "positional layout lacks binding {binding}:\n{source}"
        );
    }
}
