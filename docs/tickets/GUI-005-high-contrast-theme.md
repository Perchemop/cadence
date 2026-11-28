# Ticket: GUI-005 — High Contrast Theme

## Objective
Implement a high contrast theme for Cadence, matching the pattern from MS Project, Linear, and MS Project Web. This is a **medium priority** improvement identified in the GUI Inspiration Audit.

## Target Files
- `crates/design/src.rs` — Extend `ThemeVariant` with `HighContrastLight`/`HighContrastDark`
- `crates/ui-egui/src/lib.rs` — High contrast toggle in Appearance settings
- `crates/tokens/src.rs` — Contrast-verified color palette

## ThemeVariant Extension (from research)
Extend the existing theme system with high contrast variants:

```rust
/// Available theme variants for the application
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThemeVariant {
    Light,             // default light theme
    Dark,              // default dark theme
    HighContrastLight, // high contrast light mode
    HighContrastDark,  // high contrast dark mode
}
```

## Contrast Verification (from II.30.6 and research)
WCAG AA verified for all semantic color pairs in high contrast mode:
- `surface.base` against `text.primary`: ≥ 4.5:1
- `text.primary` against `accent`: ≥ 3:1 (with `accent.on`)
- `focus.ring` against control and surface: ≥ 3:1
- `border.strong` against adjacent surface: ≥ 3:1

## Color Palette Adjustments
High contrast mode replaces semantic colors with high-visibility alternatives:
- `surface.base` → off-white or light gray (not pure white)
- `text.primary` → pure black (not dark gray)
- `accent` → bright, distinguishable color
- `accent.on` → white or light background on accent
- `danger` → high-contrast red
- `warning` → high-contrast amber/yellow
- `success` → high-contrast green
- `border.subtle` → visible border color
- `border.strong` → strong visible border color

## System Preference Detection (from research)
- Detect system-level high contrast preference
- Provide manual override toggle in settings
- Default: follow system preference
- User can force light/dark/high contrast regardless of system

## Blocking Dependencies
- None (foundational theming improvement)

## Acceptance Criteria
- [ ] High contrast theme available in Appearance settings
- [ ] `ThemeVariant` includes: Light, Dark, HighContrastLight, HighContrastDark
- [ ] WCAG AA verified for all semantic color pairs in high contrast mode
- [ ] System preference detection works
- [ ] Manual override toggle in settings
- [ ] All interactive elements have sufficient contrast
- [ ] Focus ring clearly visible in high contrast mode
- [ ] Error and success messages have sufficient contrast
- [ ] Light and dark themes unaffected by high contrast toggle

## Verification
- Headless test: Switch to high contrast → all color pairs meet WCAG AA thresholds
- UI test: High contrast mode renders, all elements have sufficient contrast, focus visible
- Integration: High contrast works with skeleton loaders, density mode, inline editing
- Edge case: Color blindness considerations → additional differentiation beyond contrast
- Edge case: System doesn't support high contrast → graceful fallback