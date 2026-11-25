//! Design Token Validation Tests
//!
//! These tests verify that the design token system meets all requirements:
//! - WCAG 2.2 AA contrast ratios
// - Token completeness (all semantic roles covered)
// - Density scaling correctness
// - Theme parity (light/dark both valid)
// - Serialization round-trip

use cadence_ui_egui::design_tokens::*;

#[test]
fn light_colors_complete() {
    // Every semantic color role must be populated
    let c = LIGHT_COLORS;
    // Surface
    assert_ne!(c.surface_base, Color32::TRANSPARENT);
    assert_ne!(c.surface_raised, Color32::TRANSPARENT);
    assert_ne!(c.surface_sunken, Color32::TRANSPARENT);
    assert_ne!(c.surface_deep, Color32::TRANSPARENT);
    // Text
    assert_ne!(c.text_primary, Color32::TRANSPARENT);
    assert_ne!(c.text_secondary, Color32::TRANSPARENT);
    assert_ne!(c.text_muted, Color32::TRANSPARENT);
    assert_ne!(c.text_on_accent, Color32::TRANSPARENT);
    // Accent
    assert_ne!(c.accent, Color32::TRANSPARENT);
    assert_ne!(c.accent_hover, Color32::TRANSPARENT);
    assert_ne!(c.accent_active, Color32::TRANSPARENT);
    // Status
    assert_ne!(c.status_critical, Color32::TRANSPARENT);
    assert_ne!(c.status_critical_on, Color32::TRANSPARENT);
    assert_ne!(c.status_normal, Color32::TRANSPARENT);
    assert_ne!(c.status_ok, Color32::TRANSPARENT);
    assert_ne!(c.status_ok_on, Color32::TRANSPARENT);
    assert_ne!(c.status_warn, Color32::TRANSPARENT);
    assert_ne!(c.status_warn_on, Color32::TRANSPARENT);
    // Border
    assert_ne!(c.border_subtle, Color32::TRANSPARENT);
    assert_ne!(c.border_strong, Color32::TRANSPARENT);
    // Focus
    assert_ne!(c.focus_ring, Color32::TRANSPARENT);
    // Selection
    assert_ne!(c.selection_bg, Color32::TRANSPARENT);
    assert_ne!(c.selection_text, Color32::TRANSPARENT);
    // Special
    assert_ne!(c.non_working_day, Color32::TRANSPARENT);
    assert_ne!(c.status_date, Color32::TRANSPARENT);
    assert_ne!(c.overlay_scrim, Color32::TRANSPARENT);
    // Disabled
    assert_ne!(c.disabled_bg, Color32::TRANSPARENT);
    assert_ne!(c.disabled_text, Color32::TRANSPARENT);
}

#[test]
fn dark_colors_complete() {
    let c = DARK_COLORS;
    // Surface
    assert_ne!(c.surface_base, Color32::TRANSPARENT);
    assert_ne!(c.surface_raised, Color32::TRANSPARENT);
    assert_ne!(c.surface_sunken, Color32::TRANSPARENT);
    assert_ne!(c.surface_deep, Color32::TRANSPARENT);
    // Text
    assert_ne!(c.text_primary, Color32::TRANSPARENT);
    assert_ne!(c.text_secondary, Color32::TRANSPARENT);
    assert_ne!(c.text_muted, Color32::TRANSPARENT);
    assert_ne!(c.text_on_accent, Color32::TRANSPARENT);
    // Accent
    assert_ne!(c.accent, Color32::TRANSPARENT);
    assert_ne!(c.accent_hover, Color32::TRANSPARENT);
    assert_ne!(c.accent_active, Color32::TRANSPARENT);
    // Status
    assert_ne!(c.status_critical, Color32::TRANSPARENT);
    assert_ne!(c.status_critical_on, Color32::TRANSPARENT);
    assert_ne!(c.status_normal, Color32::TRANSPARENT);
    assert_ne!(c.status_ok, Color32::TRANSPARENT);
    assert_ne!(c.status_ok_on, Color32::TRANSPARENT);
    assert_ne!(c.status_warn, Color32::TRANSPARENT);
    assert_ne!(c.status_warn_on, Color32::TRANSPARENT);
    // Border
    assert_ne!(c.border_subtle, Color32::TRANSPARENT);
    assert_ne!(c.border_strong, Color32::TRANSPARENT);
    // Focus
    assert_ne!(c.focus_ring, Color32::TRANSPARENT);
    // Selection
    assert_ne!(c.selection_bg, Color32::TRANSPARENT);
    assert_ne!(c.selection_text, Color32::TRANSPARENT);
    // Special
    assert_ne!(c.non_working_day, Color32::TRANSPARENT);
    assert_ne!(c.status_date, Color32::TRANSPARENT);
    assert_ne!(c.overlay_scrim, Color32::TRANSPARENT);
    // Disabled
    assert_ne!(c.disabled_bg, Color32::TRANSPARENT);
    assert_ne!(c.disabled_text, Color32::TRANSPARENT);
}

