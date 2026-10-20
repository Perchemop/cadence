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
3. **Dates.** Calendar arithmetic uses the `time` crate (`NaiveDate`).
   Working-day logic is driven by `Calendar` values in the model.
4. **Error handling.** No panics outside tests. Rich `Result` types
   (`thiserror`) at crate boundaries.
5. **Performance target.** Smooth interaction with thousands of activities
   (engine pass over 10k activities well under 100 ms).

## CPM engine design (Phase 1 scope, foundation laid now)

- Forward pass: earliest start/finish per activity, honouring relationship
  types (FS/SS/FF/SF), lag/lead, calendars, and constraints.
- Backward pass: latest start/finish from project end; total float and free
  float derived from the two passes.
- Cycle detection: relationship graphs are validated before scheduling;
  cycles are reported as errors, never as hangs.
- The vertical slice currently implements FS links with lag and basic
  forward/backward pass over working days; the remaining relationship types,
  constraints, and calendars land in Phase 1.
