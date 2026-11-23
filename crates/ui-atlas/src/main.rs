//! Generate the UI atlas module from the pinned Roboto font.

mod atlas;

use std::{path::PathBuf, process::ExitCode};

fn run() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    let (Some(root), None) = (args.next(), args.next()) else {
        return Err("usage: subscript-typegpu-ui-atlas <repo-root>".into());
    };
    let root = PathBuf::from(root);
    let module = atlas::generate_ui_atlas(&root)?;
    let output = root.join("lib/typegpu-ui-atlas.generated.ts");
    std::fs::write(&output, module).map_err(|e| format!("write {}: {e}", output.display()))
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
