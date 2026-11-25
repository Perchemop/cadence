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
    DesignTokens, DensityLevel, FontWeight, RadiusSystem, SpacingSystem, ThemeTokens, ThemeVariant,
};
use cadence_ui_egui::theme::{parse_color, StatusSeverity, variance_color};

/// Test that all required colour roles exist and are distinct per theme.
#[test]
fn color_roles_complete_and_distinct() {
    for variant in [ThemeVariant::Light, ThemeVariant::Dark] {
        let tokens = DesignTokens::create(variant);
        let c = &tokens.color;

        // Surface roles
        assert_ne!(c.surface.base, c.surface.raised);
        assert_ne!(c.surface.raised, c.surface.sunken);
        assert_ne!(c.surface.sunken, c.surface.hover);
        assert_ne!(c.surface.hover, c.surface.active);
        assert_ne!(c.surface.active, c.surface.disabled);

        // Text roles
        assert_ne!(c.text.primary, c.text.secondary);
        assert_ne!(c.text.secondary, c.text.muted);
        assert_ne!(c.text.muted, c.text.disabled);

        // Accent roles
        assert_ne!(c.accent.primary, c.accent.hover);
        assert_ne!(c.accent.hover, c.accent.active);
        assert_ne!(c.accent.active, c.accent.subtle);
        assert_ne!(c.accent.subtle, c.accent.primary_bg);

        // Border roles
        assert_ne!(c.border.subtle, c.border.strong);
        assert_ne!(c.border.strong, c.border.accent);
        assert_ne!(c.border.accent, c.border.critical);
        assert_ne!(c.border.critical, c.border.warning);

        // Focus roles
        assert!(!c.focus.ring.is_empty());
        assert!(!c.focus.offset.is_empty());

        // Status roles - all distinct
        let status_colors = [
            &c.status.critical,
            &c.status.normal,
            &c.status.success,
            &c.status.warning,
            &c.status.variance_positive,
            &c.status.variance_negative,
        ];
        for i in 0..status_colors.len() {
            for j in i + 1..status_colors.len() {
                assert_ne!(status_colors[i], status_colors[j], "Status colors must be distinct");
            }
        }

        // Overlay roles
        assert!(!c.overlay.scrim.is_empty());
        assert!(!c.overlay.popover.is_empty());
        assert!(!c.overlay.tooltip.is_empty());
    }
}

