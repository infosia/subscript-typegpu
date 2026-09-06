//! The command-line runner of one program on one subscript tier.
//!
//! `<dev|ship> <program>` writes the program's raw output to stdout. `--coverage` adds the reached
//! facade exports to stderr, and `--measure-map-async` times one `mapAsync` wait. The dev tier
//! carries both extra modes.

use std::io::Write;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

/// Times one `mapAsync` wait of a program that exports the four measurement entries.
///
/// Returns the program's raw output. The measurement line goes to stderr, so it never reaches a
/// golden. An output whose last line is not `PASS` returns an error.
fn measure_map_async(program: &Path) -> Result<Vec<u8>, String> {
    let mut session =
        subscript_typegpu_harness::load_program(program).map_err(|error| error.to_string())?;
    session
        .call_export("enableMapAsyncMeasurement")
        .map_err(|error| error.to_string())?;
    session
        .call_export("prepareMapAsync")
        .map_err(|error| error.to_string())?;
    // Every request of the preparation completes before the clock starts, so the number covers
    // the wait alone.
    while session.async_pending() != 0 {
        session.async_step().map_err(|error| error.to_string())?;
    }

    session
        .call_export("measureMapAsync")
        .map_err(|error| error.to_string())?;
    let started = Instant::now();
    // The step count separates a slow backend from a wait that needs many pumps.
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
    // A program that failed its own checks must not report a time.
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

/// Parses the arguments and runs one program on the named tier.
///
/// Returns the program's raw output bytes. A missing argument, an unknown mode, or an extra
/// argument returns the usage line. The ship tier rejects both extra modes.
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
            // The C link needs the subscript runtime archive, which the variable names for the
            // code generator.
            let runtime = subscript_typegpu_harness::ensure_runtime_staticlib()?;
            std::env::set_var(subscript_codegen::RUNTIME_STATICLIB_ENV, runtime);
            subscript_typegpu_harness::run_ship(Path::new(&program))
        }
        _ => Err("tier must be `dev` or `ship`".to_owned()),
    }
}

/// Runs the command on a thread with the compiler's stack and writes the program output to stdout.
///
/// Any failure prints one stderr line and returns a failure exit code.
fn main() -> ExitCode {
    match subscript_typegpu_harness::run_on_compiler_stack(run).and_then(|result| result) {
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