/// Computes relative luminance (sRGB -> linear -> luminance).
fn luminance(color: Color32) -> f32 {
    let r = color.r() as f32 / 255.0;
    let g = color.g() as f32 / 255.0;
    let b = color.b() as f32 / 255.0;

    let to_linear = |c: f32| -> f32 {
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };

    0.2126 * to_linear(r) + 0.7152 * to_linear(g) + 0.0722 * to_linear(b)
}

/// Computes contrast ratio between two colors (lighter / darker).
fn contrast_ratio(lighter: Color32, darker: Color32) -> f32 {
    let l1 = luminance(lighter);
    let l2 = luminance(darker);
    (l1.max(l2) + 0.05) / (l1.min(l2) + 0.05)
}

/// Asserts that foreground meets minimum contrast against background.
fn assert_contrast(fg: Color32, bg: Color32, min_ratio: f32, context: &str) {
    let ratio = contrast_ratio(fg, bg);
    assert!(
        ratio >= min_ratio,
        "{}: contrast ratio {:.2}:1 < {:.2}:1 (fg={:02X}{:02X}{:02X}, bg={:02X}{:02X}{:02X})",
        context,
        ratio,
        min_ratio,
        fg.r(),
        fg.g(),
        fg.b(),
        bg.r(),
        bg.g(),
        bg.b()
    );
}

#[test]
fn light_theme_text_contrast() {
    let c = LIGHT_COLORS;
    // Body text (4.5:1 minimum)
    assert_contrast(
        c.text_primary,
        c.surface_raised,
        4.5,
        "light: text_primary on surface_raised",
    );
    assert_contrast(
        c.text_secondary,
        c.surface_raised,
        4.5,
        "light: text_secondary on surface_raised",
    );
    // Muted text (3:1 minimum for secondary)
    assert_contrast(
        c.text_muted,
        c.surface_raised,
        3.0,
        "light: text_muted on surface_raised",
    );
    // Text on accent (4.5:1)
    assert_contrast(
        c.text_on_accent,
        c.accent,
        4.5,
        "light: text_on_accent on accent",
    );
}

#[test]
fn dark_theme_text_contrast() {
    let c = DARK_COLORS;
    // Body text (4.5:1 minimum)
    assert_contrast(
        c.text_primary,
        c.surface_raised,
        4.5,
        "dark: text_primary on surface_raised",
    );
    assert_contrast(
        c.text_secondary,
        c.surface_raised,
        4.5,
        "dark: text_secondary on surface_raised",
    );
    // Muted text (3:1 minimum for secondary)
    assert_contrast(
        c.text_muted,
        c.surface_raised,
        3.0,
        "dark: text_muted on surface_raised",
    );
    // Text on accent (4.5:1)
    assert_contrast(
        c.text_on_accent,
        c.accent,
        4.5,
        "dark: text_on_accent on accent",
    );
}

