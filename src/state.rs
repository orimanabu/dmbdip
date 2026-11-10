use ab_glyph::PxScale;
use crate::fonts::FontFace;
use image::RgbImage;
use crate::fonts::text_size;

use crate::constants::{LayoutParams, LIST_INDENT_PER_LEVEL};
use crate::fonts::Fonts;
use crate::headings::{build_headings, is_block_folded};
use crate::mermaid::MermaidCache;
use crate::parsing::parse_markdown;
use crate::render::{heading_style, render_markdown, wrap_code_lines, wrap_heading_text};
use crate::text::{spans_to_plain, wrap_spans};
use crate::theme::Theme;
use crate::types::{Block, HeadingInfo, ListMarker};

pub(crate) struct AppState {
    pub(crate) blocks: Vec<Block>,
    pub(crate) block_source_lines: Vec<usize>,
    pub(crate) headings: Vec<HeadingInfo>,
    pub(crate) current_heading: Option<usize>,
    pub(crate) scroll_y: u32,
    pub(crate) vp_width: u32,
    pub(crate) vp_height: u32,
    pub(crate) frame: u32,
    pub(crate) img: RgbImage,
    pub(crate) block_y_positions: Vec<(usize, u32)>,
    pub(crate) margin_left: u32,
    pub(crate) theme: Theme,
    pub(crate) layout: LayoutParams,
    pub(crate) search_mode: bool,
    pub(crate) search_query: String,
    pub(crate) search_matches: Vec<usize>,
    pub(crate) search_current: usize,
    pub(crate) search_highlights: Vec<(u32, u32, u32, u32, usize)>,
    pub(crate) mermaid_cache: MermaidCache,
}

impl AppState {
    pub(crate) fn new(
        source: &str,
        fonts: &Fonts,
        vp_width: u32,
        vp_height: u32,
        theme: Theme,
        layout: LayoutParams,
    ) -> Self {
        let (blocks, block_source_lines) = parse_markdown(source);
        let headings = build_headings(&blocks);
        let current_heading = if headings.is_empty() { None } else { Some(0) };

        let mut state = AppState {
            blocks,
            block_source_lines,
            headings,
            current_heading,
            scroll_y: 0,
            vp_width,
            vp_height,
            frame: 1,
            img: RgbImage::new(1, 1),
            block_y_positions: Vec::new(),
            margin_left: 0,
            theme,
            layout,
            search_mode: false,
            search_query: String::new(),
            search_matches: Vec::new(),
            search_current: 0,
            search_highlights: Vec::new(),
            mermaid_cache: MermaidCache::new(),
        };
        state.rerender(fonts);
        state
    }

    pub(crate) fn rerender(&mut self, fonts: &Fonts) {
        let (img, positions, margin_left) = render_markdown(
            &self.blocks,
            &mut self.headings,
            self.vp_width,
            self.vp_height,
            fonts,
            &self.theme,
            &self.layout,
            &mut self.mermaid_cache,
        );
        self.img = img;
        self.block_y_positions = positions;
        self.margin_left = margin_left;
        let max_scroll = self.max_scroll();
        if self.scroll_y > max_scroll {
            self.scroll_y = max_scroll;
        }
    }

    pub(crate) fn max_scroll(&self) -> u32 {
        self.img.height().saturating_sub(self.vp_height)
    }

    pub(crate) fn navigate_heading(&mut self, direction: i32) -> bool {
        if self.headings.is_empty() {
            return false;
        }

        let current = self.current_heading.unwrap_or(0);
        let new_idx = if direction > 0 {
            let mut idx = current + 1;
            while idx < self.headings.len() {
                if !is_block_folded(self.headings[idx].block_index, &self.headings) {
                    break;
                }
                idx += 1;
            }
            if idx < self.headings.len() {
                idx
            } else {
                return false;
            }
        } else {
            if current == 0 {
                return false;
            }
            let mut idx = current - 1;
            loop {
                if !is_block_folded(self.headings[idx].block_index, &self.headings) {
                    break;
                }
                if idx == 0 {
                    return false;
                }
                idx -= 1;
            }
            idx
        };

        self.current_heading = Some(new_idx);

        let heading = &self.headings[new_idx];
        let target = heading.y_pos.saturating_sub(self.vp_height / 4);
        self.scroll_y = target.min(self.max_scroll());

        true
    }

