//! Narrow extraction of the WebIDL corpus embedded in the pinned GPUWeb spec.
//!
//! `weedle2` does not parse namespace constants.
//! The pre-pass accepts only the namespace constant form used by the pin.
//! A grammar change causes a named failure and requires a dependency review.

/// One constant removed from a namespace before the remaining IDL is parsed by
/// `weedle2`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamespaceConstant {
    /// Namespace identifier, such as `GPUBufferUsage`.
    pub namespace: String,
    /// Constant identifier, such as `COPY_DST`.
    pub name: String,
    /// Numeric value parsed from the required hexadecimal spelling.
    pub value: u64,
}

/// Deterministic result of extracting the pinned GPUWeb WebIDL.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtractedIdl {
    /// Number of `<script type=idl>` blocks found in the source document.
    pub block_count: usize,
    /// Number of namespace blocks consumed by the narrow pre-pass.
    pub namespace_count: usize,
    /// All namespace constants in document order.
    pub namespace_constants: Vec<NamespaceConstant>,
    /// Concatenated IDL with the captured namespace definitions removed.
    pub weedle_source: String,
}

/// Extracts every `<script type=idl>` block and removes the namespace-constant
/// syntax unsupported by `weedle2`.
pub fn extract_gpuweb_idl(document: &str) -> Result<ExtractedIdl, String> {
    const OPEN: &str = "<script type=idl>";
    const CLOSE: &str = "</script>";

    let mut rest = document;
    let mut blocks = Vec::new();
    while let Some(open_offset) = rest.find(OPEN) {
        rest = &rest[open_offset + OPEN.len()..];
        let close_offset = rest
            .find(CLOSE)
            .ok_or_else(|| "unterminated <script type=idl> block".to_owned())?;
        blocks.push(&rest[..close_offset]);
        rest = &rest[close_offset + CLOSE.len()..];
    }

    let source = blocks.join("\n");
    let lines: Vec<&str> = source.split_inclusive('\n').collect();
    let mut remainder: Vec<&str> = Vec::with_capacity(lines.len());
    let mut constants = Vec::new();
    let mut namespace_count = 0;
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        let Some(namespace) = parse_namespace_open(trimmed) else {
            remainder.push(line);
            index += 1;
            continue;
        };

        let mut namespace_constants = Vec::new();
        let mut cursor = index + 1;
        let mut closed = false;
        while cursor < lines.len() {
            let member = lines[cursor].trim();
            if member == "};" {
                closed = true;
                break;
            }
            if !member.is_empty() {
                let (name, value) = parse_namespace_constant(member).map_err(|detail| {
                    format!(
                        "namespace {namespace} exceeds the namespace-const pre-pass grammar: {detail}; fork-and-pin weedle2"
                    )
                })?;
                namespace_constants.push(NamespaceConstant {
                    namespace: namespace.to_owned(),
                    name,
                    value,
                });
            }
            cursor += 1;
        }
        if !closed {
            return Err(format!(
                "unterminated namespace {namespace}; fork-and-pin weedle2"
            ));
        }
        if namespace_constants.is_empty() {
            return Err(format!(
                "namespace {namespace} has no GPUFlagsConstant members; fork-and-pin weedle2"
            ));
        }

        while remainder
            .last()
            .is_some_and(|previous| is_extended_attribute_line(previous.trim()))
        {
            remainder.pop();
        }
        remainder.push("\n");
        constants.extend(namespace_constants);
        namespace_count += 1;
        index = cursor + 1;
    }

    Ok(ExtractedIdl {
        block_count: blocks.len(),
        namespace_count,
        namespace_constants: constants,
        weedle_source: remainder.concat(),
    })
}

fn parse_namespace_open(line: &str) -> Option<&str> {
    let namespace = line.strip_prefix("namespace ")?.strip_suffix(" {")?;
    is_identifier(namespace).then_some(namespace)
}

fn parse_namespace_constant(line: &str) -> Result<(String, u64), String> {
    let body = line
        .strip_prefix("const GPUFlagsConstant ")
        .and_then(|line| line.strip_suffix(';'))
        .ok_or_else(|| format!("unsupported member `{line}`"))?;
    let (name, value) = body
        .split_once('=')
        .ok_or_else(|| format!("missing `=` in `{line}`"))?;
    let name = name.trim();
    let value = value.trim();
    if !is_identifier(name) {
        return Err(format!("invalid constant name `{name}`"));
    }
    let digits = value
        .strip_prefix("0x")
        .ok_or_else(|| format!("non-hexadecimal constant value `{value}`"))?;
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("invalid hexadecimal constant value `{value}`"));
    }
    let value = u64::from_str_radix(digits, 16)
        .map_err(|_| format!("constant value `{value}` does not fit u64"))?;
    Ok((name.to_owned(), value))
}

fn is_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn is_extended_attribute_line(line: &str) -> bool {
    line.starts_with('[') && line.ends_with(']')
}
