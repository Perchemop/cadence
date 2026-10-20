# Capability Parity Tracker

Living list of capabilities. Status values: `todo`, `in progress`, `done`.

## Phase 0 — Foundation

| Capability | Status | Notes |
|---|---|---|
| Cargo workspace + crate layout | done | core / commands / persistence / app |
| GitHub Actions CI (fmt, clippy, test) | done | `.github/workflows/ci.yml` |
| Typed command layer + undo/redo | done | `cadence-commands` |
| Product name locked | done | `docs/product-name.md` |
| SQLite project file persistence | todo | Phase 0 target, JSON first |
| Minimal project model | done | Project, Activity, Relationship |

## Vertical slice (guide §12.4)

| Capability | Status | Notes |
|---|---|---|
| Create project | done | |
| Two activities | done | |
| FS link with lag | done | |
| Run CPM | done | forward/backward pass, float |
| Show dates in a table | done | CLI table via `cadence` binary |

## Phase 1 — Scheduling engine

| Capability | Status | Notes |
|---|---|---|
| WBS hierarchy | todo | |
| SS / FF / SF relationships | todo | FS + lag done |
| Multiple calendars (working time, exceptions) | todo | single implicit calendar now |
| Constraints (ASAP/ALAP, dates) | todo | |
| Total float / free float / critical path | in progress | basic float done |
| Cycle detection & diagnostics | todo | |
| Property-based engine tests | todo | |

## Phase 2 — Gantt & tables

| Capability | Status | Notes |
|---|---|---|
| Interactive Gantt | todo | egui/eframe, custom rendering |
| Activity table | todo | |
| Dependency editing | todo | |
| Zoom / pan | todo | |
| Light + dark design system | todo | |

## Later phases

Resources (Phase 3), baselines & progress/EVM (Phase 4), printing, reports,
interop, polish (Phase 5) — all `todo`.
