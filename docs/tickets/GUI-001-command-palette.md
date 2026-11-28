# Ticket: GUI-001 — Command Palette (⌘K)

## Objective
Implement a command palette (⌘K) for Cadence, matching the pattern from Linear, Planera, nPlan, and MS Project Web. This is a **critical priority** improvement identified in the GUI Inspiration Audit.

## Target Files
- `crates/ui-egui/src/command_palette.rs` — New module for command palette
- `crates/ui-egui/src/lib.rs` — Integration of command palette into main UI
- `crates/design/src/lib.rs` — Any design tokens or constants needed

## Command Palette Features (from gui-inspiration-audit.md)
- Single searchable palette for all actions (views, commands, navigation)
- Fuzzy search with recent items at top
- Keyboard shortcuts shown in palette
- Replaces F1 shortcuts dialog with unified palette
- Fuzzy matching against: view names, command names, activity IDs, WBS elements, resource names

## Keyboard Shortcut
- **Ctrl+K** (or ⌘K on macOS) to open the palette
- Esc to close
- Ctrl+Enter or Down/Enter to select

## Search Functionality
- Fuzzy text matching against all discoverable commands
- Categories: Views, Gantt Operations, Table Operations, Resources, Settings, Help
- Recent/pinable items at top
- Category filtering optional

## Implementation Details
```rust
// Command palette entry
pub struct PaletteEntry {
    pub id: String,           // unique identifier
    pub label: String,        // display text
    pub category: PaletteCategory,
    pub description: Option<String>,
    pub shortcut: Option<String>,  // keyboard shortcut if applicable
}

pub enum PaletteCategory {
    Views,
    GanttOperations,
    TableOperations,
    Resources,
    Settings,
    Help,
}
```

## UI Integration
- Palette opens as modal overlay over main UI
- Search field at top with focus placed automatically
- Results listed with category badges
- Selection executes the corresponding command
- Esc closes without action

## Blocking Dependencies
- None (foundational GUI improvement)

## Acceptance Criteria
- [ ] Ctrl+K opens command palette with search field focused
- [ ] Fuzzy search matches against views, commands, activities, resources
- [ ] Results organized by category with badges
- [ ] Esc closes palette without action
- [ ] Selection executes the commanded action
- [ ] Palette appears over main UI (non-blocking model, egui context)
- [ ] Recent/pinable items shown at top
- [ ] Help text available within palette

## Verification
- Headless test: Ctrl+K opens palette, search finds entries, selection works
- UI test: Palette renders, search matches, categories displayed, Esc closes
- Integration: Palette works with existing keyboard navigation
- Edge case: No matches found → shows appropriate message
- Edge case: Very large command set (≥100 entries) → performance remains smooth