/// Test WCAG AA contrast for critical colour pairs.
///
/// These are the minimum contrast requirements per guide II.30.7:
/// - Body text on surface: ≥ 4.5:1
/// - Large text on surface: ≥ 3:1
/// - UI boundaries (borders, focus rings) on surface: ≥ 3:1
/// - Status colours on their surface: ≥ 3:1
/// - Accent with white text: ≥ 4.5:1
#[test]
fn contrast_wcag_aa() {
    for variant in [ThemeVariant::Light, ThemeVariant::Dark] {
        let tokens = DesignTokens::create(variant);
        let c = &tokens.color;

        // Helper: compute relative luminance
        fn luminance(color: Color32) -> f32 {
            let srgb_to_linear = |c: f32| {
                if c <= 0.04045 {
                    c / 12.92
                } else {
                    ((c + 0.055) / 1.055).powf(2.4)
                }
            };
            let r = srgb_to_linear(color.r() as f32 / 255.0);
            let g = srgb_to_linear(color.g() as f32 / 255.0);
            let b = srgb_to_linear(color.b() as f32 / 255.0);
            0.2126 * r + 0.7152 * g + 0.0722 * b
        }

        fn contrast_ratio(fg: Color32, bg: Color32) -> f32 {
            let l1 = luminance(fg);
            let l2 = luminance(bg);
            let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
            (lighter + 0.05) / (darker + 0.05)
        }

        let bg_raised = parse_color(&c.surface.raised);
        let bg_base = parse_color(&c.surface.base);
        let bg_sunken = parse_color(&c.surface.sunken);

        // Body text on raised surface ≥ 4.5:1
        let text_primary = parse_color(&c.text.primary);
        let cr = contrast_ratio(text_primary, bg_raised);
        assert!(
            cr >= 4.5,
            "Body text on panel: contrast {:.2}:1 < 4.5:1 ({variant:?})",
            cr
        );

        // Muted text on raised surface ≥ 3:1 (large text threshold)
        let text_muted = parse_color(&c.text.muted);
        let cr = contrast_ratio(text_muted, bg_raised);
        assert!(
            cr >= 3.0,
            "Muted text on panel: contrast {:.2}:1 < 3:1 ({variant:?})",
            cr
        );

        // Accent with white text ≥ 4.5:1
        let accent = parse_color(&c.accent.primary);
        let on_accent = parse_color(&c.text.on_accent);
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
        let focus = parse_color(&c.focus.ring);
        let cr = contrast_ratio(focus, bg_raised);
        assert!(
            cr >= 3.0,
            "Focus ring on panel: contrast {:.2}:1 < 3:1 ({variant:?})",
            cr
        );

        // Critical on raised surface ≥ 3:1
        let critical = parse_color(&c.status.critical);
        let cr = contrast_ratio(critical, bg_raised);
        assert!(
            cr >= 3.0,
            "Critical on panel: contrast {:.2}:1 < 3:1 ({variant:?})",
            cr
        );

        // Success on raised surface ≥ 3:1
        let success = parse_color(&c.status.success);
        let cr = contrast_ratio(success, bg_raised);
        assert!(
            cr >= 3.0,
            "Success on panel: contrast {:.2}:1 < 3:1 ({variant:?})",
            cr
        );

        // Warning on raised surface ≥ 3:1
        let warning = parse_color(&c.status.warning);
        let cr = contrast_ratio(warning, bg_raised);
        assert!(
            cr >= 3.0,
            "Warning on panel: contrast {:.2}:1 < 3:1 ({variant:?})",
            cr
        );

        // Border subtle on raised surface ≥ 3:1
        let border = parse_color(&c.border.subtle);
        let cr = contrast_ratio(border, bg_raised);
        assert!(
            cr >= 3.0,
            "Border on panel: contrast {:.2}:1 < 3:1 ({variant:?})",
            cr
        );
    }
}

use egui::Color32;

/// Test serialization round-trip for all token types.
#[test]
fn serialization_roundtrip() {
    let light = DesignTokens::light();
    let dark = DesignTokens::dark();

    // Serialize to JSON
    let light_json = serde_json::to_string_pretty(&light).unwrap();
    let dark_json = serde_json::to_string_pretty(&dark).unwrap();

    // Deserialize back
    let light_parsed: DesignTokens = serde_json::from_str(&light_json).unwrap();
    let dark_parsed: DesignTokens = serde_json::from_str(&dark_json).unwrap();

    // Verify equality
    assert_eq!(light, light_parsed);
    assert_eq!(dark, dark_parsed);
}

/// Test ThemeTokens serialization.
#[test]
fn theme_tokens_serialization() {
    let themes = ThemeTokens::default();
    let json = serde_json::to_string_pretty(&themes).unwrap();
    let parsed: ThemeTokens = serde_json::from_str(&json).unwrap();
    assert_eq!(themes, parsed);
}

/// Test spacing tokens match 4px base with correct multipliers.
#[test]
fn spacing_tokens_correct_multipliers() {
    let tokens = DesignTokens::light();
    let base = tokens.spacing.base;
    assert_eq!(base, 4.0);

    let expected = [
        ("xs", 0.5),
        ("sm", 1.0),
        ("md", 2.0),
        ("lg", 3.0),
        ("xl", 4.0),
        ("2xl", 6.0),
        ("3xl", 8.0),
        ("4xl", 12.0),
        ("5xl", 16.0),
    ];

    for (name, mult) in expected {
        let value = tokens.spacing(name).unwrap();
        assert_eq!(value, base * mult, "Token {name} should be {mult}× base");
    }
}

