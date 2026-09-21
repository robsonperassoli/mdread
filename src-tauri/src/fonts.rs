//! Installed font families, from fontconfig. The app does not follow a desktop
//! theme; whatever name is stored in the config is the family that gets used.

use std::process::Command;
use std::sync::Mutex;

struct FontCache {
    fonts: Vec<String>,
}

static FONT_CACHE: Mutex<Option<FontCache>> = Mutex::new(None);

pub fn installed() -> Vec<String> {
    let mut guard = FONT_CACHE
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    if let Some(cache) = guard.as_ref() {
        return cache.fonts.clone();
    }
    let fonts = query_fonts();
    *guard = Some(FontCache {
        fonts: fonts.clone(),
    });
    fonts
}

fn query_fonts() -> Vec<String> {
    let output = Command::new("fc-list")
        .args(["-f", "%{family[0]}\n"])
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let mut names: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty() && !skip_font(name))
        .map(str::to_string)
        .collect();
    names.sort_by(|a, b| a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()));
    names.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    names
}

fn skip_font(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("emoji") || lower.contains("signwriting") || lower == "omarchy"
}
