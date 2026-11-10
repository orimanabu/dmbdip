use std::sync::Arc;

use ab_glyph::{point, Font, FontVec, GlyphId, OutlinedGlyph, PxScale, ScaleFont, VariableFont};
use font_kit::family_name::FamilyName;
use font_kit::handle::Handle;
use font_kit::properties::{Properties, Style, Weight};
use font_kit::source::SystemSource;
use image::{Rgb, RgbImage};

use crate::config::FontsConfig;

/// Families tried (in order) as fallback for glyphs missing from the primary
/// fonts, mainly to cover CJK text. The first one found on the system is used.
const DEFAULT_FALLBACK_FAMILIES: &[&str] = &[
    "Noto Sans CJK JP",
    "Noto Sans JP",
    "Source Han Sans JP",
    "Source Han Sans",
    "Hiragino Sans",
    "Hiragino Kaku Gothic ProN",
    "IPAexGothic",
    "IPAGothic",
    "TakaoGothic",
    "VL Gothic",
    "Noto Sans CJK SC",
    "Noto Sans CJK TC",
    "Noto Sans CJK KR",
    "PingFang SC",
    "Microsoft YaHei",
    "Yu Gothic",
    "Meiryo",
    "Droid Sans Fallback",
];

/// A primary font plus fallback fonts consulted, per character, for glyphs the
/// primary font lacks.
pub(crate) struct FontFace {
    primary: FontVec,
    fallbacks: Vec<Arc<FontVec>>,
}

/// ab_glyph's `PxScale` is relative to the font's ascent-descent height, which
/// differs between fonts. This ratio lets fallback glyphs match the primary
/// font's em size.
fn height_per_em(font: &FontVec) -> f32 {
    font.height_unscaled() / font.units_per_em().unwrap_or(1000.0)
}

impl FontFace {
    fn resolve(&self, c: char) -> (&FontVec, GlyphId) {
        let id = self.primary.glyph_id(c);
        if id.0 != 0 {
            return (&self.primary, id);
        }
        for fb in &self.fallbacks {
            let fid = fb.glyph_id(c);
            if fid.0 != 0 {
                return (fb, fid);
            }
        }
        (&self.primary, id)
    }

    /// Lays out `text` on a single line, calling `f` with each outlined glyph
    /// (positioned relative to the text origin). Returns (width, height), with
    /// the same conventions as `imageproc::drawing::text_size`.
    fn layout(&self, scale: PxScale, text: &str, mut f: impl FnMut(OutlinedGlyph)) -> (u32, u32) {
        if text.is_empty() {
            return (0, 0);
        }
        let primary = self.primary.as_scaled(scale);
        let ascent = primary.ascent();
        let primary_ratio = height_per_em(&self.primary);

        let mut w = 0.0f32;
        let mut prev: Option<(*const FontVec, GlyphId)> = None;

        for c in text.chars() {
            let (font, id) = self.resolve(c);
            let font_scale = if std::ptr::eq(font, &self.primary) {
                scale
            } else {
                let k = height_per_em(font) / primary_ratio;
                PxScale { x: scale.x * k, y: scale.y * k }
            };
            let scaled = font.as_scaled(font_scale);
            if let Some((pf, pid)) = prev
                && std::ptr::eq(pf, font)
            {
                w += scaled.kern(pid, id);
            }
            let glyph = id.with_scale_and_position(font_scale, point(w, ascent));
            w += scaled.h_advance(id);
            prev = Some((font as *const FontVec, id));
            if let Some(g) = font.outline_glyph(glyph) {
                f(g);
            }
        }

        let w = w.max(0.0).ceil();
        let h = primary.height().ceil().max(0.0);
        (1 + w as u32, h as u32)
    }
}

/// Width and height of `text` rendered with `face` at `scale`.
pub(crate) fn text_size(scale: PxScale, face: &FontFace, text: &str) -> (u32, u32) {
    face.layout(scale, text, |_| {})
}

/// Draws `text` at (x, y) (top-left of the line box), using fallback fonts for
/// characters missing from the primary font.
pub(crate) fn draw_text_mut(
    img: &mut RgbImage,
    color: Rgb<u8>,
    x: i32,
    y: i32,
    scale: PxScale,
    face: &FontFace,
    text: &str,
) {
    let (iw, ih) = (img.width() as i32, img.height() as i32);
    face.layout(scale, text, |g| {
        let bb = g.px_bounds();
        let x0 = x + bb.min.x.round() as i32;
        let y0 = y + bb.min.y.round() as i32;
        g.draw(|gx, gy, cov| {
            let px = x0 + gx as i32;
            let py = y0 + gy as i32;
            if px < 0 || py < 0 || px >= iw || py >= ih {
                return;
            }
            let a = cov.clamp(0.0, 1.0);
            let p = img.get_pixel_mut(px as u32, py as u32);
            for i in 0..3 {
                let v = p.0[i] as f32 * (1.0 - a) + color.0[i] as f32 * a;
                p.0[i] = v.round().clamp(0.0, 255.0) as u8;
            }
        });
    });
}

pub(crate) struct Fonts {
    pub(crate) regular: FontFace,
    pub(crate) bold: FontFace,
    pub(crate) italic: FontFace,
    pub(crate) mono: FontFace,
}

