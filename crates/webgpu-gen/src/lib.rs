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

/// Generates both artifacts from yml + policy text, byte-stably.
pub fn generate(yml_text: &str, policy_text: &str) -> Result<Generated, Error> {
    let yml: model::Yml = serde_yaml::from_str(yml_text).map_err(Error::Yaml)?;
    let policy: policy::Policy = toml::from_str(policy_text).map_err(Error::Toml)?;
    let plan = plan::build(&yml, &policy)?;
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
    let symbols = native_symbols::render(&plan);
    Ok(Generated {
        header: emit_header::render(&plan, &cenum_aliases),
        rust: emit_rust::render(&plan),
        native_symbols: symbols.source,
        export_names: symbols.names,
        cenum_aliases,
    })
}
