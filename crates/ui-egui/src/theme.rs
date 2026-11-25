//! egui Theme integration — maps DesignTokens onto egui Visuals and Style.
//!
//! This module is the single bridge between the token system (design_tokens.rs)
//! and egui's rendering. All widget styling flows through here.
//!
//! Following guide II.30.6: one token source, no raw colour literals in widget code.

use crate::design_tokens::{
    ColorTokens, DesignTokenSet, Density, FontWeight, Palette as LegacyPalette,
    RadiusSet, RadiusToken, SpacingToken, ThemeMode, ThemeRegistry, TypographyScale,
    Theme as DesignTheme, LightTheme, DarkTheme, init_theme_registry as init_registry,
    theme_registry as get_registry, theme_registry_mut as get_registry_mut,
    ACCENT, FOCUS_RING, ACCENT_ON, LIGHT_COLORS, DARK_COLORS, BASE_UNIT,
    apply as tokens_apply, apply_visuals as tokens_apply_visuals,
    light_tokens, dark_tokens,
};
use egui::{
    Color32, Context, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Stroke,
    Style, TextStyle, Visuals,
};

/// Theme variant for apply functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeVariant {
    Light,
    Dark,
}

/// Apply the full theme (tokens → egui Visuals + Style) to a context.
///
/// This is the main entry point. Call once at startup and whenever the theme
/// changes (light ↔ dark). The tokens drive everything: colours, spacing,
/// typography, radius, elevation.
pub fn apply_theme(ctx: &Context, variant: ThemeVariant) {
    let dark = matches!(variant, ThemeVariant::Dark);
    tokens_apply(ctx, dark);
}

/// Apply only the visual palette (colours, radius, elevation) without typography.
///
/// Used by headless render tests that cannot load bundled fonts (guide II.30.12).
pub fn apply_visuals_only(ctx: &Context, variant: ThemeVariant) {
    let dark = matches!(variant, ThemeVariant::Dark);
    tokens_apply_visuals(ctx, dark);
}

/// Install bundled fonts (Inter + Phosphor) onto the context.
///
/// Call once at startup before any UI renders.
pub fn install_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    // Inter weights (SIL OFL 1.1)
    let inter = [
        ("Inter-Regular", include_bytes!("../assets/fonts/Inter-Regular.ttf")),
        ("Inter-Medium", include_bytes!("../assets/fonts/Inter-Medium.ttf")),
        ("Inter-SemiBold", include_bytes!("../assets/fonts/Inter-SemiBold.ttf")),
        ("Inter-Bold", include_bytes!("../assets/fonts/Inter-Bold.ttf")),
    ];
    for (name, data) in inter {
        fonts.font_data.insert(
            name.to_string(),
            FontData::from_static(data).into(),
        );
    }

    // Phosphor Icons (MIT) — patched copy from scripts/patch-phosphor.py
    fonts.font_data.insert(
        "phosphor".to_string(),
        FontData::from_static(include_bytes!("../assets/fonts/Phosphor.ttf")).into(),
    );

    // Proportional family: icon face first, then Inter weights, then fallbacks
    let fallbacks = fonts
        .families
        .remove(&FontFamily::Proportional)
        .unwrap_or_default();
    let mut proportional = vec!["phosphor".to_string(), "Inter-Regular".to_string()];
    proportional.extend(fallbacks.iter().cloned());
    fonts.families.insert(FontFamily::Proportional, proportional);

    // Per-weight families for explicit weight selection
    for name in ["Inter-Medium", "Inter-SemiBold", "Inter-Bold"] {
        let mut family = vec!["phosphor".to_string(), name.to_string()];
        family.extend(fallbacks.iter().cloned());
        fonts.families.insert(FontFamily::Name(name.into()), family);
    }

    ctx.set_fonts(fonts);
}

/// Get the color tokens for a theme variant.
fn color_tokens(variant: ThemeVariant) -> &'static ColorTokens {
    match variant {
        ThemeVariant::Light => &LIGHT_COLORS,
        ThemeVariant::Dark => &DARK_COLORS,
    }
}

/// Get a DesignTokenSet for a theme variant.
fn token_set(variant: ThemeVariant) -> DesignTokenSet {
    match variant {
        ThemeVariant::Light => light_tokens(),
        ThemeVariant::Dark => dark_tokens(),
    }
}

/// Parse a CSS hex/rgba colour string to egui Color32.
///
/// Supports: #RRGGBB, #RRGGBBAA, rgb(...), rgba(...)
pub fn parse_color(s: &str) -> Color32 {
    let s = s.trim();
    if s.starts_with('#') {
        parse_hex(s)
    } else if s.starts_with("rgb") {
        parse_rgb(s)
    } else if s.starts_with("rgba") {
        parse_rgba(s)
    } else {
        // Fallback: transparent
        Color32::TRANSPARENT
    }
}

fn parse_hex(s: &str) -> Color32 {
    let hex = &s[1..];
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            Color32::from_rgb(r, g, b)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
            Color32::from_rgba_unmultiplied(r, g, b, a)
        }
        _ => Color32::TRANSPARENT,
    }
}

