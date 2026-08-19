//! The 16-color name palette (SPEC §5.4) — the replacement for free color picking.
//!
//! Colors travel as *named keys* (`"azure"`), never as hex or OKLCH literals. Each
//! key maps to one OKLCH hue with theme-mirrored lightness, so contrast against both
//! theme backgrounds is guaranteed by construction and nothing is clamped at runtime.
//! The property test at the bottom asserts ≥4.5:1 for all 16 keys × 2 themes and runs
//! in CI — it is the guard against someone "improving" a value later.

use serde::{Deserialize, Serialize};

/// dark: oklch(0.76 0.13 hue) · light: oklch(0.50 0.14 hue) · slate: chroma 0.02.
const DARK_L: f64 = 0.76;
const DARK_C: f64 = 0.13;
const LIGHT_L: f64 = 0.50;
const LIGHT_C: f64 = 0.14;
const SLATE_C: f64 = 0.02;

/// Theme backgrounds the palette must hold ≥4.5:1 against (SPEC §5.3 surface-0).
pub const DARK_BG: &str = "#16181C";
pub const LIGHT_BG: &str = "#F7F8F9";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Theme {
    Dark,
    Light,
}

/// One named palette entry: a key and its OKLCH hue.
#[derive(Debug, Clone, Copy)]
pub struct PaletteColor {
    pub key: &'static str,
    pub hue: f64,
    /// slate is the one desaturated entry (chroma 0.02 instead of 0.13/0.14).
    pub muted: bool,
}

const fn c(key: &'static str, hue: f64) -> PaletteColor {
    PaletteColor {
        key,
        hue,
        muted: false,
    }
}

/// The IRC 16, remapped through OKLCH. Order matters only for the picker UI.
pub const PALETTE: [PaletteColor; 16] = [
    c("ember", 32.0),
    c("rust", 50.0),
    c("amber", 68.0),
    c("brass", 90.0),
    c("lime", 118.0),
    c("fern", 145.0),
    c("mint", 162.0),
    c("teal", 180.0),
    c("cyan", 200.0),
    c("sky", 230.0),
    c("azure", 255.0),
    c("indigo", 275.0),
    c("violet", 295.0),
    c("orchid", 320.0),
    c("rose", 350.0),
    PaletteColor {
        key: "slate",
        hue: 250.0,
        muted: true,
    },
];

/// Whether `key` names a palette color. This is what the server validates wire
/// `ColorKey` values against — client-side validation alone is a defect.
pub fn is_valid_color_key(key: &str) -> bool {
    PALETTE.iter().any(|p| p.key == key)
}

/// Look up a palette entry by key.
pub fn get(key: &str) -> Option<&'static PaletteColor> {
    PALETTE.iter().find(|p| p.key == key)
}

impl PaletteColor {
    /// The OKLCH components for this color in the given theme.
    #[must_use]
    pub fn oklch(&self, theme: Theme) -> (f64, f64, f64) {
        let chroma = if self.muted {
            SLATE_C
        } else {
            match theme {
                Theme::Dark => DARK_C,
                Theme::Light => LIGHT_C,
            }
        };
        let lightness = match theme {
            Theme::Dark => DARK_L,
            Theme::Light => LIGHT_L,
        };
        (lightness, chroma, self.hue)
    }

    /// sRGB hex fallback (`#rrggbb`), gamut-clipped. The build ships these in case
    /// the target WebKitGTK's `oklch()` support falls through (SPEC §5.4).
    #[must_use]
    pub fn hex(&self, theme: Theme) -> String {
        let (l, ch, h) = self.oklch(theme);
        let (r, g, b) = oklch_to_srgb(l, ch, h);
        format!(
            "#{:02X}{:02X}{:02X}",
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8
        )
    }

    /// The `oklch()` CSS literal for this color in the given theme.
    #[must_use]
    pub fn css_oklch(&self, theme: Theme) -> String {
        let (l, ch, h) = self.oklch(theme);
        format!("oklch({l} {ch} {h})")
    }
}

