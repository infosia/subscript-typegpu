//! Rasterize the icon outlines and font cells, then pack them into atlas rows.

use std::path::Path;

use ab_glyph::{
    point, Font, FontRef, GlyphId, Outline, OutlineCurve, OutlinedGlyph, PxScaleFactor, ScaleFont,
};

pub(crate) const FONT_PATH: &str = "third_party/imgui/misc/fonts/Roboto-Medium.ttf";
const FONT_COMMIT: &str = "77fe4588e6af934b4167b91312174ca3fbb0d71a";
pub(crate) const WIDTH: usize = 256;
pub(crate) const SCALE: usize = 2;
pub(crate) const TEXT_HEIGHT: usize = 18;
const GAP: usize = 2;

// Vertices follow each icon's perimeter in logical pixels, with y down.
const CLOSE: &[(f32, f32)] = &[
    (3., 4.5),
    (4.5, 3.),
    (8., 6.5),
    (11.5, 3.),
    (13., 4.5),
    (9.5, 8.),
    (13., 11.5),
    (11.5, 13.),
    (8., 9.5),
    (4.5, 13.),
    (3., 11.5),
    (6.5, 8.),
];
const CHECK: &[(f32, f32)] = &[
    (2., 9.),
    (4., 7.),
    (7., 10.),
    (14., 3.),
    (16., 5.),
    (7., 14.),
];
const COLLAPSED: &[(f32, f32)] = &[(0., 0.), (5., 3.5), (0., 7.)];
const EXPANDED: &[(f32, f32)] = &[(0., 0.), (7., 0.), (3.5, 5.)];

struct Cell {
    width: usize,
    height: usize,
    texels: Vec<u8>,
}

impl Cell {
    fn new(width: usize, height: usize, value: u8) -> Self {
        Self {
            width,
            height,
            texels: vec![value; width * height],
        }
    }

    fn draw(&mut self, outline: OutlinedGlyph) {
        let origin = outline.px_bounds().min;
        outline.draw(|x, y, coverage| {
            let x = origin.x as i32 + x as i32;
            let y = origin.y as i32 + y as i32;
            if (0..self.width as i32).contains(&x) && (0..self.height as i32).contains(&y) {
                self.texels[y as usize * self.width + x as usize] =
                    (coverage.min(1.) * 15.).round() as u8;
            }
        });
    }
}

fn icon(width: usize, height: usize, vertices: &[(f32, f32)]) -> Cell {
    // Outline coordinates use y up. The baseline at y=0 maps them back into the cell.
    let points = vertices.iter().map(|&(x, y)| point(x, -y));
    let curves = points
        .clone()
        .zip(points.cycle().skip(1))
        .map(|(a, b)| OutlineCurve::Line(a, b))
        .collect();
    let bounds = ab_glyph::Rect {
        min: point(0., 0.),
        max: point(width as f32, -(height as f32)),
    };
    let scale = SCALE as f32;
    let shape = OutlinedGlyph::new(
        GlyphId(0).with_scale(scale),
        Outline { bounds, curves },
        PxScaleFactor {
            horizontal: scale,
            vertical: scale,
        },
    );
    let mut cell = Cell::new(width * SCALE, height * SCALE, 0);
    cell.draw(shape);
    cell
}

pub(crate) struct Atlas {
    pub(crate) rects: Vec<[usize; 4]>,
    pub(crate) texels: Vec<u8>,
}