/// Test density scaling works correctly.
#[test]
fn density_scaling_all_levels() {
    let mut tokens = DesignTokens::light();

    // Compact: 0.875×
    tokens.density.level = DensityLevel::Compact;
    assert!((tokens.density_multiplier() - 0.875).abs() < 0.001);
    assert!((tokens.scaled_spacing("md").unwrap() - 7.0).abs() < 0.01); // 8 * 0.875

    // Standard: 1.0×
    tokens.density.level = DensityLevel::Standard;
    assert!((tokens.density_multiplier() - 1.0).abs() < 0.001);
    assert!((tokens.scaled_spacing("md").unwrap() - 8.0).abs() < 0.01);

    // Comfortable: 1.125×
    tokens.density.level = DensityLevel::Comfortable;
    assert!((tokens.density_multiplier() - 1.125).abs() < 0.001);
    assert!((tokens.scaled_spacing("md").unwrap() - 9.0).abs() < 0.01); // 8 * 1.125
}

/// Test component spacing scales with density.
#[test]
fn component_spacing_scales_with_density() {
    let mut tokens = DesignTokens::light();

    tokens.density.level = DensityLevel::Compact;
    let (x, y) = tokens.scaled_item_spacing();
    assert!((x - 7.0).abs() < 0.01); // 8 * 0.875
    assert!((y - 3.5).abs() < 0.01); // 4 * 0.875

    let (bx, by) = tokens.scaled_button_padding();
    assert!((bx - 7.0).abs() < 0.01); // 8 * 0.875
    assert!((by - 3.5).abs() < 0.01); // 4 * 0.875

    assert!((tokens.scaled_panel_margin() - 10.5).abs() < 0.01); // 12 * 0.875
}

/// Test radius system has all required values.
#[test]
fn radius_system_complete() {
    let tokens = DesignTokens::light();
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
    let tokens = DesignTokens::light();
    let e = &tokens.elevation;

    // Level 0: no shadow
    assert_eq!(e.level0.blur, 0.0);
    assert_eq!(e.level0.offset_y, 0.0);

    // Level 1: subtle
    assert!(e.level1.blur > e.level0.blur);
    assert!(e.level1.offset_y > e.level0.offset_y);

    // Level 2: medium
    assert!(e.level2.blur > e.level1.blur);
    assert!(e.level2.offset_y > e.level1.offset_y);

    // Level 3: high
    assert!(e.level3.blur > e.level2.blur);
    assert!(e.level3.offset_y > e.level2.offset_y);
}

/// Test typography scale follows ~1.125 ratio.
#[test]
fn typography_scale_ratio() {
    let tokens = DesignTokens::light();
    let t = &tokens.typography;

    // Body is base (11px)
    assert_eq!(t.body.size, 11.0);

    // h3 ~ 1.27× body (14px)
    assert!((t.h3.size / t.body.size - 1.27).abs() < 0.05);

    // h2 ~ 1.64× body (18px)
    assert!((t.h2.size / t.body.size - 1.64).abs() < 0.05);

    // h1 ~ 2.0× body (22px)
    assert!((t.h1.size / t.body.size - 2.0).abs() < 0.05);

    // display ~ 2.55× body (28px)
    assert!((t.display.size / t.body.size - 2.55).abs() < 0.05);

    // Small is smaller than body
    assert!(t.small.size < t.body.size);

    // Caption is smallest
    assert!(t.caption.size < t.small.size);

    // Mono is body size
    assert_eq!(t.mono.size, t.body.size);

    // Button is body size
    assert_eq!(t.button.size, t.body.size);
}

