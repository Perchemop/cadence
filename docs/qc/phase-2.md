# Phase 2 Quality Gate

Checklist for the Phase-2 Gantt and table slices.

## Functionality

- [x] egui/eframe app shell with light/dark theme switching (P2-1).
- [x] Activity table with sortable columns, selection, inspector editing via
      commands, undo/redo (P2-2).
- [x] Gantt view with timeline, bars, dependency arrows, zoom/pan, and row
      virtualisation (P2-3).
- [x] Drag-to-create dependencies and lag editing with cycle prevention
      (P2-4).
- [x] Performance baseline: 10,000-activity chain schedules in under 1 s
      (the budget catches algorithmic regressions, not runner speed; CI runs
      it on three platforms)
      (P2-5).

## Engineering

- [x] `cargo fmt --check --all` passes.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [x] `cargo test --workspace` passes.
- [x] CI green on `main`.

## Carried into Phase 3

- Resource dictionary, assignments, histograms, leveling (P3-1..P3-3).
- Design polish and accessibility pass (P2 QC subset deferred to Phase 3).