    pub(crate) fn toggle_fold(&mut self, fonts: &Fonts) -> bool {
        if let Some(hi) = self.current_heading {
            self.headings[hi].folded = !self.headings[hi].folded;
            self.rerender(fonts);
            true
        } else {
            false
        }
    }

    pub(crate) fn cursor_info(&self) -> Option<(u32, u32, u32, [u8; 3])> {
        let hi = self.current_heading?;
        let heading = &self.headings[hi];
        let (size, _) = heading_style(&heading.level, &self.theme);
        let arrow_space = (size * 0.5) as u32 + 4;
        let cursor_x = self.margin_left.saturating_sub(arrow_space + self.layout.cursor_margin + self.layout.cursor_width);
        let c = self.theme.cursor_color.0;
        Some((cursor_x, heading.y_pos, heading.heading_height, c))
    }

    #[allow(dead_code)]
    pub(crate) fn scroll(&mut self, delta: i32) -> bool {
        let max = self.max_scroll();
        let new_y = if delta > 0 {
            (self.scroll_y + delta as u32).min(max)
        } else {
            self.scroll_y.saturating_sub((-delta) as u32)
        };
        if new_y != self.scroll_y {
            self.scroll_y = new_y;
            self.sync_cursor_to_scroll();
            true
        } else {
            false
        }
    }

    /// Update current_heading to the last visible heading at or above the
    /// top third of the viewport.
    pub(crate) fn sync_cursor_to_scroll(&mut self) {
        if self.headings.is_empty() {
            return;
        }
        let threshold = self.scroll_y + self.vp_height / 3;
        let mut best: Option<usize> = None;
        for (i, h) in self.headings.iter().enumerate() {
            if is_block_folded(h.block_index, &self.headings) {
                continue;
            }
            if h.y_pos <= threshold {
                best = Some(i);
            } else {
                break;
            }
        }
        if let Some(idx) = best {
            self.current_heading = Some(idx);
        }
    }

    /// Return the source line number of the block closest to the current scroll position.
    pub(crate) fn current_source_line(&self) -> Option<usize> {
        if self.block_y_positions.is_empty() {
            return None;
        }
        let target_y = self.scroll_y + self.vp_height / 4;
        let mut best_block_idx = self.block_y_positions[0].0;
        for &(bi, y) in &self.block_y_positions {
            if y <= target_y {
                best_block_idx = bi;
            } else {
                break;
            }
        }
        self.block_source_lines.get(best_block_idx).copied()
    }

    pub(crate) fn execute_search(&mut self, fonts: &Fonts) {
        self.search_matches.clear();
        self.search_highlights.clear();
        self.search_current = 0;
        if self.search_query.is_empty() {
            return;
        }
        let query = self.search_query.to_lowercase();
        let content_width = self.vp_width - 2 * self.margin_left;
        let mut match_idx = 0usize;

        for (pos_idx, &(bi, block_y)) in self.block_y_positions.iter().enumerate() {
            let block = &self.blocks[bi];
            if !block_contains_text(block, &query) {
                continue;
            }
            self.search_matches.push(pos_idx);

            let highlights = compute_block_highlights(
                block, block_y, &query, fonts, &self.theme,
                content_width, self.margin_left, match_idx,
                &self.headings, bi, &self.layout,
            );
            self.search_highlights.extend(highlights);
            match_idx += 1;
        }

        if let Some(&(_, hy, _, _, _)) = self.search_highlights.first() {
            self.scroll_y = hy.saturating_sub(self.vp_height / 4).min(self.max_scroll());
        }
    }

