# Progress

Source of truth for build state. "Done" means implemented **and** verified by
green CI (`main` branch).

## 2026-10-21 — P2-2 activity table

- Activity table now supports column sorting, row selection, an inspector
  with name/duration/constraint editing routed through commands (with
  undo/redo), and schedule recomputation.
- CI green on `main` (run 39578242160).

## 2026-10-21 — P2-1 egui app shell

- Added `cadence-ui-egui` crate with design tokens, light/dark theme
  switching, and a window rendering the project and scheduled dates.
- CLI gains `cadence gui [file]` (demo if no file).
- CI green on `main` (run 39577082469).

## 2026-10-21 — P1-6 property-based tests + Phase-1 QC gate

- Added `crates/core/tests/properties.rs` (proptest): working-day invariants
  for arbitrary DAGs, relationship logic and non-negative float, ALAP
  preservation of bounds/logic/project finish, constraint acceptance or
  violation, and calendar arithmetic round-trips.
- Fixed a mixed-calendar backward-pass bug found by the property tests:
  late finish is now normalised to the activity's own working calendar even
  for non-terminal activities.
- Recorded the Phase-1 QC gate in `docs/qc/phase-1.md`.
- CI green on `main` (run 39576146773, 89 tests + proptest cases).

## 2026-10-21 — P1-5 constraints

- Activities carry a `Constraint`: ASAP (default), ALAP, and start/finish
  no-earlier/no-later-than date bounds. Lower bounds raise the forward pass;
  upper bounds cap the backward pass and produce
  `SchedError::ConstraintViolated` when even the earliest schedule misses
  them.
- `ActivitySchedule` now reports `scheduled_start`/`scheduled_finish` next to
  the CPM early/late bounds. ALAP activities use their late dates, pulled
  back only where an already-scheduled successor would be violated (D10).
- Added the undoable `SetActivityConstraint` command and CLI columns for
  scheduled dates.
- Project-file schema is now version 2 with an in-place migration from
  version 1 (new `constraint_kind`/`constraint_date` columns).
- CI green on `main` (run 39574713546, 84 tests).

## 2026-10-21 — P1-4 SQLite persistence

- Project files are ordinary SQLite databases with a relational schema
  (`project`, `project_holidays`, `calendars`, `calendar_holidays`,
  `wbs_nodes`, `activities`, `relationships`) versioned with
  `PRAGMA user_version` (D9).
- Saves validate dangling WBS/calendar/relationship references before
  writing, then replace the file contents in one transaction; a failed save
  leaves the previous file intact. Loads reject newer schema versions and
  malformed data with explicit errors.
- CLI now supports `cadence save <file>` and `cadence load <file>` on top of
  the demo; `LinkType::from_abbreviation` round-trips precedence types.
- CI green on `main` (run 39566216222, 58 tests).

## 2026-10-21 — Guide updated → process retro-fit + WBS & calendars

- Adopted the updated generic guide: discovery conversation completed with
  the human; created `docs/project-brief.md`, `docs/scope.md`,
  `docs/inspiration.md`, `docs/assumptions-and-questions.md`, and all root
  tracking files; added `.opencode/agents/` definitions.
- Discovery answers recorded; multi-OS CI explicitly deferred (D5).
- P1-2 done: WBS tree (`wbs.rs`) with cycle-safe reparenting, delete guards,
  and undoable commands.
- P1-3 done: named calendars (`calendar_set.rs`), per-activity calendar
  selection, cross-calendar CPM convention (D6), undoable calendar commands.
  CI green on `main` (run 39559389569) after fixing cross-calendar
  working-day stepping with dedicated `advance`/`retreat` helpers.

## 2026-10-20 — Vertical slice complete

- Repo created (public: https://github.com/Perchemop/cadence), Phase 0
  scaffold + CI.
- P1-1 done: domain model, full CPM engine (FS/SS/FF/SF + lag, float,
  critical set, cycle detection), command layer with undo/redo, CLI demo
  printing the schedule table. 30+ unit tests. CI green after one clippy fix
  (needless lifetime in a test helper).

## Next slice

- Phase 2 (P2-1..): egui/eframe app shell, design system, then Gantt and
  tables. No further Phase-1 work remains.