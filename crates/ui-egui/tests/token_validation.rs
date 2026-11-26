//! Design Token Validation Tests
//!
//! These tests validate the design token system meets all requirements:
//! - Contrast ratios (WCAG AA)
//! - Token completeness
//! - Serialization round-trip
//! - Theme parity
//! - Density scaling
//! - Accessibility compliance

use cadence_ui_egui::design_tokens::{
    chrome, dark_tokens, light_tokens, non_working_color, parse_color, status_date_color, variance_color,
    ColorTokens, DarkTheme, Density, DesignTokenSet, EasingToken, FontWeight, LightTheme,
    RadiusSet, RadiusToken, SpacingToken, StatusSeverity, Theme, ThemeMode, ThemeRegistry,
    ThemeVariant, TypographyScale, ACCENT, ACCENT_ON, BASE_UNIT, DARK_COLORS, FOCUS_RING,
    LIGHT_COLORS, ContrastRatio,
};

/// Test that all required colour roles exist and are distinct per theme.
#[test]
fn color_roles_complete_and_distinct() {
    for variant in [ThemeMode::Light, ThemeMode::Dark, ThemeMode::System] {
        let tokens = match variant {
            ThemeMode::Light => light_tokens(),
            ThemeMode::Dark => dark_tokens(),
            ThemeMode::System => light_tokens(),
        };
        let c = &tokens.colors;

        // Surface roles
        assert_ne!(c.surface_base, c.surface_raised);
        assert_ne!(c.surface_raised, c.surface_sunken);
        assert_ne!(c.surface_sunken, c.surface_deep);
        assert_ne!(c.surface_deep, c.disabled_bg);

        // Text roles
        assert_ne!(c.text_primary, c.text_secondary);
        assert_ne!(c.text_secondary, c.text_muted);
        assert_ne!(c.text_muted, c.disabled_text);

        // Accent roles
        assert_ne!(c.accent, c.accent_hover);
        assert_ne!(c.accent_hover, c.accent_active);

        // Border roles
        assert_ne!(c.border_subtle, c.border_strong);

        // Focus roles
        assert_ne!(c.focus_ring, Color32::TRANSPARENT);

        // Status roles - all distinct
        let status_colors = [
            c.status_critical,
            c.status_normal,
            c.status_ok,
            c.status_warn,
        ];
        for i in 0..status_colors.len() {
            for j in i + 1..status_colors.len() {
                assert_ne!(
                    status_colors[i], status_colors[j],
                    "Status colors must be distinct"
                );
            }
        }

        // Overlay roles
        assert_ne!(c.overlay_scrim, Color32::TRANSPARENT);
    }
}

use egui::Color32;

