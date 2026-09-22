use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use crate::wm;

pub const SYSTEM_FONT: &str = ".SystemUIFont";
pub const CUSTOM_THEME: &str = "custom";
pub const DEFAULT_THEME: &str = "Default Dark";

const GITHUB_DARK_BG: &str = "#0d1117";
const GITHUB_DARK_FG: &str = "#f0f6fc";
const GITHUB_LIGHT_BG: &str = "#ffffff";
const GITHUB_LIGHT_FG: &str = "#1f2328";
const SEPIA_BG: &str = "#f4ecd8";
const SEPIA_FG: &str = "#5b4636";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DecorationsMode {
    /// Hide the title bar on tiling WMs, show it elsewhere.
    #[default]
    Auto,
    /// Always show close/minimize/maximize (floating desktops).
    Always,
    /// Always hide decorations.
    Never,
}

impl DecorationsMode {
    pub fn hide_title_bar(self) -> bool {
        match self {
            Self::Always => false,
            Self::Never => true,
            Self::Auto => wm::is_tiling_wm(),
        }
    }
}

impl FromStr for DecorationsMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            other => Err(format!(
                "unknown decorations mode '{other}', expected auto, always, or never"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomPalette {
    #[serde(default = "default_custom_mode")]
    pub mode: String,
    #[serde(default = "default_custom_bg")]
    pub background: String,
    #[serde(default = "default_custom_fg")]
    pub foreground: String,
    #[serde(default)]
    pub muted: Option<String>,
    #[serde(default)]
    pub border: Option<String>,
    #[serde(default)]
    pub primary: Option<String>,
}

impl Default for CustomPalette {
    fn default() -> Self {
        Self {
            mode: default_custom_mode(),
            background: default_custom_bg(),
            foreground: default_custom_fg(),
            muted: Some("#8b949e".into()),
            border: Some("#30363d".into()),
            primary: Some("#4493f8".into()),
        }
    }
}

impl CustomPalette {
    pub fn sepia() -> Self {
        Self {
            mode: "light".into(),
            background: SEPIA_BG.into(),
            foreground: SEPIA_FG.into(),
            muted: Some("#8a7357".into()),
            border: Some("#d4c4a8".into()),
            primary: Some("#8b5e3c".into()),
        }
    }

    pub fn is_dark(&self) -> bool {
        self.mode.eq_ignore_ascii_case("dark")
            || channel_sum(&self.background).is_some_and(|sum| sum <= 382)
    }
}

fn default_custom_mode() -> String {
    "dark".into()
}
fn default_custom_bg() -> String {
    GITHUB_DARK_BG.into()
}
fn default_custom_fg() -> String {
    GITHUB_DARK_FG.into()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    /// Named GPUI theme, or `"custom"`.
    pub theme: String,
    /// Installed family. `None` / `"system"` uses the platform UI font.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mono_font: Option<String>,
    pub size: f32,
    pub radius: f32,
    pub radius_lg: f32,
    pub shadow: bool,
    #[serde(default)]
    pub decorations: DecorationsMode,
    #[serde(default)]
    pub custom: CustomPalette,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: DEFAULT_THEME.into(),
            font: None,
            mono_font: None,
            size: 18.0,
            radius: 6.0,
            radius_lg: 8.0,
            shadow: true,
            decorations: DecorationsMode::Auto,
            custom: CustomPalette::default(),
        }
    }
}

