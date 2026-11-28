# Ticket: GUI-007 — Longest Path / Driving Chain Visualization

## Objective
Implement longest path / driving chain visualization in the Gantt chart, matching the pattern from P6 "longest path", nPlan "driving chain", and ALICE "critical drivers". This is a **medium priority** improvement identified in the GUI Inspiration Audit.

## Target Files
- `crates/ui-egui/src/lib.rs` — Gantt rendering with longest path toggle
- `crates/schedule-analysis/src/lib.rs` — Longest path analysis engine
- `crates/design/src.rs` — Visual style for driving chain

## Longest Path vs Critical Path (from research)
- **Critical path**: float-based (activities with zero total float)
- **Longest path / driving chain**: the chain of driving activities from project start to finish, regardless of float
- Distinct visualization: critical path in one style, driving chain in another
- Toggle in Gantt toolbar to switch between views

## Implementation Pattern (from gui-inspiration-audit.md)
- Show driving chain to project finish in different style from critical path
- Toggle in Gantt toolbar to show/hide longest path
- Distinct from critical path (float-based)
- Activities on critical path (zero total float) are a subset of the driving chain

## Visual Style (from research)
```rust
/// Style for longest path / driving chain visualization
pub enum PathStyle {
    Critical,      // critical path (zero total float) - red
    DrivingChain,  // longest path/driving chain - distinct color/style
}
```

## Gantt Rendering additions
```rust
/// Whether to show the longest path/driving chain in Gantt
pub struct GanttRenderOptions {
    pub show_critical_path: bool,
    pub show_driving_chain: bool,
    pub path_style: PathStyle,
    pub toggle_shortcut: Option<String>,  // keyboard shortcut if applicable
}
```

## Toggle Functionality
- Toolbar button: "Show Longest Path" / "Hide Longest Path"
- Keyboard shortcut (if defined)
- Persists user preference
- When toggled: activities on the driving chain re-render in distinct style
- Critical path remains visible optionally alongside

## Blocking Dependencies
- None (foundational scheduling visualization)

## Acceptance Criteria
- [ ] Gantt toolbar has "Show Longest Path" / "Hide Longest Path" toggle
- [ ] Driving chain activities rendered in distinct style from critical path
- [ ] Toggle persists user preference across sessions
- [ ] Critical path and driving chain can be shown simultaneously
- [ ] Toggle shortcut available (configurable)
- [ ] Visual distinction: different color, line style, or fill pattern

## Verification
- Headless test: Toggle longest path → driving chain activities re-render → toggle off → revert
- UI test: Longest path toggle renders distinct visual style, works with critical path
- Integration: Longest path works with existing critical path, skeleton loaders, density modes
- Edge case: Projects with no driving chain (single activity) → appropriate handling
- Edge case: Very large programs (≥10k activities) → performance remains smooth