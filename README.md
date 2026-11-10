# dmbdip - Display Markdown But Do it Pretty

![Preview](assets/preview.png)

A Rust program that renders markdown files as images and displays them in the terminal using the Kitty graphics protocol, including mermaid diagrams. Includes basic file navigation utilities.

## Usage

```
dmbdip [markdown-file-or-directory]
dmbdip --help
```

When given a markdown file, opens it full-width with the file list hidden. Press Left to reveal the file list and browse sibling files. When given a directory (or no argument), opens the file browser with a file list on the left and markdown preview on the right.

**Requirements:**  a terminal supporting the [Terminal graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/), DejaVu fonts installed. For CJK (Japanese, Chinese, ...) text, a CJK font such as Noto Sans CJK is used automatically as fallback when installed.

## Installation

### From crates.io

```
cargo install dmbdip
```

### Pre-built binaries

Download pre-built binaries for Linux (x86_64) and macOS (x86_64, Apple Silicon) from the [GitHub Releases](https://github.com/sapristi/dmbdip/releases) page.

On macOS, you'll need DejaVu fonts installed:

```
brew install font-dejavu
```

### Building from source

Requires Rust (1.85+ for edition 2024). Install via [rustup](https://rustup.rs/).

```
cargo build --release
```

Install directly with cargo:

```
cargo install --path . --root ~/.local
```

## Keybindings

### Document View

| Key | Action |
|-----|--------|
| Tab/Shift+Tab | Navigate between headings |
| Space | Toggle fold open/close |
| Right | Hide file list (full-width reading) |
| Left | Show file list / back to browser |
| Up/Down | Scroll |
| j/k | Small scroll steps |
| PgUp/PgDn | Half-page scroll |
| Home/End | Jump to top/bottom |
| / | Search text (vim-style) |
| n/N | Next/previous search match |
| h | Show keybindings help overlay |
| q/Esc/Ctrl-C | Quit |

### File Browser

| Key | Action |
|-----|--------|
| Up/Down, j/k | Move cursor |
| Right/Enter | Open file / enter subfolder |
| Left | Go to parent directory |
| e | Open in $EDITOR |
| Ctrl-H | Toggle hidden files |
| h | Show help |
| q/Esc/Ctrl-C | Quit |

## Configuration

Customize theme colors, font sizes, layout, and fonts via a TOML config file at `~/.config/dmbdip/dmbdip.toml`. All fields are optional. See [docs/configuration.md](docs/configuration.md) for the full reference.

## Development

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for architecture, tech stack, task tracking, and workflow notes.
