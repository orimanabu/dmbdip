# Development

## Tech Stack

- **Language:** Rust
- **Terminal:** Kitty (graphics protocol, raw RGB f=24, double-buffered)
- **Key crates:** `image`, `base64`, `crossterm`, `pulldown-cmark`, `ab_glyph`, `imageproc`, `notify-debouncer-mini`, `mermaid-rs-renderer`, `resvg`, `usvg`
- **Fonts:** DejaVu Sans (regular, bold, oblique) + DejaVu Sans Mono
- **Font fallback:** `fonts::FontFace` resolves each char to the primary font or a fallback (CJK) font; always use `fonts::{text_size, draw_text_mut}` instead of the imageproc versions
- **Line breaking:** `text::split_breakable` allows breaks between CJK chars (with simple kinsoku); soft breaks between CJK chars are not rendered as spaces

## Architecture

- Parse markdown into `Vec<Block>` (Heading, Paragraph, CodeBlock, Table, Metadata, List, Mermaid)
- Inline text uses `Vec<Span>` with styles: Normal, Bold, Italic, Code, Strikethrough
- `HeadingInfo` tracks each heading's position, number, fold state
- `AppState` manages blocks, headings, cursor, scroll, fold, search state
- Two-pass rendering: compute height, then draw to RgbImage (re-renders on fold/nav changes)
- Kitty display: double-buffered with alternating image IDs, raw RGB, BufWriter
- Alternate screen mode for proper key passthrough

## Workflow Rules

- Commit changes each time a task is done
- Clean git history if appropriate (squash fixup commits)
- Evaluate quality every few commits and after fixing bugs

## Manual testing

The file docs/sample.md can be used for manual testing

# Track advancement

## Todo

(no pending tasks)

## Completed Tasks

- [x] Task 0: Fix inline code spacing and code block indentation
- [x] Task 1: Hierarchical heading numbering (1., 1.1., 1.1.1.)
- [x] Task 2: Heading navigation with cursor (orange bar in left margin)
- [x] Task 3: Section folding with ▶/▼ indicators
- [x] Task 4: New keybindings (arrows for nav, space for scroll)
- [x] Task 5: CLAUDE.md as both README and CLAUDE.md with user info at top
- [x] Task 6: --help flag with usage and keybindings
- [x] Task 7: Help overlay when pressing `h`
- [x] Task 8: Vim-style search with `/`, `n`/`N` navigation
- [x] Task 9: File browser mode with two-pane layout
- [x] Task 10: TOML configuration file support (theme, layout, fonts)
- [x] Task 11: Source file viewing with syntax highlighting (syntect)
- [x] Task 12: File watcher for auto-reload on external edits (notify-debouncer-mini)
- [x] Task 13: Mermaid diagram rendering via mermaid-rs-renderer (auto dark/light, in-memory cache)
