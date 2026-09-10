//! Atlas data for the host UI library.

use std::path::Path;

/// Returns the body of the C array initializer that follows `name` in `source`.
///
/// The body is the text between the first `{` after `name` and the closing `};`.
///
/// # Errors
///
/// If `source` holds no such initializer, returns a message that names `name`.
fn initializer<'a>(source: &'a str, name: &str) -> Result<&'a str, String> {
    source
        .split_once(name)
        .and_then(|(_, tail)| tail.split_once('{'))
        .and_then(|(_, tail)| tail.split_once("};"))
        .map(|(body, _)| body)
        .ok_or_else(|| format!("atlas initializer {name} is absent"))
}

/// Read the pinned atlas and return its subscript module.
pub fn generate_ui_atlas(root: &Path) -> Result<String, String> {
    let submodule = root.join("third_party/microui");
    let gitpath = submodule.join(".git");
    // A submodule checkout carries `.git` as a file that points at the real directory. The module
    // header names the pinned commit, so the generator resolves both shapes.
    let gitdir = if gitpath.is_dir() {
        gitpath
    } else {
        let gitfile = std::fs::read_to_string(&gitpath)
            .map_err(|error| format!("read microui gitfile: {error}"))?;
        let gitdir = gitfile
            .trim()
            .strip_prefix("gitdir:")
            .ok_or("microui gitfile lacks gitdir")?
            .trim();
        submodule.join(gitdir)
    };
    let head = std::fs::read_to_string(gitdir.join("HEAD"))
        .map_err(|error| format!("read microui HEAD: {error}"))?;
    let commit = if let Some(reference) = head.trim().strip_prefix("ref: ") {
        std::fs::read_to_string(gitdir.join(reference))
            .map_err(|error| format!("read microui ref {reference}: {error}"))?
    } else {
        head
    };
    let commit = commit.trim();
    if commit.len() != 40 || !commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("microui HEAD does not resolve to a commit hash".into());
    }
    let path = root.join("third_party/microui/demo/atlas.inl");
    let source = std::fs::read_to_string(&path)
        .map_err(|error| format!("read {}: {error}", path.display()))?;
    let mut bytes = initializer(&source, "atlas_texture[")?
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(|token| {
            token
                .strip_prefix("0x")
                .and_then(|hex| u8::from_str_radix(hex, 16).ok())
                .ok_or_else(|| format!("invalid atlas byte {token}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    // The C initializer names fewer bytes than the array holds, and C zero-fills the rest. The
    // generator zero-fills the same way and rejects an initializer with more bytes (UI2).
    if bytes.len() > 128 * 128 {
        return Err(format!("atlas has {} bytes, expected 16384", bytes.len()));
    }
    bytes.resize(128 * 128, 0);
    let mut rects = [None; 100];
    for (line_index, line) in initializer(&source, "atlas[]")?
        .lines()
        .enumerate()
        .map(|(index, line)| (index, line.trim()))
        .filter(|(_, line)| !line.is_empty())
    {
        let (key, values) = line.split_once(']').ok_or("invalid atlas rect key")?;
        let key = key.trim_start_matches('[').trim();
        let index = match key {
            "MU_ICON_CLOSE" => 0,
            "MU_ICON_CHECK" => 1,
            "MU_ICON_COLLAPSED" => 2,
            "MU_ICON_EXPANDED" => 3,
            "ATLAS_WHITE" => 4,
            _ => {
                let byte = key
                    .strip_prefix("ATLAS_FONT+")
                    .ok_or_else(|| format!("unknown atlas key {key}"))?
                    .parse::<usize>()
                    .map_err(|error| format!("atlas glyph: {error}"))?;
                // microui's table carries a rect for byte 127 that the module does not export.
                if byte == 127 {
                    continue;
                }
                if !(32..=126).contains(&byte) {
                    return Err(format!("invalid atlas glyph {byte}"));
                }
                // Byte 32 is the first glyph and lands at index 5. `UI_ATLAS_FONT` exports the
                // same shift with the opposite sign, so a reader indexes by byte (UI2).
                byte - 27
            }
        };
        let body = values
            .split_once('{')
            .and_then(|(_, s)| s.split_once('}'))
            .map(|(s, _)| s)
            .ok_or("invalid atlas rect")?;
        let values = body
            .split(',')
            .map(|s| {
                s.trim()
                    .parse::<i32>()
                    .map_err(|error| format!("atlas rect: {error}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let rect: [i32; 4] = values
            .try_into()
            .map_err(|_| "atlas rect needs four values")?;
        // Icon indices are 0 through 4. Valid glyph indices are 5 through 99.
        if rects
            .get_mut(index)
            .ok_or_else(|| {
                crate::internal(
                    "ui_atlas::generate_ui_atlas",
                    format!("missing rect {index}"),
                    &subscript_compiler::Pos::new(
                        "third_party/microui/demo/atlas.inl",
                        line_index as u32 + 1,
                        1,
                    ),
                )
                .message
            })?
            .replace(rect)
            .is_some()
        {
            return Err(format!("duplicate atlas rect {key}"));
        }
    }
    let rects = rects
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or("atlas rect is absent")?;
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    let mut output = format!("// Generated from third_party/microui/demo/atlas.inl.\n// Source commit: {commit}.\n\nexport const UI_ATLAS_WIDTH: i32 = 128;\nexport const UI_ATLAS_HEIGHT: i32 = 128;\nexport const UI_ATLAS_ALPHA_HEX: string = \"{hex}\";\nexport const UI_ATLAS_WHITE: i32 = 4;\nexport const UI_ATLAS_FONT: i32 = -27;\nexport const UI_TEXT_HEIGHT: i32 = 18;\n");
    let columns = rects.iter().fold(
        [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
        |mut columns, rect| {
            for (column, value) in columns.iter_mut().zip(rect) {
                column.push(value.to_string());
            }
            columns
        },
    );
    for (values, name) in columns.iter().zip(["X", "Y", "W", "H"]) {
        let values = values.join(", ");
        output.push_str(&format!(
            "export const UI_ATLAS_RECT_{name}: i32[] = [{values}];\n"
        ));
    }
    output.push_str("\nexport function uiAtlasAlpha(): u8[] {\n  const bytes: u8[] = [];\n  for (let i: i32 = 0; i < UI_ATLAS_ALPHA_HEX.length; i += 2) {\n    const high: i32 = UI_ATLAS_ALPHA_HEX.charCodeAt(i);\n    const low: i32 = UI_ATLAS_ALPHA_HEX.charCodeAt(i + 1);\n    bytes.push(((high <= 57 ? high - 48 : high - 87) * 16\n      + (low <= 57 ? low - 48 : low - 87)) as u8);\n  }\n  return bytes;\n}\n");
    Ok(output)
}
