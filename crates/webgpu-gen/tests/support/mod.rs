//! Shared inputs for API generator integration tests.

use std::path::Path;
use std::sync::{Once, OnceLock};

fn repo_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repo root")
        .to_path_buf()
        .join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

/// Prints the integration executable's single libclang pending line.
pub fn pend_libclang() {
    static PENDING: Once = Once::new();
    PENDING.call_once(|| println!("pending: libclang — install Xcode command line tools"));
}

/// Runs bindgen, or marks the test pending when libclang is absent.
pub fn bind_header(header: &str) -> Option<String> {
    match subscript_bindgen::generate_for_header(header, "subscript-typegpu.h") {
        Ok(mirror) => Some(mirror),
        Err(error) if error.to_string().to_ascii_lowercase().contains("libclang") => {
            pend_libclang();
            None
        }
        Err(error) => panic!("generate mirror: {error}"),
    }
}

/// Generates the enum-free facade mirror used by API joins.
pub fn base_mirror() -> Option<&'static str> {
    static MIRROR: OnceLock<Option<String>> = OnceLock::new();
    MIRROR
        .get_or_init(|| {
            let facade = subscript_typegpu_webgpu_gen::generate(
                &repo_file("third_party/webgpu-headers/webgpu.yml"),
                &repo_file("crates/webgpu-gen/policy.toml"),
            )
            .expect("the facade inputs generate for an API test");
            let header = facade
                .header
                .split_inclusive('\n')
                .filter(|line| !line.contains("@subscript-cenum"))
                .collect::<String>();
            bind_header(&header)
        })
        .as_deref()
}

macro_rules! require_base_mirror {
    () => {
        match crate::support::base_mirror() {
            Some(mirror) => mirror.to_owned(),
            None => return,
        }
    };
}

pub(crate) use require_base_mirror;
