//! Cadence design system — Compatibility Layer
//!
//! This module provides the legacy API for backward compatibility during
//! migration to the comprehensive design token system in `design_tokens.rs`
//! and `theme.rs`.
//!
//! New code should use `design_tokens` and `theme` directly.

// Re-export all public types from design_tokens
pub use crate::design_tokens::{
    ColorTokens, DesignTokenSet, FontWeight, Palette as LegacyPalette, RadiusSet,
    RadiusToken, SpacingToken, ThemeMode, ThemeRegistry, TypographyScale,
    Theme as DesignTheme, LightTheme, DarkTheme, init_theme_registry as init_registry,
    theme_registry as get_registry, theme_registry as get_registry_mut,
    ACCENT, FOCUS_RING, ACCENT_ON, LIGHT_COLORS, DARK_COLORS, BASE_UNIT,
    wcag_min_ratio, contrast_ratio, relative_luminance, meets_wcag,
    SEMANTIC_CONTRAST_PAIRS,
    validate_contrast,
};

use crate::theme::{apply_theme as new_apply_theme, apply_visuals_only, install_fonts as new_install_fonts};
use egui::{Color32, Context, Visuals};

/// Semantic and structural colours for one theme (legacy Palette).
///
/// This is the old Palette struct kept for backward compatibility.
/// New code should use `design_tokens::DesignTokenSet` directly.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    /// Window background behind panels.
    pub bg: Color32,
    /// Panel, grid and popup surface.
    pub panel: Color32,
    /// Body text.
    pub text: Color32,
    /// Muted/disabled text.
    pub text_muted: Color32,
    /// Grid lines and subtle borders.
    pub grid: Color32,
    /// Critical/alert state (red).
    pub critical: Color32,
    /// Normal/baseline state.
    pub normal: Color32,
    /// Success/ok state (green).
    pub ok: Color32,
    /// Warning/caution state (amber).
    pub warn: Color32,
    /// Deep/sunken surface.
    pub deep: Color32,
    /// Non-working day background.
    pub non_working: Color32,
    /// Status date line.
    pub status_date: Color32,
    /// Focus ring for keyboard navigation.
    pub focus_ring: Color32,
}

impl Palette {
    /// Create a legacy Palette from the new token system.
    pub fn from_tokens(tokens: &DesignTokenSet) -> Self {
        let c = &tokens.colors;
        Self {
            bg: c.surface_base,
            panel: c.surface_raised,
            text: c.text_primary,
            text_muted: c.text_muted,
            grid: c.border_subtle,
            critical: c.status_critical,
            normal: c.status_normal,
            ok: c.status_ok,
            warn: c.status_warn,
            deep: c.surface_sunken,
            non_working: c.non_working,
            status_date: c.status_date,
            focus_ring: c.focus_ring,
        }
    }
}

/// The palette for the requested theme (legacy function).
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
        deep: colors.surface_sunken,
        non_working: colors.non_working,
        status_date: colors.status_date,
        focus_ring: colors.focus_ring,
    }
}

/// Translucent variant of a palette colour; the theme module owns construction.
pub fn translucent(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

/// Colour a variance: delays and overspends in red, improvements in green.
pub fn variance_color(value: Option<i64>, dark: bool) -> Color32 {
    let colors = if dark { DARK_COLORS } else { LIGHT_COLORS };
    match value {
        Some(v) if v > 0 => colors.variance_positive,
        Some(v) if v < 0 => colors.variance_negative,
        _ => colors.border_subtle,
    }
}

/// The four chrome button roles.
pub mod chrome {
    use super::*;
    use crate::theme::chrome as new_chrome;
    use crate::design_tokens::DesignTokenSet;
    use crate::theme::parse_color;
    use egui::{Button, RichText, Stroke};

    /// The one primary action per row (e.g. "New activity"): accent fill,
    /// no stroke, high-contrast label.
    pub fn primary(label: impl Into<String>) -> Button<'static> {
        // Default to light theme tokens; caller should use theme-aware version
        let tokens = DesignTokenSet::light();
        new_chrome::primary(&tokens, label)
    }

    /// Toolbar/quick actions: invisible chrome until hovered.
    pub fn ghost(label: impl Into<String>) -> Button<'static> {
        let tokens = DesignTokenSet::light();
        new_chrome::ghost(&tokens, label)
    }

    /// Secondary actions in toolbars: subtle outline.
    pub fn secondary(label: impl Into<String>) -> Button<'static> {
        let tokens = DesignTokenSet::light();
        new_chrome::secondary(&tokens, label)
    }

    /// Destructive actions: red fill, high-contrast label.
    pub fn danger(label: impl Into<String>) -> Button<'static> {
        let tokens = DesignTokenSet::light();
        new_chrome::danger(&tokens, label)
    }
}

/// Apply a palette as the global egui theme (legacy entry point).
pub fn apply_palette(ctx: &Context, p: &Palette) {
    let visuals = Visuals {
        window_fill: p.bg,
        panel_fill: p.panel,
        override_text_color: Some(p.text),
        warn_fg_color: p.warn,
        error_fg_color: p.critical,
        hyperlink_color: p.ok,
        selection: egui::style::Selection {
            bg_fill: p.focus_ring,
            stroke: Stroke::NONE,
        },
        ..Visuals::light()
    };
    ctx.set_visuals(visuals);
}

/// Apply the light theme to the context (legacy entry point).
pub fn apply_light_theme(ctx: &Context) {
    new_apply_theme(ctx, false);
}

/// Apply the dark theme to the context (legacy entry point).
pub fn apply_dark_theme(ctx: &Context) {
    new_apply_theme(ctx, true);
}

/// Apply the theme for the given mode (legacy entry point).
pub fn apply_theme(ctx: &Context, dark: bool) {
    new_apply_theme(ctx, dark);
}

/// Install the default fonts for the theme (legacy entry point).
pub fn install_fonts(ctx: &Context) {
    new_install_fonts(ctx);
}

/// Apply only visuals without changing fonts (legacy entry point).
pub fn apply_visuals(ctx: &Context, dark: bool) {
    apply_visuals_only(ctx, dark);
}

/// Apply the theme and install fonts (legacy entry point).
pub fn apply_theme_and_fonts(ctx: &Context, dark: bool) {
    new_apply_theme(ctx, dark);
    new_install_fonts(ctx);
}

/// Apply a theme by name (legacy entry point).
pub fn apply_theme_by_name(ctx: &Context, name: &str) {
    match name.to_lowercase().as_str() {
        "dark" => new_apply_theme(ctx, true),
        _ => new_apply_theme(ctx, false),
    }
}

/// Get the current theme mode from the global registry.
pub fn current_theme_mode() -> ThemeMode {
    get_registry().current()
}

/// Set the current theme mode in the global registry.
pub fn set_theme_mode(mode: ThemeMode) {
    get_registry_mut().set_mode(mode);
}

/// Initialize the global theme registry (legacy entry point).
pub fn init_theme_registry_legacy() {
    init_registry();
}

/// Get the global theme registry (legacy entry point).
pub fn theme_registry_legacy() -> &'static ThemeRegistry {
    get_registry()
}

/// Get mutable access to the global theme registry (legacy entry point).
pub fn theme_registry_mut_legacy() -> &'static mut ThemeRegistry {
    get_registry_mut()
}