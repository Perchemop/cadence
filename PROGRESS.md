# Progress

Source of truth for build state. "Done" means implemented **and** verified by
green CI (`main` branch).

## 2026-10-22 — P5-1 open interchange: Microsoft Project XML and MPX

- New `cadence-interchange` crate with `mspdi` (Microsoft Project XML) and
  `mpx` readers and writers covering activities, durations at eight working
  hours per day, constraints, relationships with lag, the resource dictionary
  with capacities, assignments and baselines.
- XML is read by an in-crate reader (`interchange::xml`): declarations,
  comments and processing instructions are skipped, entities are decoded, and
  mismatched or unterminated tags are rejected, so no dependency is added to
  the build.
- CLI: `cadence export-xml|import-xml|export-mpx|import-mpx <in> <out>`.
- Round-trip and hand-written-document tests for both formats.
- CI green on `opencode/ci` (run 39754077818, 152 tests passed).

## 2026-10-22 — P3-3 resource conflicts, leveling and resource panel

- `engine::resource_conflicts` reports every working day on which an activity
  consumes a resource beyond its capacity;
  `engine::level_resources` greedily delays the latest-starting conflicting
  activity until the over-allocation clears, keeping dependencies intact.
- `Schedule` gained a scheduling path parameterised by minimum start dates so
  leveling can probe candidate schedules without mutating the project.
- New Resources view in the egui shell: dictionary table with kind, capacity,
  assignment count and over-allocated days; create/rename/delete; capacity and
  kind editing; per-day usage against capacity; assignment toggles; leveling
  preview and an apply action that writes undoable `SetActivityConstraint`
  commands.
- CI green on `opencode/ci` (runs 39751953234 and 39748409298).

## 2026-10-22 — P3-1/P3-2/P4-1/P4-3 resources, baselines and EVM

- Resource dictionary on `Project`: `ResourceKind`, name uniqueness
  (case-insensitive, matching the SQLite constraint), and `capacity` in units
  per working day. Undoable commands for create/rename/set-capacity/set-kind/
  delete, plus assignment commands with correct inverse operations.
- `engine::resource_histogram` collects the resource units committed on each
  working day.
- Project baselines: `engine::capture_baseline` snapshots duration, start and
  finish per activity; capture/remove/restore are undoable and preserve
  position.
- Earned value metrics (`engine::evm_metrics`): planned value, earned value,
  actual cost, schedule and cost variance, SPI and CPI in basis points, with
  cost stored in minor units.
- SQLite schema v3 adds `resources.capacity` and the resource, assignment,
  baseline and cost tables and columns, with v1→v3 and v2→v3 migrations.
- Comments in the schema and decisions record the append-only table style.
- CI green on `opencode/ci` (runs 39748409298 and 39751953234).

## 2026-10-21 — P2-5 performance baseline + Phase-2 QC gate

- Added a 10,000-activity scheduling performance test and the Phase-2 QC
  gate in `docs/qc/phase-2.md`.
- CI green on `main`.

## 2026-10-21 — P2-4 dependency editing

- Drag-to-create FS links in the Gantt view with cycle detection and
  feedback; lag editing via the inspector and `SetRelationshipLag`.
- CI green on `main` (run 39586623396).

## 2026-10-21 — P2-3 Gantt rendering

- Added a Gantt view with timeline axis, activity bars, dependency arrows,
  zoom (Ctrl+wheel), scrolling, and row virtualisation.
- CI green on `main` (run 39579147001).

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