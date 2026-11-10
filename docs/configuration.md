# Configuration

dmbdip reads an optional TOML configuration file on startup. All fields are optional -- missing fields use built-in defaults.

## File Location

`$XDG_CONFIG_HOME/dmbdip/dmbdip.toml`

Falls back to `~/.config/dmbdip/dmbdip.toml` when `XDG_CONFIG_HOME` is not set.

## Error Handling

- **Missing file:** silently uses all defaults (the normal case).
- **Malformed TOML:** prints a warning to stderr, uses all defaults.
- **Invalid color value:** prints a warning to stderr for that field, uses the default for that field.
- **Unknown font name:** exits with an error (fonts are required to render).

## Full Example

```toml
[theme]
bg = "#1e1e28"
body_color = "#dcdcdc"
body_size = 18.0
code_color = "#e6b450"
code_bg = "#2d2d3a"
cursor_color = "#ffb432"
h1_color = "#64a0ff"
h1_size = 36.0
h2_color = "#50c8c8"
h2_size = 28.0
h3_color = "#78dc78"
h3_size = 22.0
meta_key_color = "#b48cff"
meta_val_color = "#c8c8c8"
table_border = "#646478"
table_header_bg = "#323241"

[layout]
margin_left = 50
margin_right = 20
paragraph_gap = 16
max_content_width = 900
h1_extra_margin = 40
block_indent = 24
scroll_step = 40
cursor_width = 4
cursor_margin = 6

[fonts]
sans = "DejaVu Sans"
mono = "DejaVu Sans Mono"
fallback = ["Noto Sans CJK JP"]

[browser]
extra_extensions = ["rs", "py", "toml", "js"]
```

## `[theme]` -- Colors and Font Sizes

Colors are 6-digit hex strings. Both `"1e1e28"` and `"#1e1e28"` are accepted.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `bg` | color | `#1e1e28` | Background color |
| `body_color` | color | `#dcdcdc` | Body text color |
| `body_size` | float | `18.0` | Body text size in pixels |
| `code_color` | color | `#e6b450` | Inline code text color |
| `code_bg` | color | `#2d2d3a` | Code block background color |
| `cursor_color` | color | `#ffb432` | Heading cursor bar color |
| `h1_color` | color | `#64a0ff` | H1 heading color |
| `h1_size` | float | `36.0` | H1 heading size in pixels |
| `h2_color` | color | `#50c8c8` | H2 heading color |
| `h2_size` | float | `28.0` | H2 heading size in pixels |
| `h3_color` | color | `#78dc78` | H3 heading color |
| `h3_size` | float | `22.0` | H3+ heading size in pixels |
| `meta_key_color` | color | `#b48cff` | YAML frontmatter key color |
| `meta_val_color` | color | `#c8c8c8` | YAML frontmatter value color |
| `table_border` | color | `#646478` | Table border line color |
| `table_header_bg` | color | `#323241` | Table header row background |

## `[layout]` -- Spacing and Dimensions

All values are in pixels. Values are clamped to safe ranges to prevent rendering issues.

| Field | Type | Default | Range | Description |
|-------|------|---------|-------|-------------|
| `margin_left` | int | `50` | 0--200 | Left margin for document content |
| `margin_right` | int | `20` | 0--200 | Right margin for document content |
| `paragraph_gap` | int | `16` | 0--200 | Vertical space between blocks |
| `max_content_width` | int | `900` | 100--4000 | Maximum content width before centering |
| `h1_extra_margin` | int | `40` | 0--200 | Extra top margin before H1 headings |
| `block_indent` | int | `24` | 0--200 | Indentation for paragraphs, code blocks, and lists |
| `scroll_step` | int | `40` | 1--500 | Pixels scrolled per step (Up/Down, j/k) |
| `cursor_width` | int | `4` | 1--20 | Width of the heading cursor bar |
| `cursor_margin` | int | `6` | 0--50 | Gap between cursor bar and text |

## `[fonts]` -- Font Families

Font family names resolved through the system font library (fontconfig on Linux, CoreText on macOS). Bold and italic variants are resolved automatically from the family name.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `sans` | string | `"DejaVu Sans"` | Sans-serif font family (used for body text, headings, lists) |
| `mono` | string | `"DejaVu Sans Mono"` | Monospace font family (used for code blocks and inline code) |
| `fallback` | list of strings | *(auto)* | Fallback font families used, in order, for characters missing from the main fonts (e.g. Japanese/Chinese text). When unset, the first installed font among common CJK families (Noto Sans CJK JP, Source Han Sans, Hiragino Sans, IPAGothic, ...) is used. |

## `[browser]` -- File Browser

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `extra_extensions` | list of strings | `[]` | Additional file extensions to show in the browser (e.g., `["rs", "py", "toml"]`). Markdown files (`.md`) are always shown. Source files are displayed with syntax highlighting using a monospace font. |
