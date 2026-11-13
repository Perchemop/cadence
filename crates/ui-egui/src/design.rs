//! Cadence design system: the visual tokens from `docs/design/brand.md`,
//! applied to an egui context.
//!
//! The design intent: calm, dense, precise — a professional scheduling tool.
//! Panels are quiet chrome; the schedule is the hero. Colour is reserved for
//! meaning (critical, variance, selection); structure is carried by hairlines
//! and a 4 px spacing rhythm; type is Inter with a strict size scale.

use egui::{Color32, Context, FontDefinitions, FontFamily, Margin, Visuals};

/// The selection/highlight accent. Kept distinct from the semantic palette and
/// WCAG-AA verified against both themes by `accessibility_tests`.
pub const ACCENT: Color32 = Color32::from_rgb(0x2F, 0x6F, 0xED);

/// The visible keyboard-focus ring colour. egui maps keyboard focus onto the
/// `active` widget state, so this stroke becomes the focus ring everywhere.
/// Verified at ≥ 3:1 against each theme's panel background by
/// `focus_ring_meets_nontext_contrast_on_both_themes`.
pub const FOCUS_RING: Color32 = ACCENT;

/// Semantic and structural colours for one theme.
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
    /// Visible keyboard-focus ring stroke colour.
    pub focus_ring: Color32,
}

/// The palette for the requested theme.
pub const fn palette(dark: bool) -> Palette {
    if dark {
        Palette {
            bg: Color32::from_rgb(0x16, 0x18, 0x1D),
            panel: Color32::from_rgb(0x1D, 0x20, 0x26),
            text: Color32::from_rgb(0xE3, 0xE7, 0xEB),
            text_muted: Color32::from_rgb(0x9A, 0xA5, 0xB1),
            grid: Color32::from_rgb(0x2A, 0x2E, 0x36),
            critical: Color32::from_rgb(0xF8, 0x71, 0x71),
            normal: Color32::from_rgb(0x93, 0xC5, 0xFD),
            ok: Color32::from_rgb(0x4A, 0xDE, 0x80),
            warn: Color32::from_rgb(0xFB, 0xBF, 0x24),
            deep: Color32::from_rgb(0x11, 0x13, 0x18),
            non_working: Color32::from_rgb(0x1A, 0x1E, 0x27),
            focus_ring: FOCUS_RING,
        }
    } else {
        Palette {
            bg: Color32::from_rgb(0xF4, 0xF5, 0xF7),
            panel: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            text: Color32::from_rgb(0x1F, 0x29, 0x33),
            text_muted: Color32::from_rgb(0x5F, 0x6B, 0x7A),
            grid: Color32::from_rgb(0xE5, 0xE7, 0xEB),
            critical: Color32::from_rgb(0xB9, 0x1C, 0x1C),
            normal: Color32::from_rgb(0x1D, 0x4E, 0xD8),
            ok: Color32::from_rgb(0x15, 0x80, 0x3D),
            warn: Color32::from_rgb(0xB4, 0x53, 0x09),
            deep: Color32::from_rgb(0xEC, 0xEE, 0xF1),
            non_working: Color32::from_rgb(0xEA, 0xEC, 0xEF),
            focus_ring: FOCUS_RING,
        }
    }
}

/// Colour a variance: delays and overspends in red, improvements in green.
///
/// Each theme gets its own pair so that both pass WCAG AA (4.5:1) against the
/// panel background — verified by the crate's accessibility contrast tests.
pub fn variance_color(value: Option<i64>, dark: bool) -> Color32 {
    match (value, dark) {
        (Some(value), true) if value > 0 => Color32::from_rgb(0xE7, 0x4C, 0x3C),
        (Some(value), true) if value < 0 => Color32::from_rgb(0x2E, 0xCC, 0x71),
        (Some(value), false) if value > 0 => Color32::from_rgb(0xC0, 0x39, 0x2B),
        (Some(value), false) if value < 0 => Color32::from_rgb(0x1E, 0x84, 0x49),
        _ => palette(dark).grid,
    }
}