/// Test font weights are correct.
#[test]
fn font_weights_correct() {
    let tokens = DesignTokens::light();
    let t = &tokens.typography;

    assert_eq!(t.display.weight, FontWeight::Bold);
    assert_eq!(t.h1.weight, FontWeight::Semibold);
    assert_eq!(t.h2.weight, FontWeight::Semibold);
    assert_eq!(t.h3.weight, FontWeight::Medium);
    assert_eq!(t.body.weight, FontWeight::Normal);
    assert_eq!(t.small.weight, FontWeight::Normal);
    assert_eq!(t.caption.weight, FontWeight::Medium);
    assert_eq!(t.mono.weight, FontWeight::Normal);
    assert_eq!(t.button.weight, FontWeight::Medium);
}

/// Test line heights are reasonable.
#[test]
fn line_heights_reasonable() {
    let tokens = DesignTokens::light();
    let t = &tokens.typography;

    // All line heights should be >= 1.0
    let specs = [
        &t.display,
        &t.h1,
        &t.h2,
        &t.h3,
        &t.body,
        &t.small,
        &t.caption,
        &t.mono,
        &t.button,
    ];

    for spec in specs {
        assert!(spec.line_height >= 1.0, "Line height must be >= 1.0");
        assert!(spec.line_height <= 2.0, "Line height should not exceed 2.0");
    }
}

/// Test motion system has all durations and easing.
#[test]
fn motion_system_complete() {
    let tokens = DesignTokens::light();
    let m = &tokens.motion;

    assert_eq!(m.duration.instant, 0);
    assert_eq!(m.duration.fast, 100);
    assert_eq!(m.duration.normal, 150);
    assert_eq!(m.duration.slow, 250);

    assert!(!m.easing.ease_out.is_empty());
    assert!(!m.easing.ease_in_out.is_empty());
    assert!(!m.easing.spring.is_empty());

    // Reduced motion overrides
    assert_eq!(m.reduced.duration_overrides.get("instant"), Some(&0));
    assert_eq!(m.reduced.duration_overrides.get("fast"), Some(&0));
    assert_eq!(m.reduced.duration_overrides.get("normal"), Some(&50));
    assert_eq!(m.reduced.duration_overrides.get("slow"), Some(&50));
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
    assert_eq!(parse_color("rgb(244, 245, 247)"), Color32::from_rgb(244, 245, 247));
    assert_eq!(parse_color("rgb(31, 41, 51)"), Color32::from_rgb(31, 41, 51));

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
    let light = DesignTokens::light();
    let dark = DesignTokens::dark();

    // Positive = delay/overspend = red family
    assert_eq!(
        variance_color(Some(5), &light),
        parse_color(&light.color.status.variance_positive)
    );
    assert_eq!(
        variance_color(Some(5), &dark),
        parse_color(&dark.color.status.variance_positive)
    );

    // Negative = improvement = green family
    assert_eq!(
        variance_color(Some(-3), &light),
        parse_color(&light.color.status.variance_negative)
    );
    assert_eq!(
        variance_color(Some(-3), &dark),
        parse_color(&dark.color.status.variance_negative)
    );

    // Zero/none = grid/subtle
    assert_eq!(
        variance_color(None, &light),
        parse_color(&light.color.border.subtle)
    );
    assert_eq!(
        variance_color(Some(0), &light),
        parse_color(&light.color.border.subtle)
    );
}

