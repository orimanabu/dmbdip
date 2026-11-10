use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use ab_glyph::PxScale;
use image::{Rgb, RgbImage};
use crate::fonts::{draw_text_mut, text_size};

use crate::fonts::Fonts;
use crate::theme::Theme;

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub(crate) struct MermaidCacheKey {
    pub(crate) source_hash: u64,
    pub(crate) target_width: u32,
    pub(crate) dark: bool,
}

pub(crate) struct MermaidCache {
    entries: HashMap<MermaidCacheKey, Arc<RgbImage>>,
}

impl MermaidCache {
    pub(crate) fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    #[allow(dead_code)]
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn get_or_render(
        &mut self,
        source: &str,
        target_width: u32,
        dark: bool,
        fonts: &Fonts,
        theme: &Theme,
    ) -> Arc<RgbImage> {
        let key = MermaidCacheKey {
            source_hash: hash_source(source),
            target_width,
            dark,
        };
        if let Some(cached) = self.entries.get(&key) {
            return Arc::clone(cached);
        }
        let image = match render_mermaid_svg(source, dark) {
            Ok(svg) => match rasterize_svg_to_rgb(&svg, target_width, theme.bg) {
                Ok(img) => img,
                Err(msg) => render_error_image(&msg, target_width, fonts, theme),
            },
            Err(msg) => render_error_image(&msg, target_width, fonts, theme),
        };
        let arc = Arc::new(image);
        self.entries.insert(key, Arc::clone(&arc));
        arc
    }
}

pub(crate) fn is_dark_bg(bg: Rgb<u8>) -> bool {
    let [r, g, b] = bg.0;
    (r as u32 + g as u32 + b as u32) < 3 * 128
}

pub(crate) fn hash_source(source: &str) -> u64 {
    let mut h = DefaultHasher::new();
    source.hash(&mut h);
    h.finish()
}

fn mermaid_theme_for(dark: bool) -> mermaid_rs_renderer::Theme {
    let mut t = mermaid_rs_renderer::Theme::modern();
    if dark {
        t.background = "#1e1e1e".to_string();
        t.primary_color = "#264f78".to_string();
        t.primary_text_color = "#d4d4d4".to_string();
        t.primary_border_color = "#569cd6".to_string();
        t.secondary_color = "#3c3c3c".to_string();
        t.tertiary_color = "#2d2d30".to_string();
        t.line_color = "#7f7f7f".to_string();
        t.text_color = "#d4d4d4".to_string();
        t.edge_label_background = "#1e1e1e".to_string();
        t.cluster_background = "#2d2d30".to_string();
        t.cluster_border = "#569cd6".to_string();
    }
    t
}

pub(crate) fn render_mermaid_svg(source: &str, dark: bool) -> Result<String, String> {
    let mut opts = mermaid_rs_renderer::RenderOptions::modern();
    opts.theme = mermaid_theme_for(dark);
    mermaid_rs_renderer::render_with_options(source, opts)
        .map_err(|e| e.to_string())
}

pub(crate) fn rasterize_svg_to_rgb(
    svg: &str,
    max_width: u32,
    bg: Rgb<u8>,
) -> Result<RgbImage, String> {
    let mut opt = usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();

    let tree = usvg::Tree::from_str(svg, &opt).map_err(|e| e.to_string())?;

    let size = tree.size();
    let intrinsic_w = size.width();
    let intrinsic_h = size.height();
    if intrinsic_w <= 0.0 || intrinsic_h <= 0.0 {
        return Err("SVG has zero intrinsic size".into());
    }

    let scale = if intrinsic_w > max_width as f32 {
        max_width as f32 / intrinsic_w
    } else {
        1.0
    };
    let out_w = (intrinsic_w * scale).round().max(1.0) as u32;
    let out_h = (intrinsic_h * scale).round().max(1.0) as u32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(out_w, out_h)
        .ok_or_else(|| "failed to allocate pixmap".to_string())?;
    pixmap.fill(resvg::tiny_skia::Color::from_rgba8(bg.0[0], bg.0[1], bg.0[2], 255));

    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // The pixmap was pre-filled with opaque `bg` before rendering, so every pixel
    // has alpha=255 (premultiplied RGB == straight RGB). Copy the channels directly.
    let mut img = RgbImage::new(out_w, out_h);
    for (dst, px) in img.pixels_mut().zip(pixmap.pixels().iter()) {
        *dst = Rgb([px.red(), px.green(), px.blue()]);
    }

    Ok(img)
}

