use ab_glyph::PxScale;
use image::RgbImage;
use imageproc::drawing::draw_filled_rect_mut;
use crate::fonts::{draw_text_mut, text_size};
use imageproc::rect::Rect;

use crate::fonts::Fonts;
use crate::theme::Theme;
use crate::types::{Span, SpanStyle};

pub(crate) fn spans_to_plain(spans: &[Span]) -> String {
    spans.iter().map(|s| s.text.as_str()).collect()
}

fn split_preserving_indent(text: &str) -> (usize, Vec<&str>) {
    let leading = text.len() - text.trim_start().len();
    let words: Vec<&str> = text.split_whitespace().collect();
    (leading, words)
}

/// Characters of scripts written without spaces between words (Han, kana,
/// CJK punctuation, fullwidth forms), where a line may break between any two
/// characters. Hangul is excluded since Korean uses spaces between words.
pub(crate) fn is_cjk(c: char) -> bool {
    matches!(c as u32,
        0x2E80..=0x2FDF      // CJK radicals, Kangxi radicals
        | 0x3000..=0x303F    // CJK symbols and punctuation
        | 0x3040..=0x30FF    // Hiragana, Katakana
        | 0x3100..=0x312F    // Bopomofo
        | 0x31C0..=0x31FF    // CJK strokes, Katakana phonetic extensions
        | 0x3200..=0x33FF    // Enclosed CJK, CJK compatibility
        | 0x3400..=0x4DBF    // CJK extension A
        | 0x4E00..=0x9FFF    // CJK unified ideographs
        | 0xF900..=0xFAFF    // CJK compatibility ideographs
        | 0xFE30..=0xFE4F    // CJK compatibility forms
        | 0xFF00..=0xFFEF    // Halfwidth and fullwidth forms
        | 0x20000..=0x3FFFF  // CJK extensions B+
    )
}

/// Characters that must not start a line (closing brackets, punctuation,
/// small kana, prolonged sound mark) — a simple form of Japanese kinsoku.
fn no_break_before(c: char) -> bool {
    "、。，．,.:;!?！？：；・）〕］｝〉》」』】〙〗〟’”｠»)]}ゝゞーァィゥェォッャュョヮヵヶぁぃぅぇぉっゃゅょゎゕゖ々〻‐゠–〜～…‥".contains(c)
}

/// Characters that must not end a line (opening brackets).
fn no_break_after(c: char) -> bool {
    "（〔［｛〈《「『【〘〖〝‘“｟«([{".contains(c)
}

/// Splits a whitespace-free word into pieces at positions where a line break
/// is allowed (around CJK characters). Pieces are joined without spaces.
pub(crate) fn split_breakable(word: &str) -> Vec<&str> {
    let mut pieces = Vec::new();
    let mut start = 0;
    let mut prev: Option<char> = None;
    for (i, c) in word.char_indices() {
        if let Some(p) = prev {
            let breakable = (is_cjk(p) || is_cjk(c)) && !no_break_before(c) && !no_break_after(p);
            if breakable {
                pieces.push(&word[start..i]);
                start = i;
            }
        }
        prev = Some(c);
    }
    pieces.push(&word[start..]);
    pieces
}

fn ends_with_cjk(spans: &[Span]) -> bool {
    spans
        .last()
        .and_then(|s| s.text.chars().last())
        .is_some_and(is_cjk)
}