/// Test status severity colors map to correct tokens.
#[test]
fn status_severity_colors() {
    let light = DesignTokens::light();

    assert_eq!(
        StatusSeverity::Info.color(&light),
        parse_color(&light.color.text.muted)
    );
    assert_eq!(
        StatusSeverity::Success.color(&light),
        parse_color(&light.color.status.success)
    );
    assert_eq!(
        StatusSeverity::Warning.color(&light),
        parse_color(&light.color.status.warning)
    );
    assert_eq!(
        StatusSeverity::Error.color(&light),
        parse_color(&light.color.status.critical)
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
    use cadence_ui_egui::theme::chrome;
    let tokens = DesignTokens::light();

    let _primary = chrome::primary(&tokens, "New");
    let _ghost = chrome::ghost(&tokens, "Action");
    let _tab_selected = chrome::tab(&tokens, true, "Table");
    let _tab_unselected = chrome::tab(&tokens, false, "Gantt");
    let _secondary = chrome::secondary(&tokens, "Cancel");
    let _danger = chrome::danger(&tokens, "Delete");
}

/// Test light and dark themes have same structure, different values.
#[test]
fn theme_parity_structure() {
    let light = DesignTokens::light();
    let dark = DesignTokens::dark();

    // Same accent in both themes
    assert_eq!(light.color.accent.primary, dark.color.accent.primary);

    // Different surfaces
    assert_ne!(light.color.surface.base, dark.color.surface.base);
    assert_ne!(light.color.surface.raised, dark.color.surface.raised);

    // Different text
    assert_ne!(light.color.text.primary, dark.color.text.primary);
    assert_ne!(light.color.text.muted, dark.color.text.muted);

    // Status colors differ (adapted for contrast)
    assert_ne!(light.color.status.critical, dark.color.status.critical);
    assert_ne!(light.color.status.success, dark.color.status.success);
    assert_ne!(light.color.status.warning, dark.color.status.warning);

    // But variance colors differ per theme
    assert_ne!(
        light.color.status.variance_positive,
        dark.color.status.variance_positive
    );
    assert_ne!(
        light.color.status.variance_negative,
        dark.color.status.variance_negative
    );

    // Typography identical
    assert_eq!(light.typography, dark.typography);

    // Spacing identical
    assert_eq!(light.spacing, dark.spacing);

    // Radius identical
    assert_eq!(light.radius, dark.radius);

    // Elevation differs (shadow colors)
    assert_ne!(light.elevation.level1.color, dark.elevation.level1.color);

    // Motion identical
    assert_eq!(light.motion, dark.motion);

    // Density identical
    assert_eq!(light.density, dark.density);
}

/// Test non-working day and status date colors exist.
#[test]
fn special_colors_exist() {
    use cadence_ui_egui::theme::{non_working_color, status_date_color};

    let light = DesignTokens::light();
    let dark = DesignTokens::dark();

    let nw_light = non_working_color(&light);
    let nw_dark = non_working_color(&dark);
    assert_ne!(nw_light, nw_dark);

    let sd_light = status_date_color(&light);
    let sd_dark = status_date_color(&dark);
    assert_ne!(sd_light, sd_dark);
}

/// Test that all token values are valid CSS colors.
#[test]
fn all_colors_valid_css() {
    fn is_valid_color(s: &str) -> bool {
        s.starts_with('#')
            || s.starts_with("rgb")
            || s.starts_with("rgba")
            || s == "transparent"
    }

    for variant in [ThemeVariant::Light, ThemeVariant::Dark] {
        let tokens = DesignTokens::create(variant);
        let c = &tokens.color;

        let all_colors = vec![
            &c.surface.base,
            &c.surface.raised,
            &c.surface.sunken,
            &c.surface.hover,
            &c.surface.active,
            &c.surface.disabled,
            &c.text.primary,
            &c.text.secondary,
            &c.text.muted,
            &c.text.disabled,
            &c.text.on_accent,
            &c.text.on_critical,
            &c.text.on_warning,
            &c.text.on_success,
            &c.accent.primary,
            &c.accent.hover,
            &c.accent.active,
            &c.accent.subtle,
            &c.accent.primary_bg,
            &c.border.subtle,
            &c.border.strong,
            &c.border.accent,
            &c.border.critical,
            &c.border.warning,
            &c.focus.ring,
            &c.focus.offset,
            &c.status.critical,
            &c.status.normal,
            &c.status.success,
            &c.status.warning,
            &c.status.variance_positive,
            &c.status.variance_negative,
            &c.status.non_working,
            &c.status.status_date,
            &c.overlay.scrim,
            &c.overlay.popover,
            &c.overlay.tooltip,
        ];

        for color in all_colors {
            assert!(
                is_valid_color(color),
                "Invalid color format: {color} ({variant:?})"
            );
        }
    }
}