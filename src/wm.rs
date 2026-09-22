//! Detect tiling window managers so we can drop client-side decorations.
//!
//! Wayland has no portable "am I tiled?" query. Other Linux apps therefore
//! treat the *session* as tiling: compositor sockets first, then colon-separated
//! `XDG_CURRENT_DESKTOP` / `XDG_SESSION_DESKTOP` tokens.

use std::env;

const TILING_DESKTOPS: &[&str] = &[
    "hyprland",
    "sway",
    "i3",
    "river",
    "niri",
    "bspwm",
    "dwm",
    "dwl",
    "qtile",
    "xmonad",
    "awesome",
    "herbstluftwm",
    "leftwm",
    "spectrwm",
    "miracle",
    "miracle-wm",
    "omarchy",
];

pub fn is_tiling_wm() -> bool {
    #[cfg(target_os = "linux")]
    {
        is_tiling_wm_from(|key| env::var(key).ok())
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

pub fn is_tiling_wm_from(get: impl Fn(&str) -> Option<String>) -> bool {
    // Live compositor sockets beat a stale desktop name inherited after a WM switch.
    if get("HYPRLAND_INSTANCE_SIGNATURE").is_some()
        || get("SWAYSOCK").filter(|s| !s.is_empty()).is_some()
        || get("I3SOCK").filter(|s| !s.is_empty()).is_some()
        || get("NIRI_SOCKET").filter(|s| !s.is_empty()).is_some()
        || get("BSPWM_SOCKET").filter(|s| !s.is_empty()).is_some()
    {
        return true;
    }

    desktop_tokens(&get)
        .iter()
        .any(|token| TILING_DESKTOPS.contains(&token.as_str()))
}

fn desktop_tokens(get: &impl Fn(&str) -> Option<String>) -> Vec<String> {
    [
        "XDG_CURRENT_DESKTOP",
        "XDG_SESSION_DESKTOP",
        "DESKTOP_SESSION",
    ]
    .into_iter()
    .filter_map(|var| get(var))
    .flat_map(|value| {
        value
            .split(':')
            .map(|part| part.trim().to_ascii_lowercase())
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::is_tiling_wm_from;

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |key| {
            pairs
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| (*v).to_string())
        }
    }

    #[test]
    fn hyprland_socket() {
        assert!(is_tiling_wm_from(env(&[(
            "HYPRLAND_INSTANCE_SIGNATURE",
            "sig"
        )])));
    }

    #[test]
    fn hyprland_desktop_name() {
        assert!(is_tiling_wm_from(env(&[(
            "XDG_CURRENT_DESKTOP",
            "Hyprland"
        )])));
    }

    #[test]
    fn omarchy_session_name() {
        assert!(is_tiling_wm_from(env(&[("DESKTOP_SESSION", "omarchy")])));
    }

    #[test]
    fn colon_separated_desktop() {
        assert!(is_tiling_wm_from(env(&[(
            "XDG_CURRENT_DESKTOP",
            "omarchy:Hyprland"
        )])));
    }

    #[test]
    fn gnome_is_not_tiling() {
        assert!(!is_tiling_wm_from(env(&[(
            "XDG_CURRENT_DESKTOP",
            "ubuntu:GNOME"
        )])));
    }

    #[test]
    fn stale_gnome_loses_to_hyprland_socket() {
        assert!(is_tiling_wm_from(env(&[
            ("XDG_CURRENT_DESKTOP", "GNOME"),
            ("HYPRLAND_INSTANCE_SIGNATURE", "sig"),
        ])));
    }

    #[test]
    fn empty_sway_socket_ignored() {
        assert!(!is_tiling_wm_from(env(&[("SWAYSOCK", "")])));
    }
}