pub(crate) fn wrap_spans(spans: &[Span], fonts: &Fonts, scale: PxScale, max_width: u32) -> Vec<Vec<Span>> {
    let mut lines: Vec<Vec<Span>> = Vec::new();
    let mut current_line: Vec<Span> = Vec::new();
    let mut current_width: u32 = 0;
    let space_w = text_size(scale, &fonts.regular, " ").0;

    for (si, span) in spans.iter().enumerate() {
        let font = span.font(fonts);
        let (leading_spaces, words) = split_preserving_indent(&span.text);

        // Whitespace-only spans (e.g. soft breaks) are handled through
        // `prev_ends_space` of the following span.
        if words.is_empty() {
            continue;
        }

        let prev_ends_space = if si > 0 {
            spans[si - 1].text.ends_with(char::is_whitespace)
        } else {
            false
        };
        // A soft break between two CJK characters must not become a space.
        let prev_is_soft_break = si > 0 && spans[si - 1].text.trim().is_empty();
        let joins_cjk = prev_is_soft_break
            && leading_spaces == 0
            && ends_with_cjk(&current_line)
            && words[0].starts_with(is_cjk);
        let need_sep_before_first =
            !current_line.is_empty() && (leading_spaces > 0 || prev_ends_space) && !joins_cjk;

        let pieces = words.iter().enumerate().flat_map(|(wi, word)| {
            split_breakable(word)
                .into_iter()
                .enumerate()
                .map(move |(pi, piece)| (wi, pi, piece))
        });
        for (wi, pi, word) in pieces {
            let word_w = text_size(scale, font, word).0;

            let need_space = if pi > 0 {
                false
            } else if wi == 0 {
                need_sep_before_first
            } else {
                true
            };

            let total = current_width + (if need_space { space_w } else { 0 }) + word_w;
            if current_width > 0 && total > max_width {
                lines.push(std::mem::take(&mut current_line));
                current_width = 0;
                if span.style == SpanStyle::Code && wi == 0 && pi == 0 && leading_spaces > 0 {
                    let indent: String = " ".repeat(leading_spaces);
                    let indent_w = text_size(scale, font, &indent).0;
                    current_line.push(Span {
                        text: indent,
                        style: span.style.clone(),
                    });
                    current_width = indent_w;
                }
            } else if need_space && current_width > 0 {
                current_width += space_w;
                if let Some(last) = current_line.last_mut() {
                    if last.style == span.style {
                        last.text.push(' ');
                    } else {
                        current_line.push(Span {
                            text: " ".to_string(),
                            style: SpanStyle::Normal,
                        });
                    }
                }
            }

            if wi == 0 && pi == 0 && current_width == 0 && leading_spaces > 0 {
                let indent: String = " ".repeat(leading_spaces);
                let indent_w = text_size(scale, font, &indent).0;
                current_line.push(Span {
                    text: format!("{}{}", indent, word),
                    style: span.style.clone(),
                });
                current_width = indent_w + word_w;
                continue;
            }

            if let Some(last) = current_line.last_mut() {
                if last.style == span.style {
                    last.text.push_str(word);
                } else {
                    current_line.push(Span {
                        text: word.to_string(),
                        style: span.style.clone(),
                    });
                }
            } else {
                current_line.push(Span {
                    text: word.to_string(),
                    style: span.style.clone(),
                });
            }
            current_width += word_w;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(Vec::new());
    }
    lines
}

pub(crate) fn draw_spans(
    img: &mut RgbImage,
    spans: &[Span],
    x: u32,
    y: u32,
    scale: PxScale,
    fonts: &Fonts,
    theme: &Theme,
) -> u32 {
    let mut cx = x;
    for span in spans {
        let font = span.font(fonts);
        let color = span.color(theme);

        if span.style == SpanStyle::Code {
            let (tw, th) = text_size(scale, font, &span.text);
            let pad = 2;
            draw_filled_rect_mut(
                img,
                Rect::at(cx as i32 - pad as i32, y as i32).of_size(tw + pad * 2, th + pad),
                theme.code_bg,
            );
        }

        draw_text_mut(img, color, cx as i32, y as i32, scale, font, &span.text);
        let (w, h) = text_size(scale, font, &span.text);

        if span.style == SpanStyle::Strikethrough && w > 0 {
            let line_y = y as i32 + (h as i32 * 5 / 9);
            let thickness = ((scale.y / 14.0).round() as u32).max(1);
            draw_filled_rect_mut(
                img,
                Rect::at(cx as i32, line_y).of_size(w, thickness),
                color,
            );
        }

        cx += w;
    }
    cx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::test_fonts;

    fn normal(text: &str) -> Span {
        Span { text: text.to_string(), style: SpanStyle::Normal }
    }

    fn line_text(line: &[Span]) -> String {
        spans_to_plain(line)
    }

    #[test]
    fn split_breakable_cjk() {
        assert_eq!(split_breakable("hello"), vec!["hello"]);
        assert_eq!(split_breakable("日本語"), vec!["日", "本", "語"]);
        assert_eq!(split_breakable("abc日本"), vec!["abc", "日", "本"]);
        // Closing punctuation stays attached to the preceding char,
        // opening brackets to the following one.
        assert_eq!(split_breakable("「本」。"), vec!["「本」。"]);
        assert_eq!(split_breakable("です。「か"), vec!["で", "す。", "「か"]);
    }

    #[test]
    fn wrap_spans_breaks_cjk_without_spaces() {
        let fonts = test_fonts();
        let scale = PxScale::from(18.0);
        let text = "日本語の文章は単語の間に空白がないので文字単位で折り返す必要があります";
        let lines = wrap_spans(&[normal(text)], &fonts, scale, 200);
        assert!(lines.len() > 1, "CJK text should wrap");
        let joined: String = lines.iter().map(|l| line_text(l)).collect();
        assert_eq!(joined, text, "no spaces should be inserted when wrapping CJK");
        for line in &lines {
            assert!(text_size(scale, &fonts.regular, &line_text(line)).0 <= 200);
        }
    }

    #[test]
    fn wrap_spans_soft_break_between_cjk_is_not_a_space() {
        let fonts = test_fonts();
        let scale = PxScale::from(18.0);
        let spans = vec![normal("日本語"), normal(" "), normal("続き"), normal(" "), normal("next")];
        let lines = wrap_spans(&spans, &fonts, scale, 2000);
        assert_eq!(lines.len(), 1);
        assert_eq!(line_text(&lines[0]), "日本語続き next");
    }

    #[test]
    fn wrap_spans_keeps_space_in_latin_text() {
        let fonts = test_fonts();
        let scale = PxScale::from(18.0);
        let spans = vec![normal("foo"), normal(" "), normal("bar")];
        let lines = wrap_spans(&spans, &fonts, scale, 2000);
        assert_eq!(line_text(&lines[0]), "foo bar");
    }

    #[test]
    fn wrap_spans_respects_width() {
        let fonts = test_fonts();
        let scale = PxScale::from(18.0);
        let spans = vec![Span {
            text: "This is a fairly long sentence that should wrap at some point when rendered".to_string(),
            style: SpanStyle::Normal,
        }];
        let lines = wrap_spans(&spans, &fonts, scale, 200);
        assert!(lines.len() > 1, "should wrap into multiple lines for narrow width");

        let lines_wide = wrap_spans(&spans, &fonts, scale, 2000);
        assert_eq!(lines_wide.len(), 1, "should fit in one line for wide width");
    }
}
