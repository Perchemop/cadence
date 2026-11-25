//! Cadence design system — Compatibility Layer
//!
//! This module provides the legacy API for backward compatibility during
//! migration to the comprehensive design token system in `design_tokens.rs`
//! and `theme.rs`.
//!
//! New code should use `design_tokens` and `theme` directly.

use crate::design_tokens::{
    ColorTokens, DesignTokenSet, FontWeight, Palette as LegacyPalette, RadiusSet,
    RadiusToken, SpacingToken, ThemeMode, ThemeRegistry, TypographyScale,
    Theme as DesignTheme, LightTheme, DarkTheme, init_theme_registry as init_registry,
    theme_registry as get_registry, theme_registry_mut as get_registry_mut,
    ACCENT, FOCUS_RING, ACCENT_ON, LIGHT_COLORS, DARK_COLORS, BASE_UNIT,
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
    /// Secondary text: units, hints, dimmed numerics.
    pub text_muted: Color32,
    /// Hairlines, table grid lines, Gantt grid.
    pub grid: Color32,
    /// Critical bars and failed checks.
    pub critical: Color32,
    /// Non-critical bars.
    pub normal: Color32,
    /// Passes and on-track state.
    pub ok: Color32,
    /// Warnings.
    pub warn: Color32,
    /// Extreme background (e.g. the Gantt canvas inside a panel).
    pub deep: Color32,
    /// Non-working day background tint (weekends/holidays on Gantt canvas).
    pub non_working: Color32,
    /// Status date line color (dotted indicator separate from today line).
    pub status_date: Color32,
    /// Visible keyboard-focus ring stroke colour.
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

    /// View tabs. Selected: accent-tinted fill, strong text and an accent
    /// hairline; idle: transparent with muted text.
    pub fn tab(dark: bool, selected: bool, label: impl Into<String>) -> Button<'static> {
        let tokens = if dark { DesignTokenSet::dark() } else { DesignTokenSet::light() };
        new_chrome::tab(&tokens, selected, label)
    }
}

/// Bundle Inter (SIL OFL 1.1) and install the type scale.
pub fn install_fonts(ctx: &Context) {
    new_install_fonts(ctx);
}

/// Apply the full theme: background, panels, widget rounding, spacing and
/// widget states, all from the design tokens.
pub fn apply(ctx: &Context, dark: bool) {
    crate::theme::apply_theme(ctx, dark);
}

/// Apply only the palette visuals — backgrounds, text colours, widget
/// states — without the type scale.
pub fn apply_visuals(ctx: &Context, dark: bool) {
    crate::theme::apply_visuals_only(ctx, dark);
}

/// Theme mode enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

/// Theme trait for dynamic theming.
pub trait Theme: Send + Sync {
    /// Apply this theme to an egui context.
    fn apply(&self, ctx: &Context);

    /// Get the colors for this theme.
    fn colors(&self) -> ThemeColors;
}

/// Colors returned by the Theme trait.
#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub accent: Color32,
    pub surface_raised: Color32,
    pub status_critical: Color32,
    pub status_normal: Color32,
}

/// Light theme implementation.
pub struct LightTheme;

impl Theme for LightTheme {
    fn apply(&self, ctx: &Context) {
        crate::theme::apply_theme(ctx, false);
    }

    fn colors(&self) -> ThemeColors {
        let tokens = DesignTokenSet::light();
        ThemeColors {
            accent: ACCENT,
            surface_raised: tokens.colors.surface_raised,
            status_critical: tokens.colors.status_critical,
            status_normal: tokens.colors.status_normal,
        }
    }
}

/// Dark theme implementation.
pub struct DarkTheme;

impl Theme for DarkTheme {
    fn apply(&self, ctx: &Context) {
        crate::theme::apply_theme(ctx, true);
    }

    fn colors(&self) -> ThemeColors {
        let tokens = DesignTokenSet::dark();
        ThemeColors {
            accent: ACCENT,
            surface_raised: tokens.colors.surface_raised,
            status_critical: tokens.colors.status_critical,
            status_normal: tokens.colors.status_normal,
        }
    }
}

/// Theme registry for dynamic theme switching.
pub struct ThemeRegistry {
    light: Box<dyn Theme>,
    dark: Box<dyn Theme>,
    current: ThemeMode,
}

impl ThemeRegistry {
    /// Create a new theme registry with default themes.
    pub fn new() -> Self {
        Self {
            light: Box::new(LightTheme),
            dark: Box::new(DarkTheme),
            current: ThemeMode::Dark,
        }
    }

    /// Get the current theme.
    pub fn current(&self) -> &dyn Theme {
        match self.current {
            ThemeMode::Light => &*self.light,
            ThemeMode::Dark => &*self.dark,
        }
    }

    /// Set the current theme mode.
    pub fn set_mode(&mut self, mode: ThemeMode) {
        self.current = mode;
    }

    /// Apply the current theme to a context.
    pub fn apply(&self, ctx: &Context) {
        self.current().apply(ctx);
    }
}

/// Global theme registry.
static mut THEME_REGISTRY: Option<ThemeRegistry> = None;

/// Initialize the global theme registry.
pub fn init_theme_registry() {
    unsafe {
        THEME_REGISTRY = Some(ThemeRegistry::new());
    }
}

/// Get the global theme registry.
pub fn theme_registry() -> &'static ThemeRegistry {
    unsafe {
        THEME_REGISTRY
            .as_ref()
            .expect("Theme registry not initialized. Call init_theme_registry() first.")
    }
}

/// Get mutable access to the global theme registry.
pub fn theme_registry_mut() -> &'static mut ThemeRegistry {
    unsafe {
        THEME_REGISTRY
            .as_mut()
            .expect("Theme registry not initialized. Call init_theme_registry() first.")
    }
}