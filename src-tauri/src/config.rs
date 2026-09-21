use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use crate::wm;

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
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// A CSS font stack or a single installed family name.
    pub font: String,
    pub size: f32,
    /// `light`, `dark`, or legacy `sepia`. Anything else follows the background luminance.
    pub theme: String,
    pub bg: String,
    pub fg: String,
    #[serde(default)]
    pub decorations: DecorationsMode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font: "Georgia, \"Iowan Old Style\", Palatino, serif".into(),
            size: 18.0,
            theme: "dark".into(),
            bg: "#0d1117".into(),
            fg: "#f0f6fc".into(),
            decorations: DecorationsMode::Auto,
        }
    }
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
    toml::from_str(&raw).unwrap_or_default()
}

impl Settings {
    /// Light GitHub syntax highlighting, as opposed to the dark theme.
    pub fn light_syntax(&self) -> bool {
        match self.theme.as_str() {
            "light" | "sepia" => true,
            "dark" => false,
            _ => channel_sum(&self.bg).is_some_and(|sum| sum > 382),
        }
    }
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
    fn named_themes_pick_syntax() {
        let mut settings = Settings::default();
        settings.theme = "dark".into();
        assert!(!settings.light_syntax());
        settings.theme = "light".into();
        assert!(settings.light_syntax());
        settings.theme = "sepia".into();
        assert!(settings.light_syntax());
    }

    #[test]
    fn unknown_theme_follows_background() {
        let mut settings = Settings::default();
        settings.theme = "custom".into();
        settings.bg = "#05182e".into();
        assert!(!settings.light_syntax());
        settings.bg = "#eff1f5".into();
        assert!(settings.light_syntax());
    }
}
