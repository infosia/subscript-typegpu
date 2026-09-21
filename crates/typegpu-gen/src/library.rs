//! Registered library sources in compiler load order.

use std::path::Path;

use subscript_compiler::{parse_import_specifiers, render_diagnostics, Diagnostic, SourceFile};

/// The registered library modules (LB1), in the order the compiler loads them.
///
/// The first `CORE_COUNT` entries are the core set that every program compiles with. Each other
/// entry loads only when an import declaration reaches it.
pub(crate) const LIBRARY_ORDER: [&str; 12] = [
    "subscript-typegpu.generated.d.ts",
    "wire-enum-aliases.generated.d.ts",
    "webgpu.ts",
    "typegpu-types.ts",
    "typegpu.ts",
    "typegpu-color.ts",
    "typegpu-noise.ts",
    "typegpu-radiance-cascades.ts",
    "typegpu-sdf.ts",
    "typegpu-sort.ts",
    "typegpu-ui-atlas.generated.ts",
    "typegpu-ui.ts",
];
/// The count of leading `LIBRARY_ORDER` entries that every program compiles with (LB1).
const CORE_COUNT: usize = 5;

/// A required library file is unreadable or a source has invalid syntax.
#[derive(Debug)]
pub enum LibraryLoadError {
    /// A read operation failed for a required file.
    Read(String),
    /// The compiler parser rejected a source.
    Parse {
        /// The source the parser rejected.
        file: SourceFile,
        /// The parser diagnostics, rendered against `file` alone.
        diagnostics: Vec<Diagnostic>,
    },
}

impl std::fmt::Display for LibraryLoadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read(message) => formatter.write_str(message),
            Self::Parse { file, diagnostics } => {
                formatter.write_str(&render_diagnostics(std::slice::from_ref(file), diagnostics))
            }
        }
    }
}

impl std::error::Error for LibraryLoadError {}

/// Reads one registered library file from `directory`.
///
/// A `.d.ts` name becomes an ambient source. Every other name becomes an ordinary source.
///
/// # Errors
///
/// If the read fails, returns a message that names the path and the cause.
fn read_library_file(directory: &Path, name: &str) -> Result<SourceFile, String> {
    let path = directory.join(name);
    let source = std::fs::read_to_string(&path)
        .map_err(|error| format!("read {}: {error}", path.display()))?;
    Ok(if name.ends_with(".d.ts") {
        SourceFile::ambient(name, source)
    } else {
        SourceFile::new(name, source)
    })
}

/// Loads the core sources and the registered modules that the program's imports reach.
///
/// The result follows library load order and excludes the program itself.
/// Unknown module specifiers remain subject to compiler diagnostics.
///
/// # Errors
///
/// If a required file is unreadable or a source has invalid syntax, returns an error.
pub fn load_library_files(
    directory: &Path,
    program: &SourceFile,
) -> Result<Vec<SourceFile>, LibraryLoadError> {
    // One slot per `LIBRARY_ORDER` entry. The slots keep load order whatever order the imports
    // reach the modules in.
    let mut files: [Option<SourceFile>; LIBRARY_ORDER.len()] = std::array::from_fn(|_| None);
    let mut pending = vec![program.clone()];
    // The core set loads unconditionally. Every other module waits for an import that reaches it.
    for (slot, name) in files.iter_mut().zip(LIBRARY_ORDER).take(CORE_COUNT) {
        let file = read_library_file(directory, name).map_err(LibraryLoadError::Read)?;
        pending.push(file.clone());
        *slot = Some(file);
    }
    while let Some(file) = pending.pop() {
        let imports =
            parse_import_specifiers(&file).map_err(|diagnostics| LibraryLoadError::Parse {
                file: file.clone(),
                diagnostics,
            })?;
        for specifier in imports {
            let Some(module) = specifier.strip_prefix("./") else {
                continue;
            };
            // A specifier that names no registered module belongs to the program's own files. The
            // compiler reports it when it cannot resolve it.
            let Some((slot, name)) = files
                .iter_mut()
                .zip(LIBRARY_ORDER)
                .find(|(_, name)| name.strip_suffix(".ts") == Some(module))
            else {
                continue;
            };
            if slot.is_none() {
                let dependency =
                    read_library_file(directory, name).map_err(LibraryLoadError::Read)?;
                pending.push(dependency.clone());
                *slot = Some(dependency);
            }
        }
    }
    Ok(files.into_iter().flatten().collect())
}
