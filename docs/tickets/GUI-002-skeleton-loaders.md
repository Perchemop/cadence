# Ticket: GUI-002 — Skeleton Loaders for Async Operations

## Objective
Implement skeleton loaders for async operations in the Cadence UI, matching the pattern from Linear, Planera, Airtable, and other modern applications. This is a **high priority** improvement identified in the GUI Inspiration Audit.

## Target Files
- `crates/design/src.rs` — Add `Skeleton` component and related design tokens
- `crates/ui-egui/src/lib.rs` — Use skeleton components in status area, project load views, report generation

## Skeleton Component Design (from gui-inspiration-audit.md)
- Three density variants: Comfortable (default), Compact, Condensed
- Update check: skeleton for status area
- Project load: skeleton for table/Gantt
- Report generation: skeleton for report preview
- Match the exact shape and size of real content
- Subtle shimmer animation, not aggressive pulsing

## Skeleton Component Types
```rust
/// Skeleton for individual table cells
pub struct TableCellSkeleton;

/// Skeleton for Gantt chart bar area
pub struct GanttSkeleton;

/// Skeleton for status/progress area
pub struct StatusSkeleton;

/// Skeleton for entire view (table or Gantt)
pub struct ViewSkeleton {
    pub density: ViewDensity,
}

/// Density variants
#[derive(Clone, Debug, PartialEq)]
pub enum ViewDensity {
    Comfortable,   // default, more padding
    Compact,       // reduced padding
    Condensed,     // minimal padding
}
```

## Implementation Pattern (from research)
- Update check: skeleton for status area showing loading state
- Project load: skeleton for table/Gantt while data loads
- Report generation: skeleton for report preview area
- Skeleton should match the layout it replaces
- Subtle animation: shimmer effect that flows directionally
- No visual difference from real content aside from animation

## Blocking Dependencies
- None (foundational UI component)

## Acceptance Criteria
- [ ] Skeleton shows for update check status area while checking
- [ ] Skeleton shows for project load (table and Gantt views) while data loads
- [ ] Skeleton shows for report generation preview
- [ ] Three density variants available: Comfortable (default), Compact, Condensed
- [ ] Skeleton matches the exact shape of real content
- [ ] Subtle shimmer animation (not aggressive pulsing)
- [ ] No visual layout shift when skeleton replaces loaded content
- [ ] Skeleton respects theme (light/dark)

## Verification
- Headless test: Trigger update check → skeleton shows → results replace skeleton
- UI test: Project loads → skeleton renders → data replaces skeleton smoothly
- Integration: Skeleton respects density preferences
- Edge case: Slow network → skeleton persists appropriately
- Edge case: Zero activities → skeleton shows appropriate empty state