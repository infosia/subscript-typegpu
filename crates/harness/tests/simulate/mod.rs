//! CL2 and CL4 host-simulation gates.

use std::path::{Path, PathBuf};

use subscript_compiler::hir::{AsyncCallee, Callee, Expr, ExprKind, Module, Stmt, TplPart};
use subscript_compiler::CheckOptions;
use subscript_typegpu_gen::Generated;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("harness crate is under the repository root")
        .to_path_buf()
}

fn is_program(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let Some(stem) = name.strip_suffix(".ts") else {
        return false;
    };
    let bytes = stem.as_bytes();
    bytes.len() >= 5
        && matches!(bytes[0], b'a' | b'b' | b'x')
        && bytes[1].is_ascii_digit()
        && bytes[2].is_ascii_digit()
        && bytes[3] == b'-'
}

fn programs() -> Vec<PathBuf> {
    let directory = repository_root().join("programs");
    let mut programs = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        .map(|entry| entry.expect("program entry").path())
        .filter(|path| is_program(path))
        .collect::<Vec<_>>();
    programs.sort();
    assert!(!programs.is_empty(), "simulate program list is empty");
    programs
}

fn simulation_spec_index(name: &str) -> Option<usize> {
    Some(match name.split('<').next().unwrap_or(name) {
        "simulateCompute" | "simulateComputeThreads" => 2,
        "simulateCompute2" => 3,
        "simulateCompute3" => 4,
        "simulateCompute4" => 5,
        _ => return None,
    })
}

fn statement_pos(statement: &Stmt) -> Option<&subscript_compiler::Pos> {
    match statement {
        Stmt::Let { pos, .. }
        | Stmt::Return { pos, .. }
        | Stmt::If { pos, .. }
        | Stmt::While { pos, .. }
        | Stmt::For { pos, .. }
        | Stmt::ForOf { pos, .. }
        | Stmt::Switch { pos, .. }
        | Stmt::Break(pos)
        | Stmt::Continue(pos) => Some(pos),
        Stmt::Expr(expression) => Some(&expression.pos),
        Stmt::Block(body) => body.first().and_then(statement_pos),
        _ => None,
    }
}

fn is_library_simulation(module: &Module, name: &str) -> bool {
    simulation_spec_index(name).is_some()
        && module.functions.iter().any(|function| {
            function.name == name
                && (function.pos.file == "typegpu.ts"
                    || function
                        .params
                        .iter()
                        .any(|parameter| parameter.pos.file == "typegpu.ts")
                    || function
                        .body
                        .first()
                        .and_then(statement_pos)
                        .is_some_and(|pos| pos.file == "typegpu.ts"))
        })
}

fn assert_pair(
    program: &Path,
    module: &Module,
    generated: &Generated,
    expression: &Expr,
    callee: &str,
    args: &[Expr],
    failures: &mut Vec<String>,
) {
    if !is_library_simulation(module, callee) {
        return;
    }
    let method = callee.split('<').next().unwrap_or(callee);
    let Some(spec_index) = simulation_spec_index(callee) else {
        return;
    };
    let call = format!("{} {method} at {}", program.display(), expression.pos);
    let Some(Expr {
        kind: ExprKind::FuncRef(kernel),
        ..
    }) = args.first()
    else {
        failures.push(format!("{call} does not pass a FuncRef kernel"));
        return;
    };
    let Some(Expr {
        kind: ExprKind::Global(declaration),
        ..
    }) = args.get(spec_index)
    else {
        failures.push(format!("{call} does not pass a Global pipeline spec"));
        return;
    };
    let Some(pipeline) = generated
        .compute_pipelines
        .iter()
        .find(|pipeline| pipeline.declaration == *declaration && pipeline.kernel == *kernel)
    else {
        failures.push(format!(
            "{call} pairs kernel `{kernel}` with pipeline `{declaration}`"
        ));
        return;
    };
    let expected = format!("{}_HOST_RUNNABLE", pipeline.declaration);
    let Some(Expr {
        kind: ExprKind::Global(constant),
        ..
    }) = args.last()
    else {
        failures.push(format!("{call} does not pass Global `{expected}`"));
        return;
    };
    if constant != &expected {
        failures.push(format!(
            "{call} passes `{constant}`, expected `{expected}` for kernel `{kernel}`"
        ));
        return;
    }
    let support_file = format!(
        "{}.typegpu.ts",
        program
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("UTF-8 program stem"),
    );
    let Some(global) = module.globals.iter().find(|global| global.name == expected) else {
        failures.push(format!("{call} cannot resolve Global `{expected}`"));
        return;
    };
    if global.pos.file != support_file {
        failures.push(format!(
            "{call} resolves `{expected}` from {}, expected {support_file}",
            global.pos.file,
        ));
    }
}

