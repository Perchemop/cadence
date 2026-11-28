//! Cadence Design Token System — Comprehensive Implementation
//!
//! This module defines the complete design token system for Cadence, mapping
//! semantic design decisions to egui Visuals/Style. It implements the full
//! specification from `docs/design/brand.md` and the discovery research
//! (CADENCE-DESIGN-TOKENS epic).
//!
//! ## Token Architecture (guide II.30.6)
//!
//! One theme module owns every colour, space, radius, elevation and text size.
//! Widget code reads semantic roles (`surface.base`, `text.muted`, `accent`,
//! `focus.ring`, …); no raw colour literals outside this module.
//!
//! ## Colour System
//!
//! Two themes (light/dark) built on a neutral slate/graphite ramp with one
//! restrained accent (`#2F6FED`) and semantic colours for schedule meaning.
//! All contrast ratios WCAG 2.2 AA verified: 4.5:1 for body text, 3:1 for
//! large text / UI boundaries / focus rings.
//!
//! ## Typography Scale
//!
//! UI face: Inter (SIL OFL 1.1). Sizes: Display (24), H1 (20), H2 (16),
//! H3 (14), Body (11), Small (9.5), Caption (8), Mono (11). Weights:
//! Regular, Medium, SemiBold, Bold.
//!
//! ## Spacing System
//!
//! 4 px base unit with multipliers: 0.5 (2), 1 (4), 2 (8), 3 (12), 4 (16),
//! 6 (24), 8 (32), 12 (48), 16 (64). Single 4 px grid everywhere.
//!
//! ## Radius System
//!
//! None (0), sm (2), md (4), lg (6), xl (8), full (9999).
//!
//! ## Elevation / Shadow System
//!
//! 4 levels: level0 (none), level1 (subtle), level2 (card), level3 (modal).
//!
//! ## Motion Specs
//!
//! Durations: fast (100ms), normal (200ms), slow (300ms). Easing: ease-out
//! (cubic-bezier(0.25, 0.46, 0.45, 0.94)). Reduced-motion: all durations 0.
//!
//! ## Density Tokens
//!
//! Compact (0.875x), Standard (1.0x), Comfortable (1.125x) — multipliers
//! applied to spacing and sizing tokens.
//!
//! ## egui Integration
//!
//! `Theme::apply` maps all tokens onto `egui::Visuals` and `egui::Style`.

use egui::{
    Color32, Context, FontData, FontDefinitions, FontFamily, FontId, Margin, Stroke, TextStyle,
    Visuals,
};
use serde::{de::Visitor, Deserialize, Serialize};
use std::fmt;
use std::sync::{Mutex, OnceLock};

// Custom serde for Color32: hex strings "#RRGGBBAA" (reads both 6- and
// 8-digit forms so older "#RRGGBB" exports still load losslessly for opaque
// colours). Color32 stores premultiplied bytes, so serializing the stored
// bytes and reading them back with `from_rgba_premultiplied` is exact.
mod color32_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub(crate) fn to_hex(color: &Color32) -> String {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            color.r(),
            color.g(),
            color.b(),
            color.a()
        )
    }

    pub(crate) fn from_hex(v: &str) -> Result<Color32, String> {
        let v = v.trim_start_matches('#');
        let digit = |s: &str| u8::from_str_radix(s, 16).map_err(|e| e.to_string());
        match v.len() {
            6 => Ok(Color32::from_rgb(
                digit(&v[0..2])?,
                digit(&v[2..4])?,
                digit(&v[4..6])?,
            )),
            8 => Ok(Color32::from_rgba_premultiplied(
                digit(&v[0..2])?,
                digit(&v[2..4])?,
                digit(&v[4..6])?,
                digit(&v[6..8])?,
            )),
            _ => Err(format!("expected 6 or 8 hex digits, got {v}")),
        }
    }

    pub fn serialize<S>(color: &Color32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&to_hex(color))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color32, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Color32Visitor;

        impl<'de> Visitor<'de> for Color32Visitor {
            type Value = Color32;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a hex color string like #RRGGBB")
            }

            fn visit_str<E>(self, v: &str) -> Result<Color32, E>
            where
                E: serde::de::Error,
            {
                let v = v.trim_start_matches('#');
                from_hex(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(Color32Visitor)
    }
}

// ============================================================================
// COLOUR TOKENS
// ============================================================================

/// Semantic colour roles for a single theme.
///
/// All colours are stored as sRGB `Color32`. Contrast is verified in
/// `tests/design_token_tests.rs` against WCAG 2.2 AA.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorTokens {
    /// Window background behind panels.
    #[serde(with = "color32_serde")]
    pub surface_base: Color32,
    /// Panel, grid, popup surface.
    #[serde(with = "color32_serde")]
    pub surface_raised: Color32,
    /// Sunken/pressed surface (active widget background).
    #[serde(with = "color32_serde")]
    pub surface_sunken: Color32,
    /// Body text.
    #[serde(with = "color32_serde")]
    pub text_primary: Color32,
    /// Secondary text: units, hints, dimmed numerics.
    #[serde(with = "color32_serde")]
    pub text_secondary: Color32,
    /// Disabled/muted text.
    #[serde(with = "color32_serde")]
    pub text_muted: Color32,
    /// Text on accent fills (primary button, selection).
    #[serde(with = "color32_serde")]
    pub text_on_accent: Color32,
    /// Primary brand accent: selection, links, focus ring.
    #[serde(with = "color32_serde")]
    pub accent: Color32,
    /// Accent hover state.
    #[serde(with = "color32_serde")]
    pub accent_hover: Color32,
    /// Accent active/pressed state.
    #[serde(with = "color32_serde")]
    pub accent_active: Color32,
    /// Critical: delays, overspends, failures.
    #[serde(with = "color32_serde")]
    pub status_critical: Color32,
    /// Critical on surface (for text on critical backgrounds).
    #[serde(with = "color32_serde")]
    pub status_critical_on: Color32,
    /// Normal: non-critical bars, info.
    #[serde(with = "color32_serde")]
    pub status_normal: Color32,
    /// OK: passes, on-track.
    #[serde(with = "color32_serde")]
    pub status_ok: Color32,
    /// OK on surface.
    #[serde(with = "color32_serde")]
    pub status_ok_on: Color32,
    /// Warning.
    #[serde(with = "color32_serde")]
    pub status_warn: Color32,
    /// Warning on surface.
    #[serde(with = "color32_serde")]
    pub status_warn_on: Color32,
    /// Hairlines, table grid lines, Gantt grid.
    #[serde(with = "color32_serde")]
    pub border_subtle: Color32,
    /// Strong borders: input outlines, selected tab underline.
    #[serde(with = "color32_serde")]
    pub border_strong: Color32,
    /// Keyboard focus ring (maps to active widget stroke in egui).
    #[serde(with = "color32_serde")]
    pub focus_ring: Color32,
    /// Selection background (table row, text selection).
    #[serde(with = "color32_serde")]
    pub selection_bg: Color32,
    /// Selection text colour.
    #[serde(with = "color32_serde")]
    pub selection_text: Color32,
    /// Extreme background (e.g., Gantt canvas inside a panel).
    #[serde(with = "color32_serde")]
    pub surface_deep: Color32,
    /// Non-working day background tint (weekends/holidays on Gantt).
    #[serde(with = "color32_serde")]
    pub non_working_day: Color32,
    /// Status date line (dotted indicator).
    #[serde(with = "color32_serde")]
    pub status_date: Color32,
    /// Overlay scrim for modals.
    #[serde(with = "color32_serde")]
    pub overlay_scrim: Color32,
    /// Disabled widget background.
    #[serde(with = "color32_serde")]
    pub disabled_bg: Color32,
    /// Disabled widget text.
    #[serde(with = "color32_serde")]
    pub disabled_text: Color32,
    /// Legacy: non-working time background (alias for non_working_day).
    #[serde(with = "color32_serde")]
    pub non_working: Color32,
    /// Legacy: positive variance (improvements, green).
    #[serde(with = "color32_serde")]
    pub variance_positive: Color32,
    /// Legacy: negative variance (delays/overspends, red).
    #[serde(with = "color32_serde")]
    pub variance_negative: Color32,
}