#[test]
fn light_theme_ui_contrast() {
    let c = LIGHT_COLORS;
    // Focus ring (3:1 non-text minimum)
    assert_contrast(
        c.focus_ring,
        c.surface_raised,
        3.0,
        "light: focus_ring on surface_raised",
    );
    // Border subtle (3:1)
    assert_contrast(
        c.border_subtle,
        c.surface_raised,
        3.0,
        "light: border_subtle on surface_raised",
    );
    // Status colors on surface (3:1 for UI components)
    assert_contrast(
        c.status_critical,
        c.surface_raised,
        3.0,
        "light: status_critical on surface_raised",
    );
    assert_contrast(
        c.status_normal,
        c.surface_raised,
        3.0,
        "light: status_normal on surface_raised",
    );
    assert_contrast(
        c.status_ok,
        c.surface_raised,
        3.0,
        "light: status_ok on surface_raised",
    );
    assert_contrast(
        c.status_warn,
        c.surface_raised,
        3.0,
        "light: status_warn on surface_raised",
    );
    // Accent as UI component (3:1)
    assert_contrast(
        c.accent,
        c.surface_raised,
        3.0,
        "light: accent on surface_raised",
    );
}

#[test]
fn dark_theme_ui_contrast() {
    let c = DARK_COLORS;
    // Focus ring (3:1 non-text minimum)
    assert_contrast(
        c.focus_ring,
        c.surface_raised,
        3.0,
        "dark: focus_ring on surface_raised",
    );
    // Border subtle (3:1)
    assert_contrast(
        c.border_subtle,
        c.surface_raised,
        3.0,
        "dark: border_subtle on surface_raised",
    );
    // Status colors on surface (3:1 for UI components)
    assert_contrast(
        c.status_critical,
        c.surface_raised,
        3.0,
        "dark: status_critical on surface_raised",
    );
    assert_contrast(
        c.status_normal,
        c.surface_raised,
        3.0,
        "dark: status_normal on surface_raised",
    );
    assert_contrast(
        c.status_ok,
        c.surface_raised,
        3.0,
        "dark: status_ok on surface_raised",
    );
    assert_contrast(
        c.status_warn,
        c.surface_raised,
        3.0,
        "dark: status_warn on surface_raised",
    );
    // Accent as UI component (3:1)
    assert_contrast(
        c.accent,
        c.surface_raised,
        3.0,
        "dark: accent on surface_raised",
    );
}

#[test]
fn light_theme_status_text_contrast() {
    let c = LIGHT_COLORS;
    // Status text on status backgrounds (4.5:1 for text)
    assert_contrast(
        c.status_critical_on,
        c.status_critical,
        4.5,
        "light: critical_on on critical",
    );
    assert_contrast(c.status_ok_on, c.status_ok, 4.5, "light: ok_on on ok");
    assert_contrast(
        c.status_warn_on,
        c.status_warn,
        4.5,
        "light: warn_on on warn",
    );
}

#[test]
fn dark_theme_status_text_contrast() {
    let c = DARK_COLORS;
    // Status text on status backgrounds (4.5:1 for text)
    assert_contrast(
        c.status_critical_on,
        c.status_critical,
        4.5,
        "dark: critical_on on critical",
    );
    assert_contrast(c.status_ok_on, c.status_ok, 4.5, "dark: ok_on on ok");
    assert_contrast(
        c.status_warn_on,
        c.status_warn,
        4.5,
        "dark: warn_on on warn",
    );
}

#[test]
fn spacing_token_values_standard() {
    assert_eq!(SpacingToken::Xs.value(Density::Standard), 2.0);
    assert_eq!(SpacingToken::Sm.value(Density::Standard), 4.0);
    assert_eq!(SpacingToken::Md.value(Density::Standard), 8.0);
    assert_eq!(SpacingToken::Lg.value(Density::Standard), 12.0);
    assert_eq!(SpacingToken::Xl.value(Density::Standard), 16.0);
    assert_eq!(SpacingToken::Xxl.value(Density::Standard), 24.0);
    assert_eq!(SpacingToken::Xxxl.value(Density::Standard), 32.0);
    assert_eq!(SpacingToken::Huge.value(Density::Standard), 48.0);
    assert_eq!(SpacingToken::Massive.value(Density::Standard), 64.0);
}