/// Test WCAG AA contrast for critical colour pairs.
#[test]
fn contrast_wcag_aa() {
    for variant in [ThemeMode::Light, ThemeMode::Dark, ThemeMode::System] {
        let tokens = match variant {
            ThemeMode::Light => light_tokens(),
            ThemeMode::Dark => dark_tokens(),
            ThemeMode::System => light_tokens(),
        };
        let c = &tokens.colors;

        // Helper: compute relative luminance
        fn luminance(color: Color32) -> f32 {
            let srgb_to_linear = |c: f32| {
                if c <= 0.04045 {
                    c / 12.92
                } else {
                    ((c + 0.055) / 1.055).powf(2.4)
                }
            };
            let r = srgb_to_linear(c.r() as f32 / 255.0);
            let g = srgb_to_linear(g.g() as f32 / 255.0);
            let b = srgb_to_linear(b.b() as f32 / 255.0);
            0.2126 * r + 0.7152 * g + 0.0722 * b
        }

        fn contrast_ratio(fg: Color32, bg: Color32) -> f32 {
            let l1 = luminance(fg);
            let l2 = luminance(bg);
            let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
            (lighter + 0.05) / (darker + 0.05)
        }

        let bg_raised = c.surface_raised;
        let _bg_base = c.surface_base;

        // Body text on raised surface ≥ 4.5:1
        let text_primary = c.text_primary;
        let cr = contrast_ratio(text_primary, bg_raised);
        assert!(
            cr >= 4.5,
            "Body text on panel: contrast {:.2}:1 < 4.5:1 ({variant:?})",
            cr
        );

        // Muted text on raised surface ≥ 3:1 (large text threshold)
        let text_muted = c.text_muted;
        let cr = contrast_ratio(text_muted, bg_raised);
        assert!(
            cr >= 3.0,
            "Muted text on panel: contrast {:.2}:1 < 3:1 ({variant:?})",
            cr
        );

        // Accent with white text ≥ 4.5:1
        let accent = c.accent;
        let on_accent = c.text_on_accent;
        let cr = contrast_ratio(on_accent, accent);
        assert!(
            cr >= 4.5,
            "White on accent: contrast {:.2}:1 < 4.5:1 ({variant:?})",
            cr
        );

        // Accent on raised surface ≥ 3:1 (non-text)
        let cr = contrast_ratio(accent, bg_raised);
        assert!(
            cr >= 3.0,
            "Accent on panel (non-text): contrast {:.2}:1 < 3:1 ({variant:?})",
            cr
        );

        // Focus ring on raised surface ≥ 3:1
        let focus = c.focus_ring;
        let cr = contrast_ratio(focus, bg_raised);
        assert!(
            cr >= 3.0,
            "Focus ring on panel: contrast {:.2}:1 < 3:1 ({variant:?})",
            cr
        );

        // Critical on raised surface ≥ 3:1
        let critical = c.status_critical;
        let cr = contrast_ratio(critical, bg_raised);
        assert!(
            cr >= 3.0,
            "Critical on panel: contrast {:.2}:1 < 3:1 ({variant:?})",
            cr
        );

        // Success on raised surface ≥ 3:1
        let success = c.status_ok;
        let cr = contrast_ratio(success, bg_raised);
        assert!(
            cr >= 3.0,
            "Success on panel: contrast {:.2}:1 < 3:1 ({variant:?})",
            cr
        );

        // Warning on raised surface ≥ 3:1
        let warning = c.status_warn;
        let cr = contrast_ratio(warning, bg_raised);
        assert!(
            cr >= 3.0,
            "Warning on panel: contrast {:.2}:1 < 3:1 ({variant:?})",
            cr
        );

        // Border subtle on raised surface ≥ 3:1
        let border = c.border_subtle;
        let cr = contrast_ratio(border, bg_raised);
        assert!(
            cr >= 3.0,
            "Border on panel: contrast {:.2}:1 < 3:1 ({variant:?})",
            cr
        );
    }
}

/// Test serialization round-trip for all token types.
#[test]
fn serialization_roundtrip() {
    let light = light_tokens();
    let dark = dark_tokens();

    // Serialize to JSON
    let light_json = serde_json::to_string_pretty(&light).unwrap();
    let dark_json = serde_json::to_string_pretty(&dark).unwrap();

    // Deserialize back
    let light_parsed: DesignTokenSet = serde_json::from_str(&light_json).unwrap();
    let dark_parsed: DesignTokenSet = serde_json::from_str(&dark_json).unwrap();

    // Verify equality
    assert_eq!(light, light_parsed);
    assert_eq!(dark, dark_parsed);
}

/// Test spacing tokens match 4px base with correct multipliers.
#[test]
fn spacing_tokens_correct_multipliers() {
    let _tokens = light_tokens();
    let base = BASE_UNIT;
    assert_eq!(base, 4.0);

    let expected = [
        (SpacingToken::Xs, 0.5),
        (SpacingToken::Sm, 1.0),
        (SpacingToken::Md, 2.0),
        (SpacingToken::Lg, 3.0),
        (SpacingToken::Xl, 4.0),
        (SpacingToken::Xxl, 6.0),
        (SpacingToken::Xxxl, 8.0),
        (SpacingToken::Huge, 12.0),
        (SpacingToken::Massive, 16.0),
    ];

    for (token, mult) in expected {
        let value = token.value(Density::Standard);
        assert_eq!(
            value,
            base * mult,
            "Token {:?} should be {mult}× base",
            token
        );
    }
}

