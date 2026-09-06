//! Serde model of the fields used from the pinned `webgpu.yml`.
//! Unknown fields do not widen the policy surface.

use serde::Deserialize;

/// Top-level `webgpu.yml` document.
#[derive(Debug, Deserialize)]
pub(crate) struct Yml {
    /// File-level constants such as `strlen` and sentinels.
    #[serde(default)]
    pub constants: Vec<Constant>,
    /// Plain enums with `uint32_t` storage and index values.
    #[serde(default)]
    pub enums: Vec<Enum>,
    /// 64-bit flag types (entry 0 = 0, entry i = `1 << (i - 1)`).
    #[serde(default)]
    pub bitflags: Vec<Bitflag>,
    /// Struct definitions.
    #[serde(default)]
    pub structs: Vec<Struct>,
    /// Callback definitions.
    #[serde(default)]
    pub callbacks: Vec<Callback>,
    /// Freestanding functions.
    #[serde(default)]
    pub functions: Vec<Function>,
    /// Objects (opaque handles with methods).
    #[serde(default)]
    pub objects: Vec<Object>,
}

/// One file-level constant.
#[derive(Debug, Deserialize)]
pub(crate) struct Constant {
    /// The yml constant name, snake_case.
    pub name: String,
    /// Symbolic value such as `usize_max` or `uint64_max`.
    #[allow(dead_code)]
    pub value: serde_yaml::Value,
}

/// One plain enum. `entries` can contain `null` placeholders that
/// still occupy a value slot.
#[derive(Debug, Deserialize)]
pub(crate) struct Enum {
    /// The yml enum name, snake_case.
    pub name: String,
    /// The entries in value order. A `null` entry holds its slot without a name.
    pub entries: Vec<Option<EnumEntry>>,
}

/// One named enum entry.
#[derive(Debug, Deserialize)]
pub(crate) struct EnumEntry {
    /// The yml entry name, snake_case.
    pub name: String,
}

/// One flag type.
#[derive(Debug, Deserialize)]
pub(crate) struct Bitflag {
    /// The yml flag-type name, snake_case.
    pub name: String,
    /// The entries in bit order, entry 0 first.
    pub entries: Vec<BitflagEntry>,
}

/// One flag entry. `value_combination` entries combine earlier entries.
#[derive(Debug, Deserialize)]
pub(crate) struct BitflagEntry {
    /// The yml entry name, snake_case.
    pub name: String,
    #[serde(default)]
    pub value_combination: Option<Vec<String>>,
}

/// One struct definition.
#[derive(Debug, Deserialize)]
pub(crate) struct Struct {
    /// The yml struct name, snake_case.
    pub name: String,
    /// `standalone`, `extensible`, or `extension` in the pinned yml.
    #[serde(rename = "type", default)]
    pub kind: String,
    /// Base descriptors accepted by an extension struct.
    #[serde(default)]
    pub extends: Vec<String>,
    /// The members in declaration order, which is the C field order.
    #[serde(default)]
    pub members: Vec<Member>,
    /// Whether webgpu.h exposes a by-value FreeMembers helper.
    #[serde(default)]
    pub free_members: bool,
}

/// One struct member.
#[derive(Debug, Deserialize)]
pub(crate) struct Member {
    /// The yml member name, snake_case.
    pub name: String,
    /// The yml type name. A prefixed form such as `struct.x` or `enum.x` names another
    /// construct.
    #[serde(rename = "type")]
    pub ty: String,
    /// The yml pointer attribute, `immutable` or `mutable`. `None` marks a by-value member.
    #[serde(default)]
    pub pointer: Option<String>,
    /// The yml marks the member optional, so it accepts NULL or an absent value.
    #[serde(default)]
    pub optional: bool,
    /// The yml default. A `constant.<name>` value names a file-level constant.
    #[serde(default)]
    pub default: Option<serde_yaml::Value>,
}

/// One callback definition.
#[derive(Debug, Deserialize)]
pub(crate) struct Callback {
    /// The yml callback name, snake_case.
    pub name: String,
    /// `callback_mode` or `immediate`.
    pub style: String,
    /// The callback arguments in declaration order.
    #[serde(default)]
    pub args: Vec<Arg>,
}

