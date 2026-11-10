use image::Rgb;
use pulldown_cmark::HeadingLevel;

use crate::fonts::{FontFace, Fonts};
use crate::theme::Theme;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SpanStyle {
    Normal,
    Bold,
    Italic,
    Code,
    Strikethrough,
}

#[derive(Clone, Debug)]
pub(crate) struct Span {
    pub(crate) text: String,
    pub(crate) style: SpanStyle,
}

impl Span {
    pub(crate) fn font<'a>(&self, fonts: &'a Fonts) -> &'a FontFace {
        match self.style {
            SpanStyle::Normal => &fonts.regular,
            SpanStyle::Bold => &fonts.bold,
            SpanStyle::Italic => &fonts.italic,
            SpanStyle::Code => &fonts.mono,
            SpanStyle::Strikethrough => &fonts.regular,
        }
    }

    pub(crate) fn color(&self, theme: &Theme) -> Rgb<u8> {
        match self.style {
            SpanStyle::Code => theme.code_color,
            _ => theme.body_color,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ListMarker {
    Bullet,
    Ordered(u64),
}

#[derive(Clone, Debug)]
pub(crate) struct ListItem {
    pub(crate) marker: ListMarker,
    pub(crate) depth: u32,
    pub(crate) spans: Vec<Span>,
}

#[derive(Clone)]
pub(crate) enum Block {
    Heading {
        level: HeadingLevel,
        spans: Vec<Span>,
    },
    Paragraph {
        spans: Vec<Span>,
    },
    CodeBlock {
        text: String,
    },
    Table {
        headers: Vec<Vec<Span>>,
        rows: Vec<Vec<Vec<Span>>>,
    },
    Metadata {
        entries: Vec<(String, String)>,
    },
    List {
        items: Vec<ListItem>,
    },
    Mermaid {
        source: String,
    },
}

#[derive(Clone)]
pub(crate) struct HeadingInfo {
    pub(crate) block_index: usize,
    pub(crate) level: HeadingLevel,
    pub(crate) number: String,
    pub(crate) folded: bool,
    /// Y position of this heading in the rendered image (set during render)
    pub(crate) y_pos: u32,
    /// Height of the heading line itself
    pub(crate) heading_height: u32,
}