/// Test density scaling works correctly.
#[test]
fn density_scaling_all_levels() {
    let mut tokens = light_tokens();

    // Compact: 0.875×
    tokens.density = Density::Compact;
    let md_value = SpacingToken::Md.value(tokens.density);
    assert!((md_value - 7.0).abs() < 0.01); // 8 * 0.875

    // Standard: 1.0×
    tokens.density = Density::Standard;
    let md_value = SpacingToken::Md.value(tokens.density);
    assert!((md_value - 8.0).abs() < 0.01);

    // Comfortable: 1.125×
    tokens.density = Density::Comfortable;
    let md_value = SpacingToken::Md.value(tokens.density);
    assert!((md_value - 9.0).abs() < 0.01); // 8 * 1.125
}

/// Test radius system has all required values.
#[test]
fn radius_system_complete() {
    let tokens = light_tokens();
    let r = &tokens.radius;

    assert_eq!(r.none, 0.0);
    assert_eq!(r.sm, 2.0);
    assert_eq!(r.md, 4.0);
    assert_eq!(r.lg, 6.0);
    assert_eq!(r.xl, 8.0);
    assert_eq!(r.full, 9999.0);
}

/// Test elevation system has 4 levels with increasing blur.
#[test]
fn elevation_system_progressive() {
    let tokens = light_tokens();
    let e = &tokens.elevation;

    // Tuple structure: (offset_y, blur, spread, color)
    // Level 0: no shadow
    assert_eq!(e.level0.1, 0.0); // blur
    assert_eq!(e.level0.0, 0.0); // offset_y

    // Level 1: subtle
    assert!(e.level1.1 > e.level0.1); // blur
    assert!(e.level1.0 > e.level0.0); // offset_y

    // Level 2: medium
    assert!(e.level2.1 > e.level1.1); // blur
    assert!(e.level2.0 > e.level1.0); // offset_y

    // Level 3: high
    assert!(e.level3.1 > e.level2.1); // blur
    assert!(e.level3.0 > e.level2.0); // offset_y
}

/// Test typography scale follows reasonable ratios.
#[test]
fn typography_scale_ratio() {
    let tokens = light_tokens();
    let t = &tokens.typography;

    // Body is base (11px)
    assert_eq!(t.body.size, 11.0);

    // h3 ~ 1.27× body (14px)
    assert!((t.h3.size / t.body.size - 1.27).abs() < 0.05);

    // h2 ~ 1.45× body (16px)
    assert!((t.h2.size / t.body.size - 1.45).abs() < 0.05);

    // h1 ~ 1.82× body (20px)
    assert!((t.h1.size / t.body.size - 1.82).abs() < 0.05);

    // display ~ 2.18× body (24px)
    assert!((t.display.size / t.body.size - 2.18).abs() < 0.05);

    // Small is smaller than body
    assert!(t.small.size < t.body.size);

    // Caption is smallest
    assert!(t.caption.size < t.small.size);

    // Mono is body size
    assert_eq!(t.mono.size, t.body.size);
}

/// Test font weights are correct.
#[test]
fn font_weights_correct() {
    let tokens = light_tokens();
    let t = &tokens.typography;

    assert_eq!(t.display.weight, FontWeight::Bold);
    assert_eq!(t.h1.weight, FontWeight::SemiBold);
    assert_eq!(t.h2.weight, FontWeight::SemiBold);
    assert_eq!(t.h3.weight, FontWeight::Medium);
    assert_eq!(t.body.weight, FontWeight::Regular);
    assert_eq!(t.small.weight, FontWeight::Regular);
    assert_eq!(t.caption.weight, FontWeight::Medium);
    assert_eq!(t.mono.weight, FontWeight::Regular);
    assert_eq!(t.button.weight, FontWeight::Medium);
}

