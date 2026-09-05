//! Atlas data for the host UI library.

use std::path::Path;

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
    let gitfile = std::fs::read_to_string(submodule.join(".git"))
        .map_err(|error| format!("read microui gitfile: {error}"))?;
    let gitdir = gitfile
        .trim()
        .strip_prefix("gitdir: ")
        .ok_or("microui gitfile lacks gitdir")?;
    let gitdir = submodule.join(gitdir);
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
    if bytes.len() > 128 * 128 {
        return Err(format!("atlas has {} bytes, expected 16384", bytes.len()));
    }
    bytes.resize(128 * 128, 0);
    let mut rects = [None; 100];
    for line in initializer(&source, "atlas[]")?
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
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
                if byte == 127 {
                    continue;
                }
                if !(32..=126).contains(&byte) {
                    return Err(format!("invalid atlas glyph {byte}"));
                }
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
        if rects[index].replace(rect).is_some() {
            return Err(format!("duplicate atlas rect {key}"));
        }
    }
    let rects = rects
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or("atlas rect is absent")?;
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    let mut output = format!("// Generated from third_party/microui/demo/atlas.inl.\n// Source commit: {commit}.\n\nexport const UI_ATLAS_WIDTH: i32 = 128;\nexport const UI_ATLAS_HEIGHT: i32 = 128;\nexport const UI_ATLAS_ALPHA_HEX: string = \"{hex}\";\nexport const UI_ATLAS_WHITE: i32 = 4;\nexport const UI_ATLAS_FONT: i32 = -27;\nexport const UI_TEXT_HEIGHT: i32 = 18;\n");
    for (column, name) in ["X", "Y", "W", "H"].iter().enumerate() {
        let values = rects
            .iter()
            .map(|rect| rect[column].to_string())
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!(
            "export const UI_ATLAS_RECT_{name}: i32[] = [{values}];\n"
        ));
    }
    output.push_str("\nexport function uiAtlasAlpha(): u8[] {\n  const bytes: u8[] = [];\n  for (let i: i32 = 0; i < UI_ATLAS_ALPHA_HEX.length; i += 2) {\n    const high: i32 = UI_ATLAS_ALPHA_HEX.charCodeAt(i);\n    const low: i32 = UI_ATLAS_ALPHA_HEX.charCodeAt(i + 1);\n    bytes.push(((high <= 57 ? high - 48 : high - 87) * 16\n      + (low <= 57 ? low - 48 : low - 87)) as u8);\n  }\n  return bytes;\n}\n");
    Ok(output)
}
