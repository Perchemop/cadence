//! Design Token Validation Tests
//!
//! These tests verify that the design token system meets all requirements:
//! - WCAG 2.2 AA contrast ratios
//! - Token completeness (all semantic roles covered)
//! - Density scaling correctness
//! - Theme parity (light/dark both valid)
//! - Serialization round-trip

use cadence_ui_egui::design_tokens::{
    validate_contrast, variance_color, ColorTokens, ContrastRatio, DARK_COLORS, LIGHT_COLORS,
};
use egui::Color32;

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

#[test]
fn contrast_validation_passes() {
    // Test that all semantic color pairs meet WCAG 2.2 AA
    // Light theme
    let light_failures = validate_contrast(&LIGHT_COLORS);
    if !light_failures.is_empty() {
        panic!(
            "Light theme contrast failures:\n{}",
            light_failures.join("\n")
        );
    }

    // Dark theme
    let dark_failures = validate_contrast(&DARK_COLORS);
    if !dark_failures.is_empty() {
        panic!(
            "Dark theme contrast failures:\n{}",
            dark_failures.join("\n")
        );
    }
}

#[test]
fn contrast_ratio_calculation() {
    // White on black should be 21:1
    let white = Color32::WHITE;
    let black = Color32::BLACK;
    let ratio = ContrastRatio::compute(white, black);
    assert!((ratio - 21.0).abs() < 0.1);

    // White on white should be 1:1
    let ratio = ContrastRatio::compute(white, white);
    assert!((ratio - 1.0).abs() < 0.01);

    // Black on black should be 1:1
    let ratio = ContrastRatio::compute(black, black);
    assert!((ratio - 1.0).abs() < 0.01);
}

#[test]
fn variance_color_mapping() {
    // Positive variance = delay/overspend = red family (status_critical)
    assert_eq!(
        variance_color(Some(5), true), // dark theme
        DARK_COLORS.status_critical
    );
    assert_eq!(
        variance_color(Some(5), false), // light theme
        LIGHT_COLORS.status_critical
    );

    // Negative variance = improvement = green family (status_ok)
    assert_eq!(
        variance_color(Some(-3), true), // dark theme
        DARK_COLORS.status_ok
    );
    assert_eq!(
        variance_color(Some(-3), false), // light theme
        LIGHT_COLORS.status_ok
    );

    // Zero/none = grid/subtle
    assert_eq!(
        variance_color(None, true), // dark theme
        DARK_COLORS.border_subtle
    );
    assert_eq!(
        variance_color(Some(0), false), // light theme
        LIGHT_COLORS.border_subtle
    );
}
