use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

fn run() -> Result<Vec<u8>, String> {
    const USAGE: &str = "usage: subscript-typegpu-harness <dev|ship> <program> [--coverage]";
    let mut arguments = std::env::args_os().skip(1);
    let tier = arguments.next().ok_or_else(|| USAGE.to_owned())?;
    let program = arguments.next().ok_or_else(|| USAGE.to_owned())?;
    let coverage = match arguments.next() {
        Some(argument) if argument == "--coverage" => true,
        Some(_) => return Err(USAGE.to_owned()),
        None => false,
    };
    if arguments.next().is_some() {
        return Err(USAGE.to_owned());
    }
    match tier.to_str() {
        Some("dev") if coverage => {
            let (mut bytes, names) =
                subscript_typegpu_harness::run_dev_with_coverage(Path::new(&program))?;
            if !bytes.is_empty() && !bytes.ends_with(b"\n") {
                bytes.push(b'\n');
            }
            for name in names {
                bytes.extend_from_slice(format!("coverage:{name}\n").as_bytes());
            }
            Ok(bytes)
        }
        Some("dev") => subscript_typegpu_harness::run_dev(Path::new(&program)),
        Some("ship") if coverage => Err("--coverage is available only for the dev tier".to_owned()),
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