    pub(crate) fn navigate_search(&mut self, forward: bool) -> bool {
        if self.search_matches.is_empty() {
            return false;
        }
        if forward {
            self.search_current = (self.search_current + 1) % self.search_matches.len();
        } else {
            self.search_current = if self.search_current == 0 {
                self.search_matches.len() - 1
            } else {
                self.search_current - 1
            };
        }
        let target_idx = self.search_current;
        if let Some(&(_, hy, _, _, _)) = self
            .search_highlights
            .iter()
            .find(|h| h.4 == target_idx)
        {
            self.scroll_y = hy.saturating_sub(self.vp_height / 4).min(self.max_scroll());
        }
        true
    }
}

fn block_contains_text(block: &Block, query: &str) -> bool {
    match block {
        Block::Heading { spans, .. } | Block::Paragraph { spans } => {
            spans_to_plain(spans).to_lowercase().contains(query)
        }
        Block::CodeBlock { text } => text.to_lowercase().contains(query),
        Block::Table { headers, rows } => {
            headers
                .iter()
                .any(|h| spans_to_plain(h).to_lowercase().contains(query))
                || rows.iter().any(|row| {
                    row.iter()
                        .any(|cell| spans_to_plain(cell).to_lowercase().contains(query))
                })
        }
        Block::Metadata { entries } => entries.iter().any(|(k, v)| {
            k.to_lowercase().contains(query) || v.to_lowercase().contains(query)
        }),
        Block::List { items } => items.iter().any(|item| {
            spans_to_plain(&item.spans).to_lowercase().contains(query)
        }),
        Block::Mermaid { source } => source.to_lowercase().contains(query),
    }
}

fn find_highlights_in_text(
    text: &str,
    query: &str,
    font: &FontFace,
    scale: PxScale,
    line_height: u32,
    x: u32,
    y: u32,
    match_idx: usize,
) -> Vec<(u32, u32, u32, u32, usize)> {
    let mut highlights = Vec::new();
    let text_lower = text.to_lowercase();
    let mut search_from = 0;
    while search_from < text_lower.len() {
        if let Some(byte_pos) = text_lower[search_from..].find(query) {
            let match_start = search_from + byte_pos;
            let match_end = match_start + query.len();
            let prefix = &text[..match_start];
            let match_text = &text[match_start..match_end];
            let x_offset = text_size(scale, font, prefix).0;
            let match_width = text_size(scale, font, match_text).0;
            highlights.push((x + x_offset, y, match_width, line_height, match_idx));
            search_from = match_end;
        } else {
            break;
        }
    }
    highlights
}