/// CSS custom properties for every palette entry in one theme, e.g.
/// `--name-azure: #8FB3FF;`. M7 wires this into the client build so the palette
/// is defined exactly once, here.
#[must_use]
pub fn css_variables(theme: Theme) -> String {
    PALETTE
        .iter()
        .map(|p| format!("--name-{}: {};", p.key, p.hex(theme)))
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// Color math. OKLCH → OKLab → linear sRGB → gamma sRGB, then WCAG contrast.
// Matrices from Björn Ottosson's OKLab reference implementation.
// ---------------------------------------------------------------------------

fn oklch_to_srgb(l: f64, chroma: f64, hue_deg: f64) -> (f64, f64, f64) {
    let hr = hue_deg.to_radians();
    let a = chroma * hr.cos();
    let b = chroma * hr.sin();

    let l_ = l + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
    let m_ = l - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
    let s_ = l - 0.089_484_177_5 * a - 1.291_485_548_0 * b;

    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    let r = 4.076_741_662_1 * l3 - 3.307_711_591_3 * m3 + 0.230_969_929_2 * s3;
    let g = -1.268_438_004_6 * l3 + 2.609_757_401_1 * m3 - 0.341_319_396_5 * s3;
    let b = -0.004_196_086_3 * l3 - 0.703_418_614_7 * m3 + 1.707_614_701_0 * s3;

    (
        gamma(r.clamp(0.0, 1.0)),
        gamma(g.clamp(0.0, 1.0)),
        gamma(b.clamp(0.0, 1.0)),
    )
}

fn gamma(linear: f64) -> f64 {
    if linear <= 0.003_130_8 {
        12.92 * linear
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

fn linearize(channel: f64) -> f64 {
    if channel <= 0.040_45 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn hex_to_rgb(hex: &str) -> (f64, f64, f64) {
    let h = hex.trim_start_matches('#');
    let parse = |i: usize| f64::from(u8::from_str_radix(&h[i..i + 2], 16).unwrap_or(0)) / 255.0;
    (parse(0), parse(2), parse(4))
}

fn relative_luminance((r, g, b): (f64, f64, f64)) -> f64 {
    0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b)
}

/// WCAG contrast ratio between two `#rrggbb` colors.
#[must_use]
pub fn contrast_ratio(hex_a: &str, hex_b: &str) -> f64 {
    let la = relative_luminance(hex_to_rgb(hex_a));
    let lb = relative_luminance(hex_to_rgb(hex_b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The CI guard from PROTOCOL §5 / AGENTS.md: every palette key must hold
    /// ≥4.5:1 against its theme background. If this fails, a palette value was
    /// changed in a way that breaks structural contrast safety — fix the value,
    /// never this test.
    #[test]
    fn all_16_keys_hold_contrast_in_both_themes() {
        for color in &PALETTE {
            for (theme, bg) in [(Theme::Dark, DARK_BG), (Theme::Light, LIGHT_BG)] {
                let hex = color.hex(theme);
                let ratio = contrast_ratio(&hex, bg);
                assert!(
                    ratio >= 4.5,
                    "{} in {theme:?} theme is {hex} → {ratio:.2}:1 against {bg}, below 4.5:1",
                    color.key
                );
            }
        }
    }

    #[test]
    fn palette_is_exactly_the_named_16() {
        assert_eq!(PALETTE.len(), 16);
        for key in [
            "ember", "rust", "amber", "brass", "lime", "fern", "mint", "teal", "cyan", "sky",
            "azure", "indigo", "violet", "orchid", "rose", "slate",
        ] {
            assert!(is_valid_color_key(key), "missing palette key {key}");
        }
        assert!(!is_valid_color_key("hotpink"));
        assert!(!is_valid_color_key("#ff00ff"));
    }

    #[test]
    fn known_conversion_sanity() {
        // oklch(1 0 0) is white; oklch(0 0 0) is black.
        let (r, g, b) = oklch_to_srgb(1.0, 0.0, 0.0);
        assert!(r > 0.999 && g > 0.999 && b > 0.999);
        let (r, g, b) = oklch_to_srgb(0.0, 0.0, 0.0);
        assert!(r < 0.001 && g < 0.001 && b < 0.001);
    }

    #[test]
    fn css_variables_emit_all_keys() {
        let css = css_variables(Theme::Dark);
        assert_eq!(css.lines().count(), 16);
        assert!(css.contains("--name-azure: #"));
    }
}
