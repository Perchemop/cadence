# Implementation Plan

Cadence is built in small vertical slices. Each slice is Definition-of-Done:
parity items implemented, tests written, CI green, tracking files updated.
The roadmap phases (from the original brief, now recorded in `docs/scope.md`)
are subdivided into slices below.

## Phase 0 — Foundation ✅ (complete)

- [x] Workspace, crate layout, licences, README
- [x] GitHub Actions CI (fmt, clippy -D warnings, tests, caching)
- [x] Typed command layer with undo/redo
- [x] Product name locked; discovery & tracking files (this docs update)
- [x] SQLite project-file persistence *(delivered as slice P1-4 below)*

## Phase 1 — Scheduling engine

- [x] P1-1: Core model + CPM engine (FS/SS/FF/SF + lag, float, critical set,
      cycle detection) + CLI schedule table
- [x] P1-2: WBS hierarchy (nodes, reparenting rules, commands, undo)
- [x] P1-3: Multiple named calendars + per-activity calendar selection +
      cross-calendar scheduling convention
- [x] P1-4: SQLite persistence for projects (open schema, versioned)
- [x] P1-5: Constraints (ASAP/ALAP, date constraints) in the engine
- [x] P1-6: Property-based engine tests (arbitrary graphs, calendar maths)
- [x] P1-7: Phase-1 QC gate (checklist in `docs/qc/`)

## Phase 2 — Gantt & tables

- [x] P2-1: egui/eframe app shell, design tokens, light+dark themes
- [x] P2-2: Activity table (columns, sorting, inline editing via commands)
- [x] P2-3: Gantt rendering at scale (virtualised rows, timeline zoom/pan)
- [x] P2-4: Dependency editing (drag links, lag editing, cycle prevention)
- [x] P2-5: Phase-2 QC gate + performance baseline (10k activities)

## Phase 3 — Resources

- [x] P3-1: Resource dictionary + assignments
- [x] P3-2: Usage histograms
- [x] P3-3: Basic resource leveling

## Phase 4 — Baselines & progress

- [x] P4-1: Multiple baselines + variance views
- [ ] P4-2: Progress updating (percent complete, actual dates)
- [x] P4-3: EVM metrics (SPI/CPI)

## Phase 5 — Polish & interop

- [x] P5-1: Open interchange export+import (Microsoft Project XML/MSPDI and MPX)
- [ ] P5-1b: JSON/CSV interchange export+import
- [ ] P5-2: Printing/PDF and reports
- [ ] P5-3: Keyboard mastery, accessibility audit, performance pass
- [ ] P5-4: Documentation, example projects, first tagged release