impl Atlas {
    pub(crate) fn from_font(bytes: &[u8]) -> Result<Self, String> {
        let font = FontRef::try_from_slice(bytes).map_err(|e| format!("{FONT_PATH}: {e}"))?;
        let scaled = font.as_scaled((TEXT_HEIGHT * SCALE) as f32);
        let glyphs = (b' '..=b'~').map(|byte| {
            let id = scaled.glyph_id(char::from(byte));
            let width = (scaled.h_advance(id) / SCALE as f32).round() as usize;
            let mut cell = Cell::new(width * SCALE, TEXT_HEIGHT * SCALE, 0);
            let glyph = id.with_scale_and_position(scaled.scale(), point(0., scaled.ascent()));
            if let Some(outline) = scaled.outline_glyph(glyph) {
                cell.draw(outline);
            }
            cell
        });
        let symbols = [
            icon(16, 16, CLOSE),
            icon(18, 18, CHECK),
            icon(5, 7, COLLAPSED),
            icon(7, 5, EXPANDED),
            Cell::new(6, 6, 15),
        ];
        let mut rects = Vec::new();
        let mut texels = Vec::new();
        let (mut x, mut y, mut row_height) = (GAP, GAP, 0);
        for cell in symbols.into_iter().chain(glyphs) {
            if cell.width == 0 || cell.width + 2 * GAP > WIDTH {
                return Err(format!("invalid atlas cell width {}", cell.width));
            }
            if x + cell.width + GAP > WIDTH {
                x = GAP;
                y += row_height + GAP;
                row_height = 0;
            }
            row_height = row_height.max(cell.height);
            texels.resize(WIDTH * (y + row_height + GAP).next_multiple_of(4), 0);
            for (dy, row) in cell.texels.chunks_exact(cell.width).enumerate() {
                let start = (y + dy) * WIDTH + x;
                texels[start..start + cell.width].copy_from_slice(row);
            }
            rects.push([x, y, cell.width, cell.height]);
            x += cell.width + GAP;
        }
        Ok(Self { rects, texels })
    }
}

pub(crate) fn encode(texels: &[u8]) -> String {
    let mut code = String::new();
    for run in texels.chunk_by(|a, b| (*a == 0) == (*b == 0)) {
        if run[0] != 0 {
            for &value in run {
                code.push(b"0123456789abcdef"[value as usize] as char);
            }
            continue;
        }
        code.extend(std::iter::repeat_n('z', run.len() / 21));
        match run.len() % 21 {
            0 => {}
            1 => code.push('0'),
            n => code.push((b'g' + n as u8 - 2) as char),
        }
    }
    code
}

pub(crate) fn generate_ui_atlas(root: &Path) -> Result<String, String> {
    let source = root.join(FONT_PATH);
    let bytes = std::fs::read(&source).map_err(|e| format!("read {}: {e}", source.display()))?;
    let atlas = Atlas::from_font(&bytes)?;
    let height = atlas.texels.len() / WIDTH;
    let code = encode(&atlas.texels);
    let mut module = format!(
        "// Generated from {FONT_PATH} (Apache-2.0).
// Source commit: {FONT_COMMIT}.

export const UI_ATLAS_WIDTH: i32 = {WIDTH};
export const UI_ATLAS_HEIGHT: i32 = {height};
export const UI_ATLAS_SCALE: i32 = {SCALE};
export const UI_ATLAS_ALPHA_CODE: string = \"{code}\";
export const UI_ATLAS_WHITE: i32 = 4;
export const UI_ATLAS_FONT: i32 = -27;
export const UI_TEXT_HEIGHT: i32 = {TEXT_HEIGHT};
"
    );
    for (i, name) in ["X", "Y", "W", "H"].into_iter().enumerate() {
        let values = atlas
            .rects
            .iter()
            .map(|rect| rect[i].to_string())
            .collect::<Vec<_>>()
            .join(", ");
        module.push_str(&format!(
            "export const UI_ATLAS_RECT_{name}: i32[] = [{values}];\n"
        ));
    }
    module.push_str(
        "
export function uiAtlasAlpha(): u8[] {
  const bytes: u8[] = [];
  for (let i: i32 = 0; i < UI_ATLAS_ALPHA_CODE.length; i += 1) {
    const code: i32 = UI_ATLAS_ALPHA_CODE.charCodeAt(i);
    if (code >= 103) {
      const count: i32 = code - 103 + 2;
      for (let j: i32 = 0; j < count; j += 1) bytes.push(0);
    } else {
      const value: i32 = code <= 57 ? code - 48 : code - 87;
      bytes.push((value * 17) as u8);
    }
  }
  return bytes;
}
",
    );
    Ok(module)
}
