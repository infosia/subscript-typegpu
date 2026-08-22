//! Owned projections of the pinned GPUWeb IDL and generated subscript_typegpu mirror.

use std::collections::{BTreeMap, BTreeSet};

use weedle::argument::Argument;
use weedle::dictionary::DictionaryMember;
use weedle::interface::InterfaceMember;
use weedle::literal::{DefaultValue, IntegerLit};
use weedle::mixin::MixinMember;
use weedle::types::{
    FloatingPointType, IntegerType, NonAnyType, RecordKeyType, ReturnType, SingleType, Type,
};
use weedle::Definition;

use crate::idl::NamespaceConstant;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum IdlType {
    Undefined,
    Named { name: String, nullable: bool },
    Promise(Box<IdlType>),
    Boolean,
    String,
    Scalar(String),
    Sequence(Box<IdlType>),
    Record { key: String, value: Box<IdlType> },
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IdlArgument {
    pub name: String,
    pub ty: IdlType,
    pub optional: bool,
    pub default: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum IdlMemberKind {
    Attribute {
        ty: IdlType,
    },
    Operation {
        return_type: IdlType,
        arguments: Vec<IdlArgument>,
    },
    DictionaryField {
        ty: IdlType,
        required: bool,
        default: Option<String>,
    },
    NamespaceConstant {
        value: u64,
    },
    EnumValue,
    Special,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IdlMember {
    pub owner: String,
    pub name: String,
    pub kind: IdlMemberKind,
}

impl IdlMember {
    pub fn key(&self) -> String {
        format!("{}.{}", self.owner, self.name)
    }
}

#[derive(Clone, Debug, Default)]
struct InterfaceDef {
    defined: bool,
    inheritance: Option<String>,
    members: Vec<IdlMember>,
}

#[derive(Clone, Debug, Default)]
struct DictionaryDef {
    inheritance: Option<String>,
    members: Vec<IdlMember>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct IdlModel {
    interfaces: BTreeMap<String, InterfaceDef>,
    mixins: BTreeMap<String, Vec<IdlMember>>,
    includes: BTreeMap<String, Vec<String>>,
    dictionaries: BTreeMap<String, DictionaryDef>,
    namespaces: BTreeMap<String, Vec<IdlMember>>,
    enums: BTreeMap<String, Vec<IdlMember>>,
    typedefs: BTreeMap<String, IdlType>,
}

impl IdlModel {
    pub fn from_definitions(
        definitions: &[Definition<'_>],
        constants: &[NamespaceConstant],
    ) -> Result<Self, String> {
        let mut model = IdlModel::default();
        for definition in definitions {
            match definition {
                Definition::Interface(interface) => {
                    let name = interface.identifier.0;
                    let entry = model.interfaces.entry(name.to_owned()).or_default();
                    if entry.defined {
                        return Err(format!("duplicate interface definition `{name}`"));
                    }
                    entry.defined = true;
                    entry.inheritance = interface
                        .inheritance
                        .as_ref()
                        .map(|inheritance| inheritance.identifier.0.to_owned());
                    append_interface_members(&mut entry.members, name, &interface.members.body)?;
                }
                Definition::PartialInterface(interface) => {
                    let name = interface.identifier.0;
                    let entry = model.interfaces.entry(name.to_owned()).or_default();
                    append_interface_members(&mut entry.members, name, &interface.members.body)?;
                }
                Definition::InterfaceMixin(mixin) => {
                    append_mixin_members(
                        &mut model.mixins,
                        mixin.identifier.0,
                        &mixin.members.body,
                    )?;
                }
                Definition::PartialInterfaceMixin(mixin) => {
                    append_mixin_members(
                        &mut model.mixins,
                        mixin.identifier.0,
                        &mixin.members.body,
                    )?;
                }
                Definition::IncludesStatement(include) => {
                    model
                        .includes
                        .entry(include.lhs_identifier.0.to_owned())
                        .or_default()
                        .push(include.rhs_identifier.0.to_owned());
                }
                Definition::Dictionary(dictionary) => {
                    let name = dictionary.identifier.0;
                    let entry = model.dictionaries.entry(name.to_owned()).or_default();
                    if entry.inheritance.is_some() {
                        return Err(format!("duplicate dictionary definition `{name}`"));
                    }
                    entry.inheritance = dictionary
                        .inheritance
                        .as_ref()
                        .map(|inheritance| inheritance.identifier.0.to_owned());
                    append_dictionary_members(entry, name, &dictionary.members.body)?;
                }
                Definition::PartialDictionary(dictionary) => {
                    let name = dictionary.identifier.0;
                    let entry = model.dictionaries.entry(name.to_owned()).or_default();
                    append_dictionary_members(entry, name, &dictionary.members.body)?;
                }
                Definition::Enum(enum_) => {
                    let name = enum_.identifier.0;
                    let members = enum_
                        .values
                        .body
                        .list
                        .iter()
                        .map(|value| IdlMember {
                            owner: name.to_owned(),
                            name: value.value.0.to_owned(),
                            kind: IdlMemberKind::EnumValue,
                        })
                        .collect();
                    if model.enums.insert(name.to_owned(), members).is_some() {
                        return Err(format!("duplicate enum definition `{name}`"));
                    }
                }
                Definition::Typedef(typedef) => {
                    let name = typedef.identifier.0.to_owned();
                    let ty = own_type(&typedef.type_.type_);
                    if model.typedefs.insert(name.clone(), ty).is_some() {
                        return Err(format!("duplicate typedef definition `{name}`"));
                    }
                }
                _ => {}
            }
        }
        for constant in constants {
            model
                .namespaces
                .entry(constant.namespace.clone())
                .or_default()
                .push(IdlMember {
                    owner: constant.namespace.clone(),
                    name: constant.name.clone(),
                    kind: IdlMemberKind::NamespaceConstant {
                        value: constant.value,
                    },
                });
        }
        Ok(model)
    }

    pub fn interface_members(&self, name: &str) -> Result<Vec<IdlMember>, String> {
        let definition = self
            .interfaces
            .get(name)
            .ok_or_else(|| format!("unknown IDL interface `{name}`"))?;
        if !definition.defined {
            return Err(format!(
                "IDL interface `{name}` has only partial definitions"
            ));
        }
        let mut members = definition.members.clone();
        if let Some(mixins) = self.includes.get(name) {
            for mixin in mixins {
                let included = self
                    .mixins
                    .get(mixin)
                    .ok_or_else(|| format!("unknown IDL mixin `{mixin}` included by `{name}`"))?;
                members.extend(included.iter().cloned().map(|mut member| {
                    member.owner = name.to_owned();
                    member
                }));
            }
        }
        ensure_unique_interface_members(name, &members)?;
        members
            .into_iter()
            .map(|member| self.resolve_member(member))
            .collect()
    }

    pub fn interface_parent(&self, name: &str) -> Result<Option<&str>, String> {
        let definition = self
            .interfaces
            .get(name)
            .ok_or_else(|| format!("unknown IDL interface `{name}`"))?;
        if !definition.defined {
            return Err(format!(
                "IDL interface `{name}` has only partial definitions"
            ));
        }
        Ok(definition.inheritance.as_deref())
    }

    pub fn dictionary_members(&self, name: &str) -> Result<Vec<IdlMember>, String> {
        let mut visiting = BTreeSet::new();
        let mut members = Vec::new();
        self.collect_dictionary_members(name, name, &mut visiting, &mut members)?;
        ensure_unique_members(name, &members)?;
        members
            .into_iter()
            .map(|member| self.resolve_member(member))
            .collect()
    }

    fn collect_dictionary_members(
        &self,
        name: &str,
        owner: &str,
        visiting: &mut BTreeSet<String>,
        output: &mut Vec<IdlMember>,
    ) -> Result<(), String> {
        if !visiting.insert(name.to_owned()) {
            return Err(format!("IDL dictionary inheritance cycle at `{name}`"));
        }
        let definition = self
            .dictionaries
            .get(name)
            .ok_or_else(|| format!("unknown IDL dictionary `{name}`"))?;
        if let Some(parent) = &definition.inheritance {
            self.collect_dictionary_members(parent, owner, visiting, output)?;
        }
        output.extend(definition.members.iter().cloned().map(|mut member| {
            member.owner = owner.to_owned();
            member
        }));
        visiting.remove(name);
        Ok(())
    }

    pub fn namespace_members(&self, name: &str) -> Result<Vec<IdlMember>, String> {
        let members = self
            .namespaces
            .get(name)
            .ok_or_else(|| format!("unknown IDL namespace `{name}`"))?
            .clone();
        ensure_unique_members(name, &members)?;
        Ok(members)
    }

    pub fn enum_members(&self, name: &str) -> Result<Vec<IdlMember>, String> {
        let members = self
            .enums
            .get(name)
            .ok_or_else(|| format!("unknown IDL enum `{name}`"))?
            .clone();
        ensure_unique_members(name, &members)?;
        Ok(members)
    }

    fn resolve_member(&self, mut member: IdlMember) -> Result<IdlMember, String> {
        member.kind = match member.kind {
            IdlMemberKind::Attribute { ty } => IdlMemberKind::Attribute {
                ty: self.resolve_type(&ty, &mut BTreeSet::new())?,
            },
            IdlMemberKind::Operation {
                return_type,
                arguments,
            } => IdlMemberKind::Operation {
                return_type: self.resolve_type(&return_type, &mut BTreeSet::new())?,
                arguments: arguments
                    .into_iter()
                    .map(|mut argument| {
                        argument.ty = self.resolve_type(&argument.ty, &mut BTreeSet::new())?;
                        Ok(argument)
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            },
            IdlMemberKind::DictionaryField {
                ty,
                required,
                default,
            } => IdlMemberKind::DictionaryField {
                ty: self.resolve_type(&ty, &mut BTreeSet::new())?,
                required,
                default,
            },
            other => other,
        };
        Ok(member)
    }

    fn resolve_type(
        &self,
        ty: &IdlType,
        visiting: &mut BTreeSet<String>,
    ) -> Result<IdlType, String> {
        match ty {
            IdlType::Named {
                name,
                nullable: false,
            } if self.typedefs.contains_key(name) => {
                if !visiting.insert(name.clone()) {
                    return Err(format!("IDL typedef cycle at `{name}`"));
                }
                let resolved = self.resolve_type(&self.typedefs[name], visiting)?;
                visiting.remove(name);
                if matches!(
                    resolved,
                    IdlType::Scalar(_) | IdlType::Boolean | IdlType::String
                ) {
                    Ok(resolved)
                } else {
                    Ok(ty.clone())
                }
            }
            IdlType::Sequence(inner) => Ok(IdlType::Sequence(Box::new(
                self.resolve_type(inner, visiting)?,
            ))),
            IdlType::Promise(inner) => Ok(IdlType::Promise(Box::new(
                self.resolve_type(inner, visiting)?,
            ))),
            IdlType::Record { .. } => Ok(ty.clone()),
            _ => Ok(ty.clone()),
        }
    }
}

fn ensure_unique_members(owner: &str, members: &[IdlMember]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for member in members {
        if !seen.insert(member.name.clone()) {
            return Err(format!("duplicate IDL member `{owner}.{}`", member.name));
        }
    }
    Ok(())
}

fn ensure_unique_interface_members(owner: &str, members: &[IdlMember]) -> Result<(), String> {
    let mut by_name: BTreeMap<&str, Vec<&IdlMemberKind>> = BTreeMap::new();
    for member in members {
        by_name.entry(&member.name).or_default().push(&member.kind);
    }
    for (name, kinds) in by_name {
        if kinds.len() == 1 {
            continue;
        }
        if !kinds
            .iter()
            .all(|kind| matches!(kind, IdlMemberKind::Operation { .. }))
        {
            return Err(format!("duplicate IDL member `{owner}.{name}`"));
        }
        let mut signatures = BTreeSet::new();
        for kind in kinds {
            if !signatures.insert(format!("{kind:?}")) {
                return Err(format!("duplicate IDL operation overload `{owner}.{name}`"));
            }
        }
    }
    Ok(())
}

fn append_interface_members(
    output: &mut Vec<IdlMember>,
    owner: &str,
    members: &[InterfaceMember<'_>],
) -> Result<(), String> {
    for (index, member) in members.iter().enumerate() {
        let (name, kind) = match member {
            InterfaceMember::Attribute(attribute) => (
                attribute.identifier.0.to_owned(),
                IdlMemberKind::Attribute {
                    ty: own_type(&attribute.type_.type_),
                },
            ),
            InterfaceMember::Operation(operation) => {
                let name = operation
                    .identifier
                    .as_ref()
                    .map(|identifier| identifier.0.to_owned())
                    .unwrap_or_else(|| format!("@anonymous-operation-{index}"));
                (
                    name,
                    IdlMemberKind::Operation {
                        return_type: own_return_type(&operation.return_type),
                        arguments: own_arguments(&operation.args.body.list),
                    },
                )
            }
            InterfaceMember::Const(constant) => {
                (constant.identifier.0.to_owned(), IdlMemberKind::Special)
            }
            InterfaceMember::Constructor(_) => ("@constructor".to_owned(), IdlMemberKind::Special),
            InterfaceMember::Iterable(_) => ("@iterable".to_owned(), IdlMemberKind::Special),
            InterfaceMember::AsyncIterable(_) => {
                ("@async-iterable".to_owned(), IdlMemberKind::Special)
            }
            InterfaceMember::Maplike(_) => ("@maplike".to_owned(), IdlMemberKind::Special),
            InterfaceMember::Setlike(_) => ("@setlike".to_owned(), IdlMemberKind::Special),
            InterfaceMember::Stringifier(_) => ("@stringifier".to_owned(), IdlMemberKind::Special),
        };
        output.push(IdlMember {
            owner: owner.to_owned(),
            name,
            kind,
        });
    }
    Ok(())
}

fn append_mixin_members(
    mixins: &mut BTreeMap<String, Vec<IdlMember>>,
    owner: &str,
    members: &[MixinMember<'_>],
) -> Result<(), String> {
    let output = mixins.entry(owner.to_owned()).or_default();
    for (index, member) in members.iter().enumerate() {
        let (name, kind) = match member {
            MixinMember::Attribute(attribute) => (
                attribute.identifier.0.to_owned(),
                IdlMemberKind::Attribute {
                    ty: own_type(&attribute.type_.type_),
                },
            ),
            MixinMember::Operation(operation) => {
                let name = operation
                    .identifier
                    .as_ref()
                    .map(|identifier| identifier.0.to_owned())
                    .unwrap_or_else(|| format!("@anonymous-operation-{index}"));
                (
                    name,
                    IdlMemberKind::Operation {
                        return_type: own_return_type(&operation.return_type),
                        arguments: own_arguments(&operation.args.body.list),
                    },
                )
            }
            MixinMember::Const(constant) => {
                (constant.identifier.0.to_owned(), IdlMemberKind::Special)
            }
            MixinMember::Stringifier(_) => ("@stringifier".to_owned(), IdlMemberKind::Special),
        };
        output.push(IdlMember {
            owner: owner.to_owned(),
            name,
            kind,
        });
    }
    Ok(())
}

fn append_dictionary_members(
    dictionary: &mut DictionaryDef,
    owner: &str,
    members: &[DictionaryMember<'_>],
) -> Result<(), String> {
    for member in members {
        dictionary.members.push(IdlMember {
            owner: owner.to_owned(),
            name: member.identifier.0.to_owned(),
            kind: IdlMemberKind::DictionaryField {
                ty: own_type(&member.type_),
                required: member.required.is_some(),
                default: member
                    .default
                    .as_ref()
                    .map(|default| own_default(&default.value))
                    .transpose()?,
            },
        });
    }
    Ok(())
}

fn own_arguments(arguments: &[Argument<'_>]) -> Vec<IdlArgument> {
    arguments
        .iter()
        .map(|argument| match argument {
            Argument::Single(argument) => IdlArgument {
                name: argument.identifier.0.to_owned(),
                ty: own_type(&argument.type_.type_),
                optional: argument.optional.is_some(),
                default: argument
                    .default
                    .as_ref()
                    .and_then(|default| own_default(&default.value).ok()),
            },
            Argument::Variadic(argument) => IdlArgument {
                name: argument.identifier.0.to_owned(),
                ty: own_type(&argument.type_),
                optional: false,
                default: None,
            },
        })
        .collect()
}

fn own_return_type(value: &ReturnType<'_>) -> IdlType {
    match value {
        ReturnType::Undefined(_) => IdlType::Undefined,
        ReturnType::Type(value) => own_type(value),
    }
}

fn own_type(value: &Type<'_>) -> IdlType {
    match value {
        Type::Single(SingleType::Any(_)) | Type::Union(_) => IdlType::Other,
        Type::Single(SingleType::NonAny(value)) => own_non_any_type(value),
    }
}

fn own_non_any_type(value: &NonAnyType<'_>) -> IdlType {
    match value {
        NonAnyType::Promise(promise) => {
            IdlType::Promise(Box::new(own_return_type(&promise.generics.body)))
        }
        NonAnyType::Identifier(identifier) => IdlType::Named {
            name: identifier.type_.0.to_owned(),
            nullable: identifier.q_mark.is_some(),
        },
        NonAnyType::Boolean(_) => IdlType::Boolean,
        NonAnyType::Integer(integer) => {
            let scalar = match &integer.type_ {
                IntegerType::Short(value) => {
                    if value.unsigned.is_some() {
                        "u16"
                    } else {
                        "i16"
                    }
                }
                IntegerType::Long(value) => {
                    if value.unsigned.is_some() {
                        "u32"
                    } else {
                        "i32"
                    }
                }
                IntegerType::LongLong(value) => {
                    if value.unsigned.is_some() {
                        "u64"
                    } else {
                        "i64"
                    }
                }
            };
            IdlType::Scalar(scalar.to_owned())
        }
        NonAnyType::FloatingPoint(value) => IdlType::Scalar(
            match value.type_ {
                FloatingPointType::Float(_) => "f32",
                FloatingPointType::Double(_) => "f64",
            }
            .to_owned(),
        ),
        NonAnyType::Byte(_) => IdlType::Scalar("i8".to_owned()),
        NonAnyType::Octet(_) => IdlType::Scalar("u8".to_owned()),
        NonAnyType::DOMString(_) | NonAnyType::USVString(_) | NonAnyType::ByteString(_) => {
            IdlType::String
        }
        NonAnyType::Sequence(sequence) => {
            IdlType::Sequence(Box::new(own_type(&sequence.type_.generics.body)))
        }
        NonAnyType::RecordType(record) => {
            let (key, _, value) = &record.type_.generics.body;
            let key = match key.as_ref() {
                RecordKeyType::Byte(_) => "ByteString",
                RecordKeyType::DOM(_) => "DOMString",
                RecordKeyType::USV(_) => "USVString",
                RecordKeyType::NonAny(_) => "other",
            };
            IdlType::Record {
                key: key.to_owned(),
                value: Box::new(own_type(value)),
            }
        }
        NonAnyType::ArrayBuffer(buffer) => IdlType::Named {
            name: "ArrayBuffer".to_owned(),
            nullable: buffer.q_mark.is_some(),
        },
        _ => IdlType::Other,
    }
}

fn own_default(value: &DefaultValue<'_>) -> Result<String, String> {
    match value {
        DefaultValue::Boolean(value) => Ok(value.0.to_string()),
        DefaultValue::String(value) => Ok(format!("{:?}", value.0)),
        DefaultValue::EmptyArray(_) => Ok("[]".to_owned()),
        DefaultValue::EmptyDictionary(_) => Ok("{}".to_owned()),
        DefaultValue::Integer(value) => Ok(match value {
            IntegerLit::Dec(value) => value.0.to_owned(),
            IntegerLit::Hex(value) => value.0.to_owned(),
            IntegerLit::Oct(value) => value.0.to_owned(),
        }),
        _ => Err("unsupported IDL default in the selected API subset".to_owned()),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MirrorParam {
    pub name: String,
    pub ty: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MirrorFunction {
    pub name: String,
    pub params: Vec<MirrorParam>,
    pub return_type: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MirrorField {
    pub name: String,
    pub ty: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MirrorEnum {
    pub members: BTreeMap<String, i64>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct MirrorModel {
    pub handles: BTreeSet<String>,
    pub functions: BTreeMap<String, MirrorFunction>,
    pub classes: BTreeMap<String, Vec<MirrorField>>,
    pub aliases: BTreeMap<String, String>,
    pub enums: BTreeMap<String, MirrorEnum>,
}

impl MirrorModel {
    pub fn parse(source: &str) -> Result<Self, String> {
        let mut model = MirrorModel::default();
        let lines: Vec<&str> = source.lines().collect();
        let mut index = 0;
        while index < lines.len() {
            let line = lines[index].trim();
            if let Some(name) = line
                .strip_prefix("interface ")
                .and_then(|line| line.strip_suffix(" {"))
            {
                if name.starts_with("SubscriptTypegpu") {
                    model.handles.insert(name.to_owned());
                }
            } else if let Some(name) = line
                .strip_prefix("declare class ")
                .and_then(|line| line.strip_suffix(" {"))
            {
                let mut fields = Vec::new();
                index += 1;
                while index < lines.len() && lines[index].trim() != "}" {
                    let member = lines[index].trim();
                    if !member.starts_with("constructor(") {
                        if let Some((field, ty)) = member
                            .strip_suffix(';')
                            .and_then(|member| member.split_once(": "))
                        {
                            fields.push(MirrorField {
                                name: field.to_owned(),
                                ty: ty.to_owned(),
                            });
                        }
                    }
                    index += 1;
                }
                if model.classes.insert(name.to_owned(), fields).is_some() {
                    return Err(format!("duplicate mirror class `{name}`"));
                }
            } else if let Some(alias) = line.strip_prefix("type ") {
                if let Some((name, target)) = alias
                    .strip_suffix(';')
                    .and_then(|alias| alias.split_once(" = "))
                {
                    model.aliases.insert(name.to_owned(), target.to_owned());
                }
            } else if let Some(name) = line
                .strip_prefix("declare enum ")
                .and_then(|line| line.strip_suffix(" {"))
            {
                let mut members = BTreeMap::new();
                index += 1;
                while index < lines.len() && lines[index].trim() != "}" {
                    let member = lines[index].trim().strip_suffix(',').ok_or_else(|| {
                        format!("malformed mirror enum member `{}`", lines[index].trim())
                    })?;
                    let (member_name, value) = member
                        .split_once(" = ")
                        .ok_or_else(|| format!("malformed mirror enum member `{member}`"))?;
                    let value = parse_mirror_integer(value)?;
                    if members.insert(member_name.to_owned(), value).is_some() {
                        return Err(format!(
                            "duplicate mirror enum member `{name}.{member_name}`"
                        ));
                    }
                    index += 1;
                }
                if index == lines.len() {
                    return Err(format!("unterminated mirror enum `{name}`"));
                }
                if model
                    .enums
                    .insert(name.to_owned(), MirrorEnum { members })
                    .is_some()
                {
                    return Err(format!("duplicate mirror enum `{name}`"));
                }
            } else if let Some(signature) = line.strip_prefix("declare function ") {
                let signature = signature
                    .strip_suffix(';')
                    .ok_or_else(|| format!("malformed mirror function `{line}`"))?;
                let (head, return_type) = signature
                    .rsplit_once("): ")
                    .ok_or_else(|| format!("malformed mirror function `{line}`"))?;
                let (name, params) = head
                    .split_once('(')
                    .ok_or_else(|| format!("malformed mirror function `{line}`"))?;
                let params = if params.is_empty() {
                    Vec::new()
                } else {
                    params
                        .split(", ")
                        .map(|param| {
                            let (name, ty) = param.split_once(": ").ok_or_else(|| {
                                format!("malformed mirror parameter `{param}` in `{name}`")
                            })?;
                            Ok(MirrorParam {
                                name: name.to_owned(),
                                ty: ty.to_owned(),
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()?
                };
                let function = MirrorFunction {
                    name: name.to_owned(),
                    params,
                    return_type: return_type.to_owned(),
                };
                if model.functions.insert(name.to_owned(), function).is_some() {
                    return Err(format!("duplicate mirror function `{name}`"));
                }
            }
            index += 1;
        }
        Ok(model)
    }
}

fn parse_mirror_integer(value: &str) -> Result<i64, String> {
    let (negative, magnitude) = value
        .strip_prefix('-')
        .map_or((false, value), |value| (true, value));
    let magnitude = if let Some(value) = magnitude.strip_prefix("0x") {
        i64::from_str_radix(value, 16)
    } else {
        magnitude.parse()
    }
    .map_err(|_| format!("invalid mirror enum integer `{value}`"))?;
    Ok(if negative { -magnitude } else { magnitude })
}