#[test]
fn spacing_token_density_scaling() {
    let base = SpacingToken::Md.value(Density::Standard);
    let compact = SpacingToken::Md.value(Density::Compact);
    let comfortable = SpacingToken::Md.value(Density::Comfortable);

    // Compact = 0.875x
    assert!((compact - base * 0.875).abs() < 0.001);
    // Comfortable = 1.125x
    assert!((comfortable - base * 1.125).abs() < 0.001);
}

#[test]
fn radius_token_values() {
    assert_eq!(RadiusToken::None.value(), 0.0);
    assert_eq!(RadiusToken::Sm.value(), 2.0);
    assert_eq!(RadiusToken::Md.value(), 4.0);
    assert_eq!(RadiusToken::Lg.value(), 6.0);
    assert_eq!(RadiusToken::Xl.value(), 8.0);
    assert_eq!(RadiusToken::Full.value(), 9999.0);
}

#[test]
fn duration_tokens() {
    assert_eq!(DurationToken::Instant.ms(), 0);
    assert_eq!(DurationToken::Fast.ms(), 100);
    assert_eq!(DurationToken::Normal.ms(), 200);
    assert_eq!(DurationToken::Slow.ms(), 300);

    // Reduced motion
    assert_eq!(DurationToken::Fast.ms_reduced(true), 0);
    assert_eq!(DurationToken::Normal.ms_reduced(true), 0);
    assert_eq!(DurationToken::Slow.ms_reduced(true), 0);
    assert_eq!(DurationToken::Fast.ms_reduced(false), 100);
}

#[test]
fn typography_scale_complete() {
    let scale = TypographyScale::default();
    // All roles present with reasonable values
    assert!(scale.display.size > 0.0);
    assert!(scale.h1.size > 0.0);
    assert!(scale.h2.size > 0.0);
    assert!(scale.h3.size > 0.0);
    assert!(scale.body.size > 0.0);
    assert!(scale.small.size > 0.0);
    assert!(scale.caption.size > 0.0);
    assert!(scale.mono.size > 0.0);
    assert!(scale.button.size > 0.0);

    // Hierarchy: display > h1 > h2 > h3 > body > small > caption
    assert!(scale.display.size > scale.h1.size);
    assert!(scale.h1.size > scale.h2.size);
    assert!(scale.h2.size > scale.h3.size);
    assert!(scale.h3.size > scale.body.size);
    assert!(scale.body.size > scale.small.size);
    assert!(scale.small.size > scale.caption.size);
}

#[test]
fn theme_registry_creation() {
    let registry = ThemeRegistry::new();
    assert_eq!(registry.mode(), ThemeMode::System);
}

#[test]
fn theme_registry_mode_switching() {
    let mut registry = ThemeRegistry::new();
    registry.set_mode(ThemeMode::Light);
    assert_eq!(registry.mode(), ThemeMode::Light);
    registry.set_mode(ThemeMode::Dark);
    assert_eq!(registry.mode(), ThemeMode::Dark);
    registry.set_mode(ThemeMode::System);
    assert_eq!(registry.mode(), ThemeMode::System);
}

#[test]
fn light_theme_not_dark() {
    let theme = LightTheme::default();
    assert!(!theme.is_dark());
}

#[test]
fn dark_theme_is_dark() {
    let theme = DarkTheme::default();
    assert!(theme.is_dark());
}

#[test]
fn theme_apply_no_panic() {
    let ctx = Context::default();
    let light = LightTheme::default();
    let dark = DarkTheme::default();

    // Should not panic
    light.apply(&ctx);
    dark.apply(&ctx);
}

#[test]
fn design_token_set_serialization() {
    let set = DesignTokenSet {
        colors: LIGHT_COLORS,
        typography: TypographyScale::default(),
        density: Density::Standard,
        radius: RadiusSet::default(),
        elevation: ElevationSet::light(),
        motion: MotionSet::default(),
    };

    let json = serde_json::to_string(&set).unwrap();
    assert!(!json.is_empty());

    let parsed: DesignTokenSet = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.colors, set.colors);
    assert_eq!(parsed.typography.body.size, set.typography.body.size);
}

