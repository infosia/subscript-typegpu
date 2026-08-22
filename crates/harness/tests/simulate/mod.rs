//! CL4 host-simulation coverage for gate programs.

#[test]
fn every_host_runnable_b_pipeline_prints_a_host_golden() {
    if !super::differential::backend_is_available() {
        return;
    }
    let _guard = super::differential::suite_lock();
    for output in super::differential::first_outputs() {
        let Some(name) = output.program().file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.starts_with('b') {
            continue;
        }
        let generated = subscript_typegpu_gen::generate(
            &subscript_typegpu_harness::program_files(output.program())
                .unwrap_or_else(|error| panic!("load {name}: {error}")),
        )
        .unwrap_or_else(|diagnostics| panic!("generate {name}: {diagnostics:?}"));
        if generated
            .support_module
            .lines()
            .any(|line| line.ends_with("_HOST_RUNNABLE: boolean = true;"))
        {
            let text = String::from_utf8_lossy(output.dev());
            assert!(
                text.lines().any(|line| line.starts_with("host:")),
                "{name} has a host-runnable pipeline but no host golden:\n{text}",
            );
        }
    }
}
