# Architecture

## Overview

Cadence is a Rust workspace with strict layering. The scheduling engine is
pure and headless; all mutation flows through a typed command layer; the UI
(egui/eframe, Phase 2) is a consumer of both.

```
app (binary)
 ├── ui-egui (Phase 2)      interactive Gantt, tables, inspector
 ├── commands                typed commands, undo/redo  ──► core
 └── persistence            SQLite + JSON/CSV/XML      ──► core
```

## Crates

| Crate                | Responsibility |
|----------------------|----------------|
| `cadence-core`       | Domain model (Project, WBS, Activity, Relationship, Resource, Calendar, Baseline) and the CPM engine. No I/O, no UI. |
| `cadence-commands`   | `Command` trait + `CommandStack`. Single mutation path; produces inverse commands for undo/redo. |
| `cadence-persistence`| SQLite project file format and open interchange (JSON first, CSV/XML later). |
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

`Project` is the aggregate root: name, start date, calendar, activities, and
relationships. `Activity` stores `original_duration` in working days plus
optional cached schedule fields (`early_start`, `early_finish`, `late_start`,
`late_finish`, `total_float`, `free_float`, `critical`). `Relationship` stores
predecessor/successor, a `LinkType` (`FS`, `SS`, `FF`, `SF`), and a signed lag
in working days (negative values are leads). IDs are `u64` newtypes.

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
- Forward pass: earliest start/finish, honouring FS/SS/FF/SF and signed lag.
- Backward pass: latest start/finish from the project finish.
- Total float = `LS - ES`; free float = minimum slack over outgoing
  relationships, or total float for terminal activities; critical = total
  float <= 0.
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

Constraints (date/ASAP/ALAP), resources, baselines, and SQLite persistence
remain later-phase work.