/// Light theme colour tokens (from brand.md) — WCAG 2.2 AA compliant.
pub static LIGHT_COLORS: ColorTokens = ColorTokens {
    surface_base: Color32::from_rgb(0xF4, 0xF5, 0xF7), // #F4F5F7
    surface_raised: Color32::from_rgb(0xFF, 0xFF, 0xFF), // #FFFFFF
    surface_sunken: Color32::from_rgb(0xE2, 0xE8, 0xF2), // #E2E8F2
    text_primary: Color32::from_rgb(0x1F, 0x29, 0x33), // #1F2933
    text_secondary: Color32::from_rgb(0x4A, 0x55, 0x68), // #4A5568 (darker for 4.5:1)
    text_muted: Color32::from_rgb(0x5F, 0x6B, 0x7A),   // #5F6B7A (4.5:1 on surface_base/raised)
    text_on_accent: Color32::WHITE,
    accent: Color32::from_rgb(0x2F, 0x6F, 0xED), // #2F6FED
    accent_hover: Color32::from_rgb(0x1E, 0x5F, 0xD6), // #1E5FD6 (darker)
    accent_active: Color32::from_rgb(0x1A, 0x52, 0xBB), // #1A52BB (darker still)
    status_critical: Color32::from_rgb(0xB9, 0x1C, 0x1C), // #B91C1C
    status_critical_on: Color32::WHITE,
    status_normal: Color32::from_rgb(0x1D, 0x4E, 0xD8), // #1D4ED8
    status_ok: Color32::from_rgb(0x15, 0x80, 0x3D),     // #15803D
    status_ok_on: Color32::WHITE,
    status_warn: Color32::from_rgb(0xB4, 0x53, 0x09), // #B45309
    status_warn_on: Color32::WHITE,
    border_subtle: Color32::from_rgb(0x78, 0x80, 0x98), // #788098 (3:1 on surface_base/raised)
    border_strong: Color32::from_rgb(0x60, 0x6C, 0x84), // #606C84 (3:1 on surface_base/raised)
    focus_ring: Color32::from_rgb(0x2F, 0x6F, 0xED),    // same as accent
    selection_bg: Color32::from_rgb(0x2F, 0x6F, 0xED),  // accent
    selection_text: Color32::WHITE,
    surface_deep: Color32::from_rgb(0xEC, 0xEE, 0xF1), // #ECEEF1
    non_working_day: Color32::from_rgb(0x80, 0x8C, 0xA8), // #808CA8 (3:1 on surface_base)
    status_date: Color32::from_rgba_premultiplied(0xB9, 0x1C, 0x1C, 128), // 50% critical
    overlay_scrim: Color32::from_rgba_premultiplied(0x00, 0x00, 0x00, 0x80), // 50% black
    disabled_bg: Color32::from_rgb(0xE8, 0xEB, 0xF0),  // #E8EBF0
    disabled_text: Color32::from_rgb(0x5F, 0x6B, 0x7A), // #5F6B7A (3:1 on disabled_bg)
    // Legacy fields
    non_working: Color32::from_rgb(0x90, 0x9C, 0xB8), // #909CB8 (3:1 on surface_base)
    variance_positive: Color32::from_rgb(0x15, 0x80, 0x3D), // same as status_ok (green)
    variance_negative: Color32::from_rgb(0xB9, 0x1C, 0x1C), // same as status_critical (red)
};

/// Dark theme colour tokens (from brand.md) — WCAG 2.2 AA compliant.
pub static DARK_COLORS: ColorTokens = ColorTokens {
    surface_base: Color32::from_rgb(0x16, 0x18, 0x1D), // #16181D
    surface_raised: Color32::from_rgb(0x1D, 0x20, 0x26), // #1D2026
    surface_sunken: Color32::from_rgb(0x2E, 0x33, 0x3D), // #2E333D
    text_primary: Color32::from_rgb(0xE3, 0xE7, 0xEB), // #E3E7EB
    text_secondary: Color32::from_rgb(0xB0, 0xB8, 0xC8), // #B0B8C8 (lighter for 4.5:1)
    text_muted: Color32::from_rgb(0x8A, 0x92, 0xA8),   // #8A92A8 (4.5:1 on surface_base/raised)
    text_on_accent: Color32::WHITE,
    accent: Color32::from_rgb(0x54, 0x72, 0x8A), // #54728A (WCAG AA: 3.51:1 on surface_base, 5.06:1 on white text)
    accent_hover: Color32::from_rgb(0x5A, 0x78, 0x96), // #5A7896 (WCAG AA: 3.86:1 on surface_base, 4.60:1 on white text)
    accent_active: Color32::from_rgb(0x54, 0x78, 0x8A), // #54788A (WCAG AA: 3.75:1 on surface_base, 4.74:1 on white text)
    status_critical: Color32::from_rgb(0xF8, 0x71, 0x71), // #F87171
    status_critical_on: Color32::from_rgb(0x16, 0x18, 0x1D), // dark surface
    status_normal: Color32::from_rgb(0x93, 0xC5, 0xFD), // #93C5FD
    status_ok: Color32::from_rgb(0x4A, 0xDE, 0x80),     // #4ADE80
    status_ok_on: Color32::from_rgb(0x16, 0x18, 0x1D),  // dark surface
    status_warn: Color32::from_rgb(0xFB, 0xBF, 0x24),   // #FBBF24
    status_warn_on: Color32::from_rgb(0x16, 0x18, 0x1D), // dark surface
    border_subtle: Color32::from_rgb(0x6A, 0x72, 0x8A), // #6A728A (3:1 on surface_base/raised)
    border_strong: Color32::from_rgb(0x7A, 0x82, 0x9A), // #7A829A (3:1 on surface_base/raised)
    focus_ring: Color32::from_rgb(0x2F, 0x6F, 0xED),    // same as accent
    selection_bg: Color32::from_rgb(0x2F, 0x6F, 0xED),  // accent
    selection_text: Color32::WHITE,
    surface_deep: Color32::from_rgb(0x11, 0x13, 0x18), // #111318
    non_working_day: Color32::from_rgb(0x68, 0x7A, 0x9A), // #687A9A (3:1 on surface_base)
    status_date: Color32::from_rgba_premultiplied(0xF8, 0x71, 0x71, 128), // 50% critical
    overlay_scrim: Color32::from_rgba_premultiplied(0x00, 0x00, 0x00, 0xA0), // 62.5% black
    disabled_bg: Color32::from_rgb(0x21, 0x24, 0x2B),  // #21242B
    disabled_text: Color32::from_rgb(0x8A, 0x92, 0xA8), // #8A92A8 (3:1 on disabled_bg)
    // Legacy fields
    non_working: Color32::from_rgb(0x68, 0x7A, 0x9A), // same as non_working_day
    variance_positive: Color32::from_rgb(0x4A, 0xDE, 0x80), // same as status_ok (green)
    variance_negative: Color32::from_rgb(0xF8, 0x71, 0x71), // same as status_critical (red)
};