fn parse_rgb(s: &str) -> Color32 {
    // rgb(r, g, b)
    let nums: Vec<u8> = s
        .trim_start_matches("rgb(")
        .trim_end_matches(')')
        .split(',')
        .filter_map(|n| n.trim().parse().ok())
        .collect();
    if nums.len() == 3 {
        Color32::from_rgb(nums[0], nums[1], nums[2])
    } else {
        Color32::TRANSPARENT
    }
}

fn parse_rgba(s: &str) -> Color32 {
    // rgba(r, g, b, a) where a is 0-1 or 0-255
    let parts: Vec<&str> = s
        .trim_start_matches("rgba(")
        .trim_end_matches(')')
        .split(',')
        .collect();
    if parts.len() == 4 {
        let r = parts[0].trim().parse().unwrap_or(0);
        let g = parts[1].trim().parse().unwrap_or(0);
        let b = parts[2].trim().parse().unwrap_or(0);
        let a_str = parts[3].trim();
        let a = if a_str.contains('.') {
            (a_str.parse::<f32>().unwrap_or(1.0) * 255.0) as u8
        } else {
            a_str.parse().unwrap_or(255)
        };
        Color32::from_rgba_unmultiplied(r, g, b, a)
    } else {
        Color32::TRANSPARENT
    }
}

/// Variance colour (delay/overspend vs improvement/underspend).
pub fn variance_color(value: Option<i64>, variant: ThemeVariant) -> Color32 {
    let colors = color_tokens(variant);
    match value {
        Some(v) if v > 0 => colors.status_critical,
        Some(v) if v < 0 => colors.status_ok,
        _ => colors.border_subtle,
    }
}

/// Non-working day background tint.
pub fn non_working_color(variant: ThemeVariant) -> Color32 {
    color_tokens(variant).non_working_day
}

/// Status date line colour.
pub fn status_date_color(variant: ThemeVariant) -> Color32 {
    color_tokens(variant).status_date
}

/// Status severity colours for toast/status messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusSeverity {
    Info,
    Success,
    Warning,
    Error,
}

impl StatusSeverity {
    /// Get the colour for this severity from tokens.
    pub fn color(self, variant: ThemeVariant) -> Color32 {
        let colors = color_tokens(variant);
        match self {
            Self::Info => colors.text_muted,
            Self::Success => colors.status_ok,
            Self::Warning => colors.status_warn,
            Self::Error => colors.status_critical,
        }
    }

    /// Classify a message by its wording.
    pub fn from_message(message: &str) -> Self {
        let lower = message.to_ascii_lowercase();
        if lower.contains("cannot")
            || lower.contains("must")
            || lower.contains("select")
            || lower.contains("empty")
            || lower.contains("invalid")
            || lower.contains("needs")
            || lower.contains("not a valid")
        {
            return Self::Error;
        }
        if lower.starts_with("saved")
            || lower.starts_with("opened")
            || lower.starts_with("activity created")
            || lower.starts_with("report")
            || lower.starts_with("gantt printed")
            || lower.starts_with("diary note")
            || lower.starts_with("production entry")
            || lower.starts_with("activity deleted")
        {
            return Self::Success;
        }
        if lower.contains("warning") || lower.contains("warn") {
            return Self::Warning;
        }
        Self::Info
    }
}

/// Chrome button helpers — primary, ghost, tab.
///
/// Every toolbar, tab, and status-bar control goes through these;
/// the shell never hand-rolls fills or strokes (guide II.30.6).
pub mod chrome {
    use super::*;
    use egui::{Button, RichText, Stroke};