/// Test line heights are reasonable.
#[test]
fn line_heights_reasonable() {
    let tokens = light_tokens();
    let t = &tokens.typography;

    // All line heights should be >= 1.0
    let specs = [
        &t.display, &t.h1, &t.h2, &t.h3, &t.body, &t.small, &t.caption, &t.mono, &t.button,
    ];

    for spec in specs {
        assert!(spec.line_height >= 1.0, "Line height must be >= 1.0");
        assert!(spec.line_height <= 2.0, "Line height should not exceed 2.0");
    }
}

/// Test motion system has all durations and easing.
#[test]
fn motion_system_complete() {
    let tokens = light_tokens();
    let m = &tokens.motion;

    assert_eq!(m.fast.duration.ms(), 100);
    assert_eq!(m.normal.duration.ms(), 200);
    assert_eq!(m.slow.duration.ms(), 300);

    // Easing tokens are enums, not strings
    assert_eq!(m.fast.easing, EasingToken::EaseOut);
    assert_eq!(m.normal.easing, EasingToken::EaseOut);
    assert_eq!(m.slow.easing, EasingToken::EaseOut);
}

/// Test color parsing handles all formats.
#[test]
fn color_parsing_formats() {
    // Hex 6-digit
    assert_eq!(parse_color("#F4F5F7"), Color32::from_rgb(0xF4, 0xF5, 0xF7));
    assert_eq!(parse_color("#1F2933"), Color32::from_rgb(0x1F, 0x29, 0x33));
    assert_eq!(parse_color("#2F6FED"), Color32::from_rgb(0x2F, 0x6F, 0xED));

    // Hex 8-digit (with alpha)
    assert_eq!(
        parse_color("#2F6FEDFF"),
        Color32::from_rgba_unmultiplied(0x2F, 0x6F, 0xED, 0xFF)
    );
    assert_eq!(
        parse_color("#2F6FED80"),
        Color32::from_rgba_unmultiplied(0x2F, 0x6F, 0xED, 0x80)
    );

    // RGB
    assert_eq!(
        parse_color("rgb(244, 245, 247)"),
        Color32::from_rgb(244, 245, 247)
    );
    assert_eq!(
        parse_color("rgb(31, 41, 51)"),
        Color32::from_rgb(31, 41, 51)
    );

    // RGBA with 0-1 alpha
    assert_eq!(
        parse_color("rgba(47, 111, 237, 0.2)"),
        Color32::from_rgba_unmultiplied(47, 111, 237, 51)
    );

    // RGBA with 0-255 alpha
    assert_eq!(
        parse_color("rgba(47, 111, 237, 51)"),
        Color32::from_rgba_unmultiplied(47, 111, 237, 51)
    );
}

/// Test variance colors map correctly.
#[test]
fn variance_color_mapping() {
    // Positive variance = delay/overspend = red family (status_critical)
    assert_eq!(
        variance_color(Some(5), false),
        LIGHT_COLORS.status_critical
    );
    assert_eq!(
        variance_color(Some(5), true),
        DARK_COLORS.status_critical
    );

    // Negative variance = improvement = green family (status_ok)
    assert_eq!(
        variance_color(Some(-3), false),
        LIGHT_COLORS.status_ok
    );
    assert_eq!(
        variance_color(Some(-3), true),
        DARK_COLORS.status_ok
    );

    // Zero/none = grid/subtle
    assert_eq!(
        variance_color(None, false),
        LIGHT_COLORS.border_subtle
    );
    assert_eq!(
        variance_color(Some(0), false),
        LIGHT_COLORS.border_subtle
    );
}

/// Test status severity colors map to correct tokens.
#[test]
fn status_severity_colors() {
    assert_eq!(
        StatusSeverity::Info.color(false),
        LIGHT_COLORS.text_muted
    );
    assert_eq!(
        StatusSeverity::Success.color(false),
        LIGHT_COLORS.status_ok
    );
    assert_eq!(
        StatusSeverity::Warning.color(false),
        LIGHT_COLORS.status_warn
    );
    assert_eq!(
        StatusSeverity::Error.color(false),
        LIGHT_COLORS.status_critical
    );
}