// ============================================================================
// TYPOGRAPHY TOKENS
// ============================================================================

/// Typography scale with size, weight, line-height, and letter-spacing.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct TypographyToken {
    /// Font size in points.
    pub size: f32,
    /// Font weight name (maps to Inter family).
    pub weight: FontWeight,
    /// Line height as a multiple of font size.
    pub line_height: f32,
    /// Letter spacing in ems.
    pub letter_spacing: f32,
}

/// Font weight variants available in Inter.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FontWeight {
    Regular,
    Medium,
    SemiBold,
    Bold,
}

impl FontWeight {
    /// Returns the FontFamily name for this weight.
    pub fn family_name(self) -> &'static str {
        match self {
            FontWeight::Regular => "Inter-Regular",
            FontWeight::Medium => "Inter-Medium",
            FontWeight::SemiBold => "Inter-SemiBold",
            FontWeight::Bold => "Inter-Bold",
        }
    }
}

/// Complete typography scale.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TypographyScale {
    /// Large display text (page titles, empty states).
    pub display: TypographyToken,
    /// Heading 1 (major section headers).
    pub h1: TypographyToken,
    /// Heading 2 (subsection headers).
    pub h2: TypographyToken,
    /// Heading 3 (minor headers, panel titles).
    pub h3: TypographyToken,
    /// Body text (default).
    pub body: TypographyToken,
    /// Small text (secondary info, timestamps).
    pub small: TypographyToken,
    /// Caption text (footnotes, hints).
    pub caption: TypographyToken,
    /// Monospace for dates, numbers, code.
    pub mono: TypographyToken,
    /// Button text.
    pub button: TypographyToken,
}

impl Default for TypographyScale {
    fn default() -> Self {
        Self {
            display: TypographyToken {
                size: 24.0,
                weight: FontWeight::Bold,
                line_height: 1.25,
                letter_spacing: -0.02,
            },
            h1: TypographyToken {
                size: 20.0,
                weight: FontWeight::SemiBold,
                line_height: 1.3,
                letter_spacing: -0.01,
            },
            h2: TypographyToken {
                size: 16.0,
                weight: FontWeight::SemiBold,
                line_height: 1.4,
                letter_spacing: 0.0,
            },
            h3: TypographyToken {
                size: 14.0,
                weight: FontWeight::Medium,
                line_height: 1.4,
                letter_spacing: 0.0,
            },
            body: TypographyToken {
                size: 11.0,
                weight: FontWeight::Regular,
                line_height: 1.5,
                letter_spacing: 0.0,
            },
            small: TypographyToken {
                size: 9.5,
                weight: FontWeight::Regular,
                line_height: 1.5,
                letter_spacing: 0.0,
            },
            caption: TypographyToken {
                size: 8.0,
                weight: FontWeight::Regular,
                line_height: 1.4,
                letter_spacing: 0.02,
            },
            mono: TypographyToken {
                size: 11.0,
                weight: FontWeight::Regular,
                line_height: 1.5,
                letter_spacing: 0.0,
            },
            button: TypographyToken {
                size: 11.0,
                weight: FontWeight::Medium,
                line_height: 1.0,
                letter_spacing: 0.0,
            },
        }
    }
}

// ============================================================================
// SPACING TOKENS
// ============================================================================

/// Base spacing unit: 4 px.
pub const BASE_UNIT: f32 = 4.0;

/// Spacing multipliers (token names map to multiplier × BASE_UNIT).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SpacingToken {
    /// 0.5 × 4 = 2px
    Xs,
    /// 1 × 4 = 4px
    Sm,
    /// 2 × 4 = 8px
    Md,
    /// 3 × 4 = 12px
    Lg,
    /// 4 × 4 = 16px
    Xl,
    /// 6 × 4 = 24px
    Xxl,
    /// 8 × 4 = 32px
    Xxxl,
    /// 12 × 4 = 48px
    Huge,
    /// 16 × 4 = 64px
    Massive,
}

impl SpacingToken {
    /// Returns the spacing value in points for the given density.
    pub fn value(self, density: Density) -> f32 {
        BASE_UNIT * self.multiplier() * density.multiplier()
    }

    fn multiplier(self) -> f32 {
        match self {
            SpacingToken::Xs => 0.5,
            SpacingToken::Sm => 1.0,
            SpacingToken::Md => 2.0,
            SpacingToken::Lg => 3.0,
            SpacingToken::Xl => 4.0,
            SpacingToken::Xxl => 6.0,
            SpacingToken::Xxxl => 8.0,
            SpacingToken::Huge => 12.0,
            SpacingToken::Massive => 16.0,
        }
    }
}

/// Density levels for toolbar/chrome scaling.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Density {
    /// Compact: 0.875× base spacing.
    #[default]
    Compact,
    /// Standard: 1.0× base spacing.
    Standard,
    /// Comfortable: 1.125× base spacing.
    Comfortable,
}

impl Density {
    /// Multiplier applied to all spacing tokens.
    pub fn multiplier(self) -> f32 {
        match self {
            Density::Compact => 0.875,
            Density::Standard => 1.0,
            Density::Comfortable => 1.125,
        }
    }

    /// Returns the next density in cycle order.
    pub fn next(self) -> Self {
        match self {
            Density::Compact => Density::Standard,
            Density::Standard => Density::Comfortable,
            Density::Comfortable => Density::Compact,
        }
    }
}

// ============================================================================
// RADIUS TOKENS
// ============================================================================

/// Border radius tokens.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RadiusToken {
    /// No rounding (0).
    None,
    /// Small (2px).
    Sm,
    /// Medium (4px) — default widget radius.
    Md,
    /// Large (6px) — window/menu radius.
    Lg,
    /// Extra large (8px).
    Xl,
    /// Full (pill/circle).
    Full,
}

impl RadiusToken {
    /// Returns the radius value in points.
    pub fn value(self) -> f32 {
        match self {
            RadiusToken::None => 0.0,
            RadiusToken::Sm => 2.0,
            RadiusToken::Md => 4.0,
            RadiusToken::Lg => 6.0,
            RadiusToken::Xl => 8.0,
            RadiusToken::Full => 9999.0,
        }
    }
}

// ============================================================================
// ELEVATION / SHADOW TOKENS
// ============================================================================

/// Elevation levels with shadow definitions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ElevationToken {
    /// No shadow (base surfaces).
    Level0,
    /// Subtle shadow (hovered cards, dropdowns).
    Level1,
    /// Card shadow (panels, tooltips).
    Level2,
    /// Modal shadow (dialogs, drawers).
    Level3,
}

