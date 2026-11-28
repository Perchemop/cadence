# Ticket: GUI-004 — Density / Compact Mode

## Objective
Implement density control (Comfortable, Compact, Condensed) for the Cadence UI, matching the pattern from Linear, Notion, Airtable, and MS Project Web. This is a **medium priority** improvement identified in the GUI Inspiration Audit.

## Target Files
- `crates/design/src.rs` — Add `Density` struct to `Tokens`, spacing scale adjustments
- `crates/ui-egui/src/lib.rs` — Density preference in `CadenceApp`, affect row height, padding, font size
- `crates/settings/src/lib.rs` — Persisted density preference

## Density System (from gui-inspiration-audit.md)
- Three densities: Comfortable (default), Compact, Condensed
- Affects: row height, padding, font size, spacing
- Persisted in user preferences

## Density Definition (from research)
```rust
/// UI density setting for the application
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiDensity {
    Comfortable,   // default, more padding, spacious
    Compact,       // reduced padding, tighter layout
    Condensed,     // minimal padding, maximum information density
}
```

## Spacing Scale Adjustments (from design system gaps)
Current: 4px base
Need: 4-8-12-16-24-32-48 scale with named tokens

Density affects the spacing token used:
- **Comfortable**: Uses spacing tokens at full scale (e.g., 16px row height, 8px padding)
- **Compact**: Uses spacing tokens scaled down (e.g., 12px row height, 6px padding)  
- **Condensed**: Uses minimal spacing tokens (e.g., 8px row height, 4px padding)

## Font Size Adjustments
- Comfortable: 14px default text size
- Compact: 12px default text size
- Condensed: 11px default text size

## Implementation in Tokens (from design system gaps)
Extend the existing `Tokens` struct with density-specific spacing:

```rust
/// UI density setting affecting spacing and typography
pub struct UiDensity {
    /// The active density setting
    pub mode: UiDensity,
    
    /// Row height multiplier per density
    pub row_height: f32,
    
    /// Base padding multiplier per density
    pub padding: f32,
    
    /// Base font size multiplier per density
    pub font_size: f32,
}
```

## Blocking Dependencies
- None (foundational UI preference)

## Acceptance Criteria
- [ ] Three density options available in Settings → Appearance
- [ ] Comfortable (default) → spacious layout with normal padding
- [ ] Compact → reduced padding, tighter layout
- [ ] Condensed → minimal padding, maximum information density
- [ ] Density persists across app restarts
- [ ] Affects: row height, padding, font size, spacing between elements
- [ ] Light and dark themes both support all density options
- [ ] Density preference remembered and applied automatically

## Verification
- Headless test: Change density → UI elements resize appropriately → restart app → density persisted
- UI test: Density selector works, all density modes render correctly, preference persisted
- Integration: Density works with existing skeleton loaders and table editing
- Edge case: Extreme density → UI remains usable, no clipping or unreadable text
- Edge case: Switch density during operation → layout reflows smoothly