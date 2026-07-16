use slint::{Brush, Color};

pub const PALETTE: &[&str] = &[
    "#2b2b2b", "#1e1e1e", "#444444", "#111111",
    "#e63946", "#2ecc71", "#1565c0", "#ff9800",
    "#9d4edd", "#00c853", "#ffb703", "#f72585",
];

pub fn parse_hex(hex: &str) -> Option<Brush> {
    let h = hex.trim().trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()?;
    let g = u8::from_str_radix(&h[2..4], 16).ok()?;
    let b = u8::from_str_radix(&h[4..6], 16).ok()?;
    Some(Brush::from(Color::from_rgb_u8(r, g, b)))
}

pub fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

pub fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let h = hex.trim().trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()?;
    let g = u8::from_str_radix(&h[2..4], 16).ok()?;
    let b = u8::from_str_radix(&h[4..6], 16).ok()?;
    Some((r, g, b))
}

pub fn brush_from_rgb(r: u8, g: u8, b: u8) -> Brush {
    Brush::from(Color::from_rgb_u8(r, g, b))
}

pub fn brush_from_hex_or_default(hex: Option<&str>, default: (u8, u8, u8)) -> Brush {
    hex.and_then(|h| parse_hex(h))
        .unwrap_or_else(|| Brush::from(Color::from_rgb_u8(default.0, default.1, default.2)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_green() {
        assert!(parse_hex("#2ecc71").is_some());
    }
}