impl ElevationToken {
    /// Returns (offset_y, blur_radius, spread, color) for light theme.
    pub fn light_shadow(self) -> (f32, f32, f32, Color32) {
        match self {
            ElevationToken::Level0 => (0.0, 0.0, 0.0, Color32::TRANSPARENT),
            ElevationToken::Level1 => (
                1.0,
                2.0,
                0.0,
                Color32::from_rgba_unmultiplied(0, 0, 0, 0x14),
            ),
            ElevationToken::Level2 => (
                4.0,
                8.0,
                -2.0,
                Color32::from_rgba_unmultiplied(0, 0, 0, 0x1A),
            ),
            ElevationToken::Level3 => (
                8.0,
                16.0,
                -4.0,
                Color32::from_rgba_unmultiplied(0, 0, 0, 0x26),
            ),
        }
    }

    /// Returns (offset_y, blur_radius, spread, color) for dark theme.
    pub fn dark_shadow(self) -> (f32, f32, f32, Color32) {
        match self {
            ElevationToken::Level0 => (0.0, 0.0, 0.0, Color32::TRANSPARENT),
            ElevationToken::Level1 => (
                1.0,
                2.0,
                0.0,
                Color32::from_rgba_unmultiplied(0, 0, 0, 0x33),
            ),
            ElevationToken::Level2 => (
                4.0,
                8.0,
                -2.0,
                Color32::from_rgba_unmultiplied(0, 0, 0, 0x4D),
            ),
            ElevationToken::Level3 => (
                8.0,
                16.0,
                -4.0,
                Color32::from_rgba_unmultiplied(0, 0, 0, 0x66),
            ),
        }
    }
}

// ============================================================================
// MOTION TOKENS
// ============================================================================

/// Animation duration tokens.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DurationToken {
    /// 0ms (instant).
    Instant,
    /// 100ms (fast: hover, focus).
    Fast,
    /// 200ms (normal: transitions).
    Normal,
    /// 300ms (slow: modals, drawers).
    Slow,
}

impl DurationToken {
    /// Returns duration in milliseconds.
    pub fn ms(self) -> u32 {
        match self {
            DurationToken::Instant => 0,
            DurationToken::Fast => 100,
            DurationToken::Normal => 200,
            DurationToken::Slow => 300,
        }
    }

    /// Returns duration in milliseconds, respecting reduced motion.
    pub fn ms_reduced(self, reduced_motion: bool) -> u32 {
        if reduced_motion {
            0
        } else {
            self.ms()
        }
    }
}

/// Easing functions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EasingToken {
    /// ease-out: cubic-bezier(0.25, 0.46, 0.45, 0.94)
    EaseOut,
    /// ease-in-out: cubic-bezier(0.42, 0.0, 0.58, 1.0)
    EaseInOut,
    /// linear
    Linear,
}

/// Motion specification combining duration and easing.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct MotionSpec {
    pub duration: DurationToken,
    pub easing: EasingToken,
}

impl MotionSpec {
    /// Fast transition (hover, focus).
    pub const FAST: Self = Self {
        duration: DurationToken::Fast,
        easing: EasingToken::EaseOut,
    };
    /// Normal transition (panel expand, tooltip).
    pub const NORMAL: Self = Self {
        duration: DurationToken::Normal,
        easing: EasingToken::EaseOut,
    };
    /// Slow transition (modal, drawer).
    pub const SLOW: Self = Self {
        duration: DurationToken::Slow,
        easing: EasingToken::EaseOut,
    };
}

// ============================================================================
// COMPLETE TOKEN SET
// ============================================================================

/// Complete design token set for one theme + density.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignTokenSet {
    pub colors: ColorTokens,
    pub typography: TypographyScale,
    pub density: Density,
    pub radius: RadiusSet,
    pub elevation: ElevationSet,
    pub motion: MotionSet,
}

/// Radius values for all tokens.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RadiusSet {
    pub none: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub full: f32,
}

impl Default for RadiusSet {
    fn default() -> Self {
        Self {
            none: RadiusToken::None.value(),
            sm: RadiusToken::Sm.value(),
            md: RadiusToken::Md.value(),
            lg: RadiusToken::Lg.value(),
            xl: RadiusToken::Xl.value(),
            full: RadiusToken::Full.value(),
        }
    }
}

/// Custom serde for an elevation shadow `(dx, dy, blur, colour)` —
/// serializes as `[dx, dy, blur, "#RRGGBBAA"]` so a serde roundtrip is
/// lossless.
mod shadow_serde {
    use super::*;
    use serde::de::{SeqAccess, Visitor};
    use serde::ser::SerializeSeq;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(
        shadow: &(f32, f32, f32, Color32),
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(4))?;
        seq.serialize_element(&shadow.0)?;
        seq.serialize_element(&shadow.1)?;
        seq.serialize_element(&shadow.2)?;
        seq.serialize_element(&color32_serde::to_hex(&shadow.3))?;
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<(f32, f32, f32, Color32), D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ShadowVisitor;

        impl<'de> Visitor<'de> for ShadowVisitor {
            type Value = (f32, f32, f32, Color32);

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a [dx, dy, blur, \"#RRGGBBAA\"] shadow array")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let missing = || serde::de::Error::custom("shadow array needs 4 elements");
                let dx = seq.next_element::<f32>()?.ok_or_else(missing)?;
                let dy = seq.next_element::<f32>()?.ok_or_else(missing)?;
                let blur = seq.next_element::<f32>()?.ok_or_else(missing)?;
                let hex = seq.next_element::<String>()?.ok_or_else(missing)?;
                let color = color32_serde::from_hex(&hex).map_err(serde::de::Error::custom)?;
                Ok((dx, dy, blur, color))
            }
        }

        deserializer.deserialize_seq(ShadowVisitor)
    }
}

/// Elevation shadows for all levels.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ElevationSet {
    #[serde(with = "shadow_serde")]
    pub level0: (f32, f32, f32, Color32),
    #[serde(with = "shadow_serde")]
    pub level1: (f32, f32, f32, Color32),
    #[serde(with = "shadow_serde")]
    pub level2: (f32, f32, f32, Color32),
    #[serde(with = "shadow_serde")]
    pub level3: (f32, f32, f32, Color32),
}

impl ElevationSet {
    fn light() -> Self {
        Self {
            level0: ElevationToken::Level0.light_shadow(),
            level1: ElevationToken::Level1.light_shadow(),
            level2: ElevationToken::Level2.light_shadow(),
            level3: ElevationToken::Level3.light_shadow(),
        }
    }
    fn dark() -> Self {
        Self {
            level0: ElevationToken::Level0.dark_shadow(),
            level1: ElevationToken::Level1.dark_shadow(),
            level2: ElevationToken::Level2.dark_shadow(),
            level3: ElevationToken::Level3.dark_shadow(),
        }
    }
}

/// Motion specs for common interactions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MotionSet {
    pub fast: MotionSpec,
    pub normal: MotionSpec,
    pub slow: MotionSpec,
}

impl Default for MotionSet {
    fn default() -> Self {
        Self {
            fast: MotionSpec::FAST,
            normal: MotionSpec::NORMAL,
            slow: MotionSpec::SLOW,
        }
    }
}

// ============================================================================
// THEME TRAIT & IMPLEMENTATION
// ============================================================================

/// A theme that can be applied to an egui Context.
pub trait Theme {
    /// Returns the colour tokens for this theme.
    fn colors(&self) -> &ColorTokens;

    /// Returns the typography scale.
    fn typography(&self) -> &TypographyScale;

    /// Returns the current density.
    fn density(&self) -> Density;

