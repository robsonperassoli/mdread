use gpui_kit::{Hsla, Rgba};

pub fn parse_hex(value: &str) -> Option<Hsla> {
    let value = value.trim().strip_prefix('#')?;
    if !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let (width, has_alpha) = match value.len() {
        3 => (1, false),
        4 => (1, true),
        6 => (2, false),
        8 => (2, true),
        _ => return None,
    };
    let component = |index: usize| {
        let start = index * width;
        let raw = u8::from_str_radix(&value[start..start + width], 16).ok()?;
        let raw = if width == 1 { raw * 0x11 } else { raw };
        Some(raw as f32 / 255.0)
    };
    Some(
        Rgba {
            r: component(0)?,
            g: component(1)?,
            b: component(2)?,
            a: if has_alpha { component(3)? } else { 1.0 },
        }
        .into(),
    )
}

pub fn to_hex(color: Hsla) -> String {
    let rgba = Rgba::from(color);
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.).round() as u32;
    if rgba.a < 1. {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            channel(rgba.r),
            channel(rgba.g),
            channel(rgba.b),
            channel(rgba.a)
        )
    } else {
        format!(
            "#{:02X}{:02X}{:02X}",
            channel(rgba.r),
            channel(rgba.g),
            channel(rgba.b)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_hex() {
        let color = parse_hex("#0d1117").expect("parse");
        assert_eq!(to_hex(color), "#0D1117");
    }
}