    /// The one primary action per row (e.g., "New activity"): accent fill,
    /// no stroke, high-contrast label.
    pub fn primary(variant: ThemeVariant, label: impl Into<String>) -> Button<'static> {
        let colors = color_tokens(variant);
        let accent = ACCENT;
        let on_accent = ACCENT_ON;
        Button::new(RichText::new(label.into()).color(on_accent).strong())
            .fill(accent)
            .stroke(Stroke::NONE)
    }

    /// Toolbar/quick actions: invisible chrome until hovered.
    pub fn ghost(variant: ThemeVariant, label: impl Into<String>) -> Button<'static> {
        Button::new(label.into())
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE)
    }

    /// View tabs. Selected: accent-tinted fill, strong text, accent hairline;
    /// idle: transparent with muted text.
    pub fn tab(variant: ThemeVariant, selected: bool, label: impl Into<String>) -> Button<'static> {
        let colors = color_tokens(variant);
        let text_color = if selected {
            colors.text_primary
        } else {
            colors.text_muted
        };
        let text = RichText::new(label.into())
            .color(text_color)
            .strong();

        // Accent with 15% opacity for selected tab background
        let accent_bg = Color32::from_rgba_unmultiplied(
            ACCENT.r(), ACCENT.g(), ACCENT.b(), 38
        );

        Button::new(text)
            .selected(selected)
            .frame(selected)
            .fill(if selected {
                accent_bg
            } else {
                Color32::TRANSPARENT
            })
            .stroke(if selected {
                Stroke::new(1.0, ACCENT)
            } else {
                Stroke::NONE
            })
    }

    /// Secondary button: subtle outline, no fill.
    pub fn secondary(variant: ThemeVariant, label: impl Into<String>) -> Button<'static> {
        let colors = color_tokens(variant);
        Button::new(label.into())
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::new(1.0, colors.border_strong))
    }

    /// Danger button: critical colour, for destructive actions.
    pub fn danger(variant: ThemeVariant, label: impl Into<String>) -> Button<'static> {
        let colors = color_tokens(variant);
        let critical = colors.status_critical;
        let on_critical = Color32::WHITE;
        Button::new(RichText::new(label.into()).color(on_critical).strong())
            .fill(critical)
            .stroke(Stroke::NONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_colors() {
        assert_eq!(parse_color("#F4F5F7"), Color32::from_rgb(0xF4, 0xF5, 0xF7));
        assert_eq!(parse_color("#1F2933"), Color32::from_rgb(0x1F, 0x29, 0x33));
        assert_eq!(parse_color("#2F6FED"), Color32::from_rgb(0x2F, 0x6F, 0xED));
        assert_eq!(parse_color("#2F6FEDFF"), Color32::from_rgba_unmultiplied(0x2F, 0x6F, 0xED, 0xFF));
        assert_eq!(parse_color("#2F6FED80"), Color32::from_rgba_unmultiplied(0x2F, 0x6F, 0xED, 0x80));
    }

    #[test]
    fn parse_rgb_colors() {
        assert_eq!(parse_color("rgb(244, 245, 247)"), Color32::from_rgb(244, 245, 247));
        assert_eq!(parse_color("rgb(31, 41, 51)"), Color32::from_rgb(31, 41, 51));
    }

    #[test]
    fn parse_rgba_colors() {
        assert_eq!(
            parse_color("rgba(47, 111, 237, 0.2)"),
            Color32::from_rgba_unmultiplied(47, 111, 237, 51)
        );
        assert_eq!(
            parse_color("rgba(47, 111, 237, 51)"),
            Color32::from_rgba_unmultiplied(47, 111, 237, 51)
        );
    }

    #[test]
    fn variance_colors() {
        // Positive variance = delay/overspend = red family (status_critical)
        assert_eq!(variance_color(Some(5), ThemeVariant::Light), LIGHT_COLORS.status_critical);
        assert_eq!(variance_color(Some(5), ThemeVariant::Dark), DARK_COLORS.status_critical);

        // Negative variance = improvement = green family (status_ok)
        assert_eq!(variance_color(Some(-3), ThemeVariant::Light), LIGHT_COLORS.status_ok);
        assert_eq!(variance_color(Some(-3), ThemeVariant::Dark), DARK_COLORS.status_ok);

        // Zero/none = grid/subtle
        assert_eq!(variance_color(None, ThemeVariant::Light), LIGHT_COLORS.border_subtle);
        assert_eq!(variance_color(Some(0), ThemeVariant::Light), LIGHT_COLORS.border_subtle);
    }

    #[test]
    fn status_severity_colors() {
        assert_eq!(StatusSeverity::Info.color(ThemeVariant::Light), LIGHT_COLORS.text_muted);
        assert_eq!(StatusSeverity::Success.color(ThemeVariant::Light), LIGHT_COLORS.status_ok);
        assert_eq!(StatusSeverity::Warning.color(ThemeVariant::Light), LIGHT_COLORS.status_warn);
        assert_eq!(StatusSeverity::Error.color(ThemeVariant::Light), LIGHT_COLORS.status_critical);
    }

    #[test]
    fn status_severity_classification() {
        assert_eq!(
            StatusSeverity::from_message("Cannot delete activity"),
            StatusSeverity::Error
        );
        assert_eq!(
            StatusSeverity::from_message("Must select an activity"),
            StatusSeverity::Error
        );
        assert_eq!(
            StatusSeverity::from_message("Invalid date"),
            StatusSeverity::Error
        );
        assert_eq!(
            StatusSeverity::from_message("Saved project"),
            StatusSeverity::Success
        );
        assert_eq!(
            StatusSeverity::from_message("Activity created"),
            StatusSeverity::Success
        );
        assert_eq!(
            StatusSeverity::from_message("Warning: over allocation"),
            StatusSeverity::Warning
        );
        assert_eq!(
            StatusSeverity::from_message("Info: loaded"),
            StatusSeverity::Info
        );
    }

    #[test]
    fn chrome_buttons_create() {
        let _primary = chrome::primary(ThemeVariant::Light, "New");
        let _ghost = chrome::ghost(ThemeVariant::Light, "Action");
        let _tab = chrome::tab(ThemeVariant::Light, true, "Table");
        let _secondary = chrome::secondary(ThemeVariant::Light, "Cancel");
        let _danger = chrome::danger(ThemeVariant::Light, "Delete");
    }
}