fn visit_expr(
    program: &Path,
    program_name: &str,
    module: &Module,
    generated: &Generated,
    expression: &Expr,
    failures: &mut Vec<String>,
    simulation_calls: &mut usize,
) {
    if expression.pos.file == program_name {
        if let ExprKind::Call {
            callee: Callee::Func(callee),
            args,
        } = &expression.kind
        {
            if is_library_simulation(module, callee) {
                *simulation_calls += 1;
            }
            assert_pair(
                program, module, generated, expression, callee, args, failures,
            );
        }
    }
    macro_rules! visit {
        ($child:expr) => {
            visit_expr(
                program,
                program_name,
                module,
                generated,
                $child,
                failures,
                simulation_calls,
            )
        };
    }
    match &expression.kind {
        ExprKind::Unary { operand, .. }
        | ExprKind::Cast(operand)
        | ExprKind::Length(operand)
        | ExprKind::Field { obj: operand, .. }
        | ExprKind::JsonResultValue(operand) => visit!(operand),
        ExprKind::Binary { left, right, .. }
        | ExprKind::Assign {
            target: left,
            value: right,
            ..
        } => {
            visit!(left);
            visit!(right);
        }
        ExprKind::Call { callee, args } => {
            match callee {
                Callee::Value(value) => visit!(value),
                Callee::Method { recv, .. } => visit!(recv),
                _ => {}
            }
            for arg in args {
                visit!(arg);
            }
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => {
            for arg in args {
                visit!(arg);
            }
        }
        ExprKind::DescriptorLit { fields, .. } => {
            for value in fields.iter().flatten() {
                visit!(value);
            }
        }
        ExprKind::Index { obj, index, .. } => {
            visit!(obj);
            visit!(index);
        }
        ExprKind::ArraySpreadLit(items) => {
            for item in items {
                visit!(&item.expr);
            }
        }
        ExprKind::Template(parts) => {
            for part in parts {
                if let TplPart::Expr(value) = part {
                    visit!(value);
                }
            }
        }
        ExprKind::Lambda { body, .. } => visit_statements(
            program,
            program_name,
            module,
            generated,
            body,
            failures,
            simulation_calls,
        ),
        ExprKind::Yield(value) => {
            if let Some(value) = value {
                visit!(value);
            }
        }
        ExprKind::AsyncCall { callee, args } => {
            if let AsyncCallee::Method { receiver, .. } = callee {
                visit!(receiver);
            }
            for arg in args {
                visit!(arg);
            }
        }
        ExprKind::Cond { cond, then, els } => {
            visit!(cond);
            visit!(then);
            visit!(els);
        }
        _ => {}
    }
}

fn visit_statements(
    program: &Path,
    program_name: &str,
    module: &Module,
    generated: &Generated,
    statements: &[Stmt],
    failures: &mut Vec<String>,
    simulation_calls: &mut usize,
) {
    macro_rules! visit {
        ($child:expr) => {
            visit_expr(
                program,
                program_name,
                module,
                generated,
                $child,
                failures,
                simulation_calls,
            )
        };
    }
    for statement in statements {
        match statement {
            Stmt::Let { init, .. } | Stmt::Expr(init) => visit!(init),
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    visit!(value);
                }
            }
            Stmt::If {
                cond, then, els, ..
            } => {
                visit!(cond);
                visit_statements(
                    program,
                    program_name,
                    module,
                    generated,
                    then,
                    failures,
                    simulation_calls,
                );
                if let Some(els) = els {
                    visit_statements(
                        program,
                        program_name,
                        module,
                        generated,
                        els,
                        failures,
                        simulation_calls,
                    );
                }
            }
            Stmt::While { cond, body, .. } => {
                visit!(cond);
                visit_statements(
                    program,
                    program_name,
                    module,
                    generated,
                    body,
                    failures,
                    simulation_calls,
                );
            }
            Stmt::For {
                init,
                cond,
                step,
                body,
                ..
            } => {
                if let Some(init) = init {
                    visit_statements(
                        program,
                        program_name,
                        module,
                        generated,
                        std::slice::from_ref(init.as_ref()),
                        failures,
                        simulation_calls,
                    );
                }
                if let Some(cond) = cond {
                    visit!(cond);
                }
                if let Some(step) = step {
                    visit!(step);
                }
                visit_statements(
                    program,
                    program_name,
                    module,
                    generated,
                    body,
                    failures,
                    simulation_calls,
                );
            }
            Stmt::ForOf { subject, body, .. } => {
                visit!(subject);
                visit_statements(
                    program,
                    program_name,
                    module,
                    generated,
                    body,
                    failures,
                    simulation_calls,
                );
            }
            Stmt::Switch { disc, cases, .. } => {
                visit!(disc);
                for case in cases {
                    if let Some(test) = &case.test {
                        visit!(test);
                    }
                    visit_statements(
                        program,
                        program_name,
                        module,
                        generated,
                        &case.body,
                        failures,
                        simulation_calls,
                    );
                }
            }
            Stmt::Block(body) => visit_statements(
                program,
                program_name,
                module,
                generated,
                body,
                failures,
                simulation_calls,
            ),
            Stmt::Break(_) | Stmt::Continue(_) => {}
            _ => {}
        }
    }
}

