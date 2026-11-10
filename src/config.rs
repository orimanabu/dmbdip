use image::Rgb;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Default)]
#[serde(default)]
pub(crate) struct Config {
    pub(crate) theme: ThemeConfig,
    pub(crate) layout: LayoutConfig,
    pub(crate) fonts: FontsConfig,
    pub(crate) browser: BrowserConfig,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub(crate) struct ThemeConfig {
    pub(crate) bg: Option<String>,
    pub(crate) body_color: Option<String>,
    pub(crate) body_size: Option<f32>,
    pub(crate) code_color: Option<String>,
    pub(crate) code_bg: Option<String>,
    pub(crate) cursor_color: Option<String>,
    pub(crate) h1_color: Option<String>,
    pub(crate) h1_size: Option<f32>,
    pub(crate) h2_color: Option<String>,
    pub(crate) h2_size: Option<f32>,
    pub(crate) h3_color: Option<String>,
    pub(crate) h3_size: Option<f32>,
    pub(crate) meta_key_color: Option<String>,
    pub(crate) meta_val_color: Option<String>,
    pub(crate) table_border: Option<String>,
    pub(crate) table_header_bg: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub(crate) struct LayoutConfig {
    pub(crate) margin_left: Option<u32>,
    pub(crate) margin_right: Option<u32>,
    pub(crate) paragraph_gap: Option<u32>,
    pub(crate) max_content_width: Option<u32>,
    pub(crate) h1_extra_margin: Option<u32>,
    pub(crate) block_indent: Option<u32>,
    pub(crate) scroll_step: Option<u32>,
    pub(crate) cursor_width: Option<u32>,
    pub(crate) cursor_margin: Option<u32>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub(crate) struct FontsConfig {
    pub(crate) sans: Option<String>,
    pub(crate) mono: Option<String>,
    pub(crate) fallback: Option<Vec<String>>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub(crate) struct BrowserConfig {
    pub(crate) extra_extensions: Option<Vec<String>>,
}

pub(crate) fn load_config() -> Config {
    let config_path = config_path();
    match std::fs::read_to_string(&config_path) {
        Ok(contents) => match toml::from_str(&contents) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Warning: malformed config file {}: {}", config_path.display(), e);
                Config::default()
            }
        },
        Err(_) => Config::default(),
    }
}

pub(crate) fn config_path() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg).join("dmbdip").join("dmbdip.toml")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".config")
            .join("dmbdip")
            .join("dmbdip.toml")
    } else {
        PathBuf::from(".config/dmbdip/dmbdip.toml")
    }
}

pub(crate) fn parse_hex_color(s: &str) -> Option<Rgb<u8>> {
    let hex = s.strip_prefix('#').unwrap_or(s);
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Rgb([r, g, b]))
}

fn color_or_default(field: &Option<String>, default: Rgb<u8>, name: &str) -> Rgb<u8> {
    match field {
        Some(s) => match parse_hex_color(s) {
            Some(c) => c,
            None => {
                eprintln!("Warning: invalid color '{}' for {}, using default", s, name);
                default
            }
        },
        None => default,
    }
}

pub(crate) fn build_theme(config: &ThemeConfig) -> crate::theme::Theme {
    crate::theme::Theme {
        bg: color_or_default(&config.bg, Rgb([30, 30, 40]), "bg"),
        body_color: color_or_default(&config.body_color, Rgb([220, 220, 220]), "body_color"),
        body_size: config.body_size.unwrap_or(18.0),
        code_color: color_or_default(&config.code_color, Rgb([230, 180, 80]), "code_color"),
        code_bg: color_or_default(&config.code_bg, Rgb([45, 45, 58]), "code_bg"),
        cursor_color: color_or_default(&config.cursor_color, Rgb([255, 180, 50]), "cursor_color"),
        h1_color: color_or_default(&config.h1_color, Rgb([100, 160, 255]), "h1_color"),
        h1_size: config.h1_size.unwrap_or(36.0),
        h2_color: color_or_default(&config.h2_color, Rgb([80, 200, 200]), "h2_color"),
        h2_size: config.h2_size.unwrap_or(28.0),
        h3_color: color_or_default(&config.h3_color, Rgb([120, 220, 120]), "h3_color"),
        h3_size: config.h3_size.unwrap_or(22.0),
        meta_key_color: color_or_default(&config.meta_key_color, Rgb([180, 140, 255]), "meta_key_color"),
        meta_val_color: color_or_default(&config.meta_val_color, Rgb([200, 200, 200]), "meta_val_color"),
        table_border: color_or_default(&config.table_border, Rgb([100, 100, 120]), "table_border"),
        table_header_bg: color_or_default(&config.table_header_bg, Rgb([50, 50, 65]), "table_header_bg"),
    }
}

