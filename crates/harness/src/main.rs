use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

fn run() -> Result<Vec<u8>, String> {
    let mut arguments = std::env::args_os().skip(1);
    let tier = arguments
        .next()
        .ok_or_else(|| "usage: subscript-typegpu-harness <dev|ship> <program>".to_owned())?;
    let program = arguments
        .next()
        .ok_or_else(|| "usage: subscript-typegpu-harness <dev|ship> <program>".to_owned())?;
    if arguments.next().is_some() {
        return Err("usage: subscript-typegpu-harness <dev|ship> <program>".to_owned());
    }
    match tier.to_str() {
        Some("dev") => subscript_typegpu_harness::run_dev(Path::new(&program)),
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
