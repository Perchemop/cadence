//! WCAG 2.2 AA contrast validation tests
//!
//! These tests verify that all semantic color role pairs meet WCAG 2.2 AA
//! contrast requirements on both Light and Dark themes. Tests are headless —
//! no display or egui rendering context needed.

use cadence_ui_egui::design_tokens::{validate_contrast, DARK_COLORS, LIGHT_COLORS};

#[test]
fn semantic_color_pairs_meet_wcag_aa_across_themes() {
    // Test all semantic color pairs against WCAG 2.2 AA
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