impl Settings {
    pub fn font_family(&self) -> String {
        match self.font.as_deref().map(str::trim) {
            None | Some("") | Some("system") => SYSTEM_FONT.into(),
            Some(name) => first_family(name).to_string(),
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn uses_system_font(&self) -> bool {
        matches!(
            self.font.as_deref().map(str::trim),
            None | Some("") | Some("system")
        )
    }

    pub fn mono_font_family(&self) -> Option<String> {
        match self.mono_font.as_deref().map(str::trim) {
            None | Some("") => None,
            Some("system") => Some(SYSTEM_FONT.into()),
            Some(name) => Some(first_family(name).to_string()),
        }
    }

    pub fn is_custom(&self) -> bool {
        self.theme == CUSTOM_THEME
    }
}

fn first_family(stack: &str) -> &str {
    stack
        .split(',')
        .next()
        .unwrap_or(stack)
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSettings {
    theme: Option<String>,
    font: Option<String>,
    mono_font: Option<String>,
    size: Option<f32>,
    radius: Option<f32>,
    #[serde(alias = "radius_lg", alias = "radius.lg")]
    radius_lg: Option<f32>,
    shadow: Option<bool>,
    decorations: Option<DecorationsMode>,
    custom: Option<CustomPalette>,
    bg: Option<String>,
    fg: Option<String>,
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("mdread")
        .join("config.toml")
}

pub fn load() -> Settings {
    let path = config_path();
    let Ok(raw) = fs::read_to_string(path) else {
        return Settings::default();
    };
    from_toml(&raw)
}

pub fn from_toml(raw: &str) -> Settings {
    let parsed: RawSettings = toml::from_str(raw).unwrap_or_default();
    normalize(parsed)
}

fn normalize(raw: RawSettings) -> Settings {
    let mut settings = Settings::default();
    if let Some(size) = raw.size {
        settings.size = size;
    }
    if let Some(radius) = raw.radius {
        settings.radius = radius;
    }
    if let Some(radius_lg) = raw.radius_lg {
        settings.radius_lg = radius_lg;
    }
    if let Some(shadow) = raw.shadow {
        settings.shadow = shadow;
    }
    if let Some(decorations) = raw.decorations {
        settings.decorations = decorations;
    }
    settings.font = normalize_font(raw.font);
    settings.mono_font = raw.mono_font.filter(|s| !s.trim().is_empty());
    if let Some(custom) = raw.custom {
        settings.custom = custom;
    }

    let theme = raw.theme.unwrap_or_else(|| DEFAULT_THEME.into());
    let bg = raw.bg;
    let fg = raw.fg;

    match theme.as_str() {
        "dark" => {
            if matches_preset(bg.as_deref(), fg.as_deref(), GITHUB_DARK_BG, GITHUB_DARK_FG) {
                settings.theme = DEFAULT_THEME.into();
            } else {
                apply_legacy_colors(&mut settings, bg, fg, "dark");
            }
        }
        "light" => {
            if matches_preset(
                bg.as_deref(),
                fg.as_deref(),
                GITHUB_LIGHT_BG,
                GITHUB_LIGHT_FG,
            ) {
                settings.theme = "Default Light".into();
            } else {
                apply_legacy_colors(&mut settings, bg, fg, "light");
            }
        }
        "sepia" => {
            settings.theme = CUSTOM_THEME.into();
            settings.custom = CustomPalette::sepia();
            if let Some(bg) = bg {
                settings.custom.background = bg;
            }
            if let Some(fg) = fg {
                settings.custom.foreground = fg;
            }
        }
        CUSTOM_THEME => {
            settings.theme = CUSTOM_THEME.into();
            if let Some(bg) = bg {
                settings.custom.background = bg;
            }
            if let Some(fg) = fg {
                settings.custom.foreground = fg;
            }
        }
        other => {
            settings.theme = other.to_string();
            if bg.is_some() || fg.is_some() {
                apply_legacy_colors(
                    &mut settings,
                    bg,
                    fg,
                    if other.to_ascii_lowercase().contains("light") {
                        "light"
                    } else {
                        "dark"
                    },
                );
            }
        }
    }

    settings
}

fn normalize_font(font: Option<String>) -> Option<String> {
    let Some(font) = font else {
        return None;
    };
    let trimmed = font.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("system") {
        return None;
    }
    Some(font)
}

fn apply_legacy_colors(
    settings: &mut Settings,
    bg: Option<String>,
    fg: Option<String>,
    mode: &str,
) {
    settings.theme = CUSTOM_THEME.into();
    if let Some(bg) = bg {
        settings.custom.background = bg;
    }
    if let Some(fg) = fg {
        settings.custom.foreground = fg;
    }
    settings.custom.mode = mode.into();
}

fn matches_preset(bg: Option<&str>, fg: Option<&str>, preset_bg: &str, preset_fg: &str) -> bool {
    match (bg, fg) {
        (None, None) => true,
        (Some(bg), Some(fg)) => eq_hex(bg, preset_bg) && eq_hex(fg, preset_fg),
        (Some(bg), None) => eq_hex(bg, preset_bg),
        (None, Some(fg)) => eq_hex(fg, preset_fg),
    }
}

fn eq_hex(a: &str, b: &str) -> bool {
    a.trim().eq_ignore_ascii_case(b)
}

fn channel_sum(color: &str) -> Option<u32> {
    let hex = color.trim().strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u32::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u32::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u32::from_str_radix(&hex[4..6], 16).ok()?;
    Some(r + g + b)
}

pub fn save(settings: &Settings) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let raw = toml::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_uses_system_font_and_named_theme() {
        let settings = Settings::default();
        assert!(settings.uses_system_font());
        assert_eq!(settings.font_family(), SYSTEM_FONT);
        assert_eq!(settings.theme, DEFAULT_THEME);
    }

    #[test]
    fn legacy_dark_github_becomes_default_dark() {
        let settings = from_toml(
            r##"
theme = "dark"
bg = "#0d1117"
fg = "#f0f6fc"
"##,
        );
        assert_eq!(settings.theme, DEFAULT_THEME);
    }

    #[test]
    fn legacy_light_github_becomes_default_light() {
        let settings = from_toml(
            r##"
theme = "light"
bg = "#ffffff"
fg = "#1f2328"
"##,
        );
        assert_eq!(settings.theme, "Default Light");
    }

    #[test]
    fn omarchy_colors_become_custom() {
        let settings = from_toml(
            r##"
theme = "dark"
bg = "#1a1b26"
fg = "#c0caf5"
"##,
        );
        assert_eq!(settings.theme, CUSTOM_THEME);
        assert_eq!(settings.custom.background, "#1a1b26");
        assert_eq!(settings.custom.foreground, "#c0caf5");
    }

    #[test]
    fn sepia_becomes_custom() {
        let settings = from_toml(r#"theme = "sepia""#);
        assert_eq!(settings.theme, CUSTOM_THEME);
        assert_eq!(settings.custom.background, SEPIA_BG);
    }

    #[test]
    fn custom_section_and_top_level_aliases() {
        let settings = from_toml(
            r##"
theme = "custom"
bg = "#111111"
fg = "#eeeeee"

[custom]
mode = "dark"
muted = "#888888"
"##,
        );
        assert_eq!(settings.theme, CUSTOM_THEME);
        assert_eq!(settings.custom.background, "#111111");
        assert_eq!(settings.custom.foreground, "#eeeeee");
        assert_eq!(settings.custom.muted.as_deref(), Some("#888888"));
    }

    #[test]
    fn omitted_font_is_system() {
        let settings = from_toml("theme = \"Default Dark\"\nsize = 18.0\n");
        assert!(settings.uses_system_font());
    }

    #[test]
    fn explicit_font_is_kept() {
        let settings = from_toml("font = \"Inter\"\n");
        assert_eq!(settings.font.as_deref(), Some("Inter"));
        assert_eq!(settings.font_family(), "Inter");
    }

    #[test]
    fn legacy_georgia_stack_is_kept() {
        let georgia = r#"Georgia, "Iowan Old Style", Palatino, serif"#;
        let settings = from_toml(&format!("font = {georgia:?}\n"));
        assert_eq!(settings.font.as_deref(), Some(georgia));
        assert_eq!(settings.font_family(), "Georgia");
    }
}