pub(crate) fn debug_config(config: &Config) {
    let path = config_path();
    let file_exists = path.exists();
    eprintln!("Config file: {}", path.display());
    eprintln!("  exists: {}", file_exists);
    if !file_exists {
        eprintln!("  (using all defaults)");
        return;
    }
    eprintln!();
    eprintln!("Overridden options:");

    let mut any = false;

    // [theme]
    macro_rules! check_opt {
        ($section:expr, $field:expr, $val:expr) => {
            if let Some(ref v) = $val {
                eprintln!("  [{}] {} = {:?}", $section, $field, v);
                any = true;
            }
        };
    }

    check_opt!("theme", "bg", config.theme.bg);
    check_opt!("theme", "body_color", config.theme.body_color);
    check_opt!("theme", "body_size", config.theme.body_size);
    check_opt!("theme", "code_color", config.theme.code_color);
    check_opt!("theme", "code_bg", config.theme.code_bg);
    check_opt!("theme", "cursor_color", config.theme.cursor_color);
    check_opt!("theme", "h1_color", config.theme.h1_color);
    check_opt!("theme", "h1_size", config.theme.h1_size);
    check_opt!("theme", "h2_color", config.theme.h2_color);
    check_opt!("theme", "h2_size", config.theme.h2_size);
    check_opt!("theme", "h3_color", config.theme.h3_color);
    check_opt!("theme", "h3_size", config.theme.h3_size);
    check_opt!("theme", "meta_key_color", config.theme.meta_key_color);
    check_opt!("theme", "meta_val_color", config.theme.meta_val_color);
    check_opt!("theme", "table_border", config.theme.table_border);
    check_opt!("theme", "table_header_bg", config.theme.table_header_bg);

    check_opt!("layout", "margin_left", config.layout.margin_left);
    check_opt!("layout", "margin_right", config.layout.margin_right);
    check_opt!("layout", "paragraph_gap", config.layout.paragraph_gap);
    check_opt!("layout", "max_content_width", config.layout.max_content_width);
    check_opt!("layout", "h1_extra_margin", config.layout.h1_extra_margin);
    check_opt!("layout", "block_indent", config.layout.block_indent);
    check_opt!("layout", "scroll_step", config.layout.scroll_step);
    check_opt!("layout", "cursor_width", config.layout.cursor_width);
    check_opt!("layout", "cursor_margin", config.layout.cursor_margin);

    check_opt!("fonts", "sans", config.fonts.sans);
    check_opt!("fonts", "mono", config.fonts.mono);
    check_opt!("fonts", "fallback", config.fonts.fallback);

    check_opt!("browser", "extra_extensions", config.browser.extra_extensions);

    if !any {
        eprintln!("  (none — file exists but all fields use defaults)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_color_with_hash() {
        assert_eq!(parse_hex_color("#1e1e28"), Some(Rgb([0x1e, 0x1e, 0x28])));
    }

    #[test]
    fn parse_hex_color_without_hash() {
        assert_eq!(parse_hex_color("dcdcdc"), Some(Rgb([0xdc, 0xdc, 0xdc])));
    }

    #[test]
    fn parse_hex_color_invalid() {
        assert_eq!(parse_hex_color("xyz"), None);
        assert_eq!(parse_hex_color("#12345"), None);
        assert_eq!(parse_hex_color(""), None);
    }

    #[test]
    fn build_theme_defaults() {
        let config = ThemeConfig::default();
        let theme = build_theme(&config);
        assert_eq!(theme.bg, Rgb([30, 30, 40]));
        assert_eq!(theme.body_size, 18.0);
    }

    #[test]
    fn build_theme_overrides() {
        let config = ThemeConfig {
            bg: Some("#ff0000".to_string()),
            body_size: Some(24.0),
            ..Default::default()
        };
        let theme = build_theme(&config);
        assert_eq!(theme.bg, Rgb([255, 0, 0]));
        assert_eq!(theme.body_size, 24.0);
    }

    #[test]
    fn load_config_missing_file() {
        // Should return defaults without error
        let config = load_config();
        assert!(config.theme.bg.is_none());
    }

    #[test]
    fn deserialize_partial_config() {
        let toml_str = "
[theme]
bg = \"#ff0000\"

[layout]
margin_left = 30
";
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.theme.bg.as_deref(), Some("#ff0000"));
        assert_eq!(config.layout.margin_left, Some(30));
        assert!(config.theme.body_color.is_none());
        assert!(config.layout.margin_right.is_none());
    }

    #[test]
    fn deserialize_browser_config() {
        let toml_str = r#"
[browser]
extra_extensions = ["rs", "py", "toml"]
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        let exts = config.browser.extra_extensions.unwrap();
        assert_eq!(exts, vec!["rs", "py", "toml"]);
    }
}