fn compute_block_highlights(
    block: &Block,
    block_y: u32,
    query: &str,
    fonts: &Fonts,
    theme: &Theme,
    content_width: u32,
    margin_left: u32,
    match_idx: usize,
    headings: &[HeadingInfo],
    block_index: usize,
    layout: &LayoutParams,
) -> Vec<(u32, u32, u32, u32, usize)> {
    let mut highlights = Vec::new();
    match block {
        Block::Paragraph { spans } => {
            let scale = PxScale::from(theme.body_size);
            let indented_width = content_width - layout.block_indent;
            let lines = wrap_spans(spans, fonts, scale, indented_width);
            let line_height = (theme.body_size * 1.4) as u32;
            let x_start = margin_left + layout.block_indent;
            let mut y = block_y;
            for line in &lines {
                let plain = spans_to_plain(line);
                highlights.extend(find_highlights_in_text(
                    &plain, query, &fonts.regular, scale, line_height,
                    x_start, y, match_idx,
                ));
                y += line_height;
            }
        }
        Block::Heading { level: _, spans } => {
            let hi = headings.iter().find(|h| h.block_index == block_index);
            if let Some(heading) = hi {
                let (lines, size, line_height) = wrap_heading_text(
                    heading, spans, fonts, theme, content_width,
                );
                let scale = PxScale::from(size);
                let mut y = block_y;
                for line in &lines {
                    let line_plain = spans_to_plain(line);
                    highlights.extend(find_highlights_in_text(
                        &line_plain, query, &fonts.bold, scale, line_height,
                        margin_left, y, match_idx,
                    ));
                    y += line_height;
                }
            }
        }
        Block::CodeBlock { text } => {
            let scale = PxScale::from(theme.body_size);
            let indented_width = content_width - layout.block_indent;
            let pad = 10u32;
            let inner_width = indented_width - pad * 2;
            let line_height = (theme.body_size * 1.4) as u32;
            let x_start = margin_left + layout.block_indent + pad;
            let mut y = block_y + pad;
            for wrapped in &wrap_code_lines(text, fonts, scale, inner_width) {
                for line in wrapped {
                    let plain = spans_to_plain(line);
                    highlights.extend(find_highlights_in_text(
                        &plain, query, &fonts.mono, scale, line_height,
                        x_start, y, match_idx,
                    ));
                    y += line_height;
                }
            }
        }
        Block::List { items } => {
            let scale = PxScale::from(theme.body_size);
            let line_height = (theme.body_size * 1.4) as u32;
            let mut y = block_y;
            for item in items {
                let indent = layout.block_indent + item.depth * LIST_INDENT_PER_LEVEL;
                let mt = match &item.marker {
                    ListMarker::Bullet => "\u{2022}  ".to_string(),
                    ListMarker::Ordered(n) => format!("{}. ", n),
                };
                let marker_w = text_size(scale, &fonts.regular, &mt).0;
                let text_width = content_width.saturating_sub(indent + marker_w);
                let x_start = margin_left + indent + marker_w;
                let lines = wrap_spans(&item.spans, fonts, scale, text_width);
                for line in &lines {
                    let plain = spans_to_plain(line);
                    highlights.extend(find_highlights_in_text(
                        &plain, query, &fonts.regular, scale, line_height,
                        x_start, y, match_idx,
                    ));
                    y += line_height;
                }
                if lines.is_empty() {
                    y += line_height;
                }
            }
        }
        Block::Mermaid { .. } => {}
        _ => {}
    }
    highlights
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{test_fonts, SAMPLE_MD};
    use crate::theme::default_theme;

    fn new_state(source: &str, fonts: &Fonts, w: u32, h: u32) -> AppState {
        AppState::new(source, fonts, w, h, default_theme(), LayoutParams::default())
    }

    #[test]
    fn app_state_navigation() {
        let fonts = test_fonts();
        let mut state = new_state(SAMPLE_MD, &fonts, 800, 600);

        assert_eq!(state.current_heading, Some(0));
        assert!(state.navigate_heading(1));
        assert_eq!(state.current_heading, Some(1));
        assert!(state.navigate_heading(1));
        assert_eq!(state.current_heading, Some(2));
        assert!(state.navigate_heading(-1));
        assert_eq!(state.current_heading, Some(1));
    }

    #[test]
    fn app_state_navigation_bounds() {
        let fonts = test_fonts();
        let mut state = new_state(SAMPLE_MD, &fonts, 800, 600);

        assert!(!state.navigate_heading(-1));
        assert_eq!(state.current_heading, Some(0));

        while state.navigate_heading(1) {}
        let last = state.current_heading.unwrap();
        assert!(!state.navigate_heading(1));
        assert_eq!(state.current_heading, Some(last));
    }

    #[test]
    fn app_state_fold_toggle() {
        let fonts = test_fonts();
        let mut state = new_state(SAMPLE_MD, &fonts, 800, 600);
        let h_before = state.img.height();

        state.toggle_fold(&fonts);
        assert!(state.headings[0].folded);
        assert!(state.img.height() < h_before);

        state.toggle_fold(&fonts);
        assert!(!state.headings[0].folded);
        assert_eq!(state.img.height(), h_before);
    }

    #[test]
    fn app_state_scroll() {
        let fonts = test_fonts();
        let mut state = new_state(SAMPLE_MD, &fonts, 800, 200);

        assert_eq!(state.scroll_y, 0);
        assert!(state.scroll(100));
        assert_eq!(state.scroll_y, 100);
        state.scroll(999999);
        assert!(state.scroll_y <= state.max_scroll());
        assert!(!state.scroll(-999999) || state.scroll_y == 0);
    }

    #[test]
    fn app_state_cursor_info() {
        let fonts = test_fonts();
        let state = new_state(SAMPLE_MD, &fonts, 800, 600);
        let ci = state.cursor_info();
        assert!(ci.is_some());
        let (x, y, h, color) = ci.unwrap();
        assert!(x < 800);
        assert!(h > 0);
        assert_eq!(color, state.theme.cursor_color.0);
        let _ = y;
    }

    #[test]
    fn search_finds_text() {
        let fonts = test_fonts();
        let mut state = new_state(SAMPLE_MD, &fonts, 800, 600);

        state.search_query = "bold".to_string();
        state.execute_search(&fonts);
        assert!(!state.search_matches.is_empty(), "should find 'bold'");
        assert!(!state.search_highlights.is_empty());
    }

    #[test]
    fn search_case_insensitive() {
        let fonts = test_fonts();
        let mut state = new_state(SAMPLE_MD, &fonts, 800, 600);

        state.search_query = "HELLO".to_string();
        state.execute_search(&fonts);
        assert!(!state.search_matches.is_empty(), "should find 'HELLO' case-insensitively");
    }

    #[test]
    fn search_no_match() {
        let fonts = test_fonts();
        let mut state = new_state(SAMPLE_MD, &fonts, 800, 600);

        state.search_query = "zzzznonexistent".to_string();
        state.execute_search(&fonts);
        assert!(state.search_matches.is_empty());
        assert!(state.search_highlights.is_empty());
    }

    #[test]
    fn search_navigation_wraps() {
        let fonts = test_fonts();
        let mut state = new_state(SAMPLE_MD, &fonts, 800, 600);

        state.search_query = "section".to_string();
        state.execute_search(&fonts);
        if state.search_matches.len() >= 2 {
            let first = state.search_current;
            state.navigate_search(true);
            assert_ne!(state.search_current, first);
            state.search_current = 0;
            state.navigate_search(false);
            assert_eq!(state.search_current, state.search_matches.len() - 1);
        }
    }

    #[test]
    fn search_in_code_block() {
        let fonts = test_fonts();
        let mut state = new_state(SAMPLE_MD, &fonts, 800, 600);

        state.search_query = "println".to_string();
        state.execute_search(&fonts);
        assert!(!state.search_matches.is_empty(), "should find text in code blocks");
    }

    #[test]
    fn search_in_table() {
        let fonts = test_fonts();
        let md = "| Name | Age |\n|------|-----|\n| Alice | 30 |\n";
        let mut state = new_state(md, &fonts, 800, 600);

        state.search_query = "alice".to_string();
        state.execute_search(&fonts);
        assert!(!state.search_matches.is_empty(), "should find text in tables");
    }

    #[test]
    fn navigation_skips_folded_headings() {
        let fonts = test_fonts();
        let md = "# Top\n## A\n### A.1\n## B\n";
        let mut state = new_state(md, &fonts, 800, 600);

        state.current_heading = Some(1);
        state.toggle_fold(&fonts);

        state.navigate_heading(1);
        assert_eq!(state.current_heading, Some(3));
    }

    #[test]
    fn empty_document() {
        let fonts = test_fonts();
        let state = new_state("", &fonts, 800, 600);
        assert!(state.headings.is_empty());
        assert!(state.current_heading.is_none());
        assert!(state.img.height() > 0);
    }

    #[test]
    fn headings_only() {
        let fonts = test_fonts();
        let md = "# One\n## Two\n## Three\n";
        let state = new_state(md, &fonts, 800, 600);
        assert_eq!(state.headings.len(), 3);
    }
}