/// One freestanding function or object method.
#[derive(Debug, Deserialize)]
pub(crate) struct Function {
    /// The yml function or method name, snake_case.
    pub name: String,
    /// The return declaration. `None` marks a `void` function.
    #[serde(default)]
    pub returns: Option<Returns>,
    /// The arguments in declaration order, receiver excluded.
    #[serde(default)]
    pub args: Vec<Arg>,
    /// `callback.<name>` for async operations.
    #[serde(default)]
    pub callback: Option<String>,
}

/// A function/method return declaration.
#[derive(Debug, Deserialize)]
pub(crate) struct Returns {
    /// The yml return type, spelled like a member type.
    #[serde(rename = "type")]
    pub ty: String,
}

/// One function, method, or callback argument.
#[derive(Debug, Deserialize)]
pub(crate) struct Arg {
    /// The yml argument name, snake_case.
    pub name: String,
    /// The yml type, spelled like a member type.
    #[serde(rename = "type")]
    pub ty: String,
    /// The yml pointer attribute, `immutable` or `mutable`. `None` marks a by-value argument.
    #[serde(default)]
    pub pointer: Option<String>,
    /// The yml marks the argument optional, so the call accepts NULL.
    #[serde(default)]
    pub optional: bool,
}

/// An object: an opaque handle with methods plus implicit
/// AddRef/Release.
#[derive(Debug, Deserialize)]
pub(crate) struct Object {
    /// The yml object name, snake_case.
    pub name: String,
    /// The methods in declaration order. AddRef and Release stay implicit.
    #[serde(default)]
    pub methods: Vec<Function>,
}

impl Yml {
    /// Looks up an object by yml name.
    pub fn object(&self, name: &str) -> Option<&Object> {
        self.objects.iter().find(|o| o.name == name)
    }

    /// Looks up a freestanding function by yml name.
    pub fn function(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// Looks up an enum by yml name.
    pub fn enum_(&self, name: &str) -> Option<&Enum> {
        self.enums.iter().find(|e| e.name == name)
    }

    /// Looks up a bitflag by yml name.
    pub fn bitflag(&self, name: &str) -> Option<&Bitflag> {
        self.bitflags.iter().find(|b| b.name == name)
    }

    /// Looks up a struct by yml name.
    pub fn struct_(&self, name: &str) -> Option<&Struct> {
        self.structs.iter().find(|s| s.name == name)
    }

    /// Looks up a callback by yml name.
    pub fn callback(&self, name: &str) -> Option<&Callback> {
        self.callbacks.iter().find(|c| c.name == name)
    }

    /// Looks up a file-level constant by yml name.
    pub fn constant(&self, name: &str) -> Option<&Constant> {
        self.constants.iter().find(|c| c.name == name)
    }
}

impl Enum {
    /// The numeric value of a named entry: its index in `entries`
    /// (nulls occupy slots), per the webgpu.yml value scheme.
    pub fn value_of(&self, entry: &str) -> Result<Option<u32>, crate::policy::PolicyError> {
        self.entries
            .iter()
            .position(|value| value.as_ref().is_some_and(|value| value.name == entry))
            .map(|index| {
                u32::try_from(index).map_err(|_| {
                    crate::internal(
                        "model::Enum::value_of",
                        format!("enum `{}` entry `{entry}` index exceeds u32", self.name),
                    )
                })
            })
            .transpose()
    }
}

impl Bitflag {
    /// The numeric value of a flag entry: entry 0 is 0, entry `i` is
    /// `1 << (i - 1)`, and `value_combination` entries OR their parts.
    pub fn value_of(&self, entry: &str) -> Option<u64> {
        let (index, found) = self
            .entries
            .iter()
            .enumerate()
            .find(|(_, e)| e.name == entry)?;
        match &found.value_combination {
            Some(parts) => parts.iter().map(|p| self.value_of(p)).sum(),
            None if index == 0 => Some(0),
            None => Some(1u64 << (index - 1)),
        }
    }
}