fn pairing_failures(program: &Path) -> (Vec<String>, usize) {
    let files = subscript_typegpu_harness::program_files(program)
        .unwrap_or_else(|error| panic!("load {}: {error}", program.display()));
    let generated = subscript_typegpu_gen::generate(&files)
        .unwrap_or_else(|diagnostics| panic!("generate {}: {diagnostics:?}", program.display()));
    let module = subscript_compiler::check_program_with(&files, &CheckOptions::default())
        .unwrap_or_else(|diagnostics| panic!("check {}: {diagnostics:?}", program.display()));
    let program_name = program
        .file_name()
        .and_then(|name| name.to_str())
        .expect("UTF-8 program name");
    let mut failures = Vec::new();
    let mut simulation_calls = 0;
    for global in &module.globals {
        visit_expr(
            program,
            program_name,
            &module,
            &generated,
            &global.init,
            &mut failures,
            &mut simulation_calls,
        );
    }
    for function in &module.functions {
        visit_statements(
            program,
            program_name,
            &module,
            &generated,
            &function.body,
            &mut failures,
            &mut simulation_calls,
        );
    }
    for class in &module.classes {
        if let Some(constructor) = &class.ctor {
            visit_statements(
                program,
                program_name,
                &module,
                &generated,
                &constructor.body,
                &mut failures,
                &mut simulation_calls,
            );
        }
        for method in &class.methods {
            visit_statements(
                program,
                program_name,
                &module,
                &generated,
                &method.body,
                &mut failures,
                &mut simulation_calls,
            );
        }
    }
    visit_statements(
        program,
        program_name,
        &module,
        &generated,
        &module.top_level,
        &mut failures,
        &mut simulation_calls,
    );
    (failures, simulation_calls)
}

#[test]
fn every_simulation_call_uses_its_generated_pipeline_pair() {
    let mut failures = Vec::new();
    for program in programs() {
        let (program_failures, simulation_calls) = pairing_failures(&program);
        failures.extend(program_failures);
        let name = program
            .file_name()
            .and_then(|name| name.to_str())
            .expect("UTF-8 program name");
        if matches!(
            name,
            "x01-live-vecadd.ts"
                | "x02-live-saxpy.ts"
                | "x03-live-particles.ts"
                | "x04-live-control-flow.ts"
                | "x09-live-switch.ts"
        ) && simulation_calls == 0
        {
            failures.push(format!(
                "{} has no simulateCompute* call required by CL3",
                program.display(),
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

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
        let required = output
            .generated()
            .compute_pipelines
            .iter()
            .filter(|pipeline| pipeline.host_runnable)
            .count();
        let text = String::from_utf8_lossy(output.dev());
        let actual = text
            .lines()
            .filter(|line| line.starts_with("host:"))
            .count();
        if name == "b13-vector-builtins.ts" {
            assert_eq!(
                actual, 5,
                "{name} must print one host line per vector method family"
            );
        }
        assert!(
            actual >= required,
            "{name} prints {actual} host lines for {required} host-runnable pipelines:\n{text}",
        );
    }
}