pub(crate) fn render_error_image(
    msg: &str,
    max_width: u32,
    fonts: &Fonts,
    theme: &Theme,
) -> RgbImage {
    let scale = PxScale::from(theme.body_size);
    let pad: u32 = 8;
    let line_h = (theme.body_size * 1.4) as u32;
    let full = format!("Mermaid error: {}", msg);

    // Measure; clamp width to max_width - 2*pad.
    let inner_max = max_width.saturating_sub(pad * 2).max(1);
    let (text_w, _) = text_size(scale, &fonts.mono, &full);
    let width = (text_w.min(inner_max) + pad * 2).min(max_width).max(1);
    let height = line_h + pad * 2;

    let mut img = RgbImage::from_pixel(width, height, theme.bg);
    draw_text_mut(
        &mut img,
        theme.code_color,
        pad as i32,
        pad as i32,
        scale,
        &fonts.mono,
        &full,
    );
    img
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::test_fonts;
    use crate::theme::default_theme;

    #[test]
    fn cache_new_is_empty() {
        let cache = MermaidCache::new();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn dark_bg_detection() {
        assert!(is_dark_bg(Rgb([0, 0, 0])));
        assert!(is_dark_bg(Rgb([30, 30, 30])));
        assert!(!is_dark_bg(Rgb([200, 200, 200])));
        assert!(!is_dark_bg(Rgb([255, 255, 255])));
    }

    #[test]
    fn hash_source_is_deterministic() {
        let a = hash_source("flowchart LR; A-->B");
        let b = hash_source("flowchart LR; A-->B");
        assert_eq!(a, b);
    }

    #[test]
    fn hash_source_differs_on_different_input() {
        let a = hash_source("flowchart LR; A-->B");
        let b = hash_source("flowchart LR; A-->C");
        assert_ne!(a, b);
    }

    #[test]
    fn render_svg_valid_diagram_returns_svg_string() {
        let result = render_mermaid_svg("flowchart LR\n    A --> B", false);
        assert!(result.is_ok(), "expected Ok, got: {:?}", result);
        let svg = result.unwrap();
        assert!(svg.contains("<svg"), "output should contain <svg tag");
    }

    #[test]
    fn render_svg_dark_palette_affects_output() {
        let light = render_mermaid_svg("flowchart LR\n    A --> B", false).unwrap();
        let dark = render_mermaid_svg("flowchart LR\n    A --> B", true).unwrap();
        // Different theme palettes produce different SVG bytes.
        assert_ne!(light, dark, "dark and light SVGs should differ");
    }

    const TINY_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50" viewBox="0 0 100 50"><rect width="100" height="50" fill="red"/></svg>"#;

    #[test]
    fn rasterize_small_svg_at_intrinsic_size() {
        let bg = Rgb([255, 255, 255]);
        let img = rasterize_svg_to_rgb(TINY_SVG, 800, bg).unwrap();
        // Interior pixel should be fully red (fill="red" on a 100x50 rect with white bg).
        let p = img.get_pixel(50, 25);
        assert_eq!(*p, Rgb([255, 0, 0]), "center pixel should be red, got {:?}", p);
        assert_eq!(img.width(), 100);
        assert_eq!(img.height(), 50);
    }

    #[test]
    fn rasterize_scales_down_when_wider_than_max() {
        let bg = Rgb([255, 255, 255]);
        let img = rasterize_svg_to_rgb(TINY_SVG, 50, bg).unwrap();
        assert_eq!(img.width(), 50);
        assert_eq!(img.height(), 25);
    }

    #[test]
    fn rasterize_never_scales_up() {
        let bg = Rgb([255, 255, 255]);
        let img = rasterize_svg_to_rgb(TINY_SVG, 5000, bg).unwrap();
        // SVG is 100 wide; max_width 5000 should NOT enlarge it.
        assert_eq!(img.width(), 100);
        assert_eq!(img.height(), 50);
    }

    #[test]
    fn rasterize_invalid_svg_returns_err() {
        let bg = Rgb([255, 255, 255]);
        let result = rasterize_svg_to_rgb("not-an-svg", 800, bg);
        assert!(result.is_err());
    }

    #[test]
    fn error_image_is_nonempty_and_respects_max_width() {
        let fonts = test_fonts();
        let theme = default_theme();
        let img = render_error_image("boom", 200, &fonts, &theme);
        assert!(img.width() > 0);
        assert!(img.height() > 0);
        assert!(img.width() <= 200);
    }

    #[test]
    fn cache_hit_returns_pointer_equal_arc() {
        let fonts = test_fonts();
        let theme = default_theme();
        let mut cache = MermaidCache::new();
        let a = cache.get_or_render("flowchart LR\n    A --> B", 400, false, &fonts, &theme);
        let b = cache.get_or_render("flowchart LR\n    A --> B", 400, false, &fonts, &theme);
        assert!(Arc::ptr_eq(&a, &b), "second call must return cached Arc");
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn cache_key_distinguishes_width() {
        let fonts = test_fonts();
        let theme = default_theme();
        let mut cache = MermaidCache::new();
        cache.get_or_render("flowchart LR\n    A --> B", 400, false, &fonts, &theme);
        cache.get_or_render("flowchart LR\n    A --> B", 600, false, &fonts, &theme);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn cache_key_distinguishes_dark_flag() {
        let fonts = test_fonts();
        let theme = default_theme();
        let mut cache = MermaidCache::new();
        cache.get_or_render("flowchart LR\n    A --> B", 400, false, &fonts, &theme);
        cache.get_or_render("flowchart LR\n    A --> B", 400, true, &fonts, &theme);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn junk_source_returns_usable_image_without_panic() {
        let fonts = test_fonts();
        let theme = default_theme();
        let mut cache = MermaidCache::new();
        let img = cache.get_or_render("@@@ bogus @@@", 400, false, &fonts, &theme);
        assert!(img.width() > 0);
        assert!(img.height() > 0);
    }

    #[test]
    fn clear_empties_the_cache() {
        let fonts = test_fonts();
        let theme = default_theme();
        let mut cache = MermaidCache::new();
        cache.get_or_render("flowchart LR\n    A --> B", 400, false, &fonts, &theme);
        assert_eq!(cache.len(), 1);
        cache.clear();
        assert_eq!(cache.len(), 0);
    }
}
