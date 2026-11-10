use ab_glyph::PxScale;
use image::RgbImage;
use crate::fonts::text_size;

use crate::constants::LayoutParams;
use crate::fonts::Fonts;
use crate::source_render::render_source;
use crate::theme::Theme;

pub(crate) struct SourceViewState {
    pub(crate) scroll_y: u32,
    pub(crate) vp_width: u32,
    pub(crate) vp_height: u32,
    pub(crate) frame: u32,
    pub(crate) img: RgbImage,
    pub(crate) theme: Theme,
    pub(crate) layout: LayoutParams,
    pub(crate) extension: String,
    pub(crate) source_text: String,
    pub(crate) search_mode: bool,
    pub(crate) search_query: String,
    pub(crate) search_matches: Vec<usize>,
    pub(crate) search_current: usize,
    pub(crate) search_highlights: Vec<(u32, u32, u32, u32, usize)>,
}

impl SourceViewState {
    pub(crate) fn new(
        source: &str,
        extension: &str,
        fonts: &Fonts,
        vp_width: u32,
        vp_height: u32,
        theme: Theme,
        layout: LayoutParams,
    ) -> Self {
        let (img, _) = render_source(source, extension, vp_width, fonts, &theme, &layout);
        SourceViewState {
            scroll_y: 0,
            vp_width,
            vp_height,
            frame: 1,
            img,
            theme,
            layout,
            extension: extension.to_string(),
            source_text: source.to_string(),
            search_mode: false,
            search_query: String::new(),
            search_matches: Vec::new(),
            search_current: 0,
            search_highlights: Vec::new(),
        }
    }

    pub(crate) fn rerender(&mut self, fonts: &Fonts) {
        let (img, _) = render_source(
            &self.source_text,
            &self.extension,
            self.vp_width,
            fonts,
            &self.theme,
            &self.layout,
        );
        self.img = img;
        let max = self.max_scroll();
        if self.scroll_y > max {
            self.scroll_y = max;
        }
    }

    pub(crate) fn max_scroll(&self) -> u32 {
        self.img.height().saturating_sub(self.vp_height)
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
            true
        } else {
            false
        }
    }

    /// Return the source line number at the current scroll position.
    pub(crate) fn current_source_line(&self) -> usize {
        let line_height = (self.theme.body_size * 1.4) as u32;
        if line_height == 0 {
            return 1;
        }
        let line_index = self.scroll_y / line_height;
        (line_index as usize) + 1
    }

    pub(crate) fn execute_search(&mut self, fonts: &Fonts) {
        self.search_matches.clear();
        self.search_highlights.clear();
        self.search_current = 0;
        if self.search_query.is_empty() {
            return;
        }

        let query = self.search_query.to_lowercase();
        let font_size = self.theme.body_size;
        let scale = PxScale::from(font_size);
        let line_height = (font_size * 1.4) as u32;

        let lines: Vec<&str> = self.source_text.lines().collect();
        let total_lines = lines.len().max(1);

        let content_width = (self.vp_width - self.layout.margin_left - self.layout.margin_right)
            .min(self.layout.max_content_width);
        let margin_left = (self.vp_width - content_width) / 2;

        // Compute ln_width and code_x to match render_source
        let digits = format!("{}", total_lines);
        let ln_width = text_size(scale, &fonts.mono, &digits).0 + 20; // LINE_NUM_MARGIN * 2
        let code_x = margin_left + ln_width + 12; // LEFT_PAD

        let mut y = self.layout.paragraph_gap;
        let mut match_idx = 0usize;

        for (_i, line) in lines.iter().enumerate() {
            let line_lower = line.to_lowercase();
            let mut search_from = 0;
            let mut found_in_line = false;
            while search_from < line_lower.len() {
                if let Some(byte_pos) = line_lower[search_from..].find(&query) {
                    let match_start = search_from + byte_pos;
                    let match_end = match_start + query.len();
                    let prefix = &line[..match_start];
                    let match_text = &line[match_start..match_end];
                    let x_offset = text_size(scale, &fonts.mono, prefix).0;
                    let match_width = text_size(scale, &fonts.mono, match_text).0;
                    self.search_highlights.push((
                        code_x + x_offset,
                        y,
                        match_width,
                        line_height,
                        match_idx,
                    ));
                    found_in_line = true;
                    search_from = match_end;
                } else {
                    break;
                }
            }
            if found_in_line {
                self.search_matches.push(match_idx);
                match_idx += 1;
            }
            y += line_height;
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
