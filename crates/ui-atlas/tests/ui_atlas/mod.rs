use crate::atlas::{encode, generate_ui_atlas, Atlas, FONT_PATH, SCALE, TEXT_HEIGHT, WIDTH};
use std::{fs, path::Path, process::Command};

const MODULE: &str = "lib/typegpu-ui-atlas.generated.ts";

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
}

fn atlas() -> Atlas {
    Atlas::from_font(&fs::read(root().join(FONT_PATH)).unwrap()).unwrap()
}

#[test]
fn atlas_regeneration_is_byte_identical() {
    let generated = generate_ui_atlas(root()).unwrap();
    let committed = fs::read(root().join(MODULE)).unwrap();
    assert!(
        generated.as_bytes() == committed,
        "atlas differs from {MODULE}"
    );
}

#[test]
fn header_names_the_font_license_and_pinned_commit() {
    let entry = Command::new("git")
        .current_dir(root())
        .args(["ls-files", "--stage", "--", "third_party/imgui"])
        .output()
        .unwrap();
    assert!(entry.status.success());
    let entry = String::from_utf8(entry.stdout).unwrap();
    let pin = entry
        .strip_prefix("160000 ")
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap();
    let module = generate_ui_atlas(root()).unwrap();
    let mut header = module.lines();
    assert_eq!(
        header.next(),
        Some("// Generated from third_party/imgui/misc/fonts/Roboto-Medium.ttf (Apache-2.0).")
    );
    assert_eq!(header.next().unwrap(), format!("// Source commit: {pin}."));
}

#[test]
fn atlas_cli_writes_the_library_result() {
    let dir = std::env::temp_dir().join(format!("typegpu-ui-atlas-test-{}", std::process::id()));
    fs::create_dir_all(dir.join("lib")).unwrap();
    let font = dir.join(FONT_PATH);
    fs::create_dir_all(font.parent().unwrap()).unwrap();
    fs::copy(root().join(FONT_PATH), &font).unwrap();
    let binary = env!("CARGO_BIN_EXE_subscript-typegpu-ui-atlas");
    let result = Command::new(binary).arg(&dir).output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(fs::read(dir.join(MODULE)).unwrap() == fs::read(root().join(MODULE)).unwrap());
    for args in [vec![], vec![".", "extra"]] {
        let result = Command::new(binary)
            .current_dir(&dir)
            .args(args)
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("<repo-root>"));
    }
    fs::write(&font, b"invalid font").unwrap();
    assert!(generate_ui_atlas(&dir).is_err());
    fs::remove_file(font).unwrap();
    assert!(generate_ui_atlas(&dir).is_err());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn code_round_trips_the_quantized_raster() {
    let atlas = atlas();
    let mut decoded = Vec::new();
    for byte in encode(&atlas.texels).bytes() {
        match byte {
            b'0'..=b'9' => decoded.push(byte - b'0'),
            b'a'..=b'f' => decoded.push(byte - b'a' + 10),
            b'g'..=b'z' => decoded.resize(decoded.len() + usize::from(byte - b'g') + 2, 0),
            _ => panic!("invalid atlas code {byte}"),
        }
    }
    assert_eq!(decoded, atlas.texels);
    for (length, code) in [
        (0, ""),
        (1, "0"),
        (2, "g"),
        (20, "y"),
        (21, "z"),
        (22, "z0"),
        (23, "zg"),
        (42, "zz"),
        (43, "zz0"),
    ] {
        assert_eq!(encode(&vec![0; length]), code);
    }
    assert_eq!(encode(&(0..=15).collect::<Vec<_>>()), "0123456789abcdef");
}

#[test]
fn rect_sizes_match_the_logical_metrics() {
    let atlas = atlas();
    for (index, &[_, _, width, height]) in atlas.rects.iter().enumerate() {
        assert_eq!(width % SCALE, 0, "rect {index} width {width}");
        assert_eq!(height % SCALE, 0, "rect {index} height {height}");
    }
    // The four icons and the white cell precede the printable ASCII glyphs.
    for &[_, _, _, height] in &atlas.rects[5..] {
        assert_eq!(height, TEXT_HEIGHT * SCALE);
    }
}

#[test]
fn rects_have_two_clear_texels_on_every_side() {
    let atlas = atlas();
    let height = atlas.texels.len() / WIDTH;
    assert_eq!(atlas.rects.len(), 100);
    assert_eq!(height % 4, 0);
    let mut occupied = vec![false; atlas.texels.len()];
    for &[x, y, w, h] in &atlas.rects {
        assert!(x >= 2 && y >= 2 && x + w + 2 <= WIDTH && y + h + 2 <= height);
        for row in y..y + h {
            for used in &mut occupied[row * WIDTH + x..row * WIDTH + x + w] {
                assert!(!*used, "overlapping cells");
                *used = true;
            }
        }
    }
    for &[x, y, w, h] in &atlas.rects {
        for row in y - 2..y + h + 2 {
            for col in x - 2..x + w + 2 {
                if (x..x + w).contains(&col) && (y..y + h).contains(&row) {
                    continue;
                }
                let index = row * WIDTH + col;
                assert!(!occupied[index], "cell inside gutter at {col}, {row}");
                assert_eq!(atlas.texels[index], 0, "ink in gutter at {col}, {row}");
            }
        }
    }
    let bottom = atlas.rects.iter().map(|[_, y, _, h]| y + h).max().unwrap();
    assert_eq!(height, (bottom + 2).next_multiple_of(4));
}
