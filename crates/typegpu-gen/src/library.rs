//! Registered library sources in compiler load order.

use std::path::Path;

use subscript_compiler::{parse_import_specifiers, render_diagnostics, Diagnostic, SourceFile};

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
const CORE_COUNT: usize = 5;

/// A required library file is unreadable or a source has invalid syntax.
#[derive(Debug)]
pub enum LibraryLoadError {
    /// A read operation failed for a required file.
    Read(String),
    /// The compiler parser rejected a source.
    Parse {
        file: SourceFile,
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
    let mut files: [Option<SourceFile>; LIBRARY_ORDER.len()] = std::array::from_fn(|_| None);
    let mut pending = vec![program.clone()];
    for (index, name) in LIBRARY_ORDER.iter().take(CORE_COUNT).enumerate() {
        let file = read_library_file(directory, name).map_err(LibraryLoadError::Read)?;
        pending.push(file.clone());
        files[index] = Some(file);
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
            let Some(index) = LIBRARY_ORDER
                .iter()
                .position(|name| name.strip_suffix(".ts") == Some(module))
            else {
                continue;
            };
            if files[index].is_none() {
                let dependency = read_library_file(directory, LIBRARY_ORDER[index])
                    .map_err(LibraryLoadError::Read)?;
                pending.push(dependency.clone());
                files[index] = Some(dependency);
            }
        }
    }
    Ok(files.into_iter().flatten().collect())
}