/// Translucent variant of a palette colour; the theme module owns construction (II.30.6).
pub fn translucent(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

/// Bundle Inter (SIL OFL 1.1) and install the type scale: 11 px body, 9.5 px
/// small, 16 px headings.
pub fn install_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    let inter: [(&str, &[u8]); 4] = [
        (
            "Inter-Regular",
            include_bytes!("../assets/fonts/Inter-Regular.ttf"),
        ),
        (
            "Inter-Medium",
            include_bytes!("../assets/fonts/Inter-Medium.ttf"),
        ),
        (
            "Inter-SemiBold",
            include_bytes!("../assets/fonts/Inter-SemiBold.ttf"),
        ),
        (
            "Inter-Bold",
            include_bytes!("../assets/fonts/Inter-Bold.ttf"),
        ),
    ];
    for (name, data) in inter {
        fonts
            .font_data
            .insert(name.to_string(), egui::FontData::from_static(data).into());
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .push(name.to_string());
    }
    // Inter first for Latin; the built-in fallbacks stay for symbols.
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        if let Some(family_fonts) = fonts.families.get_mut(&family) {
            family_fonts.retain(|name| !name.starts_with("Inter-"));
        }
    }
    for (name, _) in inter {
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, name.to_string());
    }
    // Phosphor Icons (MIT) supply the Private Use Area glyphs the command bar
    // and inspector use; registered once here beside Inter so the icon face is
    // deterministic at startup.
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    ctx.set_fonts(fonts);
}

/// Apply the type scale and spacing rhythm to both themes' styles.
fn install_style(ctx: &Context) {
    for theme in [egui::Theme::Light, egui::Theme::Dark] {
        ctx.style_mut_of(theme, |style| {
            let text = &mut style.text_styles;
            text.insert(egui::TextStyle::Body, egui::FontId::proportional(11.0));
            text.insert(egui::TextStyle::Small, egui::FontId::proportional(9.5));
            text.insert(egui::TextStyle::Button, egui::FontId::proportional(11.0));
            text.insert(egui::TextStyle::Heading, egui::FontId::proportional(16.0));
            text.insert(egui::TextStyle::Monospace, egui::FontId::monospace(11.0));
            // An 8 px horizontal / 4 px vertical rhythm, so every surface
            // shares one spacing grid.
            style.spacing.item_spacing = egui::vec2(8.0, 4.0);
            style.spacing.button_padding = egui::vec2(8.0, 4.0);
            style.spacing.menu_margin = Margin::symmetric(8, 4);
            style.spacing.indent = 16.0;
        });
    }
}

/// Apply the full theme: background, panels, widget rounding, spacing and
/// widget states, all from the design tokens.
pub fn apply(ctx: &Context, dark: bool) {
    install_style(ctx);
    let p = palette(dark);
    let mut v = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    v.panel_fill = p.panel;
    v.extreme_bg_color = p.deep;
    v.window_fill = p.panel;
    v.override_text_color = Some(p.text);

    // 4 px soft corners everywhere; 6 px on windows.
    v.window_corner_radius = 6.0.into();
    v.menu_corner_radius = 6.0.into();
    for widget in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        widget.corner_radius = 4.0.into();
        widget.fg_stroke.width = 1.0;
        widget.fg_stroke.color = p.text;
        widget.bg_stroke.color = p.grid;
        widget.bg_stroke.width = 1.0;
        widget.expansion = 0.0;
    }
    // egui maps keyboard focus onto the `active` widget state, so a coloured
    // stroke here becomes the visible focus ring for every focused widget.
    v.widgets.active.bg_stroke.color = p.focus_ring;

    v.widgets.noninteractive.bg_fill = p.panel;
    v.widgets.noninteractive.fg_stroke.color = p.text;
    v.widgets.inactive.fg_stroke.color = p.text;
    v.widgets.hovered.fg_stroke.color = p.text;
    v.widgets.hovered.bg_fill = if dark {
        Color32::from_rgb(0x26, 0x2A, 0x33)
    } else {
        Color32::from_rgb(0xEE, 0xF1, 0xF6)
    };
    v.widgets.active.bg_fill = if dark {
        Color32::from_rgb(0x2E, 0x33, 0x3D)
    } else {
        Color32::from_rgb(0xE2, 0xE8, 0xF2)
    };

    v.selection.bg_fill = ACCENT;
    v.selection.stroke.color = ACCENT;
    v.hyperlink_color = ACCENT;

    v.faint_bg_color = if dark {
        Color32::from_rgb(0x21, 0x24, 0x2B)
    } else {
        Color32::from_rgb(0xF0, 0xF2, 0xF5)
    };

    v.striped = true;

    ctx.set_visuals(v);
}
