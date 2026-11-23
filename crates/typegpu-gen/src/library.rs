//! Library and application sources in compiler load order.

use std::collections::{HashMap, HashSet};
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

/// Parses the import specifiers in source order and retains the source for diagnostics.
fn imports(file: &SourceFile) -> Result<Vec<String>, LibraryLoadError> {
    parse_import_specifiers(file).map_err(|diagnostics| LibraryLoadError::Parse {
        file: file.clone(),
        diagnostics,
    })
}

/// Returns the registered library name for a relative import specifier.
fn registered(specifier: &str) -> Option<&'static str> {
    let stem = specifier.strip_prefix("./")?;
    LIBRARY_ORDER
        .into_iter()
        .find(|name| name.strip_suffix(".ts") == Some(stem))
}

/// Loads the core libraries and transitive registered imports in library order.
fn read_libraries(
    directory: &Path,
    mut pending: Vec<&'static str>,
) -> Result<Vec<SourceFile>, LibraryLoadError> {
    pending.extend(LIBRARY_ORDER.iter().take(CORE_COUNT));
    let mut loaded = HashMap::new();
    while let Some(name) = pending.pop() {
        if let std::collections::hash_map::Entry::Vacant(slot) = loaded.entry(name) {
            let file = read_library_file(directory, name).map_err(LibraryLoadError::Read)?;
            pending.extend(imports(&file)?.iter().filter_map(|s| registered(s)));
            slot.insert(file);
        }
    }
    Ok(LIBRARY_ORDER
        .iter()
        .filter_map(|name| loaded.remove(name))
        .collect())
}

/// The state of the depth-first walk from the entry through its application modules.
struct ApplicationModules<'a> {
    /// The entry directory, which contains all application modules.
    directory: &'a Path,
    /// The import specifier for the entry's generated support module.
    support: String,
    /// The entry name and each application file name that the walk already reached.
    seen: HashSet<String>,
    /// The registered library names that the entry and its application modules import.
    libraries: Vec<&'static str>,
    /// The application sources in post-order, with the entry last.
    files: Vec<SourceFile>,
}

impl ApplicationModules<'_> {
    /// Visits imports in source order, then appends their importer (LB5).
    /// A name enters `seen` before descent, so repeated imports and cycles skip it.
    /// If a read returns `NotFound`, skips the file so the compiler can diagnose the import.
    fn visit(&mut self, file: SourceFile) -> Result<(), LibraryLoadError> {
        for specifier in imports(&file)? {
            if let Some(name) = registered(&specifier) {
                self.libraries.push(name);
                continue;
            }
            let Some(stem) = specifier.strip_prefix("./") else {
                continue;
            };
            if stem.is_empty() || stem.contains('/') || specifier == self.support {
                continue;
            }
            let name = format!("{stem}.ts");
            if Path::new(&name).file_name() != Some(name.as_ref())
                || !self.seen.insert(name.clone())
            {
                continue;
            }
            let path = self.directory.join(&name);
            match std::fs::read_to_string(&path) {
                Ok(source) => self.visit(SourceFile::new(name, source))?,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => {
                    return Err(LibraryLoadError::Read(format!(
                        "read {}: {error}",
                        path.display()
                    )))
                }
            }
        }
        self.files.push(file);
        Ok(())
    }
}

/// Loads the registered libraries, flat application modules, and entry in compiler order (LB5).
///
/// # Errors
///
/// Returns an error for unreadable sources, invalid syntax, or an invalid entry file name.
pub fn load_program_files(
    library: &Path,
    entry: &Path,
) -> Result<Vec<SourceFile>, LibraryLoadError> {
    let name = entry
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            LibraryLoadError::Read(format!(
                "program has no UTF-8 file name: {}",
                entry.display()
            ))
        })?;
    let source = std::fs::read_to_string(entry)
        .map_err(|error| LibraryLoadError::Read(format!("read {}: {error}", entry.display())))?;
    let stem = entry
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(name);
    let mut application = ApplicationModules {
        directory: entry.parent().unwrap_or(Path::new("")),
        support: format!("./{stem}.typegpu"),
        seen: HashSet::from([name.to_owned()]),
        libraries: Vec::new(),
        files: Vec::new(),
    };
    application.visit(SourceFile::entry(name, source))?;
    let mut files = read_libraries(library, application.libraries)?;
    files.extend(application.files);
    Ok(files)
}
