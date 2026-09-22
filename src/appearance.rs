use std::rc::Rc;

use gpui_kit::component::{Theme, ThemeConfig, ThemeConfigColors, ThemeMode, ThemeRegistry};
use gpui_kit::{px, App, SharedString, Window};

use crate::config::{CustomPalette, Settings, CUSTOM_THEME};

const BUNDLED_THEMES: &[&str] = &[
    include_str!("../assets/themes/adventure.json"),
    include_str!("../assets/themes/alduin.json"),
    include_str!("../assets/themes/asciinema.json"),
    include_str!("../assets/themes/aurora.json"),
    include_str!("../assets/themes/ayu.json"),
    include_str!("../assets/themes/catppuccin.json"),
    include_str!("../assets/themes/everforest.json"),
    include_str!("../assets/themes/fahrenheit.json"),
    include_str!("../assets/themes/flexoki.json"),
    include_str!("../assets/themes/gruvbox.json"),
    include_str!("../assets/themes/harper.json"),
    include_str!("../assets/themes/hybrid.json"),
    include_str!("../assets/themes/jellybeans.json"),
    include_str!("../assets/themes/kibble.json"),
    include_str!("../assets/themes/macos-classic.json"),
    include_str!("../assets/themes/mellifluous.json"),
    include_str!("../assets/themes/molokai.json"),
    include_str!("../assets/themes/solarized.json"),
    include_str!("../assets/themes/spaceduck.json"),
    include_str!("../assets/themes/tokyonight.json"),
    include_str!("../assets/themes/twilight.json"),
];

pub fn register_bundled_themes(cx: &mut App) {
    let registry = ThemeRegistry::global_mut(cx);
    for json in BUNDLED_THEMES {
        if let Err(err) = registry.load_themes_from_str(json) {
            eprintln!("mdread: ignored bundled theme: {err}");
        }
    }
}

pub fn theme_names(cx: &App) -> Vec<SharedString> {
    ThemeRegistry::global(cx)
        .sorted_themes()
        .into_iter()
        .map(|theme| theme.name.clone())
        .collect()
}

pub fn apply_settings(settings: &Settings, mut window: Option<&mut Window>, cx: &mut App) {
    if settings.is_custom() {
        let config = Rc::new(custom_theme_config(settings));
        Theme::global_mut(cx).apply_config(&config);
    } else if let Some(config) = ThemeRegistry::global(cx)
        .themes()
        .get(&SharedString::from(settings.theme.as_str()))
        .cloned()
    {
        Theme::global_mut(cx).apply_config(&config);
    } else {
        Theme::change(ThemeMode::Dark, window.as_deref_mut(), cx);
    }

    overlay_reader_prefs(settings, cx);
    Theme::sync_base(cx);
    if let Some(window) = window {
        window.refresh();
    } else {
        cx.refresh_windows();
    }
}

fn overlay_reader_prefs(settings: &Settings, cx: &mut App) {
    let theme = Theme::global_mut(cx);
    theme.font_size = px(settings.size);
    theme.font_family = settings.font_family().into();
    if let Some(mono) = settings.mono_font_family() {
        theme.mono_font_family = mono.into();
    }
    theme.radius = px(settings.radius);
    theme.radius_lg = px(settings.radius_lg);
    theme.shadow = settings.shadow;
}

fn custom_theme_config(settings: &Settings) -> ThemeConfig {
    let palette = &settings.custom;
    let mode = if palette.is_dark() {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    ThemeConfig {
        name: CUSTOM_THEME.into(),
        mode,
        font_size: Some(settings.size),
        font_family: Some(settings.font_family().into()),
        mono_font_family: settings.mono_font_family().map(Into::into),
        radius: Some(settings.radius.round() as usize),
        radius_lg: Some(settings.radius_lg.round() as usize),
        shadow: Some(settings.shadow),
        colors: colors_from_palette(palette),
        ..Default::default()
    }
}

fn colors_from_palette(palette: &CustomPalette) -> ThemeConfigColors {
    let mut colors = ThemeConfigColors::default();
    colors.background = Some(palette.background.clone().into());
    colors.foreground = Some(palette.foreground.clone().into());
    colors.muted = palette.muted.clone().map(Into::into);
    colors.muted_foreground = palette.muted.clone().map(Into::into);
    colors.border = palette.border.clone().map(Into::into);
    colors.primary = palette.primary.clone().map(Into::into);
    colors.title_bar = Some(palette.background.clone().into());
    colors.popover = Some(palette.background.clone().into());
    colors.popover_foreground = Some(palette.foreground.clone().into());
    colors
}
