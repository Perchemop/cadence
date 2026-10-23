# Architecture

## Overview

Cadence is a Rust workspace with strict layering. The scheduling engine is
pure and headless; all mutation flows through a typed command layer; the UI
(egui/eframe, Phase 2) is a consumer of both.

```
app (binary)
 ├── ui-egui (Phase 2)      interactive Gantt, tables, inspector, resources
 ├── commands                typed commands, undo/redo  ──► core
 ├── persistence            SQLite project files        ──► core
 └── interchange            MSPDI/MPX import & export  ──► core
```

## Crates

| Crate                | Responsibility |
|----------------------|----------------|
| `cadence-core`       | Domain model (Project, WBS, Activity, Relationship, Resource, Calendar, Baseline) and the CPM engine. No I/O, no UI. |
| `cadence-commands`   | `Command` trait + `CommandStack`. Single mutation path; produces inverse commands for undo/redo. |
| `cadence-persistence`| SQLite project file format: versioned schema, transactional save/load, in-place migrations. |
| `cadence-core` (report) | Markdown project reports rendered in memory; the printable artifact (D16). |
| `cadence-interchange`| Open interchange: Microsoft Project XML (MSPDI), MPX, CSV and Cadence JSON readers/writers, with in-crate XML and JSON parsers (D14). |
| `cadence-app`        | Application binary; wires the layers together. |

## Key principles

1. **Command-centric.** Every mutation of a `Project` is a typed command.
   Nothing mutates state behind the command layer.
2. **Pure engine.** `cadence-core` computes schedules from an immutable
   snapshot of the model. The engine is deterministic and side-effect free,
   which makes it trivially testable and usable from CLI/tests/future
   scripting.
3. **Dates.** Calendar arithmetic uses the `time` crate (`Date`).
   Working-day logic is driven by `Calendar` values in the model.
4. **Error handling.** No panics outside tests. Rich `Result` types
   (`thiserror`) at crate boundaries.
5. **Performance target.** Smooth interaction with thousands of activities
   (engine pass over 10k activities well under 100 ms).

## Domain model

`Project` is the aggregate root: name, start date, default calendar, named
calendars, WBS, activities, and relationships. `Activity` stores
`original_duration` in working days, a scheduling `Constraint`, and optional
cached schedule fields (`early_*`, `late_*`, `scheduled_*`, float, critical).
`Relationship` stores predecessor/successor, a `LinkType` (`FS`, `SS`, `FF`,
`SF`), and a signed lag in working days (negative values are leads). IDs are
`u64` newtypes.

`Calendar` is a serialisable value type: a Monday-first weekly working pattern
plus holiday exceptions. `is_working_day`, `next`/`prev_working_day`,
`add_working_days`, and `working_days_between` are the date-arithmetic
primitives. The default is Monday–Friday with no holidays.

`CalendarSet` holds named calendars with unique (case-insensitive) names and
stable ids; each `Activity` may select one or inherit the project default.
Keeping calendars as values (instead of behind trait objects) lets projects
round-trip through JSON.

## WBS hierarchy

The WBS is a single-rooted tree (`crates/core/src/wbs.rs`). Nodes have stable
`WbsId`s and ordered children; activities reference a node by id. Invariants
are enforced at mutation time: no orphan parents, no duplicate ids, no cycles
when reparenting. Deleting a node with children is rejected (D7); activity
assignments are checked by the command layer, which owns activity state.

## Command layer

`CommandStack` stores both the forward command and the inverse returned by
`Command::apply`, so undo and redo apply the correct direction. Commands
validate before mutating, keeping a failed command atomic. Implemented
commands: `CreateActivity`, `CreateRelationship`, `DeleteActivity`,
`DeleteRelationship`, `SetActivityDuration`, and `RenameActivity`. Deleting an
activity also removes and records its relationships so undo restores the whole
prior state.

## CPM engine design

`engine::schedule(&Project) -> Result<Schedule, SchedError>` is pure and
deterministic:

- Validate duplicate IDs and relationship endpoints.
- Detect cycles with Kahn's algorithm and report a concrete cycle path instead
  of hanging.
- Forward pass: earliest start/finish, honouring FS/SS/FF/SF, signed lag,
  and lower-bound date constraints (SNET/FNET).
- Backward pass: latest start/finish from the project finish, capped by
  upper-bound date constraints (SNLT/FNLT). An upper bound missed by the
  earliest schedule is a hard error.
- Total float = `LS - ES`; free float = minimum slack over outgoing
  relationships, or total float for terminal activities; critical = total
  float <= 0.
- Scheduled dates: early dates for ASAP activities; late dates for ALAP
  activities, pulled back only where a successor's scheduled date would be
  violated.
- Return `Schedule` with per-activity `ActivitySchedule` values and a
  `critical_path` of critical activities ordered by early start.
  `Schedule::apply_to` copies results into the optional cached `Activity`
  fields.

All date arithmetic is in working days. FS uses the conventional extra day
(`ES_succ >= EF_pred + 1 + lag`); SS, FF, and SF use their documented
constraints.

When a relationship crosses calendars, the successor's calendar governs the
forward-pass constraint (when it may start or finish) and the predecessor's
calendar governs the backward pass (how late it may finish). Activities with
no successors are normalised to the latest working day of their own calendar.
Total float is measured in each activity's working days.

## Persistence

`cadence-persistence` stores a project in a native SQLite file (D9). Schema
version 4 is relational rather than a JSON blob: `project` and
`project_holidays` hold the aggregate settings and default calendar;
`calendars` and `calendar_holidays` hold named calendars; `wbs_nodes` stores
the tree with a `preorder` column that preserves sibling order on load;
`activities` stores durations, constraints, optional WBS/calendar references,
cost, progress and recorded actual dates; `relationships` stores endpoints,
link type, and signed lag; `resources` stores the dictionary and each
resource's capacity; `activity_resources` stores assignments; and `baselines`
with `baseline_snapshots` store captured plans. Foreign keys are enabled on
every connection. Version 1, 2 and 3 files are migrated in place by adding
the missing columns and tables.

`save_project` validates the aggregate (dangling WBS, calendar, and
relationship references are errors), creates the schema when the file is new,
then replaces all rows inside one transaction. A failed save rolls back, so
the previous file survives. `load_project` distinguishes a missing file from
empty data, rejects newer schema versions, parses dates strictly, and rebuilds
`WbsTree`/`CalendarSet` through their validating constructors.

Cached schedule fields are not persisted: schedules are derived data and are
recomputed on load. Records are written in project order (and WBS pre-order)
so round-trips preserve ordering.

Constraints are documented in D10, the resource model in D11, leveling in
D12, money and percentage representation in D13, the interchange formats in
D14, and recorded progress in D15. `cadence-interchange` handles open
interchange outside the SQLite file: Microsoft Project XML (MSPDI), MPX, CSV
and Cadence JSON, each as a documented subset. Printing/PDF remains
later-phase work.

## Resources

A `Resource` is a named `kind` (Labour, Equipment, Material) with a `capacity`
in units per working day. Activities reference resources by id, and each
assignment consumes one unit per working day for the activity's duration
(D11). `engine::resource_histogram` turns a schedule into committed units per
working day, `engine::resource_conflicts` reports every day over capacity, and
`engine::level_resources` greedily delays the latest-starting conflicting
activity until the over-allocation clears (D12).
