use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use subscript_compiler::SourceFile;

pub(crate) struct TempDir(pub(crate) PathBuf);

impl TempDir {
    pub(crate) fn new(name: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "subscript-typegpu-test-{name}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        if directory.exists() {
            std::fs::remove_dir_all(&directory).expect("remove stale test directory");
        }
        std::fs::create_dir_all(&directory).expect("create test directory");
        Self(directory)
    }

    pub(crate) fn write(&self, name: &str, source: &str) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, source)
            .unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub(crate) fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("generator crate is under the repository root")
        .to_path_buf()
}

pub(crate) fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

pub(crate) fn b01_files() -> Vec<SourceFile> {
    program_files(&root().join("programs/b01-layout.ts"))
}

pub(crate) fn program_files(program: &Path) -> Vec<SourceFile> {
    subscript_typegpu_gen::load_program_files(&root().join("lib"), program)
        .expect("load program files")
}

pub(crate) fn source_files(program: SourceFile) -> Vec<SourceFile> {
    let directory = TempDir::new("source");
    program_files(&directory.write(&program.name, &program.source))
}

pub(crate) fn b_programs() -> Vec<PathBuf> {
    let directory = root().join("programs");
    let mut programs = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        .map(|entry| entry.expect("program entry").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with('b') && name.ends_with(".ts"))
        })
        .collect::<Vec<_>>();
    programs.sort();
    programs
}