#[test]
fn color_tokens_serialization() {
    let json = serde_json::to_string(&LIGHT_COLORS).unwrap();
    let _parsed: ColorTokens = serde_json::from_str(&json).unwrap();

    let json = serde_json::to_string(&DARK_COLORS).unwrap();
    let _parsed: ColorTokens = serde_json::from_str(&json).unwrap();
}

#[test]
fn density_cycle() {
    assert_eq!(Density::Compact.next(), Density::Standard);
    assert_eq!(Density::Standard.next(), Density::Comfortable);
    assert_eq!(Density::Comfortable.next(), Density::Compact);
}

#[test]
fn legacy_palette_compatibility() {
    let light_pal = palette(false);
    let dark_pal = palette(true);

    // Verify legacy palette has all expected fields
    assert_ne!(light_pal.bg, Color32::TRANSPARENT);
    assert_ne!(light_pal.panel, Color32::TRANSPARENT);
    assert_ne!(light_pal.text, Color32::TRANSPARENT);
    assert_ne!(light_pal.text_muted, Color32::TRANSPARENT);
    assert_ne!(light_pal.grid, Color32::TRANSPARENT);
    assert_ne!(light_pal.critical, Color32::TRANSPARENT);
    assert_ne!(light_pal.normal, Color32::TRANSPARENT);
    assert_ne!(light_pal.ok, Color32::TRANSPARENT);
    assert_ne!(light_pal.warn, Color32::TRANSPARENT);
    assert_ne!(light_pal.deep, Color32::TRANSPARENT);
    assert_ne!(light_pal.non_working, Color32::TRANSPARENT);
    assert_ne!(light_pal.status_date, Color32::TRANSPARENT);
    assert_ne!(light_pal.focus_ring, Color32::TRANSPARENT);

    assert_ne!(dark_pal.bg, Color32::TRANSPARENT);
    assert_ne!(dark_pal.panel, Color32::TRANSPARENT);
    // ... etc
}

#[test]
fn legacy_chrome_helpers() {
    let _btn = chrome::primary("Test");
    let _btn = chrome::ghost("Test");
    let _btn = chrome::tab(true, true, "Test");
    let _btn = chrome::tab(false, false, "Test");
}

#[test]
fn legacy_variance_color() {
    // Positive = critical (delay/overspend)
    assert_eq!(variance_color(Some(5), true), DARK_COLORS.status_critical);
    assert_eq!(variance_color(Some(5), false), LIGHT_COLORS.status_critical);
    // Negative = ok (ahead/under)
    assert_eq!(variance_color(Some(-3), true), DARK_COLORS.status_ok);
    assert_eq!(variance_color(Some(-3), false), LIGHT_COLORS.status_ok);
    // Zero/None = grid
    assert_eq!(variance_color(None, true), DARK_COLORS.border_subtle);
    assert_eq!(variance_color(Some(0), false), LIGHT_COLORS.border_subtle);
}

#[test]
fn legacy_translucent() {
    let color = Color32::from_rgb(0xFF, 0x00, 0x00);
    let translucent = translucent(color, 128);
    assert_eq!(translucent.r(), 0xFF);
    assert_eq!(translucent.g(), 0x00);
    assert_eq!(translucent.b(), 0x00);
    assert_eq!(translucent.a(), 128);
}

#[test]
fn json_export_matches_spec() {
    let json = export_tokens_json();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Verify structure
    assert!(value.get("light").is_some());
    assert!(value.get("dark").is_some());
    assert!(value.get("metadata").is_some());

    let metadata = value.get("metadata").unwrap();
    assert_eq!(metadata.get("baseUnit").unwrap(), 4);
    assert_eq!(metadata.get("fontFamily").unwrap(), "Inter");
    assert_eq!(metadata.get("iconFont").unwrap(), "Phosphor");
}
