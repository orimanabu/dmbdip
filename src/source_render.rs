use ab_glyph::PxScale;
use image::{Rgb, RgbImage};
use crate::fonts::{draw_text_mut, text_size};
use std::sync::OnceLock;
use syntect::highlighting::{ThemeSet, Style as SynStyle};
use syntect::parsing::SyntaxSet;
use syntect::easy::HighlightLines;

use crate::constants::LayoutParams;
use crate::fonts::Fonts;
use crate::theme::Theme;

fn syntax_set() -> &'static SyntaxSet {
    static SS: OnceLock<SyntaxSet> = OnceLock::new();
    SS.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn theme_set() -> &'static ThemeSet {
    static TS: OnceLock<ThemeSet> = OnceLock::new();
    TS.get_or_init(ThemeSet::load_defaults)
}

const LINE_NUM_MARGIN: u32 = 10;
const LEFT_PAD: u32 = 12;
const SYNTECT_THEME: &str = "base16-ocean.dark";

fn line_number_width(total_lines: usize, fonts: &Fonts, scale: PxScale) -> u32 {
    let digits = format!("{}", total_lines);
    text_size(scale, &fonts.mono, &digits).0 + LINE_NUM_MARGIN * 2
}

struct HighlightedToken {
    text: String,
    color: Rgb<u8>,
}

fn highlight_line<'a>(
    line: &'a str,
    highlighter: &mut HighlightLines,
    syntax_set: &SyntaxSet,
) -> Vec<HighlightedToken> {
    match highlighter.highlight_line(line, syntax_set) {
        Ok(ranges) => ranges
            .iter()
            .map(|(style, text)| HighlightedToken {
                text: text.to_string(),
                color: syn_color_to_rgb(style),
            })
            .collect(),
        Err(_) => vec![HighlightedToken {
            text: line.to_string(),
            color: Rgb([220, 220, 220]),
        }],
    }
}

fn syn_color_to_rgb(style: &SynStyle) -> Rgb<u8> {
    Rgb([style.foreground.r, style.foreground.g, style.foreground.b])
}

/// Render a full source file to an image with syntax highlighting and line numbers.
/// Returns (image, total_content_height).
pub(crate) fn render_source(
    source: &str,
    extension: &str,
    width: u32,
    fonts: &Fonts,
    theme: &Theme,
    layout: &LayoutParams,
) -> (RgbImage, u32) {
    let ss = syntax_set();
    let ts = theme_set();
    let syntax = ss
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| ss.find_syntax_plain_text());
    let syn_theme = &ts.themes[SYNTECT_THEME];
    let mut highlighter = HighlightLines::new(syntax, syn_theme);

    let font_size = theme.body_size;
    let scale = PxScale::from(font_size);
    let line_height = (font_size * 1.4) as u32;
    let lines: Vec<&str> = source.lines().collect();
    let total_lines = lines.len().max(1);

    let content_width = (width - layout.margin_left - layout.margin_right).min(layout.max_content_width);
    let margin_left = (width - content_width) / 2;
    let ln_width = line_number_width(total_lines, fonts, scale);
    let code_x = margin_left + ln_width + LEFT_PAD;

    let total_height = (total_lines as u32 * line_height) + layout.paragraph_gap * 2;

    let mut img = RgbImage::from_pixel(width, total_height.max(1), theme.code_bg);
    let dim_color = Rgb([100, 100, 120]);
    let mut y = layout.paragraph_gap;

    for (i, line) in lines.iter().enumerate() {
        // Line number
        let ln_str = format!("{}", i + 1);
        let ln_text_w = text_size(scale, &fonts.mono, &ln_str).0;
        let ln_x = margin_left + ln_width - ln_text_w - LINE_NUM_MARGIN;
        draw_text_mut(&mut img, dim_color, ln_x as i32, y as i32, scale, &fonts.mono, &ln_str);

        // Highlighted tokens
        let tokens = highlight_line(line, &mut highlighter, ss);
        let mut x = code_x;
        for token in &tokens {
            if token.text.is_empty() {
                continue;
            }
            draw_text_mut(&mut img, token.color, x as i32, y as i32, scale, &fonts.mono, &token.text);
            x += text_size(scale, &fonts.mono, &token.text).0;
        }

        y += line_height;
    }

    (img, total_height)
}

/// Render a preview of a source file (capped at max_height).
pub(crate) fn render_source_preview(
    source: &str,
    extension: &str,
    width: u32,
    max_height: u32,
    fonts: &Fonts,
    theme: &Theme,
    layout: &LayoutParams,
) -> RgbImage {
    let ss = syntax_set();
    let ts = theme_set();
    let syntax = ss
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| ss.find_syntax_plain_text());
    let syn_theme = &ts.themes[SYNTECT_THEME];
    let mut highlighter = HighlightLines::new(syntax, syn_theme);

    let font_size = theme.body_size;
    let scale = PxScale::from(font_size);
    let line_height = (font_size * 1.4) as u32;
    let lines: Vec<&str> = source.lines().collect();
    let total_lines = lines.len().max(1);

    let margin_left = 10u32;
    let ln_width = line_number_width(total_lines, fonts, scale);
    let code_x = margin_left + ln_width + LEFT_PAD;

    let mut img = RgbImage::from_pixel(width, max_height, theme.code_bg);
    let dim_color = Rgb([100, 100, 120]);
    let mut y = layout.paragraph_gap;

    for (i, line) in lines.iter().enumerate() {
        if y + line_height > max_height {
            break;
        }

        let ln_str = format!("{}", i + 1);
        let ln_text_w = text_size(scale, &fonts.mono, &ln_str).0;
        let ln_x = margin_left + ln_width - ln_text_w - LINE_NUM_MARGIN;
        draw_text_mut(&mut img, dim_color, ln_x as i32, y as i32, scale, &fonts.mono, &ln_str);

        let tokens = highlight_line(line, &mut highlighter, ss);
        let mut x = code_x;
        for token in &tokens {
            if token.text.is_empty() {
                continue;
            }
            draw_text_mut(&mut img, token.color, x as i32, y as i32, scale, &fonts.mono, &token.text);
            x += text_size(scale, &fonts.mono, &token.text).0;
        }

        y += line_height;
    }

    img
}