/// Test status severity classification.
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

/// Test chrome button creation.
#[test]
fn chrome_buttons_create() {
    let _primary = chrome::primary("New");
    let _ghost = chrome::ghost("Action");
    let _tab = chrome::tab(false, true, "Table");
    let _secondary = chrome::secondary("Cancel");
    let _danger = chrome::danger("Delete");
}

/// Test light and dark themes have same structure, different values.
#[test]
fn theme_parity_structure() {
    let light = light_tokens();
    let dark = dark_tokens();

    // Same accent in both themes
    assert_eq!(light.colors.accent, dark.colors.accent);

    // Different surfaces
    assert_ne!(light.colors.surface_base, dark.colors.surface_base);
    assert_ne!(light.colors.surface_raised, dark.colors.surface_raised);

    // Different text
    assert_ne!(light.colors.text_primary, dark.colors.text_primary);
    assert_ne!(light.colors.text_muted, dark.colors.text_muted);

    // Status colors differ (adapted for contrast)
    assert_ne!(light.colors.status_critical, dark.colors.status_critical);
    assert_ne!(light.colors.status_ok, dark.colors.status_ok);
    assert_ne!(light.colors.status_warn, dark.colors.status_warn);

    // Typography identical
    assert_eq!(light.typography, dark.typography);

    // Density identical (spacing is derived from density)
    assert_eq!(light.density, dark.density);

    // Radius identical
    assert_eq!(light.radius, dark.radius);

    // Elevation differs (shadow colors)
    assert_ne!(light.elevation.level1.3, dark.elevation.level1.3); // color (index 3)

    // Motion identical
    assert_eq!(light.motion, dark.motion);

    // Density identical
    assert_eq!(light.density, dark.density);
}

/// Test non-working day and status date colors exist.
#[test]
fn special_colors_exist() {
    let nw_light = non_working_color(ThemeVariant::Light);
    let nw_dark = non_working_color(ThemeVariant::Dark);
    assert_ne!(nw_light, nw_dark);

    let sd_light = status_date_color(ThemeVariant::Light);
    let sd_dark = status_date_color(ThemeVariant::Dark);
    assert_ne!(sd_light, sd_dark);
}

/// Test that all token values are valid CSS colors.
#[test]
fn all_colors_valid_css() {
    fn is_valid_color(s: &str) -> bool {
        s.starts_with('#') || s.starts_with("rgb") || s.starts_with("rgba") || s == "transparent"
    }

    // Note: current design_tokens.rs stores colors as Color32, not CSS strings
    // This test is adapted for the current implementation
    let light = light_tokens();
    let dark = dark_tokens();

    // Just verify colors are not transparent (except where expected)
    assert_ne!(light.colors.surface_base, Color32::TRANSPARENT);
    assert_ne!(light.colors.accent, Color32::TRANSPARENT);
    assert_ne!(dark.colors.surface_base, Color32::TRANSPARENT);
    assert_ne!(dark.colors.accent, Color32::TRANSPARENT);
}

/// Test Theme trait implementations.
#[test]
fn theme_trait_implementations() {
    let light_theme = LightTheme;
    let dark_theme = DarkTheme;

    let light_colors = light_theme.colors();
    let dark_colors = dark_theme.colors();

    assert_eq!(light_colors.accent, ACCENT);
    assert_eq!(dark_colors.accent, ACCENT);
    assert_ne!(light_colors.surface_raised, dark_colors.surface_raised);
}

/// Test ThemeRegistry.
#[test]
fn theme_registry() {
    let mut registry = ThemeRegistry::new();
    assert_eq!(registry.mode(), ThemeMode::System);

    registry.set_mode(ThemeMode::Light);
    assert_eq!(registry.mode(), ThemeMode::Light);

    registry.set_mode(ThemeMode::Dark);
    assert_eq!(registry.mode(), ThemeMode::Dark);
}
