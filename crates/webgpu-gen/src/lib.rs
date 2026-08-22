//! Byte-stable facade and API generators. The facade stage joins
//! `webgpu.yml` with policy. The API stage joins extracted GPUWeb IDL,
//! the generated facade mirror, and API policy. Both stages enforce
//! two-way policy validation and regeneration gates.
#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod api;
mod api_model;
mod emit_header;
mod emit_rust;
mod idl;
mod model;
mod naming;
mod native_symbols;
mod patterns;
mod plan;
mod policy;

use std::collections::BTreeSet;
use std::fmt;

pub use api::{generate_api, ApiError, ApiPolicyError, CEnumAlias, GeneratedApi};
pub use idl::{extract_gpuweb_idl, ExtractedIdl, NamespaceConstant};
pub use policy::PolicyError;

/// Pinned GPUWeb source files whose `script type=idl` blocks form the API model.
pub const GPUWEB_IDL_INPUTS: [&str; 2] = [
    "third_party/gpuweb/spec/index.bs",
    "third_party/gpuweb/spec/sections/copies.bs",
];

/// The two generated artifacts, as exact file bytes.
#[derive(Debug)]
pub struct Generated {
    /// `facade/subscript-typegpu.h` content.
    pub header: String,
    /// `facade/src/generated.rs` content.
    pub rust: String,
    /// Harness facade symbol table content.
    pub native_symbols: String,
    /// Facade export names in declaration order.
    pub export_names: Vec<String>,
    /// CEnum aliases derived directly from policy and yml.
    pub cenum_aliases: Vec<CEnumAlias>,
}

/// Generation failure: an input parse error or a policy error.
#[derive(Debug)]
pub enum Error {
    /// `webgpu.yml` did not parse into the model.
    Yaml(serde_yaml::Error),
    /// `policy.toml` did not parse into the policy schema.
    Toml(toml::de::Error),
    /// The two-way validation failed (F18).
    Policy(PolicyError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Yaml(e) => write!(f, "webgpu.yml: {e}"),
            Error::Toml(e) => write!(f, "policy.toml: {e}"),
            Error::Policy(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<PolicyError> for Error {
    fn from(e: PolicyError) -> Self {
        Error::Policy(e)
    }
}

fn export_exclusions(
    policy: &policy::Policy,
    plan: &plan::Plan,
) -> Result<BTreeSet<String>, Error> {
    let exports = native_symbols::export_names(plan)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut excluded = BTreeSet::new();
    for row in &policy.export_exclude {
        if row.reason.trim().is_empty() {
            return Err(Error::Policy(PolicyError::Invalid {
                entry: row.name.clone(),
                message: "export exclusion requires a reason".to_owned(),
            }));
        }
        if !exports.contains(&row.name) {
            return Err(Error::Policy(PolicyError::Unknown {
                entry: row.name.clone(),
            }));
        }
        if !excluded.insert(row.name.clone()) {
            return Err(Error::Policy(PolicyError::Duplicate {
                entry: row.name.clone(),
            }));
        }
    }
    Ok(excluded)
}

fn filter_header_exports(mut header: String, excluded: &BTreeSet<String>) -> String {
    if excluded.is_empty() {
        return header;
    }
    let retained = header
        .lines()
        .filter(|line| {
            !excluded
                .iter()
                .any(|name| line.contains(&format!(" {name}(")))
        })
        .collect::<Vec<_>>();
    let mut lines: Vec<&str> = Vec::with_capacity(retained.len());
    for line in retained {
        if line.is_empty() && lines.last().is_some_and(|previous| previous.is_empty()) {
            continue;
        }
        lines.push(line);
    }
    header = lines.join("\n");
    header.push('\n');
    header
}

fn rust_export_range(source: &str, name: &str) -> Option<std::ops::Range<usize>> {
    let marker = format!("pub extern \"C\" fn {name}(");
    let signature = source.find(&marker)?;
    let start = source[..signature].rfind("\n///").map_or_else(
        || source[..signature].rfind("#[no_mangle]"),
        |offset| Some(offset + 1),
    )?;
    let open = source[signature..].find('{')? + signature;
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut index = open;
    let mut string = false;
    let mut character = false;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if escaped {
            escaped = false;
        } else if string {
            match byte {
                b'\\' => escaped = true,
                b'"' => string = false,
                _ => {}
            }
        } else if character {
            match byte {
                b'\\' => escaped = true,
                b'\'' => character = false,
                _ => {}
            }
        } else {
            match byte {
                b'"' => string = true,
                b'\'' => character = true,
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        let mut end = index + 1;
                        while end < bytes.len() && bytes[end] == b'\n' {
                            end += 1;
                        }
                        return Some(start..end);
                    }
                }
                _ => {}
            }
        }
        index += 1;
    }
    None
}

fn filter_rust_exports(mut rust: String, excluded: &BTreeSet<String>) -> Result<String, Error> {
    for name in excluded {
        let range = rust_export_range(&rust, name).ok_or_else(|| {
            Error::Policy(PolicyError::Invalid {
                entry: name.clone(),
                message: "excluded export has no generated Rust function".to_owned(),
            })
        })?;
        rust.replace_range(range, "");
    }
    Ok(rust)
}

/// Generates both artifacts from yml + policy text, byte-stably.
pub fn generate(yml_text: &str, policy_text: &str) -> Result<Generated, Error> {
    let yml: model::Yml = serde_yaml::from_str(yml_text).map_err(Error::Yaml)?;
    let policy: policy::Policy = toml::from_str(policy_text).map_err(Error::Toml)?;
    let plan = plan::build(&yml, &policy)?;
    let excluded_exports = export_exclusions(&policy, &plan)?;
    let cenum_aliases = policy
        .api
        .as_ref()
        .map(|api| {
            api.enums
                .iter()
                .map(|public_name| CEnumAlias {
                    boundary_name: format!(
                        "SubscriptTypegpu{}",
                        public_name.strip_prefix("GPU").unwrap_or(public_name)
                    ),
                    public_name: public_name.clone(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for alias in &cenum_aliases {
        let matches = plan
            .const_sets
            .iter()
            .filter(|set| {
                matches!(set.kind, plan::ConstKind::Enum)
                    && naming::subscript_typegpu_type(&set.name) == alias.boundary_name
            })
            .count();
        if matches != 1 {
            return Err(Error::Policy(PolicyError::Invalid {
                entry: alias.public_name.clone(),
                message: format!(
                    "CEnum alias `{}` names {matches} emitted enum typedefs",
                    alias.boundary_name
                ),
            }));
        }
    }
    let rust = filter_rust_exports(
        emit_rust::render(&plan, &excluded_exports),
        &excluded_exports,
    )?;
    let symbols = native_symbols::render(&plan, &rust, &excluded_exports);
    Ok(Generated {
        header: filter_header_exports(
            emit_header::render(&plan, &cenum_aliases),
            &excluded_exports,
        ),
        rust,
        native_symbols: symbols.source,
        export_names: symbols.names,
        cenum_aliases,
    })
}