    /// Returns the radius set.
    fn radius(&self) -> &RadiusSet;

    /// Returns the elevation set.
    fn elevation(&self) -> &ElevationSet;

    /// Returns the motion set.
    fn motion(&self) -> &MotionSet;

    /// Whether reduced motion is enabled.
    fn reduced_motion(&self) -> bool;

    /// Applies this theme to the egui Context.
    fn apply(&self, ctx: &Context) {
        self.apply_visuals(ctx);
        self.apply_style(ctx);
    }

    /// Applies visuals (colours, shadows, radius) to the Context.
    fn apply_visuals(&self, ctx: &Context) {
        let colors = self.colors();
        let radius = self.radius();
        let elevation = self.elevation();

        let mut v = if self.is_dark() {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        // Base colours
        v.panel_fill = colors.surface_raised;
        v.extreme_bg_color = colors.surface_deep;
        v.window_fill = colors.surface_raised;
        v.override_text_color = Some(colors.text_primary);
        v.faint_bg_color = colors.surface_base;

        // Window / menu radius
        v.window_corner_radius = radius.lg.into();
        v.menu_corner_radius = radius.lg.into();

        // Widget states
        let widget_states = [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ];

        for widget in widget_states {
            widget.corner_radius = radius.md.into();
            widget.fg_stroke = Stroke::new(1.0, colors.text_primary);
            widget.bg_stroke = Stroke::new(1.0, colors.border_subtle);
            widget.expansion = 0.0;
        }

        // Focus ring: egui maps keyboard focus to `active` state
        v.widgets.active.bg_stroke = Stroke::new(2.0, colors.focus_ring);

        // Non-interactive
        v.widgets.noninteractive.bg_fill = colors.surface_raised;
        v.widgets.noninteractive.fg_stroke.color = colors.text_primary;

        // Inactive (default)
        v.widgets.inactive.fg_stroke.color = colors.text_primary;
        v.widgets.inactive.bg_fill = colors.surface_raised;

        // Hovered
        v.widgets.hovered.fg_stroke.color = colors.text_primary;
        v.widgets.hovered.bg_fill = if self.is_dark() {
            Color32::from_rgb(0x26, 0x2A, 0x33)
        } else {
            Color32::from_rgb(0xEE, 0xF1, 0xF6)
        };

        // Active (pressed / focused)
        v.widgets.active.fg_stroke.color = colors.text_primary;
        v.widgets.active.bg_fill = colors.surface_sunken;

        // Open (dropdown open)
        v.widgets.open.fg_stroke.color = colors.text_primary;
        v.widgets.open.bg_fill = colors.surface_sunken;

        // Selection
        v.selection.bg_fill = colors.selection_bg;
        v.selection.stroke = Stroke::new(1.0, colors.selection_bg);

        // Links
        v.hyperlink_color = colors.accent;

        // Striped tables
        v.striped = true;

        // Window shadow (level2 elevation)
        let (offset_y, blur, spread, color) = elevation.level2;
        v.window_shadow = egui::Shadow {
            offset: [0, offset_y as i8],
            blur: blur as u8,
            spread: spread as u8,
            color,
        };
        v.popup_shadow = v.window_shadow;

        ctx.set_visuals(v);
    }

    /// Applies typography and spacing to the Context Style.
    fn apply_style(&self, ctx: &Context) {
        let typography = self.typography();
        let density = self.density();
        let spacing_md = SpacingToken::Md.value(density);
        let spacing_sm = SpacingToken::Sm.value(density);
        let spacing_lg = SpacingToken::Lg.value(density);
        let spacing_xl = SpacingToken::Xl.value(density);

        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            ctx.style_mut_of(theme, |style| {
                let text = &mut style.text_styles;

                // Map typography tokens to egui TextStyle
                text.insert(
                    TextStyle::Heading,
                    FontId::new(
                        typography.h2.size,
                        FontFamily::Name(typography.h2.weight.family_name().into()),
                    ),
                );
                text.insert(
                    TextStyle::Body,
                    FontId::new(typography.body.size, FontFamily::Proportional),
                );
                text.insert(
                    TextStyle::Small,
                    FontId::new(typography.small.size, FontFamily::Proportional),
                );
                text.insert(
                    TextStyle::Button,
                    FontId::new(
                        typography.button.size,
                        FontFamily::Name(typography.button.weight.family_name().into()),
                    ),
                );
                text.insert(
                    TextStyle::Monospace,
                    FontId::new(typography.mono.size, FontFamily::Monospace),
                );

                // Spacing rhythm
                style.spacing.item_spacing = egui::vec2(spacing_md * 2.0, spacing_sm); // 8×4 base
                style.spacing.button_padding = egui::vec2(spacing_md, spacing_sm); // 8×4
                style.spacing.menu_margin = Margin::symmetric(spacing_md as i8, spacing_sm as i8);
                style.spacing.indent = spacing_xl; // 16
                style.spacing.window_margin = Margin::same(spacing_lg as i8); // 12
                style.spacing.combo_width = 160.0;
                style.spacing.slider_width = 100.0;
                style.spacing.text_edit_width = 200.0;

                // Widget visuals already set via apply_visuals
            });
        }
    }

    /// Returns true if this is a dark theme.
    fn is_dark(&self) -> bool;
}

/// Light theme implementation.
#[derive(Debug, Clone)]
pub struct LightTheme {
    colors: ColorTokens,
    typography: TypographyScale,
    density: Density,
    radius: RadiusSet,
    elevation: ElevationSet,
    motion: MotionSet,
    reduced_motion: bool,
}

impl Default for LightTheme {
    fn default() -> Self {
        Self {
            colors: LIGHT_COLORS,
            typography: TypographyScale::default(),
            density: Density::Standard,
            radius: RadiusSet::default(),
            elevation: ElevationSet::light(),
            motion: MotionSet::default(),
            reduced_motion: false,
        }
    }
}

impl Theme for LightTheme {
    fn colors(&self) -> &ColorTokens {
        &self.colors
    }

    fn typography(&self) -> &TypographyScale {
        &self.typography
    }

    fn density(&self) -> Density {
        self.density
    }

    fn radius(&self) -> &RadiusSet {
        &self.radius
    }

    fn elevation(&self) -> &ElevationSet {
        &self.elevation
    }

    fn motion(&self) -> &MotionSet {
        &self.motion
    }

    fn reduced_motion(&self) -> bool {
        self.reduced_motion
    }

    fn is_dark(&self) -> bool {
        false
    }
}

/// Dark theme implementation.
#[derive(Debug, Clone)]
pub struct DarkTheme {
    colors: ColorTokens,
    typography: TypographyScale,
    density: Density,
    radius: RadiusSet,
    elevation: ElevationSet,
    motion: MotionSet,
    reduced_motion: bool,
}

impl Default for DarkTheme {
    fn default() -> Self {
        Self {
            colors: DARK_COLORS,
            typography: TypographyScale::default(),
            density: Density::Standard,
            radius: RadiusSet::default(),
            elevation: ElevationSet::dark(),
            motion: MotionSet::default(),
            reduced_motion: false,
        }
    }
}

impl Theme for DarkTheme {
    fn colors(&self) -> &ColorTokens {
        &self.colors
    }

    fn typography(&self) -> &TypographyScale {
        &self.typography
    }

    fn density(&self) -> Density {
        self.density
    }

