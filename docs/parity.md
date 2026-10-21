# Capability Parity Tracker

Living list of capabilities. Status values: `todo`, `in progress`, `done`.

## Phase 0 — Foundation

| Capability | Status | Notes |
|---|---|---|
| Cargo workspace + crate layout | done | core / commands / persistence / app |
| GitHub Actions CI (fmt, clippy, test) | done | `.github/workflows/ci.yml` |
| Typed command layer + undo/redo | done | `cadence-commands` |
| Product name locked | done | `docs/product-name.md` |
| SQLite project file persistence | done | relational schema, `PRAGMA user_version`, transactional save/load, CLI `save`/`load` |
| Minimal project model | done | Project, Activity, Relationship |

## Vertical slice (guide §12.4)

| Capability | Status | Notes |
|---|---|---|
| Create project | done | `Project::new`, command layer |
| Two activities | done | demo builds three |
| FS link with lag | done | plus SS/FF/SF and leads |
| Run CPM | done | forward/backward pass, float, critical set |
| Show dates in a table | done | aligned CLI table via `cadence` binary |

## Phase 1 — Scheduling engine

| Capability | Status | Notes |
|---|---|---|
| WBS hierarchy | done | tree with cycle-safe reparenting, undoable commands |
| SS / FF / SF relationships | done | signed lag/lead supported |
| Working-day calendar arithmetic | done | Mon–Fri pattern + holiday exceptions |
| Multiple calendars (working time, exceptions) | done | named calendars, per-activity selection; cross-calendar convention in `architecture.md` |
| Constraints (ASAP/ALAP, dates) | done | SNET/SNLT/FNET/FNLT date bounds, scheduled vs early/late dates, undoable command |
| Total float / free float / critical path | done | critical activity set ordered by early start; longest-path extraction later |
| Cycle detection & diagnostics | done | Kahn + DFS path, reported as `SchedError::Cycle` |
| Property-based engine tests | todo | unit coverage in core and commands |
| Serde round-trip for model/schedule | done | project, link types, schedule |

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