/// Loads the font behind `handle`. Fontconfig encodes the named instance of
/// variable fonts in the upper 16 bits of the face index, so only the low bits
/// are passed to ab_glyph; the weight axis is then set explicitly.
fn load_handle(handle: &Handle, weight: Weight) -> Result<FontVec, String> {
    let font = handle.load().map_err(|e| e.to_string())?;
    let data = font.copy_font_data().ok_or("could not read font data")?;
    let index = match handle {
        Handle::Path { font_index, .. } | Handle::Memory { font_index, .. } => *font_index,
    };
    let data = Arc::try_unwrap(data).unwrap_or_else(|a| (*a).clone());
    let mut fv = FontVec::try_from_vec_and_index(data, index & 0xFFFF).map_err(|e| e.to_string())?;
    fv.set_variation(b"wght", weight.0);
    Ok(fv)
}

fn load_font(
    source: &SystemSource,
    families: &[FamilyName],
    properties: &Properties,
    name: &str,
) -> FontVec {
    let handle = match source.select_best_match(families, properties) {
        Ok(h) => h,
        Err(_) => {
            eprintln!(
                "Error: Could not find {} font.\n\
                 Install DejaVu fonts or ensure fontconfig is available.\n\
                 On Debian/Ubuntu: sudo apt install fonts-dejavu\n\
                 On Arch: sudo pacman -S ttf-dejavu\n\
                 On macOS: brew install font-dejavu",
                name
            );
            std::process::exit(1);
        }
    };
    match load_handle(&handle, properties.weight) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: Could not load {} font: {}", name, e);
            std::process::exit(1);
        }
    }
}

/// Loads fallback fonts. With an explicit list, every family found is used (in
/// order); otherwise the first available default family is used.
fn load_fallbacks(source: &SystemSource, configured: Option<&[String]>, weight: Weight) -> Vec<Arc<FontVec>> {
    let props = *Properties::new().weight(weight);
    let load = |name: &str| {
        let family = [FamilyName::Title(name.to_string())];
        source
            .select_best_match(&family, &props)
            .ok()
            .and_then(|h| load_handle(&h, weight).ok())
            .map(Arc::new)
    };
    match configured {
        Some(names) => names.iter().filter_map(|n| load(n)).collect(),
        None => DEFAULT_FALLBACK_FAMILIES
            .iter()
            .find_map(|n| load(n))
            .into_iter()
            .collect(),
    }
}

pub(crate) fn load_fonts(font_config: Option<&FontsConfig>) -> Fonts {
    let source = SystemSource::new();

    let sans_name = font_config
        .and_then(|c| c.sans.as_deref())
        .unwrap_or("DejaVu Sans");
    let mono_name = font_config
        .and_then(|c| c.mono.as_deref())
        .unwrap_or("DejaVu Sans Mono");
    let fallback_names = font_config.and_then(|c| c.fallback.as_deref());

    let sans_families = &[
        FamilyName::Title(sans_name.to_string()),
        FamilyName::SansSerif,
    ];
    let mono_families = &[
        FamilyName::Title(mono_name.to_string()),
        FamilyName::Monospace,
    ];

    let fb_regular = load_fallbacks(&source, fallback_names, Weight::NORMAL);
    let fb_bold = load_fallbacks(&source, fallback_names, Weight::BOLD);
    let face = |primary: FontVec, fallbacks: &Vec<Arc<FontVec>>| FontFace {
        primary,
        fallbacks: fallbacks.clone(),
    };

    Fonts {
        regular: face(
            load_font(&source, sans_families, &Properties::new(), sans_name),
            &fb_regular,
        ),
        bold: face(
            load_font(
                &source,
                sans_families,
                Properties::new().weight(Weight::BOLD),
                &format!("{} Bold", sans_name),
            ),
            &fb_bold,
        ),
        italic: face(
            load_font(
                &source,
                sans_families,
                Properties::new().style(Style::Italic),
                &format!("{} Italic", sans_name),
            ),
            &fb_regular,
        ),
        mono: face(
            load_font(&source, mono_families, &Properties::new(), mono_name),
            &fb_regular,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::test_fonts;

    #[test]
    fn missing_glyphs_use_fallback_with_matching_size() {
        let fonts = test_fonts();
        if fonts.regular.fallbacks.is_empty() {
            eprintln!("no CJK fallback font installed; skipping");
            return;
        }
        let (f, id) = fonts.regular.resolve('日');
        assert_ne!(id.0, 0);
        assert!(!std::ptr::eq(f, &fonts.regular.primary));
        // A CJK ideograph is roughly one em wide.
        let scale = PxScale::from(20.0);
        let em = scale.y / height_per_em(&fonts.regular.primary);
        let w = text_size(scale, &fonts.regular, "日日日日日日日日日日").0 as f32 / 10.0;
        assert!((w - em).abs() < 2.0, "width {w} vs em {em}");
    }

    #[test]
    fn primary_glyphs_do_not_use_fallback() {
        let fonts = test_fonts();
        let (f, _) = fonts.regular.resolve('a');
        assert!(std::ptr::eq(f, &fonts.regular.primary));
    }
}