    fn radius(&self) -> &RadiusSet {
        &self.radius
    }

    fn elevation(&self) -> &ElevationSet {
        &self.elevation
    }

    fn motion(&self) -> &MotionSet {
        &self.motion
    }

    fn reduced_motion(&self) -> bool {
        self.reduced_motion
    }

    fn is_dark(&self) -> bool {
        true
    }
}

// ============================================================================
// THEME REGISTRY & APPLICATION
// ============================================================================

/// Theme mode selection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

/// Global theme registry for the application.
pub struct ThemeRegistry {
    light: LightTheme,
    dark: DarkTheme,
    current_mode: ThemeMode,
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self {
            light: LightTheme::default(),
            dark: DarkTheme::default(),
            current_mode: ThemeMode::System,
        }
    }
}

impl ThemeRegistry {
    /// Creates a new registry with default themes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the theme mode.
    pub fn set_mode(&mut self, mode: ThemeMode) {
        self.current_mode = mode;
    }

    /// Returns the current theme mode.
    pub fn mode(&self) -> ThemeMode {
        self.current_mode
    }

    /// Returns the active theme based on mode and system preference.
    pub fn active_theme(&self, system_dark: bool) -> &dyn Theme {
        match self.current_mode {
            ThemeMode::Light => &self.light,
            ThemeMode::Dark => &self.dark,
            ThemeMode::System => {
                if system_dark {
                    &self.dark
                } else {
                    &self.light
                }
            }
        }
    }

    /// Returns mutable reference to light theme for customization.
    pub fn light_mut(&mut self) -> &mut LightTheme {
        &mut self.light
    }

    /// Returns mutable reference to dark theme for customization.
    pub fn dark_mut(&mut self) -> &mut DarkTheme {
        &mut self.dark
    }

    /// Applies the active theme to the context.
    pub fn apply(&self, ctx: &Context, system_dark: bool) {
        self.active_theme(system_dark).apply(ctx);
    }
}

// Global registry instance (initialized once).
static THEME_REGISTRY: OnceLock<Mutex<ThemeRegistry>> = OnceLock::new();

/// Initializes the global theme registry.
pub fn init_theme_registry() -> &'static Mutex<ThemeRegistry> {
    THEME_REGISTRY.get_or_init(|| Mutex::new(ThemeRegistry::new()))
}

/// Returns the global theme registry.
pub fn theme_registry() -> &'static Mutex<ThemeRegistry> {
    THEME_REGISTRY.get_or_init(|| Mutex::new(ThemeRegistry::new()))
}

// ============================================================================
// FONT INSTALLATION
// ============================================================================

/// Installs bundled fonts (Inter + Phosphor) into the egui Context.
pub fn install_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    // Inter font family (4 weights)
    let inter_fonts = [
        (
            "Inter-Regular",
            include_bytes!("../assets/fonts/Inter-Regular.ttf") as &[u8],
        ),
        (
            "Inter-Medium",
            include_bytes!("../assets/fonts/Inter-Medium.ttf") as &[u8],
        ),
        (
            "Inter-SemiBold",
            include_bytes!("../assets/fonts/Inter-SemiBold.ttf") as &[u8],
        ),
        (
            "Inter-Bold",
            include_bytes!("../assets/fonts/Inter-Bold.ttf") as &[u8],
        ),
    ];

    for (name, data) in inter_fonts {
        fonts
            .font_data
            .insert(name.to_string(), FontData::from_static(data).into());
    }

    // Phosphor Icons (MIT) - patched version
    fonts.font_data.insert(
        "phosphor".to_string(),
        FontData::from_static(include_bytes!("../assets/fonts/Phosphor.ttf") as &[u8]).into(),
    );

    // Build proportional family with icon font first
    let fallbacks = fonts
        .families
        .remove(&FontFamily::Proportional)
        .unwrap_or_default();

    let mut proportional = vec!["phosphor".to_string(), "Inter-Regular".to_string()];
    proportional.extend(fallbacks.iter().cloned());
    fonts
        .families
        .insert(FontFamily::Proportional, proportional);

    // Weight-specific families (each with icon font first)
    for name in ["Inter-Medium", "Inter-SemiBold", "Inter-Bold"] {
        let mut family = vec!["phosphor".to_string(), name.to_string()];
        family.extend(fallbacks.iter().cloned());
        fonts.families.insert(FontFamily::Name(name.into()), family);
    }

    ctx.set_fonts(fonts);
}

// ============================================================================
// COMPATIBILITY LAYER (for migration from design.rs)
// ============================================================================

/// Legacy palette structure for backward compatibility.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg: Color32,
    pub panel: Color32,
    pub text: Color32,
    pub text_muted: Color32,
    pub grid: Color32,
    pub critical: Color32,
    pub normal: Color32,
    pub ok: Color32,
    pub warn: Color32,
    pub deep: Color32,
    pub non_working: Color32,
    pub status_date: Color32,
    pub focus_ring: Color32,
}

/// Returns the legacy palette for the given theme (compatibility).
pub fn palette(dark: bool) -> Palette {
    let colors = if dark { DARK_COLORS } else { LIGHT_COLORS };
    Palette {
        bg: colors.surface_base,
        panel: colors.surface_raised,
        text: colors.text_primary,
        text_muted: colors.text_muted,
        grid: colors.border_subtle,
        critical: colors.status_critical,
        normal: colors.status_normal,
        ok: colors.status_ok,
        warn: colors.status_warn,
        deep: colors.surface_deep,
        non_working: colors.non_working_day,
        status_date: colors.status_date,
        focus_ring: colors.focus_ring,
    }
}

/// Legacy ACCENT constant.
pub const ACCENT: Color32 = Color32::from_rgb(0x2F, 0x6F, 0xED);

/// Legacy FOCUS_RING constant.
pub const FOCUS_RING: Color32 = ACCENT;

/// Legacy ACCENT_ON constant.
pub const ACCENT_ON: Color32 = Color32::WHITE;

/// Legacy chrome button helpers.
/// Compute the contrast ratio between two colors.
///
/// Returns the contrast ratio as a floating-point number, where 21.0 is the
/// maximum (white on black) and 1.0 is the minimum (same color on same color).
#[derive(Debug, Clone, Copy)]
pub struct ContrastRatio;
impl ContrastRatio {
    /// Compute the contrast ratio between two colors.
    pub fn compute(fg: Color32, bg: Color32) -> f32 {
        let srgb_to_linear = |c: f32| {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let r1 = srgb_to_linear(fg.r() as f32 / 255.0);
        let g1 = srgb_to_linear(fg.g() as f32 / 255.0);
        let b1 = srgb_to_linear(fg.b() as f32 / 255.0);
        let r2 = srgb_to_linear(bg.r() as f32 / 255.0);
        let g2 = srgb_to_linear(bg.g() as f32 / 255.0);
        let b2 = srgb_to_linear(bg.b() as f32 / 255.0);
        let l1 = 0.2126 * r1 + 0.7152 * g1 + 0.0722 * b1;
        let l2 = 0.2126 * r2 + 0.7152 * g2 + 0.0722 * b2;
        (l1 + 0.05) / (l2 + 0.05).max(1.0 / 21.0 / (l1 + 0.05) * (l2 + 0.05))
    }
}

pub use crate::theme::{non_working_color, parse_color, status_date_color, ThemeVariant};
/// Re-export items from sibling modules for test convenience.
pub use crate::StatusSeverity;

pub mod chrome {
    use super::*;

