//! Cadence design system — Compatibility Layer
//!
//! This module provides the legacy API for backward compatibility during
//! migration to the comprehensive design token system in `design_tokens.rs`
//! and `theme.rs`.
//!
//! New code should use `design_tokens` and `theme` directly.

// Re-export all public types from design_tokens
pub use crate::design_tokens::{
    apply as tokens_apply, apply_visuals as tokens_apply_visuals, contrast_ratio, dark_tokens,
    init_theme_registry as init_registry, light_tokens, meets_wcag, relative_luminance,
    theme_registry as tokens_theme_registry, validate_contrast, wcag_min_ratio, ColorTokens,
    DarkTheme, Density, DesignTokenSet, ElevationSet, FontWeight, LightTheme, MotionSet,
    Palette as LegacyPalette, RadiusSet, RadiusToken, SpacingToken, Theme as DesignTheme, Theme,
    ThemeMode, ThemeRegistry, TypographyScale, ACCENT, ACCENT_ON, BASE_UNIT, DARK_COLORS,
    FOCUS_RING, LIGHT_COLORS, SEMANTIC_CONTRAST_PAIRS,
};

use crate::theme::{
    apply_theme as new_apply_theme, apply_visuals_only, install_fonts as new_install_fonts,
};
use egui::{Color32, Context, Stroke, Visuals};
use std::sync::{Mutex, OnceLock};

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

/// The four chrome button roles.
pub mod chrome {
    use crate::theme::chrome as new_chrome;
    use crate::theme::ThemeVariant;
    use egui::Button;

    /// Tab button for view switching.
    pub fn tab(dark: bool, active: bool, label: impl Into<String>) -> Button<'static> {
        let variant = if dark {
            ThemeVariant::Dark
        } else {
            ThemeVariant::Light
        };
        new_chrome::tab(variant, active, label)
    }

    /// The one primary action per row (e.g. "New activity"): accent fill,
    /// no stroke, high-contrast label.
    pub fn primary(label: impl Into<String>) -> Button<'static> {
        new_chrome::primary(ThemeVariant::Light, label)
    }

    /// Toolbar/quick actions: invisible chrome until hovered.
    pub fn ghost(label: impl Into<String>) -> Button<'static> {
        new_chrome::ghost(ThemeVariant::Light, label)
    }

    /// Secondary actions in toolbars: subtle outline.
    pub fn secondary(label: impl Into<String>) -> Button<'static> {
        new_chrome::secondary(ThemeVariant::Light, label)
    }

    /// Destructive actions: red fill, high-contrast label.
    pub fn danger(label: impl Into<String>) -> Button<'static> {
        new_chrome::danger(ThemeVariant::Light, label)
    }
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
pub fn apply(ctx: &Context, dark: bool) {
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
    crate::theme::install_fonts(ctx);
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
    theme_registry().lock().unwrap().mode()
}

/// Set the current theme mode in the global registry.
pub fn set_theme_mode(mode: ThemeMode) {
    tokens_theme_registry().lock().unwrap().set_mode(mode)
}

/// Initialize the global theme registry (legacy entry point).
pub fn init_theme_registry() {
    init_registry();
}

/// Get the global theme registry (legacy entry point).
pub fn theme_registry() -> &'static Mutex<ThemeRegistry> {
    tokens_theme_registry()
}

/// Theme colors returned by the Theme trait.
#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub accent: Color32,
    pub surface_raised: Color32,
    pub status_critical: Color32,
    pub status_normal: Color32,
}

/// Light theme implementation (wraps design_tokens::LightTheme).
#[derive(Default)]
pub struct LightThemeImpl {
    inner: LightTheme,
}

impl Theme for LightThemeImpl {
    fn colors(&self) -> &ColorTokens {
        self.inner.colors()
    }

    fn typography(&self) -> &TypographyScale {
        self.inner.typography()
    }

    fn density(&self) -> Density {
        self.inner.density()
    }

    fn radius(&self) -> &RadiusSet {
        self.inner.radius()
    }

    fn elevation(&self) -> &ElevationSet {
        self.inner.elevation()
    }

    fn motion(&self) -> &MotionSet {
        self.inner.motion()
    }

    fn reduced_motion(&self) -> bool {
        self.inner.reduced_motion()
    }

    fn is_dark(&self) -> bool {
        self.inner.is_dark()
    }

    fn apply(&self, ctx: &Context) {
        self.inner.apply(ctx);
    }
}

/// Dark theme implementation (wraps design_tokens::DarkTheme).
#[derive(Default)]
pub struct DarkThemeImpl {
    inner: DarkTheme,
}

impl Theme for DarkThemeImpl {
    fn colors(&self) -> &ColorTokens {
        self.inner.colors()
    }

    fn typography(&self) -> &TypographyScale {
        self.inner.typography()
    }

    fn density(&self) -> Density {
        self.inner.density()
    }

    fn radius(&self) -> &RadiusSet {
        self.inner.radius()
    }

    fn elevation(&self) -> &ElevationSet {
        self.inner.elevation()
    }

    fn motion(&self) -> &MotionSet {
        self.inner.motion()
    }

    fn reduced_motion(&self) -> bool {
        self.inner.reduced_motion()
    }

    fn is_dark(&self) -> bool {
        self.inner.is_dark()
    }

    fn apply(&self, ctx: &Context) {
        self.inner.apply(ctx);
    }
}

/// Theme registry for dynamic theme switching (compatibility wrapper).
pub struct ThemeRegistryCompat {
    light: Box<dyn Theme + Send + Sync>,
    dark: Box<dyn Theme + Send + Sync>,
    current: ThemeMode,
}

impl ThemeRegistryCompat {
    /// Create a new theme registry with default themes.
    pub fn new() -> Self {
        Self {
            light: Box::new(LightThemeImpl::default()),
            dark: Box::new(DarkThemeImpl::default()),
            current: ThemeMode::Dark,
        }
    }

    /// Get the current theme.
    pub fn current(&self) -> &dyn Theme {
        match self.current {
            ThemeMode::Light => &*self.light,
            ThemeMode::Dark => &*self.dark,
            ThemeMode::System => &*self.dark, // Default to dark for system
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

impl Default for ThemeRegistryCompat {
    fn default() -> Self {
        Self::new()
    }
}

/// Global theme registry (compatibility wrapper).
static THEME_REGISTRY_COMPAT: OnceLock<ThemeRegistryCompat> = OnceLock::new();

/// Initialize the global theme registry (legacy entry point).
pub fn init_theme_registry_compat() {
    THEME_REGISTRY_COMPAT.set(ThemeRegistryCompat::new()).ok();
}

/// Get the global theme registry (legacy entry point).
pub fn theme_registry_compat() -> &'static ThemeRegistryCompat {
    THEME_REGISTRY_COMPAT.get_or_init(ThemeRegistryCompat::new)
}

/// Get mutable access to the global theme registry (legacy entry point).
/// Note: Returns immutable reference due to OnceLock; use theme_registry_compat() instead.
pub fn theme_registry_mut_compat() -> &'static ThemeRegistryCompat {
    THEME_REGISTRY_COMPAT.get_or_init(ThemeRegistryCompat::new)
}

/// Apply a theme using the new Theme trait (legacy entry point).
pub fn apply_theme_with_compat(ctx: &Context, theme: &dyn Theme) {
    theme.apply(ctx);
}

/// Apply a theme to an egui context using the new token system (legacy entry point).
pub fn apply_theme(ctx: &Context, dark: bool) {
    new_apply_theme(ctx, dark);
}
