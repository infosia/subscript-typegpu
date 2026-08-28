use std::io::Write;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

fn measure_map_async(program: &Path) -> Result<Vec<u8>, String> {
    let mut session =
        subscript_typegpu_harness::load_program(program).map_err(|error| error.to_string())?;
    session
        .call_export("enableMapAsyncMeasurement")
        .map_err(|error| error.to_string())?;
    session
        .call_export("prepareMapAsync")
        .map_err(|error| error.to_string())?;
    while session.async_pending() != 0 {
        session.async_step().map_err(|error| error.to_string())?;
    }

    session
        .call_export("measureMapAsync")
        .map_err(|error| error.to_string())?;
    let started = Instant::now();
    let mut async_step_calls = 0;
    while session.async_pending() != 0 {
        async_step_calls += 1;
        session.async_step().map_err(|error| error.to_string())?;
    }
    let wall_time = started.elapsed();

    session
        .call_export("cleanupMapAsync")
        .map_err(|error| error.to_string())?;
    let output = session.take_output();
    if !output.ends_with(b"PASS\n") {
        return Err(format!(
            "mapAsync measurement program did not pass:\n{}",
            String::from_utf8_lossy(&output)
        ));
    }
    eprintln!(
        "mapAsync measurement: async_step_calls={} wall_time_ns={}",
        async_step_calls,
        wall_time.as_nanos()
    );
    Ok(output)
}

fn run() -> Result<Vec<u8>, String> {
    const USAGE: &str =
        "usage: subscript-typegpu-harness <dev|ship> <program> [--coverage|--measure-map-async]";
    let mut arguments = std::env::args_os().skip(1);
    let tier = arguments.next().ok_or_else(|| USAGE.to_owned())?;
    let program = arguments.next().ok_or_else(|| USAGE.to_owned())?;
    let mode = match arguments.next() {
        Some(argument) if argument == "--coverage" => "coverage",
        Some(argument) if argument == "--measure-map-async" => "measure-map-async",
        Some(_) => return Err(USAGE.to_owned()),
        None => "run",
    };
    if arguments.next().is_some() {
        return Err(USAGE.to_owned());
    }
    match tier.to_str() {
        Some("dev") if mode == "coverage" => {
            let (bytes, names) =
                subscript_typegpu_harness::run_dev_with_coverage(Path::new(&program))?;
            eprintln!("{}", subscript_typegpu_harness::COVERAGE_SEPARATOR);
            for name in names {
                eprintln!("coverage:{name}");
            }
            Ok(bytes)
        }
        Some("dev") if mode == "measure-map-async" => measure_map_async(Path::new(&program)),
        Some("dev") => subscript_typegpu_harness::run_dev(Path::new(&program)),
        Some("ship") if mode != "run" => {
            Err("measurement and coverage modes require the dev tier".to_owned())
        }
        Some("ship") => {
            let runtime = subscript_typegpu_harness::ensure_runtime_staticlib()?;
            std::env::set_var(subscript_codegen::RUNTIME_STATICLIB_ENV, runtime);
            subscript_typegpu_harness::run_ship(Path::new(&program))
        }
        _ => Err("tier must be `dev` or `ship`".to_owned()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(bytes) => match std::io::stdout().write_all(&bytes) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("write program output: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