    pub fn primary(label: impl Into<String>) -> egui::Button<'static> {
        egui::Button::new(egui::RichText::new(label.into()).color(ACCENT_ON).strong())
            .fill(ACCENT)
            .stroke(Stroke::NONE)
    }

    pub fn ghost(label: impl Into<String>) -> egui::Button<'static> {
        egui::Button::new(label.into())
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE)
    }

    pub fn tab(dark: bool, selected: bool, label: impl Into<String>) -> egui::Button<'static> {
        let p = palette(dark);
        let text = if selected {
            egui::RichText::new(label.into()).strong().color(p.text)
        } else {
            egui::RichText::new(label.into()).color(p.text_muted)
        };
        egui::Button::new(text)
            .selected(selected)
            .frame(selected)
            .fill(if selected {
                ACCENT.linear_multiply(0.2)
            } else {
                Color32::TRANSPARENT
            })
            .stroke(if selected {
                Stroke::new(1.0, ACCENT)
            } else {
                Stroke::NONE
            })
    }
}

/// Legacy variance color function.
pub fn variance_color(value: Option<i64>, dark: bool) -> Color32 {
    let colors = if dark { DARK_COLORS } else { LIGHT_COLORS };
    match value {
        Some(v) if v > 0 => colors.status_critical,
        Some(v) if v < 0 => colors.status_ok,
        _ => colors.border_subtle,
    }
}

