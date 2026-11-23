//! Library and application sources in compiler load order.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use subscript_compiler::{
    module_key, parse_import_specifiers, render_diagnostics, resolve_module_specifier, Diagnostic,
    SourceFile,
};

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
fn read_library_file(directory: &Path, prefix: &str, name: &str) -> Result<SourceFile, String> {
    let path = directory.join(name);
    let source = std::fs::read_to_string(&path)
        .map_err(|error| format!("read {}: {error}", path.display()))?;
    let name = format!("{prefix}{name}");
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

/// Returns the directory prefix of a normalized module key, with its final slash.
pub(crate) fn directory_prefix(key: &str) -> &str {
    key.rfind('/').map_or("", |end| &key[..=end])
}

/// Returns the registered library name for a key in the entry's directory.
pub(crate) fn registered(prefix: &str, key: &str) -> Option<&'static str> {
    let local = key.strip_prefix(prefix)?;
    LIBRARY_ORDER
        .into_iter()
        .find(|name| name.strip_suffix(".ts") == Some(local))
}

/// Returns the registered library name for a declaring file in the checked program (LB5).
pub fn registered_library(
    module: &subscript_compiler::hir::Module,
    file: &str,
) -> Option<&'static str> {
    registered(
        directory_prefix(&module_key(&module.entry_pos.file)),
        &module_key(file),
    )
}

/// Returns the generated support key for the entry's compiler source name (LB5).
pub fn support_module_key(entry: &str) -> String {
    format!("{}.typegpu", module_key(entry))
}

/// Loads the core libraries and transitive registered imports in library order.
fn read_libraries(
    directory: &Path,
    prefix: &str,
    mut pending: Vec<&'static str>,
) -> Result<Vec<SourceFile>, LibraryLoadError> {
    pending.extend(LIBRARY_ORDER.iter().take(CORE_COUNT));
    let mut loaded = HashMap::new();
    while let Some(name) = pending.pop() {
        if let std::collections::hash_map::Entry::Vacant(slot) = loaded.entry(name) {
            let file =
                read_library_file(directory, prefix, name).map_err(LibraryLoadError::Read)?;
            pending.extend(imports(&file)?.iter().filter_map(|specifier| {
                registered(prefix, &resolve_module_specifier(&file.name, specifier)?)
            }));
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
    /// The current root for module keys.
    directory: &'a Path,
    /// The directory that holds the entry on disk.
    entry_directory: &'a Path,
    /// The entry directory relative to the current root, with its final slash.
    prefix: &'a str,
    /// The largest number of parent segments required by an import in this walk.
    climb: usize,
    /// The key of the entry's generated support module.
    support: String,
    /// The module keys that the walk already reached.
    seen: HashSet<String>,
    /// The registered library names that the entry and its application modules import.
    libraries: Vec<&'static str>,
    /// The application sources in post-order, with the entry last.
    files: Vec<SourceFile>,
}

/// A missing path component leaves the import unresolved.
fn missing(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
    )
}

impl ApplicationModules<'_> {
    /// Checks whether an outside key reaches a file or a reserved module after the root moves.
    fn outside_import(&self, key: &str, climb: usize) -> Result<bool, LibraryLoadError> {
        let ancestor = self.directory.ancestors().nth(climb);
        let root = ancestor
            .or_else(|| self.directory.ancestors().last())
            .ok_or_else(|| {
                LibraryLoadError::Read(format!("invalid module root: {}", self.directory.display()))
            })?;
        let tail = key.trim_start_matches("../");
        let path = root.join(format!("{tail}.ts"));
        let reserved = path.parent() == Some(self.entry_directory)
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    let key = module_key(name);
                    registered("", &key).is_some()
                        || self.support.strip_prefix(self.prefix) == Some(&key)
                });
        if !reserved {
            match std::fs::metadata(&path) {
                Ok(metadata) if metadata.is_file() => {}
                Ok(_) => return Ok(false),
                Err(error) if missing(&error) => return Ok(false),
                Err(error) => {
                    return Err(LibraryLoadError::Read(format!(
                        "read {}: {error}",
                        path.display()
                    )))
                }
            }
        }
        if ancestor.is_none() {
            return Err(LibraryLoadError::Read(format!(
                "an import leaves the file system root: {key}"
            )));
        }
        Ok(true)
    }

    /// Visits imports in source order, then appends their importer (LB5).
    /// A key enters `seen` before descent, so repeated imports and cycles skip it.
    /// A directory, `NotFound`, or `NotADirectory` leaves the import for the compiler to diagnose.
    fn visit(&mut self, file: SourceFile) -> Result<(), LibraryLoadError> {
        for specifier in imports(&file)? {
            let Some(key) = resolve_module_specifier(&file.name, &specifier) else {
                continue;
            };
            if key.is_empty() || key.rsplit('/').next() == Some("..") || key == self.support {
                continue;
            }
            if let Some(name) = registered(self.prefix, &key) {
                self.libraries.push(name);
                continue;
            }
            let climb = key.split('/').take_while(|part| *part == "..").count();
            if climb > 0 {
                if self.outside_import(&key, climb)? {
                    self.climb = self.climb.max(climb);
                }
                continue;
            }
            let name = format!("{key}.ts");
            if !self.seen.insert(key) {
                continue;
            }
            let path = self.directory.join(&name);
            match std::fs::read_to_string(&path) {
                Ok(source) => self.visit(SourceFile::new(name, source))?,
                Err(error) if missing(&error) || path.is_dir() => continue,
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

/// Loads the registered libraries, application modules, and entry in compiler order (LB5).
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
    let parent = entry
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let entry_directory = parent.canonicalize().map_err(|error| {
        LibraryLoadError::Read(format!("resolve {}: {error}", parent.display()))
    })?;
    let mut root = entry_directory.as_path();
    let mut prefix = String::new();
    loop {
        let entry_name = format!("{prefix}{name}");
        let mut application = ApplicationModules {
            directory: root,
            entry_directory: &entry_directory,
            prefix: &prefix,
            climb: 0,
            support: support_module_key(&entry_name),
            seen: HashSet::from([module_key(&entry_name)]),
            libraries: Vec::new(),
            files: Vec::new(),
        };
        application.visit(SourceFile::entry(entry_name, source.clone()))?;
        if application.climb == 0 {
            let mut files = read_libraries(library, &prefix, application.libraries)?;
            files.extend(application.files);
            return Ok(files);
        }
        for _ in 0..application.climb {
            let segment = root
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                    LibraryLoadError::Read(format!("invalid module directory: {}", root.display()))
                })?;
            prefix.insert(0, '/');
            prefix.insert_str(0, segment);
            root = root
                .parent()
                .ok_or_else(|| LibraryLoadError::Read("module root has no parent".into()))?;
        }
    }
}