/// Legacy translucent helper.
pub fn translucent(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

/// Legacy apply function (calls through to new system).
pub fn apply(ctx: &Context, dark: bool) {
    let registry = theme_registry();
    let mut registry = registry.lock().unwrap();
    let mode = if dark {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    registry.set_mode(mode);
    registry.apply(ctx, dark);
}

/// Legacy apply_visuals function.
pub fn apply_visuals(ctx: &Context, dark: bool) {
    let registry = theme_registry();
    let mut registry = registry.lock().unwrap();
    let mode = if dark {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    registry.set_mode(mode);
    registry.active_theme(dark).apply_visuals(ctx);
}

// ============================================================================
// DESIGN TOKENS JSON EXPORT
// ============================================================================

/// Exports the complete design token set as JSON for documentation/tooling.
pub fn export_tokens_json() -> String {
    let light_set = DesignTokenSet::light();
    let dark_set = DesignTokenSet::dark();

    serde_json::json!({
        "light": light_set,
        "dark": dark_set,
        "metadata": {
            "version": "1.0.0",
            "baseUnit": BASE_UNIT,
            "fontFamily": "Inter",
            "iconFont": "Phosphor",
        }
    })
    .to_string()
}

/// Create a DesignTokenSet for the light theme.
pub fn light_tokens() -> DesignTokenSet {
    DesignTokenSet {
        colors: LIGHT_COLORS,
        typography: TypographyScale::default(),
        density: Density::Standard,
        radius: RadiusSet::default(),
        elevation: ElevationSet::light(),
        motion: MotionSet::default(),
    }
}

/// Create a DesignTokenSet for the dark theme.
pub fn dark_tokens() -> DesignTokenSet {
    DesignTokenSet {
        colors: DARK_COLORS,
        typography: TypographyScale::default(),
        density: Density::Standard,
        radius: RadiusSet::default(),
        elevation: ElevationSet::dark(),
        motion: MotionSet::default(),
    }
}

impl DesignTokenSet {
    /// Create a DesignTokenSet for the light theme.
    pub fn light() -> Self {
        light_tokens()
    }

    /// Create a DesignTokenSet for the dark theme.
    pub fn dark() -> Self {
        dark_tokens()
    }
}

// ============================================================================
// WCAG 2.2 AA CONTRAST VALIDATION
// ============================================================================

/// WCAG 2.2 conformance level for contrast checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WcagLevel {
    /// Normal text: 4.5:1
    AaNormal,
    /// Large text (≥18pt or ≥14pt bold): 3:1
    AaLarge,
    /// Enhanced: 7:1 normal, 4.5:1 large
    AaaNormal,
}

/// Returns the minimum contrast ratio for the given WCAG level.
pub fn wcag_min_ratio(level: WcagLevel) -> f32 {
    match level {
        WcagLevel::AaNormal => 4.5,
        WcagLevel::AaLarge => 3.0,
        WcagLevel::AaaNormal => 7.0,
    }
}

/// Linearizes an sRGB component (0-255) to linear light (0.0-1.0).
fn linearize(c: f32) -> f32 {
    let c = c / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Computes the relative luminance of an sRGB color.
///
/// Formula per WCAG 2.2: L = 0.2126 * R + 0.7152 * G + 0.0722 * B
pub fn relative_luminance(c: Color32) -> f32 {
    let r = linearize(c.r() as f32);
    let g = linearize(c.g() as f32);
    let b = linearize(c.b() as f32);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Computes the contrast ratio between two colors.
///
/// Returns a value ≥ 1.0. Higher is more contrast.
/// Formula per WCAG 2.2: (L1 + 0.05) / (L2 + 0.05) where L1 ≥ L2
pub fn contrast_ratio(fg: Color32, bg: Color32) -> f32 {
    let l1 = relative_luminance(fg);
    let l2 = relative_luminance(bg);
    if l1 > l2 {
        (l1 + 0.05) / (l2 + 0.05)
    } else {
        (l2 + 0.05) / (l1 + 0.05)
    }
}

/// Checks if the contrast between two colors meets the given WCAG level.
///
/// Returns true if the contrast ratio meets or exceeds the minimum for the level.
pub fn meets_wcag(fg: Color32, bg: Color32, level: WcagLevel) -> bool {
    contrast_ratio(fg, bg) >= wcag_min_ratio(level)
}

/// Semantic color role pairs to validate for WCAG 2.2 AA compliance.
///
/// Each entry: (fg_role, bg_role, min_level, description)
pub const SEMANTIC_CONTRAST_PAIRS: &[(&str, &str, WcagLevel, &str)] = &[
    // Body text — 4.5:1
    (
        "text_primary",
        "surface_base",
        WcagLevel::AaNormal,
        "Body text on main surface",
    ),
    (
        "text_primary",
        "surface_raised",
        WcagLevel::AaNormal,
        "Body text on raised surface",
    ),
    (
        "text_primary",
        "surface_sunken",
        WcagLevel::AaNormal,
        "Body text on sunken surface",
    ),
    (
        "text_secondary",
        "surface_base",
        WcagLevel::AaNormal,
        "Secondary text on main surface",
    ),
    (
        "text_secondary",
        "surface_raised",
        WcagLevel::AaNormal,
        "Secondary text on raised surface",
    ),
    (
        "text_muted",
        "surface_base",
        WcagLevel::AaNormal,
        "Muted text on main surface",
    ),
    (
        "text_muted",
        "surface_raised",
        WcagLevel::AaNormal,
        "Muted text on raised surface",
    ),
    (
        "text_disabled",
        "surface_base",
        WcagLevel::AaLarge,
        "Disabled text on main surface",
    ),
    // UI components — 3:1
    (
        "accent",
        "surface_base",
        WcagLevel::AaLarge,
        "Accent on main surface",
    ),
    (
        "text_on_accent",
        "accent",
        WcagLevel::AaNormal,
        "Accent on-text on accent",
    ),
    (
        "accent_hover",
        "surface_base",
        WcagLevel::AaLarge,
        "Accent hover on main surface",
    ),
    (
        "accent_active",
        "surface_base",
        WcagLevel::AaLarge,
        "Accent active on main surface",
    ),
    (
        "border_subtle",
        "surface_base",
        WcagLevel::AaLarge,
        "Subtle border on main surface",
    ),
    (
        "border_strong",
        "surface_base",
        WcagLevel::AaLarge,
        "Strong border on main surface",
    ),
    (
        "focus_ring",
        "surface_base",
        WcagLevel::AaLarge,
        "Focus ring on main surface",
    ),
    (
        "focus_ring",
        "surface_raised",
        WcagLevel::AaLarge,
        "Focus ring on raised surface",
    ),
    (
        "border_subtle",
        "surface_raised",
        WcagLevel::AaLarge,
        "Subtle border on raised surface",
    ),
    (
        "border_strong",
        "surface_raised",
        WcagLevel::AaLarge,
        "Strong border on raised surface",
    ),
    (
        "grid_border",
        "surface_base",
        WcagLevel::AaLarge,
        "Grid border on main surface",
    ),
    // Status colors on surfaces
    (
        "status_critical",
        "surface_base",
        WcagLevel::AaLarge,
        "Critical on main surface",
    ),
    (
        "status_normal",
        "surface_base",
        WcagLevel::AaLarge,
        "Normal on main surface",
    ),
    (
        "status_ok",
        "surface_base",
        WcagLevel::AaLarge,
        "Success on main surface",
    ),
    (
        "status_warn",
        "surface_base",
        WcagLevel::AaLarge,
        "Warning on main surface",
    ),
    (
        "non_working_day",
        "surface_base",
        WcagLevel::AaLarge,
        "Non-working day on main surface",
    ),
];

/// Validates all semantic color pairs against WCAG 2.2 AA.
///
/// Returns a vector of failed checks with details.
pub fn validate_contrast(color_tokens: &ColorTokens) -> Vec<String> {
    let mut failures = Vec::new();

    // Helper to get a color by role name
    let get_color = |role: &str| -> Option<Color32> {
        match role {
            "surface_base" => Some(color_tokens.surface_base),
            "surface_raised" => Some(color_tokens.surface_raised),
            "surface_sunken" => Some(color_tokens.surface_sunken),
            "text_primary" => Some(color_tokens.text_primary),
            "text_secondary" => Some(color_tokens.text_secondary),
            "text_muted" => Some(color_tokens.text_muted),
            "text_muted_on_accent" => Some(color_tokens.text_muted), // alias
            "text_disabled" => Some(color_tokens.disabled_text),
            "accent" => Some(color_tokens.accent),
            "accent_hover" => Some(color_tokens.accent_hover),
            "accent_active" => Some(color_tokens.accent_active),
            "text_on_accent" => Some(color_tokens.text_on_accent),
            "status_critical" => Some(color_tokens.status_critical),
            "status_normal" => Some(color_tokens.status_normal),
            "status_ok" => Some(color_tokens.status_ok),
            "status_warn" => Some(color_tokens.status_warn),
            "border_subtle" => Some(color_tokens.border_subtle),
            "border_strong" => Some(color_tokens.border_strong),
            "focus_ring" => Some(color_tokens.focus_ring),
            "grid_border" => Some(color_tokens.border_subtle), // alias
            "non_working_day" => Some(color_tokens.non_working_day),
            _ => None,
        }
    };

    for (fg_role, bg_role, level, description) in SEMANTIC_CONTRAST_PAIRS {
        let fg = get_color(fg_role);
        let bg = get_color(bg_role);

        match (fg, bg) {
            (Some(fg), Some(bg)) => {
                let ratio = contrast_ratio(fg, bg);
                let min_ratio = wcag_min_ratio(*level);
                if ratio < min_ratio {
                    failures.push(format!(
                        "FAIL: {} — {} on {} = {:.2}:1 < {:.1}:1",
                        description, fg_role, bg_role, ratio, min_ratio
                    ));
                }
            }
            (None, _) => {
                failures.push(format!("MISSING FG: {} ({})", fg_role, description));
            }
            (_, None) => {
                failures.push(format!("MISSING BG: {} ({})", bg_role, description));
            }
        }
    }

    failures
}

#[cfg(test)]
mod contrast_tests {
    use super::*;

    #[test]
    fn light_colors_meet_wcag_aa() {
        let failures = validate_contrast(&LIGHT_COLORS);
        if !failures.is_empty() {
            panic!("Light theme contrast failures:\n{}", failures.join("\n"));
        }
    }

    #[test]
    fn dark_colors_meet_wcag_aa() {
        let failures = validate_contrast(&DARK_COLORS);
        if !failures.is_empty() {
            panic!("Dark theme contrast failures:\n{}", failures.join("\n"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_tokens_serializable() {
        let json = serde_json::to_string(&LIGHT_COLORS).unwrap();
        assert!(!json.is_empty());
        let _parsed: ColorTokens = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn typography_scale_default() {
        let scale = TypographyScale::default();
        assert_eq!(scale.body.size, 11.0);
        assert_eq!(scale.h2.size, 16.0);
        assert_eq!(scale.mono.size, 11.0);
    }

    #[test]
    fn spacing_token_values() {
        assert_eq!(SpacingToken::Sm.value(Density::Standard), 4.0);
        assert_eq!(SpacingToken::Md.value(Density::Standard), 8.0);
        assert_eq!(SpacingToken::Lg.value(Density::Standard), 12.0);
        assert_eq!(SpacingToken::Xl.value(Density::Standard), 16.0);

        // Density scaling
        assert!((SpacingToken::Md.value(Density::Compact) - 7.0).abs() < 0.001);
        assert!((SpacingToken::Md.value(Density::Comfortable) - 9.0).abs() < 0.001);
    }

    #[test]
    fn radius_token_values() {
        assert_eq!(RadiusToken::None.value(), 0.0);
        assert_eq!(RadiusToken::Sm.value(), 2.0);
        assert_eq!(RadiusToken::Md.value(), 4.0);
        assert_eq!(RadiusToken::Lg.value(), 6.0);
        assert_eq!(RadiusToken::Xl.value(), 8.0);
    }

    #[test]
    fn duration_tokens() {
        assert_eq!(DurationToken::Instant.ms(), 0);
        assert_eq!(DurationToken::Fast.ms(), 100);
        assert_eq!(DurationToken::Normal.ms(), 200);
        assert_eq!(DurationToken::Slow.ms(), 300);

        assert_eq!(DurationToken::Fast.ms_reduced(true), 0);
        assert_eq!(DurationToken::Fast.ms_reduced(false), 100);
    }

    #[test]
    fn theme_registry_default() {
        let registry = ThemeRegistry::new();
        assert_eq!(registry.mode(), ThemeMode::System);
    }

    #[test]
    fn export_tokens_json_valid() {
        let json = export_tokens_json();
        assert!(!json.is_empty());
        let _parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn light_theme_is_not_dark() {
        let theme = LightTheme::default();
        assert!(!theme.is_dark());
    }

    #[test]
    fn dark_theme_is_dark() {
        let theme = DarkTheme::default();
        assert!(theme.is_dark());
    }
}
